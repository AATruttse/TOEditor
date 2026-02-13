//! Comprehensive tests for Config module

use toeditor::config::Settings;
use tempfile::TempDir;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_settings_save_and_load() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("settings.toml");
    
    // Create settings
    let settings = Settings {
        symbol_style: "RF".to_string(),
        color_scheme: "dark".to_string(),
        language: "ru".to_string(),
        database_path: Some(PathBuf::from("/tmp/test.db")),
    };
    
    // Mock config_dir to return temp directory
    // Note: This test verifies serialization/deserialization
    let toml = toml::to_string(&settings).unwrap();
    fs::write(&config_path, toml).unwrap();
    
    // Load settings
    let content = fs::read_to_string(&config_path).unwrap();
    let loaded: Settings = toml::from_str(&content).unwrap();
    
    assert_eq!(loaded.symbol_style, "RF");
    assert_eq!(loaded.color_scheme, "dark");
    assert_eq!(loaded.language, "ru");
    assert_eq!(loaded.database_path, Some(PathBuf::from("/tmp/test.db")));
}

#[test]
fn test_settings_config_dir() {
    // Test that config_dir returns a valid path
    let config_dir = Settings::config_dir();
    assert!(config_dir.is_ok());
    let path = config_dir.unwrap();
    assert!(path.to_string_lossy().contains("toeditor") || path.to_string_lossy().contains("TOEditor"));
}

#[test]
fn test_settings_data_dir() {
    // Test that data_dir returns a valid path
    let data_dir = Settings::data_dir();
    assert!(data_dir.is_ok());
    let path = data_dir.unwrap();
    assert!(path.to_string_lossy().contains("toeditor") || path.to_string_lossy().contains("TOEditor"));
}

#[test]
fn test_settings_default_database_path() {
    // Test that default_database_path returns a valid path
    let db_path = Settings::default_database_path();
    assert!(db_path.is_ok());
    let path = db_path.unwrap();
    assert!(path.file_name().unwrap() == "toeditor.db");
}

#[test]
fn test_settings_default() {
    let settings = Settings::default();
    assert_eq!(settings.symbol_style, "NATO");
    assert_eq!(settings.color_scheme, "light");
    assert_eq!(settings.language, "en");
    assert_eq!(settings.database_path, None);
}

#[test]
fn test_settings_serialization_roundtrip() {
    let original = Settings {
        symbol_style: "Custom".to_string(),
        color_scheme: "dark".to_string(),
        language: "ru".to_string(),
        database_path: Some(PathBuf::from("/custom/path.db")),
    };
    
    let toml = toml::to_string(&original).unwrap();
    let deserialized: Settings = toml::from_str(&toml).unwrap();
    
    assert_eq!(original.symbol_style, deserialized.symbol_style);
    assert_eq!(original.color_scheme, deserialized.color_scheme);
    assert_eq!(original.language, deserialized.language);
    assert_eq!(original.database_path, deserialized.database_path);
}
