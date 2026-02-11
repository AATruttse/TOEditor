//! Repository for Unit operations

use anyhow::Result;
use rusqlite::{params, Connection};
use crate::models::{Unit, Equipment, Personnel};

/// Repository for unit database operations
pub struct UnitRepo<'a> {
    conn: &'a Connection,
}

impl<'a> UnitRepo<'a> {
    /// Create new repository
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Create a new unit
    pub fn create(&self, library_id: i64, unit: &mut Unit) -> Result<()> {
        self.conn.execute(
            "INSERT INTO units (library_id, name, unit_type, parent_id)
             VALUES (?1, ?2, ?3, ?4)",
            params![library_id, unit.name, unit.unit_type, unit.parent_id],
        )?;
        unit.id = Some(self.conn.last_insert_rowid());
        
        // Save personnel
        for personnel in &mut unit.personnel {
            self.create_personnel(unit.id.unwrap(), personnel)?;
        }
        
        // Save equipment
        for equipment in &mut unit.equipment {
            self.create_equipment(unit.id.unwrap(), equipment)?;
        }
        
        Ok(())
    }

    /// Create personnel entry
    pub fn create_personnel(&self, unit_id: i64, personnel: &mut Personnel) -> Result<()> {
        self.conn.execute(
            "INSERT INTO personnel (unit_id, position, rank) VALUES (?1, ?2, ?3)",
            params![unit_id, personnel.position, personnel.rank],
        )?;
        Ok(())
    }

    /// Create equipment entry
    pub fn create_equipment(&self, unit_id: i64, equipment: &Equipment) -> Result<()> {
        self.conn.execute(
            "INSERT INTO equipment (unit_id, name, quantity) VALUES (?1, ?2, ?3)",
            params![unit_id, equipment.name, equipment.quantity],
        )?;
        Ok(())
    }

    /// Get unit by ID with all related data
    pub fn get_by_id(&self, id: i64) -> Result<Option<Unit>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, unit_type, parent_id FROM units WHERE id = ?1"
        )?;
        
        let mut rows = stmt.query_map(params![id], |row| {
            Ok(Unit {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                unit_type: row.get(2)?,
                parent_id: row.get(3)?,
                personnel: Vec::new(), // Loaded separately
                equipment: Vec::new(),  // Loaded separately
                children: Vec::new(),  // Loaded separately
            })
        })?;

        if let Some(Ok(mut unit)) = rows.next() {
            // Load personnel
            unit.personnel = self.load_personnel(id)?;
            
            // Load equipment
            unit.equipment = self.load_equipment(id)?;
            
            // Load children
            unit.children = self.load_children(id)?;
            
            Ok(Some(unit))
        } else {
            Ok(None)
        }
    }

    /// Load personnel for a unit
    fn load_personnel(&self, unit_id: i64) -> Result<Vec<Personnel>> {
        let mut stmt = self.conn.prepare(
            "SELECT position, rank FROM personnel WHERE unit_id = ?1"
        )?;
        
        let rows = stmt.query_map(params![unit_id], |row| {
            Ok(Personnel {
                position: row.get(0)?,
                rank: row.get(1)?,
            })
        })?;

        let mut personnel = Vec::new();
        for row in rows {
            personnel.push(row?);
        }
        Ok(personnel)
    }

    /// Load equipment for a unit
    fn load_equipment(&self, unit_id: i64) -> Result<Vec<Equipment>> {
        let mut stmt = self.conn.prepare(
            "SELECT name, quantity FROM equipment WHERE unit_id = ?1"
        )?;
        
        let rows = stmt.query_map(params![unit_id], |row| {
            Ok(Equipment {
                name: row.get(0)?,
                quantity: row.get(1)?,
            })
        })?;

        let mut equipment = Vec::new();
        for row in rows {
            equipment.push(row?);
        }
        Ok(equipment)
    }

    /// Load child units
    fn load_children(&self, parent_id: i64) -> Result<Vec<Unit>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, unit_type, parent_id FROM units WHERE parent_id = ?1"
        )?;
        
        let rows = stmt.query_map(params![parent_id], |row| {
            Ok(Unit {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                unit_type: row.get(2)?,
                parent_id: Some(row.get(3)?),
                personnel: Vec::new(),
                equipment: Vec::new(),
                children: Vec::new(),
            })
        })?;

        let mut children = Vec::new();
        for row in rows {
            if let Ok(mut child) = row {
                let child_id = child.id.unwrap();
                child.personnel = self.load_personnel(child_id)?;
                child.equipment = self.load_equipment(child_id)?;
                child.children = self.load_children(child_id)?;
                children.push(child);
            }
        }
        Ok(children)
    }

    /// Get all units for a library
    pub fn get_by_library_id(&self, library_id: i64) -> Result<Vec<Unit>> {
        let mut stmt = self.conn.prepare(
            "SELECT id FROM units WHERE library_id = ?1 AND parent_id IS NULL"
        )?;
        
        let rows = stmt.query_map(params![library_id], |row| {
            Ok(row.get::<_, i64>(0)?)
        })?;

        let mut units = Vec::new();
        for row in rows {
            if let Ok(unit_id) = row {
                if let Some(unit) = self.get_by_id(unit_id)? {
                    units.push(unit);
                }
            }
        }
        Ok(units)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::db::repositories::LibraryRepo;
    use crate::models::Library;

    #[test]
    fn test_create_unit() {
        let db = Database::open_in_memory().unwrap();
        let lib_repo = LibraryRepo::new(db.conn());
        let mut library = Library::new("Test".to_string(), "US".to_string(), "2003".to_string());
        lib_repo.create(&mut library).unwrap();
        
        let repo = UnitRepo::new(db.conn());
        let mut unit = Unit::new("Squad".to_string(), "Squad".to_string());
        repo.create(library.id.unwrap(), &mut unit).unwrap();
        assert!(unit.id.is_some());
    }
}
