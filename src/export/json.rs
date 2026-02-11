//! JSON export functionality

use anyhow::Result;
use crate::models::Library;
use std::path::Path;

/// Export library to JSON file
pub fn export_json(library: &Library, path: &Path) -> Result<()> {
    let json = serde_json::to_string_pretty(library)?;
    std::fs::write(path, json)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Library;
    use tempfile::NamedTempFile;

    #[test]
    fn test_export_json() {
        let library = Library::new("Test".to_string(), "US".to_string(), "2003".to_string());
        let file = NamedTempFile::new().unwrap();
        export_json(&library, file.path()).unwrap();
        
        let content = std::fs::read_to_string(file.path()).unwrap();
        assert!(content.contains("Test"));
        assert!(content.contains("US"));
    }
}
