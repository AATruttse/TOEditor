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
