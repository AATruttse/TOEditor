//! Version control and snapshot management

use serde::{Deserialize, Serialize};
use crate::models::Library;

/// Trait for versioned entities
pub trait Versioned {
    /// Get current version number
    fn version(&self) -> i64;
    
    /// Increment version
    fn increment_version(&mut self);
}

/// Snapshot of a library at a specific version
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Snapshot {
    /// Snapshot ID
    pub id: Option<i64>,
    /// Library ID
    pub library_id: i64,
    /// Version number
    pub version: i64,
    /// Timestamp (Unix epoch)
    pub timestamp: i64,
    /// Serialized library data
    pub data: String,
    /// Optional description/comment
    pub description: Option<String>,
}

impl Snapshot {
    /// Create a new snapshot
    pub fn new(library_id: i64, version: i64, data: String) -> Self {
        Self {
            id: None,
            library_id,
            version,
            timestamp: chrono::Utc::now().timestamp(),
            data,
            description: None,
        }
    }

    /// Create snapshot with description
    pub fn with_description(
        library_id: i64,
        version: i64,
        data: String,
        description: String,
    ) -> Self {
        Self {
            id: None,
            library_id,
            version,
            timestamp: chrono::Utc::now().timestamp(),
            data,
            description: Some(description),
        }
    }
}

/// Calculate diff between two library snapshots
pub fn diff_snapshots(_old: &Snapshot, _new: &Snapshot) -> String {
    // TODO: Implement diff calculation
    "Diff not yet implemented".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_creation() {
        let snapshot = Snapshot::new(1, 1, "{}".to_string());
        assert_eq!(snapshot.library_id, 1);
        assert_eq!(snapshot.version, 1);
        assert!(snapshot.timestamp > 0);
    }
}
