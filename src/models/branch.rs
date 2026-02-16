//! Branches of service (роды войск) and their categories per library.

use serde::{Deserialize, Serialize};

/// A branch of service for a library (e.g. Infantry, Armor).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Branch {
    pub id: Option<i64>,
    pub library_id: i64,
    /// Optional category (e.g. "Боевые", "ПВО").
    pub category_id: Option<i64>,
    pub name_ru: String,
    pub name_en: String,
}

impl Branch {
    pub fn new(library_id: i64, name_ru: String, name_en: String) -> Self {
        Self {
            id: None,
            library_id,
            category_id: None,
            name_ru,
            name_en,
        }
    }

    pub fn with_category(library_id: i64, category_id: Option<i64>, name_ru: String, name_en: String) -> Self {
        Self {
            id: None,
            library_id,
            category_id,
            name_ru,
            name_en,
        }
    }
}

/// Category of branches (e.g. "Боевые", "ПВО") — per library, editable, exportable.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BranchCategory {
    pub id: Option<i64>,
    pub library_id: i64,
    pub name_ru: String,
    pub name_en: String,
}

impl BranchCategory {
    pub fn new(library_id: i64, name_ru: String, name_en: String) -> Self {
        Self {
            id: None,
            library_id,
            name_ru,
            name_en,
        }
    }
}

/// Default branch categories for every new library.
pub fn default_branch_categories(library_id: i64) -> Vec<BranchCategory> {
    let pairs: &[(&str, &str)] = &[
        ("Боевые", "Combat"),
        ("Артиллерия", "Artillery"),
        ("ПВО", "Air defense"),
        ("Армейская авиация", "Army aviation"),
        ("Боевое обеспечение", "Combat support"),
        ("Тыловое обеспечение", "Logistics support"),
    ];
    pairs
        .iter()
        .map(|(ru, en)| BranchCategory::new(library_id, ru.to_string(), en.to_string()))
        .collect()
}

/// Default branches created for every new library, with category index (0..=5).
/// Can be edited per library (add/remove, e.g. remove Cavalry).
/// Category indices: 0=Боевые, 1=Артиллерия, 2=ПВО, 3=Армейская авиация, 4=Боевое обеспечение, 5=Тыловое обеспечение.
pub fn default_branches(library_id: i64) -> Vec<(Branch, usize)> {
    let items: &[(&str, &str, usize)] = &[
        ("Пехота", "Infantry", 0),
        ("Бронетанковые войска", "Armor", 0),
        ("Артиллерия", "Artillery", 1),
        ("Ракетные войска", "Rocket forces", 0),
        ("Авиация", "Aviation", 3),
        ("ВМФ", "Navy", 0),
        ("Войска ПВО", "Air defense", 2),
        ("Инженерные войска", "Engineers", 4),
        ("Войска связи", "Signals", 4),
        ("Тыл", "Logistics", 5),
        ("Кавалерия", "Cavalry", 0),
        ("Разведка", "Reconnaissance", 0),
        ("РХБЗ", "Chemical defense", 4),
    ];
    items
        .iter()
        .map(|(ru, en, cat_idx)| {
            let b = Branch::new(library_id, ru.to_string(), en.to_string());
            (b, *cat_idx)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_branch_new() {
        let b = Branch::new(1, "Пехота".to_string(), "Infantry".to_string());
        assert_eq!(b.id, None);
        assert_eq!(b.library_id, 1);
        assert_eq!(b.category_id, None);
        assert_eq!(b.name_ru, "Пехота");
        assert_eq!(b.name_en, "Infantry");
    }

    #[test]
    fn test_branch_with_category() {
        let b = Branch::with_category(1, Some(5), "Пехота".to_string(), "Infantry".to_string());
        assert_eq!(b.id, None);
        assert_eq!(b.library_id, 1);
        assert_eq!(b.category_id, Some(5));
        assert_eq!(b.name_ru, "Пехота");
        assert_eq!(b.name_en, "Infantry");
    }

    #[test]
    fn test_branch_with_category_none() {
        let b = Branch::with_category(1, None, "Пехота".to_string(), "Infantry".to_string());
        assert_eq!(b.category_id, None);
    }

    #[test]
    fn test_branch_serialization() {
        let b = Branch::with_category(1, Some(3), "Пехота".to_string(), "Infantry".to_string());
        let json = serde_json::to_string(&b).unwrap();
        let deserialized: Branch = serde_json::from_str(&json).unwrap();
        assert_eq!(b, deserialized);
    }

    #[test]
    fn test_branch_category_new() {
        let c = BranchCategory::new(1, "Боевые".to_string(), "Combat".to_string());
        assert_eq!(c.id, None);
        assert_eq!(c.library_id, 1);
        assert_eq!(c.name_ru, "Боевые");
        assert_eq!(c.name_en, "Combat");
    }

    #[test]
    fn test_branch_category_serialization() {
        let c = BranchCategory::new(1, "ПВО".to_string(), "Air defense".to_string());
        let json = serde_json::to_string(&c).unwrap();
        let deserialized: BranchCategory = serde_json::from_str(&json).unwrap();
        assert_eq!(c, deserialized);
    }

    #[test]
    fn test_default_branch_categories() {
        let cats = default_branch_categories(1);
        assert_eq!(cats.len(), 6);
        assert!(cats.iter().all(|c| c.library_id == 1));
        assert!(cats.iter().all(|c| c.id.is_none()));
        assert!(cats.iter().any(|c| c.name_en == "Combat"));
        assert!(cats.iter().any(|c| c.name_en == "Artillery"));
        assert!(cats.iter().any(|c| c.name_en == "Air defense"));
        assert!(cats.iter().any(|c| c.name_en == "Army aviation"));
        assert!(cats.iter().any(|c| c.name_en == "Combat support"));
        assert!(cats.iter().any(|c| c.name_en == "Logistics support"));
    }

    #[test]
    fn test_default_branches() {
        let branches = default_branches(1);
        assert!(branches.len() >= 10);
        assert!(branches.iter().all(|(b, _)| b.library_id == 1));
        assert!(branches.iter().all(|(b, _)| b.id.is_none()));
        assert!(branches.iter().all(|(b, _)| b.category_id.is_none()));
        assert!(branches.iter().any(|(b, _)| b.name_en == "Infantry"));
        assert!(branches.iter().any(|(b, _)| b.name_en == "Armor"));
        assert!(branches.iter().any(|(b, _)| b.name_en == "Cavalry"));
    }

    #[test]
    fn test_default_branches_category_indices_valid() {
        let categories = default_branch_categories(1);
        let branches = default_branches(1);
        for (_, cat_idx) in &branches {
            assert!(*cat_idx < categories.len(),
                "Category index {} out of range (max {})", cat_idx, categories.len() - 1);
        }
    }
}
