//! Branches of service (роды войск) per library.

use serde::{Deserialize, Serialize};

/// A branch of service for a library (e.g. Infantry, Armor).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Branch {
    pub id: Option<i64>,
    pub library_id: i64,
    pub name_ru: String,
    pub name_en: String,
}

impl Branch {
    pub fn new(library_id: i64, name_ru: String, name_en: String) -> Self {
        Self {
            id: None,
            library_id,
            name_ru,
            name_en,
        }
    }
}

/// Default branches created for every new library.
/// Can be edited per library (add/remove, e.g. remove Cavalry).
pub fn default_branches(library_id: i64) -> Vec<Branch> {
    let pairs: &[(&str, &str)] = &[
        ("Пехота", "Infantry"),
        ("Бронетанковые войска", "Armor"),
        ("Артиллерия", "Artillery"),
        ("Ракетные войска", "Rocket forces"),
        ("Авиация", "Aviation"),
        ("ВМФ", "Navy"),
        ("Войска ПВО", "Air defense"),
        ("Инженерные войска", "Engineers"),
        ("Войска связи", "Signals"),
        ("Тыл", "Logistics"),
        ("Кавалерия", "Cavalry"),
        ("Разведка", "Reconnaissance"),
        ("РХБЗ", "Chemical defense"),
    ];
    pairs
        .iter()
        .map(|(ru, en)| Branch::new(library_id, ru.to_string(), en.to_string()))
        .collect()
}
