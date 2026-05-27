use super::*;

#[derive(Debug, Clone)]
pub(super) struct PreviousPlayerSample {
    pub(super) rigid_body: boxcars::RigidBody,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct DirectionalBumpCandidate {
    pub(super) score: f32,
    pub(super) closing_speed: f32,
    pub(super) victim_impulse: f32,
    pub(super) initiator_slowdown: f32,
}

#[derive(Debug, Clone, Default)]
pub struct BumpCalculator {
    pub(super) player_stats: HashMap<PlayerId, BumpPlayerStats>,
    pub(super) player_teams: HashMap<PlayerId, bool>,
    pub(super) team_zero_stats: BumpTeamStats,
    pub(super) team_one_stats: BumpTeamStats,
    pub(super) events: Vec<BumpEvent>,
    pub(super) previous_players: HashMap<PlayerId, PreviousPlayerSample>,
    pub(super) last_seen_pair_frame: HashMap<(PlayerId, PlayerId), usize>,
}
