//! Tests for configuration and settings

use toeditor::config::Settings;
use std::path::PathBuf;

#[test]
fn test_default_settings() {
    let settings = Settings::default();
    assert_eq!(settings.symbol_style, "NATO");
    assert_eq!(settings.color_scheme, "light");
    assert_eq!(settings.language, "en");
    assert!(settings.database_path.is_none());
}

#[test]
fn test_settings_serialization() {
    let settings = Settings {
        symbol_style: "RF".to_string(),
        color_scheme: "dark".to_string(),
        language: "ru".to_string(),
        database_path: Some(PathBuf::from("/tmp/test.db")),
    };
    
    // Test that settings can be serialized to TOML
    let toml = toml::to_string(&settings).expect("Should serialize to TOML");
    assert!(toml.contains("symbol_style = \"RF\""));
    assert!(toml.contains("language = \"ru\""));
}

#[test]
fn test_settings_deserialization() {
    let toml_content = r#"
symbol_style = "NATO"
color_scheme = "default"
language = "ru"
database_path = "/tmp/test.db"
"#;
    
    let settings: Settings = toml::from_str(toml_content).expect("Should deserialize from TOML");
    assert_eq!(settings.symbol_style, "NATO");
    assert_eq!(settings.language, "ru");
    assert!(settings.database_path.is_some());
}
