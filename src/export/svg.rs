//! SVG export functionality for organizational charts

use anyhow::Result;
use crate::models::Library;
use std::path::Path;

/// Export library to SVG organizational chart
pub fn export_svg(library: &Library, path: &Path) -> Result<()> {
    // TODO: Implement SVG export using svg crate or plotters
    // For now, create a placeholder SVG
    let svg_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="800" height="600">
  <text x="400" y="300" text-anchor="middle" font-size="24">
    SVG Export - TODO: Implement organizational chart rendering
  </text>
  <text x="400" y="330" text-anchor="middle" font-size="16">
    Library: {library_name}
  </text>
</svg>"#;
    
    let svg_content = svg_content.replace("{library_name}", &library.name);
    std::fs::write(path, svg_content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Library;
    use tempfile::NamedTempFile;

    #[test]
    fn test_export_svg() {
        let library = Library::new("Test".to_string(), "US".to_string(), "2003".to_string());
        let file = NamedTempFile::new().unwrap();
        export_svg(&library, file.path()).unwrap();
        
        let content = std::fs::read_to_string(file.path()).unwrap();
        assert!(content.contains("Test"));
        assert!(content.contains("svg"));
    }
}
