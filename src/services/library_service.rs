//! Library service for managing libraries with version control

use anyhow::Result;
use rusqlite::Connection;
use crate::models::{Library, Snapshot, default_branches, default_branch_categories};
use crate::db::repositories::{LibraryRepo, VersionRepo, BranchRepo, BranchCategoryRepo};

/// Service for library operations with automatic version management
pub struct LibraryService<'a> {
    library_repo: LibraryRepo<'a>,
    version_repo: VersionRepo<'a>,
    branch_repo: BranchRepo<'a>,
    branch_category_repo: BranchCategoryRepo<'a>,
}

impl<'a> LibraryService<'a> {
    /// Create new library service
    pub fn new(conn: &'a Connection) -> Self {
        Self {
            library_repo: LibraryRepo::new(conn),
            version_repo: VersionRepo::new(conn),
            branch_repo: BranchRepo::new(conn),
            branch_category_repo: BranchCategoryRepo::new(conn),
        }
    }

    /// Create a new library, initial snapshot, default branch categories, and default branches
    pub fn create_library(&self, mut library: Library) -> Result<Library> {
        self.library_repo.create(&mut library)?;
        
        if let Some(lib_id) = library.id {
            let data = serde_json::to_string(&library)?;
            let mut snapshot = Snapshot::new(lib_id, library.version, data);
            self.version_repo.create(&mut snapshot)?;

            let mut category_ids = Vec::new();
            for mut cat in default_branch_categories(lib_id) {
                self.branch_category_repo.create(&mut cat)?;
                if let Some(id) = cat.id {
                    category_ids.push(id);
                }
            }
            for (mut branch, cat_idx) in default_branches(lib_id) {
                branch.category_id = category_ids.get(cat_idx).copied();
                self.branch_repo.create(&mut branch)?;
            }
        }
        
        Ok(library)
    }

    /// Save library (update if exists, create if new) and create snapshot
    pub fn save_library(&self, mut library: Library, create_snapshot: bool) -> Result<Library> {
        if library.id.is_none() {
            // New library
            self.create_library(library)
        } else {
            // Update existing library
            if create_snapshot {
                library.increment_version();
            }
            self.library_repo.update(&library)?;
            
            // Create snapshot if requested
            if create_snapshot {
                if let Some(lib_id) = library.id {
                    let data = serde_json::to_string(&library)?;
                    let mut snapshot = Snapshot::new(lib_id, library.version, data);
                    self.version_repo.create(&mut snapshot)?;
                }
            }
            
            Ok(library)
        }
    }

    /// Get library by ID
    pub fn get_library(&self, id: i64) -> Result<Option<Library>> {
        self.library_repo.get_by_id(id)
    }

    /// List all libraries
    pub fn list_libraries(&self) -> Result<Vec<Library>> {
        self.library_repo.list_all()
    }

    /// Search libraries
    pub fn search_libraries(&self, query: &str) -> Result<Vec<Library>> {
        self.library_repo.search(query)
    }

    /// Delete library (and all its versions)
    pub fn delete_library(&self, id: i64) -> Result<()> {
        self.library_repo.delete(id)
    }

    /// Get all versions for a library
    pub fn get_library_versions(&self, library_id: i64) -> Result<Vec<Snapshot>> {
        self.version_repo.list_by_library(library_id)
    }

    /// Get latest version snapshot for a library
    pub fn get_latest_version(&self, library_id: i64) -> Result<Option<Snapshot>> {
        self.version_repo.get_latest(library_id)
    }

    /// Restore library from a specific version snapshot
    pub fn restore_from_version(&self, library_id: i64, version: i64) -> Result<Option<Library>> {
        let snapshots = self.version_repo.list_by_library(library_id)?;
        let snapshot = snapshots.iter().find(|s| s.version == version);
        
        if let Some(snap) = snapshot {
            let library: Library = serde_json::from_str(&snap.data)?;
            Ok(Some(library))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::db::repositories::BranchRepo;

    #[test]
    fn test_create_library_with_snapshot() {
        let db = Database::open_in_memory().unwrap();
        let service = LibraryService::new(db.conn());
        
        let library = Library::new(
            "Test".to_string(),
            "US".to_string(),
            "2003".to_string(),
            "Author".to_string(),
        );
        
        let created = service.create_library(library).unwrap();
        assert!(created.id.is_some());
        assert_eq!(created.version, 1);
        
        // Check snapshot was created
        let snapshots = service.get_library_versions(created.id.unwrap()).unwrap();
        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].version, 1);
    }

    #[test]
    fn test_save_library_with_snapshot() {
        let db = Database::open_in_memory().unwrap();
        let service = LibraryService::new(db.conn());
        
        let mut library = Library::new(
            "Test".to_string(),
            "US".to_string(),
            "2003".to_string(),
            "Author".to_string(),
        );
        
        // Create initial version
        library = service.create_library(library).unwrap();
        assert_eq!(library.version, 1);
        
        // Update and save with snapshot
        library.name = "Updated Test".to_string();
        library = service.save_library(library, true).unwrap();
        assert_eq!(library.version, 2);
        
        // Check both snapshots exist
        let snapshots = service.get_library_versions(library.id.unwrap()).unwrap();
        assert_eq!(snapshots.len(), 2);
    }

    #[test]
    fn test_search_libraries() {
        let db = Database::open_in_memory().unwrap();
        let service = LibraryService::new(db.conn());
        
        let lib1 = Library::new(
            "US Army 2003".to_string(),
            "US".to_string(),
            "2003".to_string(),
            "John Doe".to_string(),
        );
        let lib2 = Library::new(
            "Russian Forces".to_string(),
            "RU".to_string(),
            "2020".to_string(),
            "Jane Smith".to_string(),
        );
        
        service.create_library(lib1).unwrap();
        service.create_library(lib2).unwrap();
        
        // Search by name (more specific)
        let results = service.search_libraries("Army").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "US Army 2003");
        
        // Search by author
        let results = service.search_libraries("John").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].author, "John Doe");
    }

    #[test]
    fn test_create_library_creates_default_branches() {
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
        let branch_repo = BranchRepo::new(db.conn());
        let branches = branch_repo.list_by_library(lib_id).unwrap();
        assert!(!branches.is_empty());
        assert!(branches.iter().any(|b| b.name_en == "Infantry"));
    }

    #[test]
    fn test_restore_from_version() {
        let db = Database::open_in_memory().unwrap();
        let service = LibraryService::new(db.conn());
        
        let mut library = Library::new(
            "Test".to_string(),
            "US".to_string(),
            "2003".to_string(),
            "Author".to_string(),
        );
        library = service.create_library(library).unwrap();
        
        // Update to version 2
        library.name = "Updated".to_string();
        library = service.save_library(library, true).unwrap();
        
        // Restore from version 1
        let restored = service.restore_from_version(library.id.unwrap(), 1).unwrap();
        assert!(restored.is_some());
        assert_eq!(restored.unwrap().name, "Test");
    }

    #[test]
    fn test_create_library_creates_default_categories() {
        let db = Database::open_in_memory().unwrap();
        let service = LibraryService::new(db.conn());
        let library = Library::new(
            "Test".to_string(), "US".to_string(), "2003".to_string(), "Author".to_string(),
        );
        let created = service.create_library(library).unwrap();
        let lib_id = created.id.unwrap();

        let cat_repo = crate::db::repositories::BranchCategoryRepo::new(db.conn());
        let categories = cat_repo.list_by_library(lib_id).unwrap();
        assert_eq!(categories.len(), 6);
        assert!(categories.iter().any(|c| c.name_en == "Combat"));
        assert!(categories.iter().any(|c| c.name_en == "Artillery"));
        assert!(categories.iter().any(|c| c.name_en == "Air defense"));
        assert!(categories.iter().any(|c| c.name_en == "Army aviation"));
        assert!(categories.iter().any(|c| c.name_en == "Combat support"));
        assert!(categories.iter().any(|c| c.name_en == "Logistics support"));
    }

    #[test]
    fn test_create_library_branches_have_category_ids() {
        let db = Database::open_in_memory().unwrap();
        let service = LibraryService::new(db.conn());
        let library = Library::new(
            "Test".to_string(), "US".to_string(), "2003".to_string(), "Author".to_string(),
        );
        let created = service.create_library(library).unwrap();
        let lib_id = created.id.unwrap();

        let branch_repo = BranchRepo::new(db.conn());
        let branches = branch_repo.list_by_library(lib_id).unwrap();
        // All default branches should have a category_id assigned
        for b in &branches {
            assert!(b.category_id.is_some(),
                "Branch '{}' has no category_id", b.name_en);
        }
    }

    #[test]
    fn test_list_libraries() {
        let db = Database::open_in_memory().unwrap();
        let service = LibraryService::new(db.conn());

        assert!(service.list_libraries().unwrap().is_empty());

        let lib1 = Library::new("Alpha".to_string(), "US".to_string(), "2003".to_string(), "A".to_string());
        let lib2 = Library::new("Bravo".to_string(), "RU".to_string(), "2020".to_string(), "B".to_string());
        service.create_library(lib1).unwrap();
        service.create_library(lib2).unwrap();

        let list = service.list_libraries().unwrap();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_save_library_without_snapshot() {
        let db = Database::open_in_memory().unwrap();
        let service = LibraryService::new(db.conn());

        let mut library = Library::new(
            "Test".to_string(), "US".to_string(), "2003".to_string(), "Author".to_string(),
        );
        library = service.create_library(library).unwrap();
        assert_eq!(library.version, 1);

        // Update without creating a snapshot
        library.name = "Updated".to_string();
        library = service.save_library(library, false).unwrap();
        // Version should NOT be incremented
        assert_eq!(library.version, 1);

        // Only the initial snapshot should exist
        let snapshots = service.get_library_versions(library.id.unwrap()).unwrap();
        assert_eq!(snapshots.len(), 1);
    }

    #[test]
    fn test_restore_from_nonexistent_version() {
        let db = Database::open_in_memory().unwrap();
        let service = LibraryService::new(db.conn());

        let library = Library::new(
            "Test".to_string(), "US".to_string(), "2003".to_string(), "Author".to_string(),
        );
        let created = service.create_library(library).unwrap();

        let result = service.restore_from_version(created.id.unwrap(), 999).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_get_latest_version() {
        let db = Database::open_in_memory().unwrap();
        let service = LibraryService::new(db.conn());

        let mut library = Library::new(
            "Test".to_string(), "US".to_string(), "2003".to_string(), "Author".to_string(),
        );
        library = service.create_library(library).unwrap();

        library.name = "V2".to_string();
        library = service.save_library(library, true).unwrap();

        let latest = service.get_latest_version(library.id.unwrap()).unwrap();
        assert!(latest.is_some());
        assert_eq!(latest.unwrap().version, 2);
    }

    #[test]
    fn test_get_library_nonexistent() {
        let db = Database::open_in_memory().unwrap();
        let service = LibraryService::new(db.conn());
        let result = service.get_library(999).unwrap();
        assert!(result.is_none());
    }
}
