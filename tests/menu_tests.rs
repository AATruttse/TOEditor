//! Tests for menu functionality and language switching

use toeditor::i18n::Language;

#[test]
fn test_language_enum() {
    // Test language enum functionality
    let en = Language::English;
    let ru = Language::Russian;
    
    assert_eq!(en.code(), "en");
    assert_eq!(ru.code(), "ru");
    
    assert_eq!(en.name(), "English");
    assert_eq!(ru.name(), "Русский");
}

#[test]
fn test_language_from_code() {
    assert_eq!(Language::from_code("en"), Language::English);
    assert_eq!(Language::from_code("ru"), Language::Russian);
    assert_eq!(Language::from_code("unknown"), Language::English); // Default
    assert_eq!(Language::from_code(""), Language::English); // Default
}

#[test]
fn test_language_switching_logic() {
    // Test that language codes are correctly parsed
    let test_cases = vec![
        ("en", Language::English),
        ("ru", Language::Russian),
        ("EN", Language::English), // Case insensitive would be nice, but current impl is case sensitive
        ("RU", Language::English), // Falls back to English
    ];
    
    for (code, expected) in test_cases {
        let result = Language::from_code(code);
        assert_eq!(result, expected, "Failed for code: {}", code);
    }
}
