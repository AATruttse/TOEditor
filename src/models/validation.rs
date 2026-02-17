//! Input validation for domain models

/// Validation error with field name and message
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

/// Validate a library before creation or update.
/// Returns a list of validation errors (empty if valid).
pub fn validate_library(name: &str, country: &str, era: &str) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    let name = name.trim();
    if name.is_empty() {
        errors.push(ValidationError {
            field: "name".to_string(),
            message: "Library name cannot be empty".to_string(),
        });
    } else if name.len() > 200 {
        errors.push(ValidationError {
            field: "name".to_string(),
            message: "Library name cannot exceed 200 characters".to_string(),
        });
    }

    let country = country.trim();
    if country.is_empty() {
        errors.push(ValidationError {
            field: "country".to_string(),
            message: "Country cannot be empty".to_string(),
        });
    }

    let era = era.trim();
    if era.is_empty() {
        errors.push(ValidationError {
            field: "era".to_string(),
            message: "Era cannot be empty".to_string(),
        });
    }

    errors
}

/// Validate a branch name pair.
pub fn validate_branch(name_ru: &str, name_en: &str) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    if name_ru.trim().is_empty() && name_en.trim().is_empty() {
        errors.push(ValidationError {
            field: "name".to_string(),
            message: "At least one name (Russian or English) must be provided".to_string(),
        });
    }

    if name_ru.len() > 200 {
        errors.push(ValidationError {
            field: "name_ru".to_string(),
            message: "Russian name cannot exceed 200 characters".to_string(),
        });
    }

    if name_en.len() > 200 {
        errors.push(ValidationError {
            field: "name_en".to_string(),
            message: "English name cannot exceed 200 characters".to_string(),
        });
    }

    errors
}

/// Validate a formation level.
pub fn validate_formation_level(name_ru: &str, name_en: &str, ordinal: i32) -> Vec<ValidationError> {
    let mut errors = validate_branch(name_ru, name_en);

    if ordinal < 0 {
        errors.push(ValidationError {
            field: "standard_level_ordinal".to_string(),
            message: "Standard level ordinal cannot be negative".to_string(),
        });
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_library_valid() {
        let errors = validate_library("US Army 2003", "US", "2003");
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_library_empty_name() {
        let errors = validate_library("", "US", "2003");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].field, "name");
    }

    #[test]
    fn test_validate_library_whitespace_name() {
        let errors = validate_library("   ", "US", "2003");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].field, "name");
    }

    #[test]
    fn test_validate_library_empty_country() {
        let errors = validate_library("Test", "", "2003");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].field, "country");
    }

    #[test]
    fn test_validate_library_empty_era() {
        let errors = validate_library("Test", "US", "");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].field, "era");
    }

    #[test]
    fn test_validate_library_multiple_errors() {
        let errors = validate_library("", "", "");
        assert_eq!(errors.len(), 3);
    }

    #[test]
    fn test_validate_library_long_name() {
        let long_name = "a".repeat(201);
        let errors = validate_library(&long_name, "US", "2003");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].field, "name");
    }

    #[test]
    fn test_validate_branch_valid() {
        let errors = validate_branch("Пехота", "Infantry");
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_branch_one_name_ok() {
        assert!(validate_branch("Пехота", "").is_empty());
        assert!(validate_branch("", "Infantry").is_empty());
    }

    #[test]
    fn test_validate_branch_both_empty() {
        let errors = validate_branch("", "");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].field, "name");
    }

    #[test]
    fn test_validate_branch_long_names() {
        let long = "a".repeat(201);
        let errors = validate_branch(&long, &long);
        assert_eq!(errors.len(), 2);
    }

    #[test]
    fn test_validate_formation_level_valid() {
        let errors = validate_formation_level("Рота", "Company", 3);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_formation_level_negative_ordinal() {
        let errors = validate_formation_level("Рота", "Company", -1);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].field, "standard_level_ordinal");
    }

    #[test]
    fn test_validation_error_display() {
        let err = ValidationError {
            field: "name".to_string(),
            message: "cannot be empty".to_string(),
        };
        assert_eq!(format!("{}", err), "name: cannot be empty");
    }
}
