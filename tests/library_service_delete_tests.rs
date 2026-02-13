//! Tests for LibraryService delete functionality

use toeditor::db::Database;
use toeditor::services::LibraryService;
use toeditor::models::Library;

#[test]
fn test_delete_library_service() {
    let db = Database::open_in_memory().unwrap();
    let service = LibraryService::new(db.conn());
    
    let library = Library::new(
        "Test".to_string(),
        "US".to_string(),
        "2003".to_string(),
        "Author".to_string(),
    );
    let created = service.create_library(library).unwrap();
    let lib_id = created.id.unwrap();
    
    // Verify library exists
    let retrieved = service.get_library(lib_id).unwrap();
    assert!(retrieved.is_some());
    
    // Delete library
    service.delete_library(lib_id).unwrap();
    
    // Verify library is gone
    let retrieved = service.get_library(lib_id).unwrap();
    assert!(retrieved.is_none());
    
    // Verify snapshots are also gone (cascade delete)
    let snapshots = service.get_library_versions(lib_id).unwrap();
    assert_eq!(snapshots.len(), 0);
}

#[test]
fn test_delete_library_with_versions() {
    let db = Database::open_in_memory().unwrap();
    let service = LibraryService::new(db.conn());
    
    let mut library = Library::new(
        "Test".to_string(),
        "US".to_string(),
        "2003".to_string(),
        "Author".to_string(),
    );
    library = service.create_library(library).unwrap();
    let lib_id = library.id.unwrap();
    
    // Create multiple versions
    library.name = "Updated".to_string();
    let _ = service.save_library(library, true).unwrap();
    
    let mut library2 = service.get_library(lib_id).unwrap().unwrap();
    library2.name = "Updated Again".to_string();
    let _ = service.save_library(library2, true).unwrap();
    
    // Verify versions exist
    let snapshots = service.get_library_versions(lib_id).unwrap();
    assert_eq!(snapshots.len(), 3); // Initial + 2 updates
    
    // Delete library
    service.delete_library(lib_id).unwrap();
    
    // Verify library and all versions are gone
    let retrieved = service.get_library(lib_id).unwrap();
    assert!(retrieved.is_none());
    
    let snapshots = service.get_library_versions(lib_id).unwrap();
    assert_eq!(snapshots.len(), 0);
}
