//! Repository for Version/Snapshot operations

use anyhow::Result;
use rusqlite::{params, Connection};
use crate::models::Snapshot;

/// Repository for snapshot database operations
pub struct VersionRepo<'a> {
    conn: &'a Connection,
}

impl<'a> VersionRepo<'a> {
    /// Create new repository
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Create a new snapshot
    pub fn create(&self, snapshot: &mut Snapshot) -> Result<()> {
        self.conn.execute(
            "INSERT INTO snapshots (library_id, version, timestamp, data, description)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                snapshot.library_id,
                snapshot.version,
                snapshot.timestamp,
                snapshot.data,
                snapshot.description
            ],
        )?;
        snapshot.id = Some(self.conn.last_insert_rowid());
        Ok(())
    }

    /// Get latest snapshot for a library
    pub fn get_latest(&self, library_id: i64) -> Result<Option<Snapshot>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, library_id, version, timestamp, data, description
             FROM snapshots
             WHERE library_id = ?1
             ORDER BY version DESC
             LIMIT 1"
        )?;
        
        let mut rows = stmt.query_map(params![library_id], |row| {
            Ok(Snapshot {
                id: Some(row.get(0)?),
                library_id: row.get(1)?,
                version: row.get(2)?,
                timestamp: row.get(3)?,
                data: row.get(4)?,
                description: row.get(5)?,
            })
        })?;

        match rows.next() {
            Some(Ok(snapshot)) => Ok(Some(snapshot)),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    /// Get all snapshots for a library
    pub fn list_by_library(&self, library_id: i64) -> Result<Vec<Snapshot>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, library_id, version, timestamp, data, description
             FROM snapshots
             WHERE library_id = ?1
             ORDER BY version DESC"
        )?;
        
        let rows = stmt.query_map(params![library_id], |row| {
            Ok(Snapshot {
                id: Some(row.get(0)?),
                library_id: row.get(1)?,
                version: row.get(2)?,
                timestamp: row.get(3)?,
                data: row.get(4)?,
                description: row.get(5)?,
            })
        })?;

        let mut snapshots = Vec::new();
        for row in rows {
            snapshots.push(row?);
        }
        Ok(snapshots)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;

    #[test]
    fn test_create_snapshot() {
        let db = Database::open_in_memory().unwrap();
        let repo = VersionRepo::new(db.conn());
        let mut snapshot = Snapshot::new(1, 1, "{}".to_string());
        repo.create(&mut snapshot).unwrap();
        assert!(snapshot.id.is_some());
    }
}
