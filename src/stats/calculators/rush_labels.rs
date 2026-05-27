use super::*;

pub(super) const RUSH_TEAM_LABELS: [StatLabel; 2] = [
    StatLabel::new("team", "team_zero"),
    StatLabel::new("team", "team_one"),
];
pub(super) const RUSH_ATTACKER_LABELS: [StatLabel; 2] = [
    StatLabel::new("attackers", "2"),
    StatLabel::new("attackers", "3"),
];
pub(super) const RUSH_DEFENDER_LABELS: [StatLabel; 3] = [
    StatLabel::new("defenders", "1"),
    StatLabel::new("defenders", "2"),
    StatLabel::new("defenders", "3"),
];

pub(super) fn rush_team_label(is_team_0: bool) -> StatLabel {
    if is_team_0 {
        StatLabel::new("team", "team_zero")
    } else {
        StatLabel::new("team", "team_one")
    }
}

pub(super) fn rush_attackers_label(attackers: usize) -> StatLabel {
    StatLabel::new(
        "attackers",
        match attackers {
            2 => "2",
            3 => "3",
            _ => "other",
        },
    )
}

pub(super) fn rush_defenders_label(defenders: usize) -> StatLabel {
    StatLabel::new(
        "defenders",
        match defenders {
            1 => "1",
            2 => "2",
            3 => "3",
            _ => "other",
        },
    )
}
