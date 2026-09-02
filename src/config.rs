// load/save LEETCODE_SESSION + csrftoken from ~/.config/lt-cli/config.toml

use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::error::{LeetCodeError, Result};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default)]
pub struct Config {
    pub leetcode_session: Option<String>,
    pub csrf_token: Option<String>,
    pub storage_path: Option<PathBuf>,
}

impl Config {
    pub fn load() -> Result<Self> {
        Self::load_from(&Self::get_config_filepath()?)
    }

    pub fn save(&self) -> Result<()> {
        self.save_to(&Self::get_config_filepath()?)
    }

    pub fn require_session(&self) -> Result<(&str, &str)> {
        match (&self.leetcode_session, &self.csrf_token) {
            (Some(session), Some(csrf)) => Ok((session.as_str(), csrf.as_str())),
            _ => Err(LeetCodeError::NotAuthenticated),
        }
    }

    pub fn require_storage_dir(&self) -> Result<&Path> {
        match &self.storage_path {
            Some(path) => Ok(path),
            None => Err(LeetCodeError::NoStorageDir),
        }
    }

    /* ===== HELPERS ===== */
    fn get_config_filepath() -> Result<PathBuf> {
        let config_dir = dirs::config_dir().ok_or_else(|| LeetCodeError::ConfigDir)?;
        let config_filepath = config_dir.join("lc-cli/config.toml");

        Ok(config_filepath)
    }

    fn load_from(path: &Path) -> Result<Self> {
        let contents = fs::read_to_string(path)?;
        Ok(toml::from_str(&contents)?)
    }

    fn save_to(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, toml::to_string(self)?)?;
        Ok(())
    }
}

/*
 * ========== TESTING ==========
 */
#[cfg(test)]
mod tests {
    use super::*;

    fn sample_config() -> Config {
        Config {
            leetcode_session: Some("sess123".to_string()),
            csrf_token: Some("csrf456".to_string()),
            storage_path: Some(PathBuf::from("/tmp/some-storage-dir")),
        }
    }

    mod require_session_tests {
        use super::*;

        #[test]
        fn require_session_ok_when_both_present() {
            let config = sample_config();
            let (session, csrf) = config.require_session().unwrap();
            assert_eq!(session, "sess123");
            assert_eq!(csrf, "csrf456");
        }

        #[test]
        fn require_session_errors_when_session_missing() {
            let config = Config {
                leetcode_session: None,
                ..sample_config()
            };
            assert!(matches!(
                config.require_session(),
                Err(LeetCodeError::NotAuthenticated)
            ));
        }

        #[test]
        fn require_session_errors_when_csrf_missing() {
            let config = Config {
                csrf_token: None,
                ..sample_config()
            };
            assert!(matches!(
                config.require_session(),
                Err(LeetCodeError::NotAuthenticated)
            ));
        }

        #[test]
        fn require_session_errors_when_both_missing() {
            let config = Config {
                leetcode_session: None,
                csrf_token: None,
                ..sample_config()
            };
            assert!(matches!(
                config.require_session(),
                Err(LeetCodeError::NotAuthenticated)
            ));
        }
    }

    mod require_storage_dir_tests {
        use super::*;

        #[test]
        fn ok_when_present() {
            let config = sample_config();
            let path = config.require_storage_dir().unwrap();
            assert_eq!(path, Path::new("/tmp/some-storage-dir"));
        }

        #[test]
        fn errors_when_missing() {
            let config = Config {
                storage_path: None,
                ..sample_config()
            };
            assert!(matches!(
                config.require_storage_dir(),
                Err(LeetCodeError::NoStorageDir)
            ));
        }
    }

    mod toml_tests {
        use super::*;

        #[test]
        fn toml_round_trip_preserves_values() {
            let original = sample_config();
            let serialized = toml::to_string(&original).expect("serialize should succeed");
            let deserialized: Config =
                toml::from_str(&serialized).expect("deserialize should succeed");

            assert_eq!(deserialized.leetcode_session, original.leetcode_session);
            assert_eq!(deserialized.csrf_token, original.csrf_token);
            assert_eq!(deserialized.storage_path, original.storage_path);
        }

        #[test]
        fn toml_round_trip_handles_missing_fields() {
            let partial_toml = r#"leetcode_session = "only_this_one""#;
            let config: Config = toml::from_str(partial_toml).expect("should deserialize");
            assert_eq!(config.leetcode_session, Some("only_this_one".to_string()));
            assert_eq!(config.csrf_token, None);
            assert_eq!(config.storage_path, None);
        }
    }

    // --- load_from/save_to: real filesystem, but a temp path passed
    // explicitly — never touches dirs::config_dir() or your real config,
    // on any OS, unlike the old env-var-based version of this test.
    mod load_and_save_tests {
        use super::*;

        #[test]
        fn save_then_load_round_trips_through_real_disk() {
            let temp_path = std::env::temp_dir()
                .join(format!("leetcode-cli-test-{}", std::process::id()))
                .join("config.toml");

            let config = sample_config();
            config.save_to(&temp_path).expect("save should succeed");
            let loaded = Config::load_from(&temp_path).expect("load should succeed");

            assert_eq!(loaded.leetcode_session, config.leetcode_session);
            assert_eq!(loaded.csrf_token, config.csrf_token);
            assert_eq!(loaded.storage_path, config.storage_path);

            std::fs::remove_dir_all(temp_path.parent().unwrap()).ok();
        }
    }
}
