use std::{
    path::{Path, PathBuf},
    process::Command,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned();

    let schema_path = workspace_root.join("insim_o3/insim_schema.json");
    let script_path = workspace_root.join("insim_o3/scripts/generate_packets.py");

    write_schema(&schema_path)?;
    run_generator(&script_path, &schema_path)?;

    Ok(())
}

fn write_schema(output: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let schema = schemars::schema_for!(insim::Packet);
    let json = serde_json::to_string_pretty(&schema)?;
    std::fs::write(output, json)?;
    println!("wrote {}", output.display());
    Ok(())
}

fn run_generator(script: &Path, schema: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // --no-project skips a `uv sync` that would otherwise trigger a maturin
    // build of the insim_o3 extension. The generator only uses stdlib +
    // shells out to `uv tool run ruff`, so no project venv is needed.
    let status = Command::new("uv")
        .args(["run", "--no-project", "python", script.to_str().unwrap()])
        .current_dir(schema.parent().unwrap())
        .status()?;

    if !status.success() {
        return Err(format!("generator script exited with {status}").into());
    }

    Ok(())
}
