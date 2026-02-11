//! Import functionality for external files

use anyhow::Result;
use crate::models::Library;
use std::path::Path;

/// Import library from JSON file
pub fn import_json(path: &Path) -> Result<Library> {
    let content = std::fs::read_to_string(path)?;
    let library: Library = serde_json::from_str(&content)?;
    Ok(library)
}

/// Import library from YAML file
pub fn import_yaml(path: &Path) -> Result<Library> {
    use yaml_rust::YamlLoader;
    let content = std::fs::read_to_string(path)?;
    let docs = YamlLoader::load_from_str(&content)?;
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
        let library = Library::new("Test".to_string(), "US".to_string(), "2003".to_string());
        let json = serde_json::to_string(&library).unwrap();
        let file = NamedTempFile::new().unwrap();
        std::fs::write(file.path(), json).unwrap();
        
        let imported = import_json(file.path()).unwrap();
        assert_eq!(imported.name, "Test");
    }
}
