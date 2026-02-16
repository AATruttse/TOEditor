//! Repository for branch categories (категории родов войск) per library.

use anyhow::Result;
use rusqlite::{params, Connection};
use crate::models::BranchCategory;

pub struct BranchCategoryRepo<'a> {
    conn: &'a Connection,
}

impl<'a> BranchCategoryRepo<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn create(&self, cat: &mut BranchCategory) -> Result<()> {
        self.conn.execute(
            "INSERT INTO branch_categories (library_id, name_ru, name_en) VALUES (?1, ?2, ?3)",
            params![cat.library_id, cat.name_ru, cat.name_en],
        )?;
        cat.id = Some(self.conn.last_insert_rowid());
        Ok(())
    }

    pub fn get_by_id(&self, id: i64) -> Result<Option<BranchCategory>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, library_id, name_ru, name_en FROM branch_categories WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], |row| {
            Ok(BranchCategory {
                id: Some(row.get(0)?),
                library_id: row.get(1)?,
                name_ru: row.get(2)?,
                name_en: row.get(3)?,
            })
        })?;
        match rows.next() {
            Some(Ok(c)) => Ok(Some(c)),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    pub fn list_by_library(&self, library_id: i64) -> Result<Vec<BranchCategory>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, library_id, name_ru, name_en FROM branch_categories WHERE library_id = ?1 ORDER BY id",
        )?;
        let rows = stmt.query_map(params![library_id], |row| {
            Ok(BranchCategory {
                id: Some(row.get(0)?),
                library_id: row.get(1)?,
                name_ru: row.get(2)?,
                name_en: row.get(3)?,
            })
        })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    pub fn update(&self, cat: &BranchCategory) -> Result<()> {
        let id = cat.id.ok_or_else(|| anyhow::anyhow!("BranchCategory has no id"))?;
        self.conn.execute(
            "UPDATE branch_categories SET name_ru = ?1, name_en = ?2 WHERE id = ?3",
            params![cat.name_ru, cat.name_en, id],
        )?;
        Ok(())
    }

    pub fn delete(&self, id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM branch_categories WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn delete_by_library(&self, library_id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM branch_categories WHERE library_id = ?1", params![library_id])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::db::repositories::LibraryRepo;
    use crate::models::{Library, BranchCategory};

    fn setup() -> (Database, i64) {
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
        (db, lib_id)
    }

    #[test]
    fn test_branch_category_create_and_get() {
        let (db, lib_id) = setup();
        let repo = BranchCategoryRepo::new(db.conn());

        let mut cat = BranchCategory::new(lib_id, "Боевые".to_string(), "Combat".to_string());
        repo.create(&mut cat).unwrap();
        assert!(cat.id.is_some());

        let loaded = repo.get_by_id(cat.id.unwrap()).unwrap();
        assert!(loaded.is_some());
        let c = loaded.unwrap();
        assert_eq!(c.name_ru, "Боевые");
        assert_eq!(c.name_en, "Combat");
        assert_eq!(c.library_id, lib_id);
    }

    #[test]
    fn test_branch_category_list_by_library() {
        let (db, lib_id) = setup();
        let repo = BranchCategoryRepo::new(db.conn());

        let mut c1 = BranchCategory::new(lib_id, "Боевые".to_string(), "Combat".to_string());
        let mut c2 = BranchCategory::new(lib_id, "ПВО".to_string(), "Air defense".to_string());
        repo.create(&mut c1).unwrap();
        repo.create(&mut c2).unwrap();

        let list = repo.list_by_library(lib_id).unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].name_en, "Combat");
        assert_eq!(list[1].name_en, "Air defense");
    }

    #[test]
    fn test_branch_category_list_by_library_empty() {
        let (db, lib_id) = setup();
        let repo = BranchCategoryRepo::new(db.conn());
        let list = repo.list_by_library(lib_id).unwrap();
        assert!(list.is_empty());
    }

    #[test]
    fn test_branch_category_update() {
        let (db, lib_id) = setup();
        let repo = BranchCategoryRepo::new(db.conn());

        let mut cat = BranchCategory::new(lib_id, "Боевые".to_string(), "Combat".to_string());
        repo.create(&mut cat).unwrap();

        let mut updated = cat.clone();
        updated.name_ru = "Артиллерия".to_string();
        updated.name_en = "Artillery".to_string();
        repo.update(&updated).unwrap();

        let after = repo.get_by_id(cat.id.unwrap()).unwrap().unwrap();
        assert_eq!(after.name_ru, "Артиллерия");
        assert_eq!(after.name_en, "Artillery");
    }

    #[test]
    fn test_branch_category_update_without_id_fails() {
        let (db, lib_id) = setup();
        let repo = BranchCategoryRepo::new(db.conn());
        let cat = BranchCategory::new(lib_id, "Боевые".to_string(), "Combat".to_string());
        let result = repo.update(&cat);
        assert!(result.is_err());
    }

    #[test]
    fn test_branch_category_delete() {
        let (db, lib_id) = setup();
        let repo = BranchCategoryRepo::new(db.conn());

        let mut cat = BranchCategory::new(lib_id, "Боевые".to_string(), "Combat".to_string());
        repo.create(&mut cat).unwrap();
        assert!(repo.get_by_id(cat.id.unwrap()).unwrap().is_some());

        repo.delete(cat.id.unwrap()).unwrap();
        assert!(repo.get_by_id(cat.id.unwrap()).unwrap().is_none());
    }

    #[test]
    fn test_branch_category_delete_by_library() {
        let (db, lib_id) = setup();
        let repo = BranchCategoryRepo::new(db.conn());

        let mut c1 = BranchCategory::new(lib_id, "Боевые".to_string(), "Combat".to_string());
        let mut c2 = BranchCategory::new(lib_id, "ПВО".to_string(), "Air defense".to_string());
        repo.create(&mut c1).unwrap();
        repo.create(&mut c2).unwrap();
        assert_eq!(repo.list_by_library(lib_id).unwrap().len(), 2);

        repo.delete_by_library(lib_id).unwrap();
        assert!(repo.list_by_library(lib_id).unwrap().is_empty());
    }

    #[test]
    fn test_branch_category_get_nonexistent() {
        let (db, _lib_id) = setup();
        let repo = BranchCategoryRepo::new(db.conn());
        let result = repo.get_by_id(999).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_branch_category_isolation_between_libraries() {
        let db = Database::open_in_memory().unwrap();
        let lib_repo = LibraryRepo::new(db.conn());
        let mut lib1 = Library::new("Lib1".to_string(), "US".to_string(), "2003".to_string(), "A".to_string());
        let mut lib2 = Library::new("Lib2".to_string(), "RU".to_string(), "2020".to_string(), "B".to_string());
        lib_repo.create(&mut lib1).unwrap();
        lib_repo.create(&mut lib2).unwrap();

        let repo = BranchCategoryRepo::new(db.conn());
        let mut c1 = BranchCategory::new(lib1.id.unwrap(), "Боевые".to_string(), "Combat".to_string());
        let mut c2 = BranchCategory::new(lib2.id.unwrap(), "ПВО".to_string(), "Air defense".to_string());
        repo.create(&mut c1).unwrap();
        repo.create(&mut c2).unwrap();

        let list1 = repo.list_by_library(lib1.id.unwrap()).unwrap();
        let list2 = repo.list_by_library(lib2.id.unwrap()).unwrap();
        assert_eq!(list1.len(), 1);
        assert_eq!(list2.len(), 1);
        assert_eq!(list1[0].name_en, "Combat");
        assert_eq!(list2[0].name_en, "Air defense");
    }
}
