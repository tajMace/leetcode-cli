// load/save LEETCODE_SESSION + csrftoken from ~/.config/leetcode-cli/config.toml

use std::{fs, path::PathBuf};

use crate::error::{LeetCodeError, Result};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Config {
    leetcode_session: Option<String>,
    csrf_token: Option<String>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_filepath = get_config_filepath()?;
        let config_string = fs::read_to_string(&config_filepath)?;

        Ok(toml::from_str::<Self>(&config_string)?)
    }

    pub fn save(&self) -> Result<()> {
        let config_filepath = get_config_filepath()?;
        let contents = toml::to_string(&self)?;

        fs::create_dir_all(
            &config_filepath
                .parent()
                .expect("No parent dir of config filepath"),
        )?;
        Ok(fs::write(config_filepath, contents)?)
    }

    pub fn require_session(&self) -> Result<(&str, &str)> {
        match (&self.leetcode_session, &self.csrf_token) {
            (Some(session), Some(csrf)) => Ok((session.as_str(), csrf.as_str())),
            _ => Err(LeetCodeError::NotAuthenticated),
        }
    }
}

fn get_config_filepath() -> Result<PathBuf> {
    let config_dir = dirs::config_dir().ok_or_else(|| LeetCodeError::ConfigDir)?;
    let config_filepath = config_dir.join("lc-cli/config.toml");

    Ok(config_filepath)
}

/*
 * ========== TESTING ==========
 */

#[cfg(test)]
mod tests {
    use super::*;

    // --- require_session: pure logic, no filesystem involved ---
    mod require_session_tests {
        use super::*;

        #[test]
        fn require_session_ok_when_both_present() {
            let config = Config {
                leetcode_session: Some("sess123".to_string()),
                csrf_token: Some("csrf456".to_string()),
            };
            let result = config.require_session();
            assert!(result.is_ok());
            let (session, csrf) = result.unwrap();
            assert_eq!(session, "sess123");
            assert_eq!(csrf, "csrf456");
        }

        #[test]
        fn require_session_errors_when_session_missing() {
            let config = Config {
                leetcode_session: None,
                csrf_token: Some("csrf456".to_string()),
            };
            assert!(matches!(
                config.require_session(),
                Err(LeetCodeError::NotAuthenticated)
            ));
        }

        #[test]
        fn require_session_errors_when_csrf_missing() {
            let config = Config {
                leetcode_session: Some("sess123".to_string()),
                csrf_token: None,
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
            };
            assert!(matches!(
                config.require_session(),
                Err(LeetCodeError::NotAuthenticated)
            ));
        }
    }

    // --- TOML round-trip: no filesystem, just serialize then deserialize ---
    mod toml_tests {
        use super::*;

        #[test]
        fn toml_round_trip_preserves_values() {
            let original = Config {
                leetcode_session: Some("sess123".to_string()),
                csrf_token: Some("csrf456".to_string()),
            };
            let serialized = toml::to_string(&original).expect("serialize should succeed");
            let deserialized: Config =
                toml::from_str(&serialized).expect("deserialize should succeed");

            assert_eq!(deserialized.leetcode_session, original.leetcode_session);
            assert_eq!(deserialized.csrf_token, original.csrf_token);
        }

        #[test]
        fn toml_round_trip_handles_missing_fields() {
            // Simulates a config.toml someone hand-wrote with only one field set
            let partial_toml = r#"leetcode_session = "only_this_one""#;
            let config: Config = toml::from_str(partial_toml).expect("should deserialize");
            assert_eq!(config.leetcode_session, Some("only_this_one".to_string()));
            assert_eq!(config.csrf_token, None);
        }
    }

    // --- load()/save(): real filesystem, but redirected to a temp dir ---
    mod load_and_save_tests {
        use super::*;

        #[test]
        fn save_then_load_round_trips_through_real_disk() {
            let temp_dir =
                std::env::temp_dir().join(format!("leetcode-cli-test-{}", std::process::id()));
            std::fs::create_dir_all(&temp_dir).expect("failed to create temp test dir");

            // SAFETY-ish note: this mutates process-wide env state for the
            // duration of this test. Fine for a single-threaded test run;
            // worth knowing if you ever see flaky failures under `cargo test`
            // running suites in parallel.
            unsafe { std::env::set_var("XDG_CONFIG_HOME", &temp_dir) }

            let config = Config {
                leetcode_session: Some("real_session".to_string()),
                csrf_token: Some("real_csrf".to_string()),
            };

            config.save().expect("save should succeed");
            let loaded = Config::load().expect("load should succeed");

            assert_eq!(loaded.leetcode_session, config.leetcode_session);
            assert_eq!(loaded.csrf_token, config.csrf_token);

            std::fs::remove_dir_all(&temp_dir).ok(); // best-effort cleanup
        }
    }
}
