//! Core domain models: Library, Unit, Equipment, Personnel

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A library contains multiple units and metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Library {
    /// Unique identifier
    pub id: Option<i64>,
    /// Library name (e.g., "US ARMY 2003")
    pub name: String,
    /// Country code (e.g., "US", "RU")
    pub country: String,
    /// Era/period (e.g., "2003", "Cold War")
    pub era: String,
    /// Author of the library
    pub author: String,
    /// Current version number
    pub version: i64,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Units in this library
    pub units: Vec<Unit>,
}

impl Library {
    /// Create a new library
    pub fn new(name: String, country: String, era: String, author: String) -> Self {
        Self {
            id: None,
            name,
            country,
            era,
            author,
            version: 1,
            tags: Vec::new(),
            units: Vec::new(),
        }
    }

    /// Increment version number
    pub fn increment_version(&mut self) {
        self.version += 1;
    }

    /// Set version number
    pub fn set_version(&mut self, version: i64) {
        self.version = version;
    }

    /// Add a unit to the library
    pub fn add_unit(&mut self, unit: Unit) {
        self.units.push(unit);
    }

    /// Get total personnel count across all units
    pub fn total_personnel(&self) -> usize {
        self.units.iter().map(|u| u.total_personnel()).sum()
    }
}

/// A unit represents a formation (squad, platoon, company, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Unit {
    /// Unique identifier
    pub id: Option<i64>,
    /// Unit name
    pub name: String,
    /// Unit type (e.g., "Squad", "Platoon", "Company")
    pub unit_type: String,
    /// Parent unit ID (for hierarchy)
    pub parent_id: Option<i64>,
    /// Personnel positions
    pub personnel: Vec<Personnel>,
    /// Equipment list
    pub equipment: Vec<Equipment>,
    /// Child units
    pub children: Vec<Unit>,
}

impl Unit {
    /// Create a new unit
    pub fn new(name: String, unit_type: String) -> Self {
        Self {
            id: None,
            name,
            unit_type,
            parent_id: None,
            personnel: Vec::new(),
            equipment: Vec::new(),
            children: Vec::new(),
        }
    }

    /// Add personnel position
    pub fn add_personnel(&mut self, personnel: Personnel) {
        self.personnel.push(personnel);
    }

    /// Add equipment
    pub fn add_equipment(&mut self, equipment: Equipment) {
        self.equipment.push(equipment);
    }

    /// Add child unit
    pub fn add_child(&mut self, child: Unit) {
        self.children.push(child);
    }

    /// Get total personnel count (including children)
    pub fn total_personnel(&self) -> usize {
        let direct = self.personnel.len();
        let children_total: usize = self.children.iter().map(|c| c.total_personnel()).sum();
        direct + children_total
    }

    /// Get total equipment count (including children)
    pub fn total_equipment(&self) -> HashMap<String, usize> {
        let mut totals = HashMap::new();
        
        // Add direct equipment
        for eq in &self.equipment {
            *totals.entry(eq.name.clone()).or_insert(0) += eq.quantity;
        }
        
        // Add child equipment
        for child in &self.children {
            let child_totals = child.total_equipment();
            for (name, qty) in child_totals {
                *totals.entry(name).or_insert(0) += qty;
            }
        }
        
        totals
    }
}

/// Equipment item (weapon, vehicle, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Equipment {
    /// Equipment name (e.g., "M4 Carbine", "M1 Abrams")
    pub name: String,
    /// Quantity
    pub quantity: usize,
}

impl Equipment {
    /// Create new equipment
    pub fn new(name: String, quantity: usize) -> Self {
        Self { name, quantity }
    }
}

/// Personnel position with optional rank
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Personnel {
    /// Position name (e.g., "Rifleman", "Squad Leader")
    pub position: String,
    /// Optional rank (e.g., "PFC", "SGT", "CPT")
    pub rank: Option<String>,
}

impl Personnel {
    /// Create new personnel
    pub fn new(position: String) -> Self {
        Self {
            position,
            rank: None,
        }
    }

    /// Create personnel with rank
    pub fn with_rank(position: String, rank: String) -> Self {
        Self {
            position,
            rank: Some(rank),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_creation() {
        let lib = Library::new(
            "Test".to_string(),
            "US".to_string(),
            "2003".to_string(),
            "Author".to_string(),
        );
        assert_eq!(lib.name, "Test");
        assert_eq!(lib.country, "US");
        assert_eq!(lib.era, "2003");
        assert_eq!(lib.author, "Author");
        assert_eq!(lib.version, 1);
        assert_eq!(lib.units.len(), 0);
    }

    #[test]
    fn test_library_version() {
        let mut lib = Library::new(
            "Test".to_string(),
            "US".to_string(),
            "2003".to_string(),
            "Author".to_string(),
        );
        assert_eq!(lib.version, 1);
        lib.increment_version();
        assert_eq!(lib.version, 2);
        lib.set_version(5);
        assert_eq!(lib.version, 5);
    }

    #[test]
    fn test_unit_creation() {
        let unit = Unit::new("Alpha Squad".to_string(), "Squad".to_string());
        assert_eq!(unit.name, "Alpha Squad");
        assert_eq!(unit.unit_type, "Squad");
        assert_eq!(unit.total_personnel(), 0);
    }

    #[test]
    fn test_unit_personnel_count() {
        let mut unit = Unit::new("Squad".to_string(), "Squad".to_string());
        unit.add_personnel(Personnel::new("Rifleman".to_string()));
        unit.add_personnel(Personnel::new("Squad Leader".to_string()));
        assert_eq!(unit.total_personnel(), 2);
    }

    #[test]
    fn test_unit_with_children() {
        let mut platoon = Unit::new("Platoon".to_string(), "Platoon".to_string());
        let mut squad1 = Unit::new("Squad 1".to_string(), "Squad".to_string());
        squad1.add_personnel(Personnel::new("Rifleman".to_string()));
        let mut squad2 = Unit::new("Squad 2".to_string(), "Squad".to_string());
        squad2.add_personnel(Personnel::new("Rifleman".to_string()));
        
        platoon.add_child(squad1);
        platoon.add_child(squad2);
        
        assert_eq!(platoon.total_personnel(), 2);
    }
}
