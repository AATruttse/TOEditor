//! Application configuration and settings

use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Tactical symbol style (NATO, RF, etc.)
    pub symbol_style: String,
    /// Color scheme
    pub color_scheme: String,
    /// Language code (en, ru, etc.)
    pub language: String,
    /// Database path
    pub database_path: Option<PathBuf>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            symbol_style: "NATO".to_string(),
            color_scheme: "default".to_string(),
            language: "en".to_string(),
            database_path: None,
        }
    }
}

impl Settings {
    /// Load settings from file
    pub fn load() -> Result<Self> {
        let config_path = Self::config_dir()?.join("settings.toml");
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let settings: Settings = toml::from_str(&content)?;
            Ok(settings)
        } else {
            Ok(Settings::default())
        }
    }

    /// Save settings to file
    pub fn save(&self) -> Result<()> {
        let config_dir = Self::config_dir()?;
        std::fs::create_dir_all(&config_dir)?;
        let config_path = config_dir.join("settings.toml");
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;
        Ok(())
    }

    /// Get application config directory
    pub fn config_dir() -> Result<PathBuf> {
        let dirs = ProjectDirs::from("com", "toeditor", "TOEditor")
            .ok_or_else(|| anyhow::anyhow!("Failed to get project directories"))?;
        Ok(dirs.config_dir().to_path_buf())
    }

    /// Get application data directory
    pub fn data_dir() -> Result<PathBuf> {
        let dirs = ProjectDirs::from("com", "toeditor", "TOEditor")
            .ok_or_else(|| anyhow::anyhow!("Failed to get project directories"))?;
        Ok(dirs.data_dir().to_path_buf())
    }

    /// Get default database path
    pub fn default_database_path() -> Result<PathBuf> {
        let data_dir = Self::data_dir()?;
        std::fs::create_dir_all(&data_dir)?;
        Ok(data_dir.join("toeditor.db"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();
        assert_eq!(settings.symbol_style, "NATO");
        assert_eq!(settings.language, "en");
    }
}
