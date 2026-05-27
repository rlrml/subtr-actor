use super::*;

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct RotationPlayerEvent {
    pub time: f32,
    pub frame: usize,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    pub is_team_0: bool,
    pub active: bool,
    pub became_first_man_count: u32,
    pub lost_first_man_count: u32,
    pub current_role_state: RoleState,
    pub current_depth_state: PlayDepthState,
}

impl RotationPlayerEvent {
    pub(crate) fn new(
        frame: &FrameInfo,
        player: PlayerId,
        is_team_0: bool,
        active: bool,
        current_role_state: RoleState,
        current_depth_state: PlayDepthState,
    ) -> Self {
        Self {
            time: frame.time,
            frame: frame.frame_number,
            player,
            is_team_0,
            active,
            became_first_man_count: 0,
            lost_first_man_count: 0,
            current_role_state,
            current_depth_state,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct RotationTeamEvent {
    pub time: f32,
    pub frame: usize,
    pub is_team_0: bool,
    pub first_man_changes_for_team: u32,
    pub rotation_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RotationPlayerEventState {
    pub active: bool,
    pub current_role_state: RoleState,
    pub current_depth_state: PlayDepthState,
}
