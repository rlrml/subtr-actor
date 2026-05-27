use super::*;

#[derive(Debug, Clone)]
pub(crate) struct PendingBackboardBounce {
    pub(super) player_id: PlayerId,
    pub(super) is_team_0: bool,
    pub(super) time: f32,
    pub(super) frame: usize,
}
