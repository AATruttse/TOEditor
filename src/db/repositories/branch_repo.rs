//! Repository for branches (роды войск) per library.

use anyhow::Result;
use rusqlite::{params, Connection};
use crate::models::Branch;

pub struct BranchRepo<'a> {
    conn: &'a Connection,
}

impl<'a> BranchRepo<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn create(&self, branch: &mut Branch) -> Result<()> {
        self.conn.execute(
            "INSERT INTO branches (library_id, category_id, name_ru, name_en) VALUES (?1, ?2, ?3, ?4)",
            params![branch.library_id, branch.category_id, branch.name_ru, branch.name_en],
        )?;
        branch.id = Some(self.conn.last_insert_rowid());
        Ok(())
    }

    pub fn get_by_id(&self, id: i64) -> Result<Option<Branch>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, library_id, category_id, name_ru, name_en FROM branches WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], |row| {
            Ok(Branch {
                id: Some(row.get(0)?),
                library_id: row.get(1)?,
                category_id: row.get(2)?,
                name_ru: row.get(3)?,
                name_en: row.get(4)?,
            })
        })?;
        match rows.next() {
            Some(Ok(b)) => Ok(Some(b)),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    pub fn list_by_library(&self, library_id: i64) -> Result<Vec<Branch>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, library_id, category_id, name_ru, name_en FROM branches WHERE library_id = ?1 ORDER BY id",
        )?;
        let rows = stmt.query_map(params![library_id], |row| {
            Ok(Branch {
                id: Some(row.get(0)?),
                library_id: row.get(1)?,
                category_id: row.get(2)?,
                name_ru: row.get(3)?,
                name_en: row.get(4)?,
            })
        })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    pub fn update(&self, branch: &Branch) -> Result<()> {
        let id = branch.id.ok_or_else(|| anyhow::anyhow!("Branch has no id"))?;
        self.conn.execute(
            "UPDATE branches SET name_ru = ?1, name_en = ?2 WHERE id = ?3",
            params![branch.name_ru, branch.name_en, id],
        )?;
        Ok(())
    }

    pub fn delete(&self, id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM branches WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Delete all branches for a library (e.g. before replacing with imported/copied list).
    pub fn delete_by_library(&self, library_id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM branches WHERE library_id = ?1", params![library_id])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::db::repositories::LibraryRepo;
    use crate::models::{Library, default_branches};

    #[test]
    fn test_branch_crud() {
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

        let repo = BranchRepo::new(db.conn());
        let mut branch = Branch::with_category(lib_id, None, "Пехота".to_string(), "Infantry".to_string());
        repo.create(&mut branch).unwrap();
        assert!(branch.id.is_some());

        let loaded = repo.get_by_id(branch.id.unwrap()).unwrap();
        assert!(loaded.is_some());
        let b = loaded.unwrap();
        assert_eq!(b.name_ru, "Пехота");
        assert_eq!(b.name_en, "Infantry");

        let list = repo.list_by_library(lib_id).unwrap();
        assert_eq!(list.len(), 1);

        let mut updated = b.clone();
        updated.name_ru = "Мотострелки".to_string();
        repo.update(&updated).unwrap();
        let after = repo.get_by_id(branch.id.unwrap()).unwrap().unwrap();
        assert_eq!(after.name_ru, "Мотострелки");

        repo.delete(branch.id.unwrap()).unwrap();
        assert!(repo.get_by_id(branch.id.unwrap()).unwrap().is_none());
    }

    #[test]
    fn test_default_branches_created() {
        let db = Database::open_in_memory().unwrap();
        let lib_repo = LibraryRepo::new(db.conn());
        let branch_repo = BranchRepo::new(db.conn());
        let mut library = Library::new(
            "Test".to_string(),
            "US".to_string(),
            "2003".to_string(),
            "Author".to_string(),
        );
        lib_repo.create(&mut library).unwrap();
        let lib_id = library.id.unwrap();

        for (mut b, _cat_idx) in default_branches(lib_id) {
            branch_repo.create(&mut b).unwrap();
        }
        let list = branch_repo.list_by_library(lib_id).unwrap();
        assert!(list.len() >= 10);
        assert!(list.iter().any(|b| b.name_en == "Infantry"));
        assert!(list.iter().any(|b| b.name_en == "Cavalry"));
    }

    #[test]
    fn test_branch_delete_by_library() {
        let db = Database::open_in_memory().unwrap();
        let lib_repo = LibraryRepo::new(db.conn());
        let mut library = Library::new(
            "Test".to_string(), "US".to_string(), "2003".to_string(), "Author".to_string(),
        );
        lib_repo.create(&mut library).unwrap();
        let lib_id = library.id.unwrap();

        let repo = BranchRepo::new(db.conn());
        let mut b1 = Branch::new(lib_id, "Пехота".to_string(), "Infantry".to_string());
        let mut b2 = Branch::new(lib_id, "Танки".to_string(), "Armor".to_string());
        repo.create(&mut b1).unwrap();
        repo.create(&mut b2).unwrap();
        assert_eq!(repo.list_by_library(lib_id).unwrap().len(), 2);

        repo.delete_by_library(lib_id).unwrap();
        assert!(repo.list_by_library(lib_id).unwrap().is_empty());
    }

    #[test]
    fn test_branch_with_category_roundtrip() {
        let db = Database::open_in_memory().unwrap();
        let lib_repo = LibraryRepo::new(db.conn());
        let mut library = Library::new(
            "Test".to_string(), "US".to_string(), "2003".to_string(), "Author".to_string(),
        );
        lib_repo.create(&mut library).unwrap();
        let lib_id = library.id.unwrap();

        let repo = BranchRepo::new(db.conn());
        let mut branch = Branch::with_category(
            lib_id, Some(42), "Пехота".to_string(), "Infantry".to_string(),
        );
        repo.create(&mut branch).unwrap();

        let loaded = repo.get_by_id(branch.id.unwrap()).unwrap().unwrap();
        assert_eq!(loaded.category_id, Some(42));
        assert_eq!(loaded.name_en, "Infantry");
    }

    #[test]
    fn test_branch_update_does_not_update_category_id() {
        // NOTE: This test documents a known limitation — BranchRepo::update
        // only updates name_ru and name_en, NOT category_id. The category is
        // updated via the UI model (delete all + recreate pattern).
        let db = Database::open_in_memory().unwrap();
        let lib_repo = LibraryRepo::new(db.conn());
        let mut library = Library::new(
            "Test".to_string(), "US".to_string(), "2003".to_string(), "Author".to_string(),
        );
        lib_repo.create(&mut library).unwrap();
        let lib_id = library.id.unwrap();

        let repo = BranchRepo::new(db.conn());
        let mut branch = Branch::with_category(
            lib_id, Some(10), "Пехота".to_string(), "Infantry".to_string(),
        );
        repo.create(&mut branch).unwrap();

        // Update the branch (only names are updated by the current implementation)
        let mut updated = branch.clone();
        updated.name_ru = "Мотострелки".to_string();
        updated.category_id = Some(99);
        repo.update(&updated).unwrap();

        let after = repo.get_by_id(branch.id.unwrap()).unwrap().unwrap();
        assert_eq!(after.name_ru, "Мотострелки");
        // category_id is NOT updated by update() — remains the original value
        assert_eq!(after.category_id, Some(10));
    }

    #[test]
    fn test_branch_get_nonexistent() {
        let db = Database::open_in_memory().unwrap();
        let repo = BranchRepo::new(db.conn());
        let result = repo.get_by_id(999).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_branch_update_without_id_fails() {
        let db = Database::open_in_memory().unwrap();
        let repo = BranchRepo::new(db.conn());
        let branch = Branch::new(1, "Пехота".to_string(), "Infantry".to_string());
        let result = repo.update(&branch);
        assert!(result.is_err());
    }

    #[test]
    fn test_branch_isolation_between_libraries() {
        let db = Database::open_in_memory().unwrap();
        let lib_repo = LibraryRepo::new(db.conn());
        let mut lib1 = Library::new("L1".to_string(), "US".to_string(), "2003".to_string(), "A".to_string());
        let mut lib2 = Library::new("L2".to_string(), "RU".to_string(), "2020".to_string(), "B".to_string());
        lib_repo.create(&mut lib1).unwrap();
        lib_repo.create(&mut lib2).unwrap();

        let repo = BranchRepo::new(db.conn());
        let mut b1 = Branch::new(lib1.id.unwrap(), "Пехота".to_string(), "Infantry".to_string());
        let mut b2 = Branch::new(lib2.id.unwrap(), "Танки".to_string(), "Armor".to_string());
        repo.create(&mut b1).unwrap();
        repo.create(&mut b2).unwrap();

        let list1 = repo.list_by_library(lib1.id.unwrap()).unwrap();
        let list2 = repo.list_by_library(lib2.id.unwrap()).unwrap();
        assert_eq!(list1.len(), 1);
        assert_eq!(list2.len(), 1);
        assert_eq!(list1[0].name_en, "Infantry");
        assert_eq!(list2[0].name_en, "Armor");
    }
}
