use std::fs;
use std::path::Path;

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
        let path = Path::new(CONFIG_FILENAME);
        if !path.exists() {
            return Ok(Self::default());
        }

        let raw = fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", CONFIG_FILENAME))?;
        let config: PttoConfig =
            toml::from_str(&raw).with_context(|| format!("failed to parse {}", CONFIG_FILENAME))?;
        Ok(config)
    }
}
