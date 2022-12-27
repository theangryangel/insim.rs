pub(crate) mod definition;
pub(crate) mod path;

use miette::{IntoDiagnostic, Result};
use std::{fs, path::PathBuf};

pub(crate) fn read(config_path: &PathBuf) -> Result<definition::Config> {
    if !config_path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("config file not found: {}", config_path.display()),
        ))
        .into_diagnostic();
    }

    let config_content = fs::read_to_string(config_path).into_diagnostic()?;
    knuffel::parse::<definition::Config>(config_path.to_str().unwrap(), &config_content)
        .into_diagnostic()
}
