//! Edge case tests for import functions

use toeditor::import::{import_json, import_json_with_versions};
use toeditor::models::Library;
use tempfile::NamedTempFile;

#[test]
fn test_import_json_empty_library() {
    let library = Library::new(
        "Empty".to_string(),
        "US".to_string(),
        "2003".to_string(),
        "Author".to_string(),
    );
    let json = serde_json::to_string(&library).unwrap();
    let file = NamedTempFile::new().unwrap();
    std::fs::write(file.path(), json).unwrap();
    
    let imported = import_json(file.path()).unwrap();
    assert_eq!(imported.name, "Empty");
    assert_eq!(imported.units.len(), 0);
}

#[test]
fn test_import_json_with_units() {
    let mut library = Library::new(
        "Test".to_string(),
        "US".to_string(),
        "2003".to_string(),
        "Author".to_string(),
    );
    library.add_unit(toeditor::models::Unit::new("Unit 1".to_string(), "Company".to_string()));
    let json = serde_json::to_string(&library).unwrap();
    let file = NamedTempFile::new().unwrap();
    std::fs::write(file.path(), json).unwrap();
    
    let imported = import_json(file.path()).unwrap();
    assert_eq!(imported.name, "Test");
    assert_eq!(imported.units.len(), 1);
    assert_eq!(imported.units[0].name, "Unit 1");
}

#[test]
fn test_import_json_with_versions_plain_library() {
    let library = Library::new(
        "Test".to_string(),
        "US".to_string(),
        "2003".to_string(),
        "Author".to_string(),
    );
    let json = serde_json::to_string(&library).unwrap();
    let file = NamedTempFile::new().unwrap();
    std::fs::write(file.path(), json).unwrap();
    
    let (imported, versions) = import_json_with_versions(file.path()).unwrap();
    assert_eq!(imported.name, "Test");
    assert!(versions.is_none()); // Plain library has no versions
}

#[test]
fn test_import_json_with_versions_with_versions() {
    let library = Library::new(
        "Test".to_string(),
        "US".to_string(),
        "2003".to_string(),
        "Author".to_string(),
    );
    let export_data = serde_json::json!({
        "library": library,
        "versions": [
            {
                "version": 1,
                "timestamp": 1234567890,
                "description": "Initial version",
                "data": "{}"
            },
            {
                "version": 2,
                "timestamp": 1234567900,
                "description": "Updated version",
                "data": "{}"
            }
        ]
    });
    let json = serde_json::to_string(&export_data).unwrap();
    let file = NamedTempFile::new().unwrap();
    std::fs::write(file.path(), json).unwrap();
    
    let (imported, versions) = import_json_with_versions(file.path()).unwrap();
    assert_eq!(imported.name, "Test");
    assert!(versions.is_some());
    assert_eq!(versions.unwrap().len(), 2);
}

#[test]
fn test_import_json_invalid_file() {
    let file = NamedTempFile::new().unwrap();
    std::fs::write(file.path(), "invalid json").unwrap();
    
    let result = import_json(file.path());
    assert!(result.is_err());
}

#[test]
fn test_import_json_nonexistent_file() {
    let result = import_json(std::path::Path::new("/nonexistent/file.json"));
    assert!(result.is_err());
}
