//! CSV export functionality

use anyhow::Result;
use crate::models::Library;
use std::path::Path;

/// Export library to CSV file.
///
/// Produces a CSV with library metadata header, then one row per unit
/// listing personnel count and equipment summary.
pub fn export_csv(library: &Library, path: &Path) -> Result<()> {
    let mut lines = Vec::new();

    // Header row
    lines.push("Unit,Type,Parent,Personnel,Equipment".to_string());

    fn write_units(
        units: &[crate::models::Unit],
        parent_name: &str,
        lines: &mut Vec<String>,
    ) {
        for unit in units {
            let personnel_count = unit.personnel.len();
            let equipment_summary: Vec<String> = unit
                .equipment
                .iter()
                .map(|e| format!("{}x{}", e.quantity, e.name))
                .collect();
            let eq_str = equipment_summary.join("; ");

            // Escape CSV fields containing commas or quotes
            let name = csv_escape(&unit.name);
            let utype = csv_escape(&unit.unit_type);
            let parent = csv_escape(parent_name);
            let eq_escaped = csv_escape(&eq_str);

            lines.push(format!(
                "{},{},{},{},{}",
                name, utype, parent, personnel_count, eq_escaped
            ));

            write_units(&unit.children, &unit.name, lines);
        }
    }

    write_units(&library.units, "", &mut lines);

    // If no units, still output the library info as a comment line
    if library.units.is_empty() {
        lines.push(format!(
            "# Library: {} | Country: {} | Era: {}",
            library.name, library.country, library.era
        ));
    }

    std::fs::write(path, lines.join("\n"))?;
    Ok(())
}

/// Escape a string for CSV: wrap in quotes if it contains comma, quote, or newline.
fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Library, Unit, Personnel, Equipment};
    use tempfile::NamedTempFile;

    #[test]
    fn test_export_csv_empty_library() {
        let library = Library::new(
            "Test".to_string(),
            "US".to_string(),
            "2003".to_string(),
            "Author".to_string(),
        );
        let file = NamedTempFile::new().unwrap();
        export_csv(&library, file.path()).unwrap();

        let content = std::fs::read_to_string(file.path()).unwrap();
        assert!(content.contains("Unit,Type,Parent,Personnel,Equipment"));
        assert!(content.contains("Test"));
    }

    #[test]
    fn test_export_csv_with_units() {
        let mut library = Library::new(
            "Test".to_string(),
            "US".to_string(),
            "2003".to_string(),
            "Author".to_string(),
        );
        let mut unit = Unit::new("1st Platoon".to_string(), "platoon".to_string());
        unit.personnel.push(Personnel::new("Platoon Leader".to_string()));
        unit.equipment.push(Equipment::new("Rifle".to_string(), 30));
        library.units.push(unit);

        let file = NamedTempFile::new().unwrap();
        export_csv(&library, file.path()).unwrap();

        let content = std::fs::read_to_string(file.path()).unwrap();
        assert!(content.contains("1st Platoon"));
        assert!(content.contains("platoon"));
        assert!(content.contains("30xRifle"));
    }

    #[test]
    fn test_csv_escape_plain() {
        assert_eq!(csv_escape("hello"), "hello");
    }

    #[test]
    fn test_csv_escape_comma() {
        assert_eq!(csv_escape("hello, world"), "\"hello, world\"");
    }

    #[test]
    fn test_csv_escape_quotes() {
        assert_eq!(csv_escape("say \"hi\""), "\"say \"\"hi\"\"\"");
    }
}
