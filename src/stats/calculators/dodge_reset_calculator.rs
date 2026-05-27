use super::*;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DodgeResetCalculator {
    pub(super) player_stats: HashMap<PlayerId, DodgeResetStats>,
    pub(super) events: Vec<DodgeResetEvent>,
    pub(super) on_ball_events: Vec<DodgeRefreshedEvent>,
    pub(super) confirmed_flip_reset_events: Vec<ConfirmedFlipResetEvent>,
    pub(super) pending_on_ball_resets: HashMap<PlayerId, DodgeRefreshedEvent>,
    pub(super) pending_reset_dodge_started: HashSet<PlayerId>,
    pub(super) previous_dodge_active: HashMap<PlayerId, bool>,
}

impl DodgeResetCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, DodgeResetStats> {
        &self.player_stats
    }

    pub fn events(&self) -> &[DodgeResetEvent] {
        &self.events
    }

    pub fn on_ball_events(&self) -> &[DodgeRefreshedEvent] {
        &self.on_ball_events
    }

    pub fn confirmed_flip_reset_events(&self) -> &[ConfirmedFlipResetEvent] {
        &self.confirmed_flip_reset_events
    }

    pub(super) fn player<'a>(
        players: &'a PlayerFrameState,
        player_id: &PlayerId,
    ) -> Option<&'a PlayerSample> {
        players
            .players
            .iter()
            .find(|player| &player.player_id == player_id)
    }
}
