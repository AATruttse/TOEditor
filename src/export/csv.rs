//! CSV export functionality

use anyhow::Result;
use crate::models::Library;
use std::path::Path;

/// Export library to CSV file
pub fn export_csv(library: &Library, path: &Path) -> Result<()> {
    // TODO: Implement CSV export using csv crate
    // For now, create a placeholder file
    let mut csv_content = String::from("Library,Country,Era,Units\n");
    csv_content.push_str(&format!(
        "{},{},{},{}\n",
        library.name,
        library.country,
        library.era,
        library.units.len()
    ));
    std::fs::write(path, csv_content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Library;
    use tempfile::NamedTempFile;

    #[test]
    fn test_export_csv() {
        let library = Library::new(
            "Test".to_string(),
            "US".to_string(),
            "2003".to_string(),
            "Author".to_string(),
        );
        let file = NamedTempFile::new().unwrap();
        export_csv(&library, file.path()).unwrap();
        
        let content = std::fs::read_to_string(file.path()).unwrap();
        assert!(content.contains("Test"));
    }
}
