use super::*;

#[derive(Debug, Clone)]
pub(crate) struct PendingPassTouch {
    pub(super) player: PlayerId,
    pub(super) is_team_0: bool,
    pub(super) time: f32,
    pub(super) frame: usize,
    pub(super) ball_position: glam::Vec3,
    pub(super) from_fifty_fifty: bool,
}
