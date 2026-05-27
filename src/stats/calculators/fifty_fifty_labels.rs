use super::*;

pub(super) const FIFTY_FIFTY_PHASE_LABELS: [StatLabel; 2] = [
    StatLabel::new("phase", "open_play"),
    StatLabel::new("phase", "kickoff"),
];
pub(super) const FIFTY_FIFTY_TEAM_OUTCOME_LABELS: [StatLabel; 3] = [
    StatLabel::new("winning_team", "team_zero"),
    StatLabel::new("winning_team", "team_one"),
    StatLabel::new("winning_team", "neutral"),
];
pub(super) const FIFTY_FIFTY_POSSESSION_LABELS: [StatLabel; 3] = [
    StatLabel::new("possession_after", "team_zero"),
    StatLabel::new("possession_after", "team_one"),
    StatLabel::new("possession_after", "neutral"),
];
pub(super) const FIFTY_FIFTY_PLAYER_OUTCOME_LABELS: [StatLabel; 3] = [
    StatLabel::new("outcome", "win"),
    StatLabel::new("outcome", "loss"),
    StatLabel::new("outcome", "neutral"),
];
pub(super) const FIFTY_FIFTY_PLAYER_POSSESSION_LABELS: [StatLabel; 3] = [
    StatLabel::new("possession_after", "self"),
    StatLabel::new("possession_after", "opponent"),
    StatLabel::new("possession_after", "neutral"),
];
pub(super) const FIFTY_FIFTY_TOUCH_DODGE_STATE_LABELS: [StatLabel; 2] = [
    StatLabel::new("dodge_state", "no_dodge"),
    StatLabel::new("dodge_state", "dodge"),
];
pub(super) const FIFTY_FIFTY_TEAM_ZERO_DODGE_STATE_LABELS: [StatLabel; 2] = [
    StatLabel::new("team_zero_dodge_state", "no_dodge"),
    StatLabel::new("team_zero_dodge_state", "dodge"),
];
pub(super) const FIFTY_FIFTY_TEAM_ONE_DODGE_STATE_LABELS: [StatLabel; 2] = [
    StatLabel::new("team_one_dodge_state", "no_dodge"),
    StatLabel::new("team_one_dodge_state", "dodge"),
];

pub(super) fn fifty_fifty_phase_label(is_kickoff: bool) -> StatLabel {
    if is_kickoff {
        StatLabel::new("phase", "kickoff")
    } else {
        StatLabel::new("phase", "open_play")
    }
}

pub(super) fn fifty_fifty_team_outcome_label(team_is_team_0: Option<bool>) -> StatLabel {
    match team_is_team_0 {
        Some(true) => StatLabel::new("winning_team", "team_zero"),
        Some(false) => StatLabel::new("winning_team", "team_one"),
        None => StatLabel::new("winning_team", "neutral"),
    }
}

pub(super) fn fifty_fifty_possession_label(team_is_team_0: Option<bool>) -> StatLabel {
    match team_is_team_0 {
        Some(true) => StatLabel::new("possession_after", "team_zero"),
        Some(false) => StatLabel::new("possession_after", "team_one"),
        None => StatLabel::new("possession_after", "neutral"),
    }
}

pub(super) fn fifty_fifty_player_outcome_label(
    player_team_is_team_0: bool,
    winning_team_is_team_0: Option<bool>,
) -> StatLabel {
    match winning_team_is_team_0 {
        Some(team_is_team_0) if team_is_team_0 == player_team_is_team_0 => {
            StatLabel::new("outcome", "win")
        }
        Some(_) => StatLabel::new("outcome", "loss"),
        None => StatLabel::new("outcome", "neutral"),
    }
}

pub(super) fn fifty_fifty_player_possession_label(
    player_team_is_team_0: bool,
    possession_team_is_team_0: Option<bool>,
) -> StatLabel {
    match possession_team_is_team_0 {
        Some(team_is_team_0) if team_is_team_0 == player_team_is_team_0 => {
            StatLabel::new("possession_after", "self")
        }
        Some(_) => StatLabel::new("possession_after", "opponent"),
        None => StatLabel::new("possession_after", "neutral"),
    }
}

pub(super) fn fifty_fifty_touch_dodge_state_label(dodge_contact: bool) -> StatLabel {
    if dodge_contact {
        StatLabel::new("dodge_state", "dodge")
    } else {
        StatLabel::new("dodge_state", "no_dodge")
    }
}

pub(super) fn fifty_fifty_team_zero_dodge_state_label(dodge_contact: bool) -> StatLabel {
    if dodge_contact {
        StatLabel::new("team_zero_dodge_state", "dodge")
    } else {
        StatLabel::new("team_zero_dodge_state", "no_dodge")
    }
}

pub(super) fn fifty_fifty_team_one_dodge_state_label(dodge_contact: bool) -> StatLabel {
    if dodge_contact {
        StatLabel::new("team_one_dodge_state", "dodge")
    } else {
        StatLabel::new("team_one_dodge_state", "no_dodge")
    }
}
