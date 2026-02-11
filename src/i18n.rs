//! Internationalization support

use anyhow::Result;

/// Supported languages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    English,
    Russian,
}

impl Language {
    /// Get language code
    pub fn code(&self) -> &'static str {
        match self {
            Language::English => "en",
            Language::Russian => "ru",
        }
    }

    /// Get language name
    pub fn name(&self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Russian => "Русский",
        }
    }

    /// Parse from code
    pub fn from_code(code: &str) -> Self {
        match code {
            "ru" => Language::Russian,
            _ => Language::English,
        }
    }
}

/// Translation manager
pub struct TranslationManager {
    current_language: Language,
}

impl TranslationManager {
    /// Create new translation manager
    pub fn new() -> Self {
        Self {
            current_language: Language::English,
        }
    }

    /// Load language from settings
    pub fn load_from_settings(&mut self) -> Result<()> {
        if let Ok(settings) = crate::config::Settings::load() {
            self.current_language = Language::from_code(&settings.language);
        }
        Ok(())
    }

    /// Set current language
    pub fn set_language(&mut self, lang: Language) {
        self.current_language = lang;
    }

    /// Get current language
    pub fn current_language(&self) -> Language {
        self.current_language
    }
}

impl Default for TranslationManager {
    fn default() -> Self {
        Self::new()
    }
}
