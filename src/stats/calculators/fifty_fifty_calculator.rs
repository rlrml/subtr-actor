use super::*;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FiftyFiftyCalculator {
    stats: FiftyFiftyStats,
    player_stats: HashMap<PlayerId, FiftyFiftyPlayerStats>,
    events: Vec<FiftyFiftyEvent>,
}

impl FiftyFiftyCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn stats(&self) -> &FiftyFiftyStats {
        &self.stats
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, FiftyFiftyPlayerStats> {
        &self.player_stats
    }

    pub fn events(&self) -> &[FiftyFiftyEvent] {
        &self.events
    }

    pub(super) fn apply_event(&mut self, event: &FiftyFiftyEvent) {
        self.stats.record_event(event);

        if let Some(player_id) = event.team_zero_player.as_ref() {
            let stats = self.player_stats.entry(player_id.clone()).or_default();
            stats.record_event(true, event);
        }
        if let Some(player_id) = event.team_one_player.as_ref() {
            let stats = self.player_stats.entry(player_id.clone()).or_default();
            stats.record_event(false, event);
        }

        self.events.push(event.clone());
    }

    pub fn update(&mut self, fifty_fifty_state: &FiftyFiftyState) -> SubtrActorResult<()> {
        for event in &fifty_fifty_state.resolved_events {
            self.apply_event(event);
        }
        Ok(())
    }
}
