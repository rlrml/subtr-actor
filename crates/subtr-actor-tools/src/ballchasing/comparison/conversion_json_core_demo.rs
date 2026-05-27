use serde_json::Value;

use super::super::super::comparable_types::{ComparableCoreStats, ComparableDemoStats};
use super::json_number;

pub(crate) fn comparable_core_from_json(stats: Option<&Value>) -> ComparableCoreStats {
    ComparableCoreStats {
        score: json_number(stats, "score"),
        goals: json_number(stats, "goals"),
        assists: json_number(stats, "assists"),
        saves: json_number(stats, "saves"),
        shots: json_number(stats, "shots"),
        shooting_percentage: json_number(stats, "shooting_percentage"),
    }
}

pub(crate) fn comparable_demo_from_json(stats: Option<&Value>) -> ComparableDemoStats {
    ComparableDemoStats {
        inflicted: json_number(stats, "inflicted"),
        taken: json_number(stats, "taken"),
    }
}

pub(crate) fn comparable_team_demo_from_json(stats: Option<&Value>) -> ComparableDemoStats {
    ComparableDemoStats {
        inflicted: json_number(stats, "inflicted"),
        taken: None,
    }
}
