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
            "INSERT INTO libraries (name, country, era, author, version, tags, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                library.name,
                library.country,
                library.era,
                library.author,
                library.version,
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
            "SELECT id, name, country, era, author, version, tags FROM libraries WHERE id = ?1"
        )?;
        
        let mut rows = stmt.query_map(params![id], |row| {
            let tags_json: String = row.get(6)?;
            let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
            Ok(Library {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                country: row.get(2)?,
                era: row.get(3)?,
                author: row.get(4)?,
                version: row.get(5)?,
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
            "SELECT id, name, country, era, author, version, tags FROM libraries ORDER BY name"
        )?;
        
        let rows = stmt.query_map([], |row| {
            let tags_json: String = row.get(6)?;
            let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
            Ok(Library {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                country: row.get(2)?,
                era: row.get(3)?,
                author: row.get(4)?,
                version: row.get(5)?,
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

    /// Search libraries by name, country, era, author, or tags
    pub fn search(&self, query: &str) -> Result<Vec<Library>> {
        let search_pattern = format!("%{}%", query);
        let mut stmt = self.conn.prepare(
            "SELECT id, name, country, era, author, version, tags 
             FROM libraries 
             WHERE name LIKE ?1 
                OR country LIKE ?1 
                OR era LIKE ?1 
                OR author LIKE ?1 
                OR tags LIKE ?1
             ORDER BY name"
        )?;
        
        let rows = stmt.query_map(params![search_pattern], |row| {
            let tags_json: String = row.get(6)?;
            let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
            Ok(Library {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                country: row.get(2)?,
                era: row.get(3)?,
                author: row.get(4)?,
                version: row.get(5)?,
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
        let id = library.id.ok_or_else(|| anyhow::anyhow!("Cannot update library without id"))?;
        let now = chrono::Utc::now().timestamp();
        self.conn.execute(
            "UPDATE libraries SET name = ?1, country = ?2, era = ?3, author = ?4, version = ?5, tags = ?6, updated_at = ?7
             WHERE id = ?8",
            params![
                library.name,
                library.country,
                library.era,
                library.author,
                library.version,
                serde_json::to_string(&library.tags)?,
                now,
                id
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
        let mut library = Library::new(
            "Test".to_string(),
            "US".to_string(),
            "2003".to_string(),
            "Author".to_string(),
        );
        repo.create(&mut library).unwrap();
        assert!(library.id.is_some());
        assert_eq!(library.version, 1);
    }

    #[test]
    fn test_get_library() {
        let db = Database::open_in_memory().unwrap();
        let repo = LibraryRepo::new(db.conn());
        let mut library = Library::new(
            "Test".to_string(),
            "US".to_string(),
            "2003".to_string(),
            "Author".to_string(),
        );
        repo.create(&mut library).unwrap();
        
        let retrieved = repo.get_by_id(library.id.unwrap()).unwrap();
        assert!(retrieved.is_some());
        let lib = retrieved.unwrap();
        assert_eq!(lib.name, "Test");
        assert_eq!(lib.author, "Author");
        assert_eq!(lib.version, 1);
    }

    #[test]
    fn test_search_libraries() {
        let db = Database::open_in_memory().unwrap();
        let repo = LibraryRepo::new(db.conn());
        
        let mut lib1 = Library::new(
            "US Army 2003".to_string(),
            "US".to_string(),
            "2003".to_string(),
            "John Doe".to_string(),
        );
        lib1.tags.push("army".to_string());
        repo.create(&mut lib1).unwrap();
        
        let mut lib2 = Library::new(
            "Russian Forces".to_string(),
            "RU".to_string(),
            "2020".to_string(),
            "Jane Smith".to_string(),
        );
        repo.create(&mut lib2).unwrap();
        
        // Search by name
        let results = repo.search("US Army").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "US Army 2003");
        
        // Search by country
        let results = repo.search("RU").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].country, "RU");
        
        // Search by author
        let results = repo.search("John").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].author, "John Doe");
    }

    #[test]
    fn test_update_library() {
        let db = Database::open_in_memory().unwrap();
        let repo = LibraryRepo::new(db.conn());
        let mut library = Library::new(
            "Test".to_string(),
            "US".to_string(),
            "2003".to_string(),
            "Author".to_string(),
        );
        repo.create(&mut library).unwrap();
        
        library.name = "Updated Test".to_string();
        library.author = "New Author".to_string();
        library.increment_version();
        repo.update(&library).unwrap();
        
        let retrieved = repo.get_by_id(library.id.unwrap()).unwrap().unwrap();
        assert_eq!(retrieved.name, "Updated Test");
        assert_eq!(retrieved.author, "New Author");
        assert_eq!(retrieved.version, 2);
    }

    #[test]
    fn test_update_library_without_id_fails() {
        let db = Database::open_in_memory().unwrap();
        let repo = LibraryRepo::new(db.conn());
        let library = Library::new(
            "Test".to_string(),
            "US".to_string(),
            "2003".to_string(),
            "Author".to_string(),
        );
        // library.id is None — should return an error, not panic
        let result = repo.update(&library);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("without id"));
    }
}
