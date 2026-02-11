//! Repository for Library operations

use anyhow::Result;
use rusqlite::{params, Connection};
use crate::models::Library;

/// Repository for library database operations
pub struct LibraryRepo<'a> {
    conn: &'a Connection,
}

impl<'a> LibraryRepo<'a> {
    /// Create new repository
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Create a new library
    pub fn create(&self, library: &mut Library) -> Result<()> {
        let now = chrono::Utc::now().timestamp();
        self.conn.execute(
            "INSERT INTO libraries (name, country, era, tags, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                library.name,
                library.country,
                library.era,
                serde_json::to_string(&library.tags)?,
                now,
                now
            ],
        )?;
        library.id = Some(self.conn.last_insert_rowid());
        Ok(())
    }

    /// Get library by ID
    pub fn get_by_id(&self, id: i64) -> Result<Option<Library>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, country, era, tags FROM libraries WHERE id = ?1"
        )?;
        
        let mut rows = stmt.query_map(params![id], |row| {
            let tags_json: String = row.get(4)?;
            let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
            Ok(Library {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                country: row.get(2)?,
                era: row.get(3)?,
                tags,
                units: Vec::new(), // Units loaded separately
            })
        })?;

        match rows.next() {
            Some(Ok(lib)) => Ok(Some(lib)),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    /// List all libraries
    pub fn list_all(&self) -> Result<Vec<Library>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, country, era, tags FROM libraries ORDER BY name"
        )?;
        
        let rows = stmt.query_map([], |row| {
            let tags_json: String = row.get(4)?;
            let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
            Ok(Library {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                country: row.get(2)?,
                era: row.get(3)?,
                tags,
                units: Vec::new(),
            })
        })?;

        let mut libraries = Vec::new();
        for row in rows {
            libraries.push(row?);
        }
        Ok(libraries)
    }

    /// Update library
    pub fn update(&self, library: &Library) -> Result<()> {
        let now = chrono::Utc::now().timestamp();
        self.conn.execute(
            "UPDATE libraries SET name = ?1, country = ?2, era = ?3, tags = ?4, updated_at = ?5
             WHERE id = ?6",
            params![
                library.name,
                library.country,
                library.era,
                serde_json::to_string(&library.tags)?,
                now,
                library.id.unwrap()
            ],
        )?;
        Ok(())
    }

    /// Delete library
    pub fn delete(&self, id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM libraries WHERE id = ?1", params![id])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;

    #[test]
    fn test_create_library() {
        let db = Database::open_in_memory().unwrap();
        let repo = LibraryRepo::new(db.conn());
        let mut library = Library::new("Test".to_string(), "US".to_string(), "2003".to_string());
        repo.create(&mut library).unwrap();
        assert!(library.id.is_some());
    }

    #[test]
    fn test_get_library() {
        let db = Database::open_in_memory().unwrap();
        let repo = LibraryRepo::new(db.conn());
        let mut library = Library::new("Test".to_string(), "US".to_string(), "2003".to_string());
        repo.create(&mut library).unwrap();
        
        let retrieved = repo.get_by_id(library.id.unwrap()).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Test");
    }
}
