use super::*;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ActiveWavedashCandidate {
    pub(super) is_team_0: bool,
    pub(super) dodge_time: f32,
    pub(super) dodge_frame: usize,
    pub(super) dodge_position: [f32; 3],
    pub(super) start_horizontal_speed: f32,
    pub(super) start_height: f32,
}

impl ActiveWavedashCandidate {
    pub(super) fn new(frame: &FrameInfo, player: &PlayerSample, position: glam::Vec3) -> Self {
        Self {
            is_team_0: player.is_team_0,
            dodge_time: frame.time,
            dodge_frame: frame.frame_number,
            dodge_position: position.to_array(),
            start_horizontal_speed: WavedashCalculator::horizontal_speed(player),
            start_height: position.z,
        }
    }
}
