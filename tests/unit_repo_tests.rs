//! Comprehensive tests for UnitRepo

use toeditor::db::Database;
use toeditor::db::repositories::{LibraryRepo, UnitRepo};
use toeditor::models::{Library, Unit, Equipment, Personnel};

#[test]
fn test_create_unit_with_personnel_and_equipment() {
    let db = Database::open_in_memory().unwrap();
    let lib_repo = LibraryRepo::new(db.conn());
    let mut library = Library::new(
        "Test".to_string(),
        "US".to_string(),
        "2003".to_string(),
        "Author".to_string(),
    );
    lib_repo.create(&mut library).unwrap();
    
    let repo = UnitRepo::new(db.conn());
    let mut unit = Unit::new("Squad".to_string(), "Squad".to_string());
    unit.add_personnel(Personnel::new("Rifleman".to_string()));
    unit.add_personnel(Personnel::with_rank("Squad Leader".to_string(), "SGT".to_string()));
    unit.add_equipment(Equipment::new("M4 Carbine".to_string(), 9));
    unit.add_equipment(Equipment::new("M249 SAW".to_string(), 1));
    
    repo.create(library.id.unwrap(), &mut unit).unwrap();
    assert!(unit.id.is_some());
    
    // Verify unit was saved with all data
    let retrieved = repo.get_by_id(unit.id.unwrap()).unwrap();
    assert!(retrieved.is_some());
    let u = retrieved.unwrap();
    assert_eq!(u.name, "Squad");
    assert_eq!(u.personnel.len(), 2);
    assert_eq!(u.equipment.len(), 2);
    assert_eq!(u.personnel[0].position, "Rifleman");
    assert_eq!(u.personnel[1].position, "Squad Leader");
    assert_eq!(u.personnel[1].rank, Some("SGT".to_string()));
    assert_eq!(u.equipment[0].name, "M4 Carbine");
    assert_eq!(u.equipment[0].quantity, 9);
}

#[test]
fn test_get_by_id_with_children() {
    let db = Database::open_in_memory().unwrap();
    let lib_repo = LibraryRepo::new(db.conn());
    let mut library = Library::new(
        "Test".to_string(),
        "US".to_string(),
        "2003".to_string(),
        "Author".to_string(),
    );
    lib_repo.create(&mut library).unwrap();
    
    let repo = UnitRepo::new(db.conn());
    
    // Create parent unit
    let mut platoon = Unit::new("Platoon".to_string(), "Platoon".to_string());
    repo.create(library.id.unwrap(), &mut platoon).unwrap();
    let platoon_id = platoon.id.unwrap();
    
    // Create child units
    let mut squad1 = Unit::new("Squad 1".to_string(), "Squad".to_string());
    squad1.parent_id = Some(platoon_id);
    repo.create(library.id.unwrap(), &mut squad1).unwrap();
    
    let mut squad2 = Unit::new("Squad 2".to_string(), "Squad".to_string());
    squad2.parent_id = Some(platoon_id);
    repo.create(library.id.unwrap(), &mut squad2).unwrap();
    
    // Retrieve parent and verify children are loaded
    let retrieved = repo.get_by_id(platoon_id).unwrap();
    assert!(retrieved.is_some());
    let p = retrieved.unwrap();
    assert_eq!(p.children.len(), 2);
    assert_eq!(p.children[0].name, "Squad 1");
    assert_eq!(p.children[1].name, "Squad 2");
}

#[test]
fn test_get_by_library_id() {
    let db = Database::open_in_memory().unwrap();
    let lib_repo = LibraryRepo::new(db.conn());
    
    let mut lib1 = Library::new(
        "Library 1".to_string(),
        "US".to_string(),
        "2003".to_string(),
        "Author".to_string(),
    );
    lib_repo.create(&mut lib1).unwrap();
    
    let mut lib2 = Library::new(
        "Library 2".to_string(),
        "RU".to_string(),
        "2020".to_string(),
        "Author".to_string(),
    );
    lib_repo.create(&mut lib2).unwrap();
    
    let repo = UnitRepo::new(db.conn());
    
    // Create units for library 1
    let mut unit1 = Unit::new("Unit 1".to_string(), "Company".to_string());
    repo.create(lib1.id.unwrap(), &mut unit1).unwrap();
    
    let mut unit2 = Unit::new("Unit 2".to_string(), "Battalion".to_string());
    repo.create(lib1.id.unwrap(), &mut unit2).unwrap();
    
    // Create unit for library 2
    let mut unit3 = Unit::new("Unit 3".to_string(), "Company".to_string());
    repo.create(lib2.id.unwrap(), &mut unit3).unwrap();
    
    // Get units for library 1
    let units = repo.get_by_library_id(lib1.id.unwrap()).unwrap();
    assert_eq!(units.len(), 2);
    assert!(units.iter().any(|u| u.name == "Unit 1"));
    assert!(units.iter().any(|u| u.name == "Unit 2"));
    
    // Get units for library 2
    let units = repo.get_by_library_id(lib2.id.unwrap()).unwrap();
    assert_eq!(units.len(), 1);
    assert_eq!(units[0].name, "Unit 3");
}

#[test]
fn test_get_by_id_nonexistent() {
    let db = Database::open_in_memory().unwrap();
    let repo = UnitRepo::new(db.conn());
    
    let result = repo.get_by_id(999).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_create_personnel() {
    let db = Database::open_in_memory().unwrap();
    let lib_repo = LibraryRepo::new(db.conn());
    let mut library = Library::new(
        "Test".to_string(),
        "US".to_string(),
        "2003".to_string(),
        "Author".to_string(),
    );
    lib_repo.create(&mut library).unwrap();
    
    let repo = UnitRepo::new(db.conn());
    let mut unit = Unit::new("Squad".to_string(), "Squad".to_string());
    repo.create(library.id.unwrap(), &mut unit).unwrap();
    
    // Add personnel directly
    let mut personnel = Personnel::with_rank("Commander".to_string(), "CPT".to_string());
    repo.create_personnel(unit.id.unwrap(), &mut personnel).unwrap();
    
    // Verify personnel was saved
    let retrieved = repo.get_by_id(unit.id.unwrap()).unwrap().unwrap();
    assert_eq!(retrieved.personnel.len(), 1);
    assert_eq!(retrieved.personnel[0].position, "Commander");
    assert_eq!(retrieved.personnel[0].rank, Some("CPT".to_string()));
}

#[test]
fn test_create_equipment() {
    let db = Database::open_in_memory().unwrap();
    let lib_repo = LibraryRepo::new(db.conn());
    let mut library = Library::new(
        "Test".to_string(),
        "US".to_string(),
        "2003".to_string(),
        "Author".to_string(),
    );
    lib_repo.create(&mut library).unwrap();
    
    let repo = UnitRepo::new(db.conn());
    let mut unit = Unit::new("Squad".to_string(), "Squad".to_string());
    repo.create(library.id.unwrap(), &mut unit).unwrap();
    
    // Add equipment directly
    let equipment = Equipment::new("M1 Abrams".to_string(), 4);
    repo.create_equipment(unit.id.unwrap(), &equipment).unwrap();
    
    // Verify equipment was saved
    let retrieved = repo.get_by_id(unit.id.unwrap()).unwrap().unwrap();
    assert_eq!(retrieved.equipment.len(), 1);
    assert_eq!(retrieved.equipment[0].name, "M1 Abrams");
    assert_eq!(retrieved.equipment[0].quantity, 4);
}
