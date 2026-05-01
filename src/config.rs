use std::fs;
use std::io::ErrorKind;

use anyhow::{Context, Result};
use serde::Deserialize;

const CONFIG_FILENAME: &str = ".ptto.toml";

#[derive(Debug, Default, Deserialize, Clone)]
pub struct PttoConfig {
    pub host: Option<String>,
    pub domain: Option<String>,
    pub ssh_key: Option<String>,
}

impl PttoConfig {
    pub fn load() -> Result<Self> {
        let raw = match fs::read_to_string(CONFIG_FILENAME) {
            Ok(contents) => contents,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Self::default()),
            Err(error) => {
                return Err(error).with_context(|| format!("failed to read {}", CONFIG_FILENAME));
            }
        };
        let config: PttoConfig =
            toml::from_str(&raw).with_context(|| format!("failed to parse {}", CONFIG_FILENAME))?;
        Ok(config)
    }
}
