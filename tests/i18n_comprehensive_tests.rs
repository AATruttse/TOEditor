//! Comprehensive tests for i18n module

use toeditor::i18n::{Language, TranslationManager};

#[test]
fn test_translation_manager_new() {
    let tm = TranslationManager::new();
    assert_eq!(tm.current_language(), Language::English);
}

#[test]
fn test_translation_manager_default() {
    let tm = TranslationManager::default();
    assert_eq!(tm.current_language(), Language::English);
}

#[test]
fn test_translation_manager_set_language() {
    let mut tm = TranslationManager::new();
    assert_eq!(tm.current_language(), Language::English);
    
    tm.set_language(Language::Russian);
    assert_eq!(tm.current_language(), Language::Russian);
    
    tm.set_language(Language::English);
    assert_eq!(tm.current_language(), Language::English);
}

#[test]
fn test_translation_manager_load_from_settings() {
    let mut tm = TranslationManager::new();
    
    // This should not panic even if settings file doesn't exist
    let result = tm.load_from_settings();
    assert!(result.is_ok());
    
    // Language should be set based on settings or default to English
    // We can't easily test the actual loading without mocking, but we can test it doesn't crash
}

#[test]
fn test_language_code() {
    assert_eq!(Language::English.code(), "en");
    assert_eq!(Language::Russian.code(), "ru");
}

#[test]
fn test_language_name() {
    assert_eq!(Language::English.name(), "English");
    assert_eq!(Language::Russian.name(), "Русский");
}

#[test]
fn test_language_from_code() {
    assert_eq!(Language::from_code("en"), Language::English);
    assert_eq!(Language::from_code("ru"), Language::Russian);
    assert_eq!(Language::from_code("unknown"), Language::English); // Default
    assert_eq!(Language::from_code(""), Language::English); // Default
    assert_eq!(Language::from_code("FR"), Language::English); // Default
}

#[test]
fn test_language_equality() {
    assert_eq!(Language::English, Language::English);
    assert_eq!(Language::Russian, Language::Russian);
    assert_ne!(Language::English, Language::Russian);
}
