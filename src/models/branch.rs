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
