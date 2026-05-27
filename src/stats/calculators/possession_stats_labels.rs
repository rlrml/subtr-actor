use super::*;

pub(super) fn team_relative_labeled_time(
    labeled_time: &LabeledFloatSums,
    is_team_zero: bool,
) -> LabeledFloatSums {
    let mut relative_labeled_time = LabeledFloatSums::default();
    for entry in &labeled_time.entries {
        relative_labeled_time.add(
            entry
                .labels
                .iter()
                .map(|label| team_relative_possession_label(label, is_team_zero)),
            entry.value,
        );
    }
    relative_labeled_time
}

fn team_relative_possession_label(label: &StatLabel, is_team_zero: bool) -> StatLabel {
    match (label.key, label.value) {
        ("possession_state", "team_zero") => StatLabel::new(
            "possession_state",
            if is_team_zero { "own" } else { "opponent" },
        ),
        ("possession_state", "team_one") => StatLabel::new(
            "possession_state",
            if is_team_zero { "opponent" } else { "own" },
        ),
        ("field_third", "team_zero_third") => StatLabel::new(
            "field_third",
            if is_team_zero {
                "defensive_third"
            } else {
                "offensive_third"
            },
        ),
        ("field_third", "team_one_third") => StatLabel::new(
            "field_third",
            if is_team_zero {
                "offensive_third"
            } else {
                "defensive_third"
            },
        ),
        _ => label.clone(),
    }
}
