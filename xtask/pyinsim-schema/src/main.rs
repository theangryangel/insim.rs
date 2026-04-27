use std::{
    path::{Path, PathBuf},
    process::Command,
};

use clap::Parser;
use serde::Deserialize;

#[derive(Debug, Parser)]
#[command(about = "Generate insim_schema.json and pyinsim/_types.py from Rust packet types")]
struct Cli {
    /// Verify outputs are up to date without writing
    #[arg(long, default_value_t = false)]
    check: bool,
}

#[derive(Deserialize)]
struct PyProject {
    project: PyProjectProject,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct PyProjectProject {
    requires_python: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned();

    let schema_path = workspace_root.join("pyinsim/insim_schema.json");
    let types_path = workspace_root.join("pyinsim/python/insim_rs/_types.py");
    let pyproject_path = workspace_root.join("pyinsim/pyproject.toml");

    let pyproject: PyProject = toml::from_str(&std::fs::read_to_string(&pyproject_path)?)?;
    let python_version = requires_python_to_target(&pyproject.project.requires_python)?;

    generate_schema(&schema_path, cli.check)?;
    generate_types(&schema_path, &types_path, &python_version, cli.check)?;

    Ok(())
}

/// Extract `major.minor` from a PEP 440 `requires-python` specifier.
///
/// Takes the version from the first clause and strips any patch component,
/// e.g. `>=3.12,<4` → `"3.12"`.
fn requires_python_to_target(spec: &str) -> Result<String, Box<dyn std::error::Error>> {
    let clause = spec.split(',').next().unwrap_or(spec);
    let start = clause
        .find(|c: char| c.is_ascii_digit())
        .ok_or_else(|| format!("no version number in requires-python = \"{spec}\""))?;
    let version = &clause[start..];
    let mut parts = version.split('.');
    let major = parts.next().ok_or("missing major version")?;
    let minor = parts.next().ok_or("missing minor version")?;
    Ok(format!("{major}.{minor}"))
}

/// Restructure `Packet.oneOf` so datamodel-codegen generates named classes.
///
/// schemars emits each variant as `{$ref, properties: {type: {const: …}}, required: [type]}`.
/// datamodel-codegen can't match that compound form back to the named `$defs` entry, so it
/// mints numbered duplicates (`Packet1`, `Packet2`, …).
///
/// We fix this by:
/// 1. Injecting the `type` discriminator field into each named `$defs` struct.
/// 2. Replacing each compound oneOf entry with a plain `{"$ref": "…"}`.
///
/// After this transform datamodel-codegen generates `class Ncn(BaseModel): type: Literal["Ncn"]`
/// and `Packet = RootModel[Ncn | Mso | …]` using the named classes throughout.
fn postprocess_packet_discriminators(schema: &mut serde_json::Value) {
    // Collect ($ref, type const) pairs from oneOf entries before any mutation.
    let pairs: Vec<(String, String)> = schema
        .get("oneOf")
        .and_then(|v| v.as_array())
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|entry| {
            let ref_val = entry.get("$ref")?.as_str()?.to_owned();
            let type_const = entry
                .get("properties")?
                .get("type")?
                .get("const")?
                .as_str()?
                .to_owned();
            Some((ref_val, type_const))
        })
        .collect();

    // Inject the `type` discriminator field into each referenced $defs struct.
    for (ref_val, type_const) in &pairs {
        let def_name = ref_val.strip_prefix("#/$defs/").unwrap_or(ref_val.as_str());

        if let Some(def) = schema
            .get_mut("$defs")
            .and_then(|d| d.get_mut(def_name))
            .and_then(|d| d.as_object_mut())
        {
            def.entry("properties")
                .or_insert_with(|| serde_json::json!({}))
                .as_object_mut()
                .unwrap()
                .insert(
                    "type".to_owned(),
                    serde_json::json!({"type": "string", "const": type_const, "default": type_const}),
                );

            let required = def
                .entry("required")
                .or_insert_with(|| serde_json::json!([]));
            if let Some(arr) = required.as_array_mut() {
                if !arr.iter().any(|v| v.as_str() == Some("type")) {
                    arr.push(serde_json::json!("type"));
                }
            }
        }
    }

    // Replace each compound oneOf entry with a plain $ref.
    if let Some(one_of) = schema.get_mut("oneOf").and_then(|v| v.as_array_mut()) {
        for entry in one_of.iter_mut() {
            if let Some(ref_val) = entry
                .get("$ref")
                .and_then(|v| v.as_str())
                .map(|s| s.to_owned())
            {
                *entry = serde_json::json!({"$ref": ref_val});
            }
        }
    }
}

fn generate_schema(output: &Path, check: bool) -> Result<(), Box<dyn std::error::Error>> {
    let schema = schemars::schema_for!(insim::Packet);
    let mut schema_value: serde_json::Value =
        serde_json::to_value(&schema).expect("schemars produced non-serialisable schema");

    postprocess_packet_discriminators(&mut schema_value);

    let generated = serde_json::to_string_pretty(&schema_value)
        .expect("post-processed schema is non-serialisable");

    if check {
        let existing = std::fs::read_to_string(output)
            .map_err(|_| format!("{} not found — run without --check first", output.display()))?;
        if existing != generated {
            return Err(format!(
                "{} is out of date. Run `cargo run -p xtask-pyinsim-schema` to regenerate.",
                output.display()
            )
            .into());
        }
        println!("ok: {}", output.display());
    } else {
        std::fs::write(output, generated)?;
        println!("wrote {}", output.display());
    }

    Ok(())
}

fn generate_types(
    schema: &Path,
    output: &Path,
    python_version: &str,
    check: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let effective_output = if check {
        std::env::temp_dir().join("pyinsim_types_check.py")
    } else {
        output.to_owned()
    };

    let status = Command::new("uv")
        .args([
            "tool",
            "run",
            "--from",
            "datamodel-code-generator[ruff]",
            "datamodel-codegen",
            "--input",
            schema.to_str().unwrap(),
            "--input-file-type",
            "jsonschema",
            "--output",
            effective_output.to_str().unwrap(),
            "--output-model-type",
            "pydantic_v2.BaseModel",
            "--use-union-operator",
            "--use-annotated",
            "--disable-timestamp",
            "--field-constraints",
            "--use-default",
            "--collapse-root-models",
            "--target-python-version",
            python_version,
            "--formatters",
            "ruff-format",
            "ruff-check",
            "--type-mappings",
            "integer+uint8=int32",
            "integer+uint16=int32",
            "integer+int16=int32",
            "integer+uint32=int64",
            "integer+uint64=int64",
            "integer+uint=int64",
        ])
        .current_dir(schema.parent().unwrap())
        .status()?;

    if !status.success() {
        return Err(format!("uv run datamodel-codegen exited with {status}").into());
    }

    if check {
        let existing = std::fs::read_to_string(output)
            .map_err(|_| format!("{} not found — run without --check first", output.display()))?;
        let generated = std::fs::read_to_string(&effective_output)?;
        if existing != generated {
            return Err(format!(
                "{} is out of date. Run `cargo run -p xtask-pyinsim-schema` to regenerate.",
                output.display()
            )
            .into());
        }
        println!("ok: {}", output.display());
    } else {
        println!("wrote {}", output.display());
    }

    Ok(())
}
