//! Database connection and migration management

pub mod repositories;

use anyhow::Result;
use rusqlite::Connection;
use std::path::Path;

/// Database connection wrapper
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Open or create database at path
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        let db = Self { conn };
        db.run_migrations()?;
        Ok(db)
    }

    /// Open in-memory database (for testing)
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        let db = Self { conn };
        db.run_migrations()?;
        Ok(db)
    }

    /// Get underlying connection (for repositories)
    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    /// Current schema version. Increment when adding new migrations.
    #[cfg(test)]
    const CURRENT_SCHEMA_VERSION: i64 = 3;

    /// Get current schema version from the database (0 if table does not exist).
    fn schema_version(&self) -> i64 {
        self.conn
            .query_row(
                "SELECT version FROM schema_version ORDER BY version DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0)
    }

    /// Record a schema version after a migration succeeds.
    fn set_schema_version(&self, version: i64) -> Result<()> {
        self.conn.execute(
            "INSERT INTO schema_version (version, applied_at) VALUES (?1, strftime('%s','now'))",
            rusqlite::params![version],
        )?;
        Ok(())
    }

    /// Run database migrations sequentially based on schema version.
    fn run_migrations(&self) -> Result<()> {
        // The schema_version table itself is always created first
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_version (
                version INTEGER PRIMARY KEY,
                applied_at INTEGER NOT NULL
            );"
        )?;

        let current = self.schema_version();

        if current < 1 {
            self.migrate_v1()?;
            self.set_schema_version(1)?;
        }
        if current < 2 {
            self.migrate_v2()?;
            self.set_schema_version(2)?;
        }
        if current < 3 {
            self.migrate_v3()?;
            self.set_schema_version(3)?;
        }

        Ok(())
    }

    /// V1: Core tables - libraries, units, personnel, equipment, snapshots + indexes
    fn migrate_v1(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS libraries (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                country TEXT NOT NULL,
                era TEXT NOT NULL,
                author TEXT NOT NULL DEFAULT '',
                version INTEGER NOT NULL DEFAULT 1,
                tags TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )?;
        // Migrate existing databases: add author and version columns if they don't exist
        let _ = self.conn.execute(
            "ALTER TABLE libraries ADD COLUMN author TEXT NOT NULL DEFAULT ''",
            [],
        );
        let _ = self.conn.execute(
            "ALTER TABLE libraries ADD COLUMN version INTEGER NOT NULL DEFAULT 1",
            [],
        );

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS units (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                library_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                unit_type TEXT NOT NULL,
                parent_id INTEGER,
                FOREIGN KEY (library_id) REFERENCES libraries(id) ON DELETE CASCADE,
                FOREIGN KEY (parent_id) REFERENCES units(id) ON DELETE CASCADE
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS personnel (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                unit_id INTEGER NOT NULL,
                position TEXT NOT NULL,
                rank TEXT,
                FOREIGN KEY (unit_id) REFERENCES units(id) ON DELETE CASCADE
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS equipment (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                unit_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                quantity INTEGER NOT NULL,
                FOREIGN KEY (unit_id) REFERENCES units(id) ON DELETE CASCADE
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS snapshots (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                library_id INTEGER NOT NULL,
                version INTEGER NOT NULL,
                timestamp INTEGER NOT NULL,
                data TEXT NOT NULL,
                description TEXT,
                FOREIGN KEY (library_id) REFERENCES libraries(id) ON DELETE CASCADE,
                UNIQUE(library_id, version)
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_units_library_id ON units(library_id)",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_units_parent_id ON units(parent_id)",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_snapshots_library_id ON snapshots(library_id)",
            [],
        )?;

        Ok(())
    }

    /// V2: Formation levels table + index
    fn migrate_v2(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS formation_levels (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                library_id INTEGER NOT NULL,
                name_ru TEXT NOT NULL,
                name_en TEXT NOT NULL,
                standard_level_ordinal INTEGER NOT NULL,
                FOREIGN KEY (library_id) REFERENCES libraries(id) ON DELETE CASCADE
            )",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_formation_levels_library_id ON formation_levels(library_id)",
            [],
        )?;
        Ok(())
    }

    /// V3: Branch categories and branches tables + indexes + category_id column
    fn migrate_v3(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS branch_categories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                library_id INTEGER NOT NULL,
                name_ru TEXT NOT NULL,
                name_en TEXT NOT NULL,
                FOREIGN KEY (library_id) REFERENCES libraries(id) ON DELETE CASCADE
            )",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_branch_categories_library_id ON branch_categories(library_id)",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS branches (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                library_id INTEGER NOT NULL,
                name_ru TEXT NOT NULL,
                name_en TEXT NOT NULL,
                FOREIGN KEY (library_id) REFERENCES libraries(id) ON DELETE CASCADE
            )",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_branches_library_id ON branches(library_id)",
            [],
        )?;
        let _ = self.conn.execute("ALTER TABLE branches ADD COLUMN category_id INTEGER", []);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_creation() {
        let db = Database::open_in_memory().unwrap();
        // Verify tables exist by querying schema
        let mut stmt = db.conn().prepare("SELECT name FROM sqlite_master WHERE type='table'").unwrap();
        let tables: Vec<String> = stmt.query_map([], |row| row.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        
        assert!(tables.contains(&"libraries".to_string()));
        assert!(tables.contains(&"units".to_string()));
        assert!(tables.contains(&"personnel".to_string()));
        assert!(tables.contains(&"equipment".to_string()));
        assert!(tables.contains(&"snapshots".to_string()));
        assert!(tables.contains(&"formation_levels".to_string()));
        assert!(tables.contains(&"branch_categories".to_string()));
        assert!(tables.contains(&"branches".to_string()));
    }

    #[test]
    fn test_database_migration_idempotency() {
        // Running migrations twice should not cause errors
        let db = Database::open_in_memory().unwrap();
        // run_migrations was already called in open_in_memory.
        // Call it again to verify idempotency.
        db.run_migrations().unwrap();

        // Verify tables still exist and are usable
        let mut stmt = db.conn().prepare("SELECT name FROM sqlite_master WHERE type='table'").unwrap();
        let tables: Vec<String> = stmt.query_map([], |row| row.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        assert!(tables.contains(&"libraries".to_string()));
        assert!(tables.contains(&"branches".to_string()));
        assert!(tables.contains(&"branch_categories".to_string()));
    }

    #[test]
    fn test_schema_version_tracking() {
        let db = Database::open_in_memory().unwrap();
        assert_eq!(db.schema_version(), Database::CURRENT_SCHEMA_VERSION);

        // Verify schema_version table has entries
        let count: i64 = db.conn()
            .query_row("SELECT COUNT(*) FROM schema_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, Database::CURRENT_SCHEMA_VERSION);
    }

    #[test]
    fn test_schema_version_table_exists() {
        let db = Database::open_in_memory().unwrap();
        let mut stmt = db.conn().prepare("SELECT name FROM sqlite_master WHERE type='table'").unwrap();
        let tables: Vec<String> = stmt.query_map([], |row| row.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        assert!(tables.contains(&"schema_version".to_string()));
    }

    #[test]
    fn test_database_indexes_created() {
        let db = Database::open_in_memory().unwrap();
        let mut stmt = db.conn().prepare("SELECT name FROM sqlite_master WHERE type='index'").unwrap();
        let indexes: Vec<String> = stmt.query_map([], |row| row.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();

        assert!(indexes.contains(&"idx_units_library_id".to_string()));
        assert!(indexes.contains(&"idx_units_parent_id".to_string()));
        assert!(indexes.contains(&"idx_snapshots_library_id".to_string()));
        assert!(indexes.contains(&"idx_formation_levels_library_id".to_string()));
        assert!(indexes.contains(&"idx_branches_library_id".to_string()));
        assert!(indexes.contains(&"idx_branch_categories_library_id".to_string()));
    }

    #[test]
    fn test_foreign_keys_enabled() {
        let db = Database::open_in_memory().unwrap();
        let fk_enabled: bool = db.conn()
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .unwrap();
        assert!(fk_enabled, "Foreign keys should be enabled");
    }

    #[test]
    fn test_foreign_key_enforcement() {
        let db = Database::open_in_memory().unwrap();
        // Trying to insert a unit with a non-existent library_id should fail
        let result = db.conn().execute(
            "INSERT INTO units (library_id, name, unit_type) VALUES (9999, 'Ghost', 'squad')",
            [],
        );
        assert!(result.is_err(), "FK violation should be rejected");
    }

    #[test]
    fn test_database_file_based() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // Create and populate database
        {
            let db = Database::open(&db_path).unwrap();
            let lib_repo = crate::db::repositories::LibraryRepo::new(db.conn());
            let mut lib = crate::models::Library::new(
                "Test".to_string(), "US".to_string(), "2003".to_string(), "A".to_string(),
            );
            lib_repo.create(&mut lib).unwrap();
        }

        // Reopen and verify data persists
        {
            let db = Database::open(&db_path).unwrap();
            let lib_repo = crate::db::repositories::LibraryRepo::new(db.conn());
            let libs = lib_repo.list_all().unwrap();
            assert_eq!(libs.len(), 1);
            assert_eq!(libs[0].name, "Test");
        }
    }
}
