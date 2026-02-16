//! Repository for custom formation levels (per library).

use anyhow::Result;
use rusqlite::{params, Connection};
use crate::models::CustomFormationLevel;

pub struct FormationLevelRepo<'a> {
    conn: &'a Connection,
}

impl<'a> FormationLevelRepo<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn create(&self, level: &mut CustomFormationLevel) -> Result<()> {
        self.conn.execute(
            "INSERT INTO formation_levels (library_id, name_ru, name_en, standard_level_ordinal)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                level.library_id,
                level.name_ru,
                level.name_en,
                level.standard_level_ordinal,
            ],
        )?;
        level.id = Some(self.conn.last_insert_rowid());
        Ok(())
    }

    pub fn get_by_id(&self, id: i64) -> Result<Option<CustomFormationLevel>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, library_id, name_ru, name_en, standard_level_ordinal
             FROM formation_levels WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], |row| {
            Ok(CustomFormationLevel {
                id: Some(row.get(0)?),
                library_id: row.get(1)?,
                name_ru: row.get(2)?,
                name_en: row.get(3)?,
                standard_level_ordinal: row.get(4)?,
            })
        })?;
        match rows.next() {
            Some(Ok(l)) => Ok(Some(l)),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    pub fn list_by_library(&self, library_id: i64) -> Result<Vec<CustomFormationLevel>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, library_id, name_ru, name_en, standard_level_ordinal
             FROM formation_levels WHERE library_id = ?1 ORDER BY standard_level_ordinal, id",
        )?;
        let rows = stmt.query_map(params![library_id], |row| {
            Ok(CustomFormationLevel {
                id: Some(row.get(0)?),
                library_id: row.get(1)?,
                name_ru: row.get(2)?,
                name_en: row.get(3)?,
                standard_level_ordinal: row.get(4)?,
            })
        })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    pub fn update(&self, level: &CustomFormationLevel) -> Result<()> {
        let id = level.id.ok_or_else(|| anyhow::anyhow!("CustomFormationLevel has no id"))?;
        self.conn.execute(
            "UPDATE formation_levels SET name_ru = ?1, name_en = ?2, standard_level_ordinal = ?3
             WHERE id = ?4",
            params![
                level.name_ru,
                level.name_en,
                level.standard_level_ordinal,
                id,
            ],
        )?;
        Ok(())
    }

    pub fn delete(&self, id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM formation_levels WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Delete all formation levels for a library.
    pub fn delete_by_library(&self, library_id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM formation_levels WHERE library_id = ?1", params![library_id])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::db::repositories::LibraryRepo;
    use crate::models::Library;

    #[test]
    fn test_custom_formation_level_crud() {
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

        let repo = FormationLevelRepo::new(db.conn());
        let mut level = CustomFormationLevel::new(
            lib_id,
            "отделение".to_string(),
            "squad".to_string(),
            1,
        );
        repo.create(&mut level).unwrap();
        assert!(level.id.is_some());

        let loaded = repo.get_by_id(level.id.unwrap()).unwrap();
        assert!(loaded.is_some());
        let l = loaded.unwrap();
        assert_eq!(l.name_ru, "отделение");
        assert_eq!(l.name_en, "squad");
        assert_eq!(l.standard_level_ordinal, 1);

        let list = repo.list_by_library(lib_id).unwrap();
        assert_eq!(list.len(), 1);

        let mut updated = l.clone();
        updated.name_ru = "взвод".to_string();
        updated.standard_level_ordinal = 3;
        repo.update(&updated).unwrap();

        let after = repo.get_by_id(level.id.unwrap()).unwrap().unwrap();
        assert_eq!(after.name_ru, "взвод");
        assert_eq!(after.standard_level_ordinal, 3);

        repo.delete(level.id.unwrap()).unwrap();
        assert!(repo.get_by_id(level.id.unwrap()).unwrap().is_none());
    }

    #[test]
    fn test_formation_level_delete_by_library() {
        let db = Database::open_in_memory().unwrap();
        let lib_repo = LibraryRepo::new(db.conn());
        let mut library = Library::new(
            "Test".to_string(), "US".to_string(), "2003".to_string(), "Author".to_string(),
        );
        lib_repo.create(&mut library).unwrap();
        let lib_id = library.id.unwrap();

        let repo = FormationLevelRepo::new(db.conn());
        let mut l1 = CustomFormationLevel::new(lib_id, "взвод".to_string(), "platoon".to_string(), 3);
        let mut l2 = CustomFormationLevel::new(lib_id, "рота".to_string(), "company".to_string(), 4);
        repo.create(&mut l1).unwrap();
        repo.create(&mut l2).unwrap();
        assert_eq!(repo.list_by_library(lib_id).unwrap().len(), 2);

        repo.delete_by_library(lib_id).unwrap();
        assert!(repo.list_by_library(lib_id).unwrap().is_empty());
    }

    #[test]
    fn test_formation_level_get_nonexistent() {
        let db = Database::open_in_memory().unwrap();
        let repo = FormationLevelRepo::new(db.conn());
        let result = repo.get_by_id(999).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_formation_level_update_without_id_fails() {
        let db = Database::open_in_memory().unwrap();
        let repo = FormationLevelRepo::new(db.conn());
        let level = CustomFormationLevel::new(1, "взвод".to_string(), "platoon".to_string(), 3);
        let result = repo.update(&level);
        assert!(result.is_err());
    }
}
