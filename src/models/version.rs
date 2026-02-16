//! Version control and snapshot management

use serde::{Deserialize, Serialize};

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
        assert_eq!(snapshot.id, None);
        assert_eq!(snapshot.description, None);
        assert_eq!(snapshot.data, "{}");
    }

    #[test]
    fn test_snapshot_with_description() {
        let snapshot = Snapshot::with_description(1, 2, "data".to_string(), "desc".to_string());
        assert_eq!(snapshot.library_id, 1);
        assert_eq!(snapshot.version, 2);
        assert_eq!(snapshot.data, "data");
        assert_eq!(snapshot.description, Some("desc".to_string()));
        assert!(snapshot.timestamp > 0);
        assert_eq!(snapshot.id, None);
    }

    #[test]
    fn test_diff_snapshots_stub() {
        let s1 = Snapshot::new(1, 1, "{}".to_string());
        let s2 = Snapshot::new(1, 2, "{\"a\":1}".to_string());
        let diff = diff_snapshots(&s1, &s2);
        assert!(!diff.is_empty());
    }

    #[test]
    fn test_snapshot_serialization() {
        let snapshot = Snapshot::with_description(1, 1, "data".to_string(), "test".to_string());
        let json = serde_json::to_string(&snapshot).unwrap();
        let deserialized: Snapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(snapshot, deserialized);
    }
}
