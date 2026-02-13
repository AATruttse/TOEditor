//! Import functionality for external files

use anyhow::Result;
use crate::models::Library;
use std::path::Path;

/// Import library from JSON file (supports both single library and library with versions)
pub fn import_json(path: &Path) -> Result<Library> {
    let content = std::fs::read_to_string(path)?;
    
    // Try to parse as library with versions first
    if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
        if data.is_object() && data.get("library").is_some() {
            // It's a library with versions - extract just the library
            let library: Library = serde_json::from_value(data["library"].clone())?;
            return Ok(library);
        }
    }
    
    // Otherwise, parse as plain library
    let library: Library = serde_json::from_str(&content)?;
    Ok(library)
}

/// Import library with versions from JSON file
/// Returns the library and optionally a list of version data
pub fn import_json_with_versions(path: &Path) -> Result<(Library, Option<Vec<serde_json::Value>>)> {
    let content = std::fs::read_to_string(path)?;
    let data: serde_json::Value = serde_json::from_str(&content)?;
    
    if data.is_object() && data.get("library").is_some() {
        let library: Library = serde_json::from_value(data["library"].clone())?;
        let versions = data.get("versions")
            .and_then(|v| v.as_array())
            .map(|v| v.clone());
        Ok((library, versions))
    } else {
        // Plain library without versions
        let library: Library = serde_json::from_value(data)?;
        Ok((library, None))
    }
}

/// Import library from YAML file
pub fn import_yaml(_path: &Path) -> Result<Library> {
    // TODO: Implement YAML parsing to Library
    anyhow::bail!("YAML import not yet implemented")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Library;
    use tempfile::NamedTempFile;

    #[test]
    fn test_import_json() {
        let library = Library::new(
            "Test".to_string(),
            "US".to_string(),
            "2003".to_string(),
            "Author".to_string(),
        );
        let json = serde_json::to_string(&library).unwrap();
        let file = NamedTempFile::new().unwrap();
        std::fs::write(file.path(), json).unwrap();
        
        let imported = import_json(file.path()).unwrap();
        assert_eq!(imported.name, "Test");
        assert_eq!(imported.author, "Author");
    }

    #[test]
    fn test_import_json_with_versions() {
        let library = Library::new(
            "Test".to_string(),
            "US".to_string(),
            "2003".to_string(),
            "Author".to_string(),
        );
        let export_data = serde_json::json!({
            "library": library,
            "versions": [
                {
                    "version": 1,
                    "timestamp": 1234567890,
                    "description": "Initial version",
                    "data": "{}"
                }
            ]
        });
        let json = serde_json::to_string(&export_data).unwrap();
        let file = NamedTempFile::new().unwrap();
        std::fs::write(file.path(), json).unwrap();
        
        let (imported, versions) = import_json_with_versions(file.path()).unwrap();
        assert_eq!(imported.name, "Test");
        assert!(versions.is_some());
        assert_eq!(versions.unwrap().len(), 1);
    }
}
