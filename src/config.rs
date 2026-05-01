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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::{Mutex, MutexGuard, OnceLock};

    use super::{PttoConfig, CONFIG_FILENAME};

    fn cwd_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    struct CwdGuard {
        original_dir: PathBuf,
        _lock: MutexGuard<'static, ()>,
    }

    impl CwdGuard {
        fn enter(path: &std::path::Path) -> Self {
            let lock = cwd_lock().lock().expect("cwd lock should not be poisoned");
            let original_dir = std::env::current_dir().expect("cwd");
            std::env::set_current_dir(path).expect("switch cwd");
            Self {
                original_dir,
                _lock: lock,
            }
        }
    }

    impl Drop for CwdGuard {
        fn drop(&mut self) {
            std::env::set_current_dir(&self.original_dir).expect("restore cwd");
        }
    }

    #[test]
    fn load_returns_defaults_when_file_missing() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let _cwd_guard = CwdGuard::enter(temp_dir.path());

        let config = PttoConfig::load().expect("load should succeed");

        assert!(config.host.is_none());
        assert!(config.domain.is_none());
        assert!(config.ssh_key.is_none());
    }

    #[test]
    fn load_parses_values_from_ptto_toml() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let _cwd_guard = CwdGuard::enter(temp_dir.path());
        std::fs::write(
            CONFIG_FILENAME,
            "host = \"root@host\"\ndomain = \"example.com\"\nssh_key = \"~/.ssh/id_ed25519\"\n",
        )
        .expect("write config");

        let config = PttoConfig::load().expect("load should parse");

        assert_eq!(config.host.as_deref(), Some("root@host"));
        assert_eq!(config.domain.as_deref(), Some("example.com"));
        assert_eq!(config.ssh_key.as_deref(), Some("~/.ssh/id_ed25519"));
    }

    #[test]
    fn load_surfaces_parse_errors_with_context() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let _cwd_guard = CwdGuard::enter(temp_dir.path());
        std::fs::write(CONFIG_FILENAME, "not = { valid = toml").expect("write config");

        let err = PttoConfig::load().expect_err("invalid toml should fail");

        assert!(err.to_string().contains("failed to parse .ptto.toml"));
    }
}
