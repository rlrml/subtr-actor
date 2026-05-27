use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum AirDribbleOrigin {
    GroundToAir,
    WallToAir,
}

pub(crate) const AIR_DRIBBLE_ORIGIN_LABELS: [StatLabel; 2] = [
    StatLabel::new("origin", "ground_to_air"),
    StatLabel::new("origin", "wall_to_air"),
];

pub(crate) fn air_dribble_origin_label(origin: AirDribbleOrigin) -> StatLabel {
    StatLabel::new("origin", origin.as_label_value())
}

impl AirDribbleOrigin {
    pub fn as_label_value(self) -> &'static str {
        match self {
            Self::GroundToAir => "ground_to_air",
            Self::WallToAir => "wall_to_air",
        }
    }
}
