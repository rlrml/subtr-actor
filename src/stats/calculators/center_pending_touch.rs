use super::*;

#[derive(Debug, Clone)]
pub(crate) struct PendingCenterTouch {
    pub(super) player: PlayerId,
    pub(super) is_team_0: bool,
    pub(super) time: f32,
    pub(super) frame: usize,
    pub(super) ball_position: glam::Vec3,
}
