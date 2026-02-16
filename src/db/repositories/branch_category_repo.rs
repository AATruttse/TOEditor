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
