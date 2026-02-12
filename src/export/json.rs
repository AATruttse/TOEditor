//! JSON export functionality

use anyhow::Result;
use crate::models::{Library, Snapshot};
use crate::db::repositories::VersionRepo;
use std::path::Path;

/// Export options for library export
#[derive(Debug, Clone)]
pub enum ExportMode {
    /// Export only the current/latest version
    LatestOnly,
    /// Export library with all version snapshots
    WithAllVersions,
}

/// Export library to JSON file
pub fn export_json(library: &Library, path: &Path) -> Result<()> {
    export_json_with_mode(library, path, ExportMode::LatestOnly, None)
}

/// Export library to JSON file with version control support
pub fn export_json_with_mode(
    library: &Library,
    path: &Path,
    mode: ExportMode,
    version_repo: Option<&VersionRepo>,
) -> Result<()> {
    match mode {
        ExportMode::LatestOnly => {
            let json = serde_json::to_string_pretty(library)?;
            std::fs::write(path, json)?;
        }
        ExportMode::WithAllVersions => {
            let mut export_data = serde_json::json!({
                "library": library,
                "versions": serde_json::Value::Null,
            });
            
            if let Some(repo) = version_repo {
                if let Some(lib_id) = library.id {
                    let snapshots = repo.list_by_library(lib_id)?;
                    let versions: Vec<_> = snapshots.iter().map(|s| {
                        serde_json::json!({
                            "version": s.version,
                            "timestamp": s.timestamp,
                            "description": s.description,
                            "data": s.data,
                        })
                    }).collect();
                    export_data["versions"] = serde_json::json!(versions);
                }
            }
            
            let json = serde_json::to_string_pretty(&export_data)?;
            std::fs::write(path, json)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Library;
    use tempfile::NamedTempFile;

    #[test]
    fn test_export_json() {
        let library = Library::new(
            "Test".to_string(),
            "US".to_string(),
            "2003".to_string(),
            "Author".to_string(),
        );
        let file = NamedTempFile::new().unwrap();
        export_json(&library, file.path()).unwrap();
        
        let content = std::fs::read_to_string(file.path()).unwrap();
        assert!(content.contains("Test"));
        assert!(content.contains("US"));
        assert!(content.contains("Author"));
    }

    #[test]
    fn test_export_json_with_versions() {
        use crate::db::Database;
        use crate::db::repositories::{LibraryRepo, VersionRepo};
        
        let db = Database::open_in_memory().unwrap();
        let mut library = Library::new(
            "Test".to_string(),
            "US".to_string(),
            "2003".to_string(),
            "Author".to_string(),
        );
        let lib_repo = LibraryRepo::new(db.conn());
        lib_repo.create(&mut library).unwrap();
        
        let version_repo = VersionRepo::new(db.conn());
        let mut snapshot = Snapshot::new(
            library.id.unwrap(),
            1,
            serde_json::to_string(&library).unwrap(),
        );
        version_repo.create(&mut snapshot).unwrap();
        
        let file = NamedTempFile::new().unwrap();
        export_json_with_mode(
            &library,
            file.path(),
            crate::export::json::ExportMode::WithAllVersions,
            Some(&version_repo),
        ).unwrap();
        
        let content = std::fs::read_to_string(file.path()).unwrap();
        assert!(content.contains("Test"));
        assert!(content.contains("versions"));
    }
}
