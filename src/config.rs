use std::{env, path::PathBuf, process::Command};

use serde::{Deserialize, Serialize};

use crate::get_powershell_path;

#[derive(Debug, Serialize, Deserialize)]
pub struct Scoop {
    pub last_update: Option<String>,
    pub virustotal_api_key: Option<String>,
    pub scoop_repo: Option<String>,
    pub scoop_branch: Option<String>,
}

impl Scoop {
    /// Converts the config path into the [`Scoop`] struct
    ///
    /// # Errors
    /// - The file was not valid UTF-8
    /// - The read file was did not match the expected structure
    pub fn read() -> std::io::Result<Self> {
        let config_path = Self::get_path();

        let config = std::fs::read_to_string(config_path)?;

        let config: Self = serde_json::from_str(&config)?;

        Ok(config)
    }

    /// Gets the scoop config path
    ///
    /// # Panics
    /// - The config directory does not exist
    pub fn get_path() -> PathBuf {
        let xdg_config = env::var("XFG_CONFIG_HOME").map(PathBuf::from);
        let user_profile = env::var("USERPROFILE").map(|path| PathBuf::from(path).join(".config"));

        let path = match (xdg_config, user_profile) {
            (Ok(path), _) | (_, Ok(path)) => path,
            _ => panic!("Could not find config directory"),
        }
        .join("scoop")
        .join("config.json");

        assert!(path.exists(), "Could not find config file");

        path
    }

    /// Update the last time the scoop was updated
    ///
    /// # Panics
    /// - The powershell path does not exist
    pub fn update_last_update_time(&mut self) {
        // TODO: Move to using chrono for time serialization
        let date_time = Command::new(get_powershell_path().unwrap())
            .args([
                "-NoProfile",
                "-Command",
                "[System.DateTime]::Now.ToString('o')",
            ])
            .output()
            .expect("Failed to get time from powershell");

        let stdout_raw = date_time.stdout;
        let mut stdout = String::from_utf8(stdout_raw).unwrap();

        // Remove '\r' from the end of the string
        stdout.pop();
        // Remove '\n' from the end of the string
        stdout.pop();

        self.last_update = Some(stdout);
    }

    /// Save the modified scoop config
    ///
    /// # Errors
    /// - The struct could not be serialized to JSON
    pub fn save(&self) -> std::io::Result<()> {
        let config_path = Self::get_path();

        let config = serde_json::to_string_pretty(self)?;

        std::fs::write(config_path, config)?;

        Ok(())
    }
}
