//! Tests for LibraryRepo delete functionality

use toeditor::db::Database;
use toeditor::db::repositories::{LibraryRepo, UnitRepo};
use toeditor::models::{Library, Unit};

#[test]
fn test_delete_library() {
    let db = Database::open_in_memory().unwrap();
    let repo = LibraryRepo::new(db.conn());
    
    let mut library = Library::new(
        "Test".to_string(),
        "US".to_string(),
        "2003".to_string(),
        "Author".to_string(),
    );
    repo.create(&mut library).unwrap();
    let lib_id = library.id.unwrap();
    
    // Verify library exists
    let retrieved = repo.get_by_id(lib_id).unwrap();
    assert!(retrieved.is_some());
    
    // Delete library
    repo.delete(lib_id).unwrap();
    
    // Verify library is gone
    let retrieved = repo.get_by_id(lib_id).unwrap();
    assert!(retrieved.is_none());
}

#[test]
fn test_delete_library_cascades_to_units() {
    let db = Database::open_in_memory().unwrap();
    let lib_repo = LibraryRepo::new(db.conn());
    let unit_repo = UnitRepo::new(db.conn());
    
    let mut library = Library::new(
        "Test".to_string(),
        "US".to_string(),
        "2003".to_string(),
        "Author".to_string(),
    );
    lib_repo.create(&mut library).unwrap();
    let lib_id = library.id.unwrap();
    
    // Create units
    let mut unit1 = Unit::new("Unit 1".to_string(), "Company".to_string());
    unit_repo.create(lib_id, &mut unit1).unwrap();
    let unit1_id = unit1.id.unwrap();
    
    let mut unit2 = Unit::new("Unit 2".to_string(), "Battalion".to_string());
    unit_repo.create(lib_id, &mut unit2).unwrap();
    let unit2_id = unit2.id.unwrap();
    
    // Verify units exist
    assert!(unit_repo.get_by_id(unit1_id).unwrap().is_some());
    assert!(unit_repo.get_by_id(unit2_id).unwrap().is_some());
    
    // Delete library (should cascade delete units)
    lib_repo.delete(lib_id).unwrap();
    
    // Verify units are gone (cascade delete)
    assert!(unit_repo.get_by_id(unit1_id).unwrap().is_none());
    assert!(unit_repo.get_by_id(unit2_id).unwrap().is_none());
}

#[test]
fn test_delete_nonexistent_library() {
    let db = Database::open_in_memory().unwrap();
    let repo = LibraryRepo::new(db.conn());
    
    // Should not panic, just do nothing
    let result = repo.delete(999);
    assert!(result.is_ok());
}
