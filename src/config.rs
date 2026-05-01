use std::fs;
use std::io::ErrorKind;
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
        Self::load_from_path(Path::new(CONFIG_FILENAME))
    }

    fn load_from_path(path: &Path) -> Result<Self> {
        let raw = match fs::read_to_string(path) {
            Ok(contents) => contents,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Self::default()),
            Err(error) => {
                return Err(error).with_context(|| format!("failed to read {}", path.display()))
            }
        };
        let config: PttoConfig =
            toml::from_str(&raw).with_context(|| format!("failed to parse {}", path.display()))?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::{PttoConfig, CONFIG_FILENAME};

    #[test]
    fn load_returns_defaults_when_file_missing() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let config_path = temp_dir.path().join(CONFIG_FILENAME);

        let config = PttoConfig::load_from_path(&config_path).expect("load should succeed");

        assert!(config.host.is_none());
        assert!(config.domain.is_none());
        assert!(config.ssh_key.is_none());
    }

    #[test]
    fn load_parses_values_from_ptto_toml() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let config_path = temp_dir.path().join(CONFIG_FILENAME);
        std::fs::write(
            &config_path,
            "host = \"root@host\"\ndomain = \"example.com\"\nssh_key = \"~/.ssh/id_ed25519\"\n",
        )
        .expect("write config");

        let config = PttoConfig::load_from_path(&config_path).expect("load should parse");

        assert_eq!(config.host.as_deref(), Some("root@host"));
        assert_eq!(config.domain.as_deref(), Some("example.com"));
        assert_eq!(config.ssh_key.as_deref(), Some("~/.ssh/id_ed25519"));
    }

    #[test]
    fn load_surfaces_parse_errors_with_context() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let config_path = temp_dir.path().join(CONFIG_FILENAME);
        std::fs::write(&config_path, "not = { valid = toml").expect("write config");

        let err = PttoConfig::load_from_path(&config_path).expect_err("invalid toml should fail");

        let err_message = err.to_string();
        assert!(err_message.contains("failed to parse"));
        assert!(err_message.contains(CONFIG_FILENAME));
    }
}
