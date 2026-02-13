//! Comprehensive tests for model methods

use toeditor::models::{Library, Unit, Equipment, Personnel};

#[test]
fn test_equipment_new() {
    let eq = Equipment::new("M4 Carbine".to_string(), 10);
    assert_eq!(eq.name, "M4 Carbine");
    assert_eq!(eq.quantity, 10);
}

#[test]
fn test_personnel_new() {
    let p = Personnel::new("Rifleman".to_string());
    assert_eq!(p.position, "Rifleman");
    assert_eq!(p.rank, None);
}

#[test]
fn test_personnel_with_rank() {
    let p = Personnel::with_rank("Squad Leader".to_string(), "SGT".to_string());
    assert_eq!(p.position, "Squad Leader");
    assert_eq!(p.rank, Some("SGT".to_string()));
}

#[test]
fn test_unit_total_equipment() {
    let mut unit = Unit::new("Squad".to_string(), "Squad".to_string());
    
    // Add direct equipment
    unit.add_equipment(Equipment::new("M4 Carbine".to_string(), 9));
    unit.add_equipment(Equipment::new("M249 SAW".to_string(), 1));
    
    let totals = unit.total_equipment();
    assert_eq!(totals.get("M4 Carbine"), Some(&9));
    assert_eq!(totals.get("M249 SAW"), Some(&1));
}

#[test]
fn test_unit_total_equipment_with_children() {
    let mut platoon = Unit::new("Platoon".to_string(), "Platoon".to_string());
    
    // Add direct equipment
    platoon.add_equipment(Equipment::new("Radio".to_string(), 1));
    
    // Add child units with equipment
    let mut squad1 = Unit::new("Squad 1".to_string(), "Squad".to_string());
    squad1.add_equipment(Equipment::new("M4 Carbine".to_string(), 9));
    squad1.add_equipment(Equipment::new("M249 SAW".to_string(), 1));
    
    let mut squad2 = Unit::new("Squad 2".to_string(), "Squad".to_string());
    squad2.add_equipment(Equipment::new("M4 Carbine".to_string(), 9));
    squad2.add_equipment(Equipment::new("M249 SAW".to_string(), 1));
    
    platoon.add_child(squad1);
    platoon.add_child(squad2);
    
    let totals = platoon.total_equipment();
    assert_eq!(totals.get("Radio"), Some(&1));
    assert_eq!(totals.get("M4 Carbine"), Some(&18)); // 9 + 9
    assert_eq!(totals.get("M249 SAW"), Some(&2)); // 1 + 1
}

#[test]
fn test_unit_total_equipment_nested_children() {
    let mut company = Unit::new("Company".to_string(), "Company".to_string());
    
    let mut platoon = Unit::new("Platoon".to_string(), "Platoon".to_string());
    platoon.add_equipment(Equipment::new("Radio".to_string(), 1));
    
    let mut squad = Unit::new("Squad".to_string(), "Squad".to_string());
    squad.add_equipment(Equipment::new("M4 Carbine".to_string(), 9));
    platoon.add_child(squad);
    
    company.add_child(platoon);
    
    let totals = company.total_equipment();
    assert_eq!(totals.get("Radio"), Some(&1));
    assert_eq!(totals.get("M4 Carbine"), Some(&9));
}

#[test]
fn test_library_total_personnel() {
    let mut library = Library::new(
        "Test".to_string(),
        "US".to_string(),
        "2003".to_string(),
        "Author".to_string(),
    );
    
    let mut unit1 = Unit::new("Unit 1".to_string(), "Company".to_string());
    unit1.add_personnel(Personnel::new("Commander".to_string()));
    unit1.add_personnel(Personnel::new("XO".to_string()));
    
    let mut unit2 = Unit::new("Unit 2".to_string(), "Battalion".to_string());
    unit2.add_personnel(Personnel::new("Commander".to_string()));
    
    library.add_unit(unit1);
    library.add_unit(unit2);
    
    assert_eq!(library.total_personnel(), 3);
}

#[test]
fn test_library_total_personnel_with_children() {
    let mut library = Library::new(
        "Test".to_string(),
        "US".to_string(),
        "2003".to_string(),
        "Author".to_string(),
    );
    
    let mut platoon = Unit::new("Platoon".to_string(), "Platoon".to_string());
    platoon.add_personnel(Personnel::new("Platoon Leader".to_string()));
    
    let mut squad1 = Unit::new("Squad 1".to_string(), "Squad".to_string());
    squad1.add_personnel(Personnel::new("Rifleman".to_string()));
    squad1.add_personnel(Personnel::new("Rifleman".to_string()));
    
    let mut squad2 = Unit::new("Squad 2".to_string(), "Squad".to_string());
    squad2.add_personnel(Personnel::new("Rifleman".to_string()));
    
    platoon.add_child(squad1);
    platoon.add_child(squad2);
    
    library.add_unit(platoon);
    
    // Total: 1 (platoon leader) + 2 (squad1) + 1 (squad2) = 4
    assert_eq!(library.total_personnel(), 4);
}

#[test]
fn test_library_add_unit() {
    let mut library = Library::new(
        "Test".to_string(),
        "US".to_string(),
        "2003".to_string(),
        "Author".to_string(),
    );
    
    assert_eq!(library.units.len(), 0);
    
    let unit = Unit::new("Unit 1".to_string(), "Company".to_string());
    library.add_unit(unit);
    
    assert_eq!(library.units.len(), 1);
    assert_eq!(library.units[0].name, "Unit 1");
}

#[test]
fn test_unit_add_child() {
    let mut parent = Unit::new("Parent".to_string(), "Company".to_string());
    let child = Unit::new("Child".to_string(), "Platoon".to_string());
    
    assert_eq!(parent.children.len(), 0);
    parent.add_child(child);
    assert_eq!(parent.children.len(), 1);
    assert_eq!(parent.children[0].name, "Child");
}

#[test]
fn test_unit_add_personnel() {
    let mut unit = Unit::new("Squad".to_string(), "Squad".to_string());
    assert_eq!(unit.personnel.len(), 0);
    
    unit.add_personnel(Personnel::new("Rifleman".to_string()));
    assert_eq!(unit.personnel.len(), 1);
    assert_eq!(unit.personnel[0].position, "Rifleman");
}

#[test]
fn test_unit_add_equipment() {
    let mut unit = Unit::new("Squad".to_string(), "Squad".to_string());
    assert_eq!(unit.equipment.len(), 0);
    
    unit.add_equipment(Equipment::new("M4 Carbine".to_string(), 10));
    assert_eq!(unit.equipment.len(), 1);
    assert_eq!(unit.equipment[0].name, "M4 Carbine");
    assert_eq!(unit.equipment[0].quantity, 10);
}
