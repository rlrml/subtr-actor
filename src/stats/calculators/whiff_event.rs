use super::*;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename_all = "snake_case")]
pub enum WhiffEventKind {
    #[default]
    Whiff,
    BeatenToBall,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct WhiffEvent {
    #[serde(default)]
    pub kind: WhiffEventKind,
    pub time: f32,
    pub frame: usize,
    pub resolved_time: f32,
    pub resolved_frame: usize,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    pub is_team_0: bool,
    pub closest_approach_distance: f32,
    pub forward_alignment: f32,
    pub approach_speed: f32,
    pub dodge_active: bool,
    pub aerial: bool,
}

impl WhiffEvent {
    pub(super) fn labels(&self) -> [StatLabel; 2] {
        [
            vertical_state_label(self.aerial),
            whiff_dodge_state_label(self.dodge_active),
        ]
    }
}
