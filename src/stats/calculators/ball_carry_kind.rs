use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum BallCarryKind {
    Carry,
    AirDribble,
}

pub(crate) const BALL_CARRY_KIND_LABELS: [StatLabel; 2] = [
    StatLabel::new("kind", "carry"),
    StatLabel::new("kind", "air_dribble"),
];

pub(crate) fn ball_carry_kind_label(kind: BallCarryKind) -> StatLabel {
    match kind {
        BallCarryKind::Carry => StatLabel::new("kind", "carry"),
        BallCarryKind::AirDribble => StatLabel::new("kind", "air_dribble"),
    }
}
