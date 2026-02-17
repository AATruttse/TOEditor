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

/// Calculate diff between two library snapshots.
///
/// Compares the JSON `data` fields and produces a human-readable summary of
/// what changed between the old and new snapshots. If the data is not valid
/// JSON, falls back to a string comparison.
pub fn diff_snapshots(old: &Snapshot, new: &Snapshot) -> String {
    let mut changes = Vec::new();

    changes.push(format!(
        "Version {} -> {}",
        old.version, new.version
    ));

    if old.data == new.data {
        changes.push("No data changes.".to_string());
        return changes.join("\n");
    }

    // Try to parse as Library JSON and compare key fields
    let old_val: Result<serde_json::Value, _> = serde_json::from_str(&old.data);
    let new_val: Result<serde_json::Value, _> = serde_json::from_str(&new.data);

    match (old_val, new_val) {
        (Ok(old_v), Ok(new_v)) => {
            diff_json_values("", &old_v, &new_v, &mut changes);
            if changes.len() == 1 {
                changes.push("Data changed but no structural differences detected.".to_string());
            }
        }
        _ => {
            changes.push("Data format changed (could not parse as JSON).".to_string());
        }
    }

    changes.join("\n")
}

/// Recursively compare two JSON values and report differences.
fn diff_json_values(
    path: &str,
    old: &serde_json::Value,
    new: &serde_json::Value,
    changes: &mut Vec<String>,
) {
    use serde_json::Value;

    if old == new {
        return;
    }

    match (old, new) {
        (Value::Object(old_map), Value::Object(new_map)) => {
            for (key, old_val) in old_map {
                let field_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", path, key)
                };
                match new_map.get(key) {
                    Some(new_val) => diff_json_values(&field_path, old_val, new_val, changes),
                    None => changes.push(format!("Removed: {}", field_path)),
                }
            }
            for key in new_map.keys() {
                if !old_map.contains_key(key) {
                    let field_path = if path.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", path, key)
                    };
                    changes.push(format!("Added: {}", field_path));
                }
            }
        }
        (Value::Array(old_arr), Value::Array(new_arr)) => {
            if old_arr.len() != new_arr.len() {
                changes.push(format!(
                    "Changed: {} (array length {} -> {})",
                    path,
                    old_arr.len(),
                    new_arr.len()
                ));
            }
            let min_len = old_arr.len().min(new_arr.len());
            for i in 0..min_len {
                let item_path = format!("{}[{}]", path, i);
                diff_json_values(&item_path, &old_arr[i], &new_arr[i], changes);
            }
        }
        _ => {
            changes.push(format!("Changed: {} ({} -> {})", path, old, new));
        }
    }
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
    fn test_diff_snapshots_identical() {
        let s1 = Snapshot::new(1, 1, "{\"name\":\"A\"}".to_string());
        let s2 = Snapshot::new(1, 2, "{\"name\":\"A\"}".to_string());
        let diff = diff_snapshots(&s1, &s2);
        assert!(diff.contains("No data changes"));
    }

    #[test]
    fn test_diff_snapshots_field_changed() {
        let s1 = Snapshot::new(1, 1, "{\"name\":\"A\"}".to_string());
        let s2 = Snapshot::new(1, 2, "{\"name\":\"B\"}".to_string());
        let diff = diff_snapshots(&s1, &s2);
        assert!(diff.contains("Changed: name"));
        assert!(diff.contains("Version 1 -> 2"));
    }

    #[test]
    fn test_diff_snapshots_field_added() {
        let s1 = Snapshot::new(1, 1, "{\"name\":\"A\"}".to_string());
        let s2 = Snapshot::new(1, 2, "{\"name\":\"A\",\"country\":\"US\"}".to_string());
        let diff = diff_snapshots(&s1, &s2);
        assert!(diff.contains("Added: country"));
    }

    #[test]
    fn test_diff_snapshots_field_removed() {
        let s1 = Snapshot::new(1, 1, "{\"name\":\"A\",\"era\":\"2020\"}".to_string());
        let s2 = Snapshot::new(1, 2, "{\"name\":\"A\"}".to_string());
        let diff = diff_snapshots(&s1, &s2);
        assert!(diff.contains("Removed: era"));
    }

    #[test]
    fn test_diff_snapshots_array_length_change() {
        let s1 = Snapshot::new(1, 1, "{\"units\":[1,2]}".to_string());
        let s2 = Snapshot::new(1, 2, "{\"units\":[1,2,3]}".to_string());
        let diff = diff_snapshots(&s1, &s2);
        assert!(diff.contains("array length 2 -> 3"));
    }

    #[test]
    fn test_diff_snapshots_invalid_json() {
        let s1 = Snapshot::new(1, 1, "not json".to_string());
        let s2 = Snapshot::new(1, 2, "also not json".to_string());
        let diff = diff_snapshots(&s1, &s2);
        assert!(diff.contains("could not parse as JSON"));
    }

    #[test]
    fn test_snapshot_serialization() {
        let snapshot = Snapshot::with_description(1, 1, "data".to_string(), "test".to_string());
        let json = serde_json::to_string(&snapshot).unwrap();
        let deserialized: Snapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(snapshot, deserialized);
    }
}
