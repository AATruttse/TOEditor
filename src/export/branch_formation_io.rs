//! Export/import and copy for branches and formation levels (per-library data).

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use crate::models::{Branch, CustomFormationLevel};
use crate::db::repositories::{BranchRepo, FormationLevelRepo};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchExport {
    pub name_ru: String,
    pub name_en: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormationLevelExport {
    pub name_ru: String,
    pub name_en: String,
    pub standard_level_ordinal: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BranchesFile {
    pub branches: Vec<BranchExport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FormationLevelsFile {
    pub formation_levels: Vec<FormationLevelExport>,
}

/// Export branches to a JSON file.
pub fn export_branches_to_path(path: &Path, branches: &[Branch]) -> Result<()> {
    let data: Vec<BranchExport> = branches
        .iter()
        .map(|b| BranchExport {
            name_ru: b.name_ru.clone(),
            name_en: b.name_en.clone(),
        })
        .collect();
    let file = BranchesFile { branches: data };
    let json = serde_json::to_string_pretty(&file)?;
    std::fs::write(path, json)?;
    Ok(())
}

/// Import branches from a JSON file. Returns the list (without library_id); caller inserts into DB.
pub fn import_branches_from_path(path: &Path) -> Result<Vec<BranchExport>> {
    let json = std::fs::read_to_string(path)?;
    let file: BranchesFile = serde_json::from_str(&json)?;
    Ok(file.branches)
}

/// Export formation levels to a JSON file.
pub fn export_formation_levels_to_path(path: &Path, levels: &[CustomFormationLevel]) -> Result<()> {
    let data: Vec<FormationLevelExport> = levels
        .iter()
        .map(|l| FormationLevelExport {
            name_ru: l.name_ru.clone(),
            name_en: l.name_en.clone(),
            standard_level_ordinal: l.standard_level_ordinal,
        })
        .collect();
    let file = FormationLevelsFile {
        formation_levels: data,
    };
    let json = serde_json::to_string_pretty(&file)?;
    std::fs::write(path, json)?;
    Ok(())
}

/// Import formation levels from a JSON file.
pub fn import_formation_levels_from_path(path: &Path) -> Result<Vec<FormationLevelExport>> {
    let json = std::fs::read_to_string(path)?;
    let file: FormationLevelsFile = serde_json::from_str(&json)?;
    Ok(file.formation_levels)
}

/// Copy all branches from source library to target library (replaces target's branches).
pub fn copy_branches_between_libraries(
    branch_repo: &BranchRepo,
    source_library_id: i64,
    target_library_id: i64,
) -> Result<()> {
    let branches = branch_repo.list_by_library(source_library_id)?;
    branch_repo.delete_by_library(target_library_id)?;
    for mut b in branches {
        b.id = None;
        b.library_id = target_library_id;
        branch_repo.create(&mut b)?;
    }
    Ok(())
}

/// Copy all formation levels from source library to target library (replaces target's).
pub fn copy_formation_levels_between_libraries(
    level_repo: &FormationLevelRepo,
    source_library_id: i64,
    target_library_id: i64,
) -> Result<()> {
    let levels = level_repo.list_by_library(source_library_id)?;
    level_repo.delete_by_library(target_library_id)?;
    for mut l in levels {
        l.id = None;
        l.library_id = target_library_id;
        level_repo.create(&mut l)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::db::repositories::{BranchRepo, FormationLevelRepo, LibraryRepo};
    use crate::models::{CustomFormationLevel, Library};
    use tempfile::NamedTempFile;

    #[test]
    fn test_export_import_branches_roundtrip() {
        let db = Database::open_in_memory().unwrap();
        let lib_repo = LibraryRepo::new(db.conn());
        let mut library = Library::new(
            "Test".to_string(),
            "US".to_string(),
            "2003".to_string(),
            "Author".to_string(),
        );
        lib_repo.create(&mut library).unwrap();
        let lib_id = library.id.unwrap();
        let branches = vec![
            Branch::new(lib_id, "Пехота".to_string(), "Infantry".to_string()),
            Branch::new(lib_id, "Танкисты".to_string(), "Armor".to_string()),
        ];
        let path = NamedTempFile::new().unwrap().into_temp_path();
        export_branches_to_path(path.as_ref(), &branches).unwrap();
        let imported = import_branches_from_path(path.as_ref()).unwrap();
        assert_eq!(imported.len(), 2);
        assert_eq!(imported[0].name_ru, "Пехота");
        assert_eq!(imported[0].name_en, "Infantry");
        assert_eq!(imported[1].name_en, "Armor");
    }

    #[test]
    fn test_export_import_formation_levels_roundtrip() {
        let path = NamedTempFile::new().unwrap().into_temp_path();
        let levels = vec![
            CustomFormationLevel::new(1, "взвод".to_string(), "platoon".to_string(), 3),
            CustomFormationLevel::new(1, "рота".to_string(), "company".to_string(), 4),
        ];
        export_formation_levels_to_path(path.as_ref(), &levels).unwrap();
        let imported = import_formation_levels_from_path(path.as_ref()).unwrap();
        assert_eq!(imported.len(), 2);
        assert_eq!(imported[0].name_ru, "взвод");
        assert_eq!(imported[0].standard_level_ordinal, 3);
        assert_eq!(imported[1].name_en, "company");
    }

    #[test]
    fn test_copy_branches_between_libraries() {
        let db = Database::open_in_memory().unwrap();
        let lib_repo = LibraryRepo::new(db.conn());
        let branch_repo = BranchRepo::new(db.conn());
        let mut lib1 = Library::new("Lib1".to_string(), "US".to_string(), "2003".to_string(), "A".to_string());
        let mut lib2 = Library::new("Lib2".to_string(), "RU".to_string(), "2020".to_string(), "B".to_string());
        lib_repo.create(&mut lib1).unwrap();
        lib_repo.create(&mut lib2).unwrap();
        let id1 = lib1.id.unwrap();
        let id2 = lib2.id.unwrap();
        let mut b1 = Branch::new(id1, "Пехота".to_string(), "Infantry".to_string());
        branch_repo.create(&mut b1).unwrap();
        copy_branches_between_libraries(&branch_repo, id1, id2).unwrap();
        let target_branches = branch_repo.list_by_library(id2).unwrap();
        assert_eq!(target_branches.len(), 1);
        assert_eq!(target_branches[0].name_en, "Infantry");
        assert_eq!(target_branches[0].library_id, id2);
    }

    #[test]
    fn test_copy_formation_levels_between_libraries() {
        let db = Database::open_in_memory().unwrap();
        let lib_repo = LibraryRepo::new(db.conn());
        let level_repo = FormationLevelRepo::new(db.conn());
        let mut lib1 = Library::new("Lib1".to_string(), "US".to_string(), "2003".to_string(), "A".to_string());
        let mut lib2 = Library::new("Lib2".to_string(), "RU".to_string(), "2020".to_string(), "B".to_string());
        lib_repo.create(&mut lib1).unwrap();
        lib_repo.create(&mut lib2).unwrap();
        let id1 = lib1.id.unwrap();
        let id2 = lib2.id.unwrap();
        let mut l1 = CustomFormationLevel::new(id1, "взвод".to_string(), "platoon".to_string(), 3);
        level_repo.create(&mut l1).unwrap();
        copy_formation_levels_between_libraries(&level_repo, id1, id2).unwrap();
        let target_levels = level_repo.list_by_library(id2).unwrap();
        assert_eq!(target_levels.len(), 1);
        assert_eq!(target_levels[0].name_en, "platoon");
        assert_eq!(target_levels[0].library_id, id2);
    }
}
