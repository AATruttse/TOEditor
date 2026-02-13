//! Edge case tests for export functions

use toeditor::export::{export_json, export_csv, export_svg};
use toeditor::models::Library;
use tempfile::NamedTempFile;

#[test]
fn test_export_json_empty_library() {
    let library = Library::new(
        "Empty".to_string(),
        "US".to_string(),
        "2003".to_string(),
        "Author".to_string(),
    );
    let file = NamedTempFile::new().unwrap();
    export_json(&library, file.path()).unwrap();
    
    let content = std::fs::read_to_string(file.path()).unwrap();
    assert!(content.contains("Empty"));
    // JSON might format as "units": [] or "units":[] - check for both
    assert!(content.contains("\"units\":[]") || content.contains("\"units\": []"));
}

#[test]
fn test_export_json_library_with_units() {
    let mut library = Library::new(
        "Test".to_string(),
        "US".to_string(),
        "2003".to_string(),
        "Author".to_string(),
    );
    library.add_unit(toeditor::models::Unit::new("Unit 1".to_string(), "Company".to_string()));
    library.add_unit(toeditor::models::Unit::new("Unit 2".to_string(), "Battalion".to_string()));
    
    let file = NamedTempFile::new().unwrap();
    export_json(&library, file.path()).unwrap();
    
    let content = std::fs::read_to_string(file.path()).unwrap();
    assert!(content.contains("Test"));
    assert!(content.contains("Unit 1"));
    assert!(content.contains("Unit 2"));
}

#[test]
fn test_export_csv_empty_library() {
    let library = Library::new(
        "Empty".to_string(),
        "US".to_string(),
        "2003".to_string(),
        "Author".to_string(),
    );
    let file = NamedTempFile::new().unwrap();
    export_csv(&library, file.path()).unwrap();
    
    let content = std::fs::read_to_string(file.path()).unwrap();
    assert!(content.contains("Empty"));
    assert!(content.contains("0")); // Zero units
}

#[test]
fn test_export_csv_library_with_units() {
    let mut library = Library::new(
        "Test".to_string(),
        "US".to_string(),
        "2003".to_string(),
        "Author".to_string(),
    );
    library.add_unit(toeditor::models::Unit::new("Unit 1".to_string(), "Company".to_string()));
    
    let file = NamedTempFile::new().unwrap();
    export_csv(&library, file.path()).unwrap();
    
    let content = std::fs::read_to_string(file.path()).unwrap();
    assert!(content.contains("Test"));
    assert!(content.contains("1")); // One unit
}

#[test]
fn test_export_svg_empty_library() {
    let library = Library::new(
        "Empty".to_string(),
        "US".to_string(),
        "2003".to_string(),
        "Author".to_string(),
    );
    let file = NamedTempFile::new().unwrap();
    export_svg(&library, file.path()).unwrap();
    
    let content = std::fs::read_to_string(file.path()).unwrap();
    assert!(content.contains("Empty"));
    assert!(content.contains("svg"));
}

#[test]
fn test_export_svg_library_with_special_chars() {
    let library = Library::new(
        "Test & <Library>".to_string(),
        "US".to_string(),
        "2003".to_string(),
        "Author".to_string(),
    );
    let file = NamedTempFile::new().unwrap();
    export_svg(&library, file.path()).unwrap();
    
    let content = std::fs::read_to_string(file.path()).unwrap();
    assert!(content.contains("Test & <Library>"));
    assert!(content.contains("svg"));
}
