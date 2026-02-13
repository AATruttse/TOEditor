//! Comprehensive tests for VersionRepo

use toeditor::db::Database;
use toeditor::db::repositories::{LibraryRepo, VersionRepo};
use toeditor::models::{Library, Snapshot};

#[test]
fn test_get_latest_snapshot() {
    let db = Database::open_in_memory().unwrap();
    let lib_repo = LibraryRepo::new(db.conn());
    let mut library = Library::new(
        "Test".to_string(),
        "US".to_string(),
        "2003".to_string(),
        "Author".to_string(),
    );
    lib_repo.create(&mut library).unwrap();
    
    let repo = VersionRepo::new(db.conn());
    
    // Create multiple snapshots
    let mut snapshot1 = Snapshot::new(library.id.unwrap(), 1, r#"{"name":"Test","version":1}"#.to_string());
    repo.create(&mut snapshot1).unwrap();
    
    let mut snapshot2 = Snapshot::new(library.id.unwrap(), 2, r#"{"name":"Test","version":2}"#.to_string());
    repo.create(&mut snapshot2).unwrap();
    
    let mut snapshot3 = Snapshot::new(library.id.unwrap(), 3, r#"{"name":"Test","version":3}"#.to_string());
    repo.create(&mut snapshot3).unwrap();
    
    // Get latest snapshot
    let latest = repo.get_latest(library.id.unwrap()).unwrap();
    assert!(latest.is_some());
    let snap = latest.unwrap();
    assert_eq!(snap.version, 3);
    assert_eq!(snap.library_id, library.id.unwrap());
}

#[test]
fn test_get_latest_nonexistent() {
    let db = Database::open_in_memory().unwrap();
    let repo = VersionRepo::new(db.conn());
    
    let latest = repo.get_latest(999).unwrap();
    assert!(latest.is_none());
}

#[test]
fn test_list_by_library() {
    let db = Database::open_in_memory().unwrap();
    let lib_repo = LibraryRepo::new(db.conn());
    
    let mut lib1 = Library::new(
        "Library 1".to_string(),
        "US".to_string(),
        "2003".to_string(),
        "Author".to_string(),
    );
    lib_repo.create(&mut lib1).unwrap();
    
    let mut lib2 = Library::new(
        "Library 2".to_string(),
        "RU".to_string(),
        "2020".to_string(),
        "Author".to_string(),
    );
    lib_repo.create(&mut lib2).unwrap();
    
    let repo = VersionRepo::new(db.conn());
    
    // Create snapshots for library 1
    let mut snap1 = Snapshot::new(lib1.id.unwrap(), 1, "{}".to_string());
    repo.create(&mut snap1).unwrap();
    
    let mut snap2 = Snapshot::new(lib1.id.unwrap(), 2, "{}".to_string());
    repo.create(&mut snap2).unwrap();
    
    // Create snapshot for library 2
    let mut snap3 = Snapshot::new(lib2.id.unwrap(), 1, "{}".to_string());
    repo.create(&mut snap3).unwrap();
    
    // List snapshots for library 1
    let snapshots = repo.list_by_library(lib1.id.unwrap()).unwrap();
    assert_eq!(snapshots.len(), 2);
    assert_eq!(snapshots[0].version, 2); // Should be ordered DESC
    assert_eq!(snapshots[1].version, 1);
    
    // List snapshots for library 2
    let snapshots = repo.list_by_library(lib2.id.unwrap()).unwrap();
    assert_eq!(snapshots.len(), 1);
    assert_eq!(snapshots[0].version, 1);
}

#[test]
fn test_list_by_library_empty() {
    let db = Database::open_in_memory().unwrap();
    let lib_repo = LibraryRepo::new(db.conn());
    let mut library = Library::new(
        "Test".to_string(),
        "US".to_string(),
        "2003".to_string(),
        "Author".to_string(),
    );
    lib_repo.create(&mut library).unwrap();
    
    let repo = VersionRepo::new(db.conn());
    let snapshots = repo.list_by_library(library.id.unwrap()).unwrap();
    assert_eq!(snapshots.len(), 0);
}

#[test]
fn test_snapshot_with_description() {
    let db = Database::open_in_memory().unwrap();
    let lib_repo = LibraryRepo::new(db.conn());
    let mut library = Library::new(
        "Test".to_string(),
        "US".to_string(),
        "2003".to_string(),
        "Author".to_string(),
    );
    lib_repo.create(&mut library).unwrap();
    
    let repo = VersionRepo::new(db.conn());
    let mut snapshot = Snapshot::with_description(
        library.id.unwrap(),
        1,
        "{}".to_string(),
        "Initial version".to_string(),
    );
    repo.create(&mut snapshot).unwrap();
    
    let retrieved = repo.get_latest(library.id.unwrap()).unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().description, Some("Initial version".to_string()));
}
