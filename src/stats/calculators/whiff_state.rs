use super::*;

#[derive(Debug, Clone, PartialEq)]
pub(super) struct ActiveWhiffCandidate {
    pub(super) player: PlayerId,
    pub(super) is_team_0: bool,
    pub(super) start_time: f32,
    pub(super) closest_time: f32,
    pub(super) closest_frame: usize,
    pub(super) closest_approach_distance: f32,
    pub(super) forward_alignment: f32,
    pub(super) approach_speed: f32,
    pub(super) dodge_active: bool,
    pub(super) aerial: bool,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct WhiffCalculator {
    pub(super) player_stats: HashMap<PlayerId, WhiffStats>,
    pub(super) active_candidates: HashMap<PlayerId, ActiveWhiffCandidate>,
    pub(super) events: Vec<WhiffEvent>,
    pub(super) current_last_whiff_player: Option<PlayerId>,
}
