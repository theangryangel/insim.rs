use std::{
    path::{Path, PathBuf},
    process::Command,
};

use serde::Deserialize;

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
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned();

    let schema_path = workspace_root.join("insim_o3/insim_schema.json");
    let types_path = workspace_root.join("insim_o3/python/insim_o3/packets.py");
    let pyproject_path = workspace_root.join("insim_o3/pyproject.toml");

    let pyproject: PyProject = toml::from_str(&std::fs::read_to_string(&pyproject_path)?)?;
    let python_version = requires_python_to_target(&pyproject.project.requires_python)?;

    let variants = generate_schema(&schema_path)?;
    generatepackets(&schema_path, &types_path, &python_version, &variants)?;

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

/// Reshape the schemars-emitted schema for datamodel-codegen, returning the
/// ordered list of variant class names.
///
/// Why we mutate:
/// - schemars emits each variant in `oneOf` as `{$ref, properties: {type:
///   {const: …}}, required: [type]}`. datamodel-codegen can't match that
///   compound form back to the named `$defs` entry, so it mints numbered
///   duplicates (`Packet1`, `Packet2`, …). We fix this by injecting the
///   `type` discriminator field into each named `$defs` struct and replacing
///   each oneOf entry with a plain `{"$ref": …}`.
/// - We then strip `oneOf` (and the `title`/`description` that go with it)
///   from the top level entirely. Without those, datamodel-codegen does not
///   emit a wrapper `Packet` class, so the Python side does not have to
///   truncate one out. The Python `AnyPacket` alias is built directly from
///   the returned variant list.
///
/// The returned variant names are in the same order schemars emitted them.
fn prepare_schema_for_codegen(schema: &mut serde_json::Value) -> Vec<String> {
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

    // Capture variant names in oneOf order for the Python `AnyPacket` alias.
    let variants: Vec<String> = pairs
        .iter()
        .map(|(ref_val, _)| {
            ref_val
                .strip_prefix("#/$defs/")
                .unwrap_or(ref_val.as_str())
                .to_owned()
        })
        .collect();

    // Drop top-level fields that would make datamodel-codegen emit a wrapper
    // class.  Without `oneOf` it has no union to model; without `title` it
    // has no name to assign.
    if let Some(obj) = schema.as_object_mut() {
        obj.remove("oneOf");
        obj.remove("title");
        obj.remove("description");
    }

    variants
}

fn generate_schema(output: &Path) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let schema = schemars::schema_for!(insim::Packet);
    let mut schema_value: serde_json::Value =
        serde_json::to_value(&schema).expect("schemars produced non-serialisable schema");

    let variants = prepare_schema_for_codegen(&mut schema_value);

    let generated = serde_json::to_string_pretty(&schema_value)
        .expect("post-processed schema is non-serialisable");

    std::fs::write(output, generated)?;
    println!("wrote {}", output.display());

    Ok(variants)
}

fn generatepackets(
    schema: &Path,
    output: &Path,
    python_version: &str,
    variants: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
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
            output.to_str().unwrap(),
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

    postprocess_generated_packets(output, variants)?;
    println!("wrote {}", output.display());

    Ok(())
}

/// Append the `AnyPacket` Python type alias and clean up dead code that
/// datamodel-codegen leaves at the top of the file.
///
/// The schema-side prepare step strips `oneOf` so there is no real top-level
/// schema, but datamodel-codegen still emits a placeholder
/// `class Model(RootModel[Any]): root: Any` plus an `Any` typing import.
/// We delete that pair and then append the `AnyPacket` alias built from the
/// variant list captured during schema preparation.
fn postprocess_generated_packets(
    path: &Path,
    variants: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut contents = std::fs::read_to_string(path)?;

    // Appease ruff.
    contents = replace_once(
        &contents,
        "from typing import Annotated, Any, Literal\n",
        "from typing import Annotated, Literal\n",
        path,
    )?;

    // Strip the placeholder root model.  Two leading newlines are part of the
    // ruff-formatted block separator above, two trailing newlines are the
    // separator below.
    contents = replace_once(
        &contents,
        "\nclass Model(RootModel[Any]):\n    root: Any\n\n",
        "",
        path,
    )?;

    if !contents.ends_with('\n') {
        contents.push('\n');
    }
    contents.push_str("\n\ntype AnyPacket = (\n");
    for (i, name) in variants.iter().enumerate() {
        if i == 0 {
            contents.push_str(&format!("    {name}\n"));
        } else {
            contents.push_str(&format!("    | {name}\n"));
        }
    }
    contents.push_str(")\n");

    std::fs::write(path, contents)?;
    Ok(())
}

/// Replace exactly one occurrence of `needle` with `replacement` in
/// `contents`, returning a clear error if the count is not 1.
fn replace_once(
    contents: &str,
    needle: &str,
    replacement: &str,
    path: &Path,
) -> Result<String, Box<dyn std::error::Error>> {
    let count = contents.matches(needle).count();
    if count != 1 {
        return Err(format!(
            "expected exactly one occurrence of {:?} in {}, found {}",
            needle.lines().next().unwrap_or(needle),
            path.display(),
            count
        )
        .into());
    }
    Ok(contents.replacen(needle, replacement, 1))
}
