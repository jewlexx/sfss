use anyhow::Context;
use chrono::DateTime;

#[derive(Debug, serde::Deserialize, serde::Serialize, Default)]
/// SFSU Specific Config
pub struct Config {
    pub telemetry: Telemetry,
}

impl Config {
    pub fn config_path() -> anyhow::Result<std::path::PathBuf> {
        let dir = dirs::config_dir()
            .context("Couldn't find config directory")?
            .join("sfsu");

        if dir.try_exists().is_ok_and(|exists| !exists) {
            std::fs::create_dir_all(&dir)?;
        }

        let path = dir.join("config.json");

        Ok(path)
    }

    pub fn load() -> anyhow::Result<Self> {
        let path = Self::config_path()?;

        if path.try_exists().is_ok_and(|exists| !exists) {
            let config = Self::default();
            serde_json::to_writer(std::fs::File::create(&path)?, &config)?;
            return Ok(config);
        }

        let contents = std::fs::read_to_string(&path)?;

        Ok(serde_json::from_str(&contents)?)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::config_path()?;
        serde_json::to_writer(std::fs::File::create(&path)?, self)?;
        Ok(())
    }

    pub fn enable_telemetry(&mut self) {
        self.telemetry.enabled = true;
        self.telemetry.notified_at = Some(chrono::Utc::now());
    }

    pub fn disable_telemetry(&mut self) {
        self.telemetry.enabled = false;
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Telemetry {
    pub enabled: bool,
    pub notified_at: Option<DateTime<chrono::Utc>>,
}

impl Default for Telemetry {
    fn default() -> Self {
        Self {
            enabled: true,
            notified_at: None,
        }
    }
}
