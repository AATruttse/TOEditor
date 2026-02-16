//! Formation hierarchy levels: standard (fixed) and custom (per-library).

use serde::{Deserialize, Serialize};

/// Ordinal for standard formation levels (0 = fire team, 11 = front).
pub const STANDARD_LEVEL_COUNT: usize = 12;

/// Standard military formation levels in hierarchical order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(i32)]
pub enum StandardFormationLevel {
    FireTeam = 0,
    Squad = 1,
    Section = 2,
    Platoon = 3,
    Company = 4,
    Battalion = 5,
    Regiment = 6,
    Brigade = 7,
    Division = 8,
    Corps = 9,
    Army = 10,
    Front = 11,
}

impl StandardFormationLevel {
    /// English name.
    pub fn name_en(&self) -> &'static str {
        match self {
            Self::FireTeam => "fire team",
            Self::Squad => "squad",
            Self::Section => "section",
            Self::Platoon => "platoon",
            Self::Company => "company",
            Self::Battalion => "battalion",
            Self::Regiment => "regiment",
            Self::Brigade => "brigade",
            Self::Division => "division",
            Self::Corps => "corps",
            Self::Army => "army",
            Self::Front => "front",
        }
    }

    /// Russian name.
    pub fn name_ru(&self) -> &'static str {
        match self {
            Self::FireTeam => "огневая группа",
            Self::Squad => "отделение",
            Self::Section => "секция",
            Self::Platoon => "взвод",
            Self::Company => "рота",
            Self::Battalion => "батальон",
            Self::Regiment => "полк",
            Self::Brigade => "бригада",
            Self::Division => "дивизия",
            Self::Corps => "корпус",
            Self::Army => "армия",
            Self::Front => "фронт",
        }
    }

    /// Ordinal (0..=11).
    pub fn ordinal(&self) -> i32 {
        *self as i32
    }

    /// All standard levels in order.
    pub fn all() -> [StandardFormationLevel; STANDARD_LEVEL_COUNT] {
        [
            Self::FireTeam,
            Self::Squad,
            Self::Section,
            Self::Platoon,
            Self::Company,
            Self::Battalion,
            Self::Regiment,
            Self::Brigade,
            Self::Division,
            Self::Corps,
            Self::Army,
            Self::Front,
        ]
    }

    /// From ordinal; returns None if out of range.
    pub fn from_ordinal(n: i32) -> Option<Self> {
        match n {
            0 => Some(Self::FireTeam),
            1 => Some(Self::Squad),
            2 => Some(Self::Section),
            3 => Some(Self::Platoon),
            4 => Some(Self::Company),
            5 => Some(Self::Battalion),
            6 => Some(Self::Regiment),
            7 => Some(Self::Brigade),
            8 => Some(Self::Division),
            9 => Some(Self::Corps),
            10 => Some(Self::Army),
            11 => Some(Self::Front),
            _ => None,
        }
    }
}

/// Custom formation level for a library: custom name(s) mapped to a standard level.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CustomFormationLevel {
    pub id: Option<i64>,
    pub library_id: i64,
    /// Russian name (e.g. "отделение")
    pub name_ru: String,
    /// English name (e.g. "squad")
    pub name_en: String,
    /// Which standard level this custom level corresponds to (0..=11).
    pub standard_level_ordinal: i32,
}

impl CustomFormationLevel {
    pub fn new(library_id: i64, name_ru: String, name_en: String, standard_level_ordinal: i32) -> Self {
        Self {
            id: None,
            library_id,
            name_ru,
            name_en,
            standard_level_ordinal,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_level_count() {
        assert_eq!(STANDARD_LEVEL_COUNT, 12);
        assert_eq!(StandardFormationLevel::all().len(), STANDARD_LEVEL_COUNT);
    }

    #[test]
    fn test_standard_level_names_en() {
        assert_eq!(StandardFormationLevel::FireTeam.name_en(), "fire team");
        assert_eq!(StandardFormationLevel::Squad.name_en(), "squad");
        assert_eq!(StandardFormationLevel::Section.name_en(), "section");
        assert_eq!(StandardFormationLevel::Platoon.name_en(), "platoon");
        assert_eq!(StandardFormationLevel::Company.name_en(), "company");
        assert_eq!(StandardFormationLevel::Battalion.name_en(), "battalion");
        assert_eq!(StandardFormationLevel::Regiment.name_en(), "regiment");
        assert_eq!(StandardFormationLevel::Brigade.name_en(), "brigade");
        assert_eq!(StandardFormationLevel::Division.name_en(), "division");
        assert_eq!(StandardFormationLevel::Corps.name_en(), "corps");
        assert_eq!(StandardFormationLevel::Army.name_en(), "army");
        assert_eq!(StandardFormationLevel::Front.name_en(), "front");
    }

    #[test]
    fn test_standard_level_names_ru() {
        assert_eq!(StandardFormationLevel::FireTeam.name_ru(), "огневая группа");
        assert_eq!(StandardFormationLevel::Squad.name_ru(), "отделение");
        assert_eq!(StandardFormationLevel::Platoon.name_ru(), "взвод");
        assert_eq!(StandardFormationLevel::Company.name_ru(), "рота");
        assert_eq!(StandardFormationLevel::Battalion.name_ru(), "батальон");
        assert_eq!(StandardFormationLevel::Front.name_ru(), "фронт");
    }

    #[test]
    fn test_standard_level_ordinal() {
        assert_eq!(StandardFormationLevel::FireTeam.ordinal(), 0);
        assert_eq!(StandardFormationLevel::Squad.ordinal(), 1);
        assert_eq!(StandardFormationLevel::Front.ordinal(), 11);
    }

    #[test]
    fn test_standard_level_from_ordinal_valid() {
        for level in StandardFormationLevel::all() {
            let recovered = StandardFormationLevel::from_ordinal(level.ordinal());
            assert_eq!(recovered, Some(level));
        }
    }

    #[test]
    fn test_standard_level_from_ordinal_invalid() {
        assert_eq!(StandardFormationLevel::from_ordinal(-1), None);
        assert_eq!(StandardFormationLevel::from_ordinal(12), None);
        assert_eq!(StandardFormationLevel::from_ordinal(100), None);
    }

    #[test]
    fn test_standard_level_all_ordered() {
        let all = StandardFormationLevel::all();
        for i in 0..all.len() {
            assert_eq!(all[i].ordinal(), i as i32);
        }
    }

    #[test]
    fn test_custom_formation_level_new() {
        let level = CustomFormationLevel::new(1, "взвод".to_string(), "platoon".to_string(), 3);
        assert_eq!(level.id, None);
        assert_eq!(level.library_id, 1);
        assert_eq!(level.name_ru, "взвод");
        assert_eq!(level.name_en, "platoon");
        assert_eq!(level.standard_level_ordinal, 3);
    }

    #[test]
    fn test_custom_formation_level_serialization() {
        let level = CustomFormationLevel::new(1, "рота".to_string(), "company".to_string(), 4);
        let json = serde_json::to_string(&level).unwrap();
        let deserialized: CustomFormationLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(level, deserialized);
    }
}
