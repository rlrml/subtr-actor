use super::*;

impl BoostCalculator {
    pub(super) const PICKUP_MATCH_FRAME_WINDOW: usize = 3;

    pub fn new() -> Self {
        Self::with_config(BoostCalculatorConfig::default())
    }

    pub fn with_config(config: BoostCalculatorConfig) -> Self {
        Self {
            config,
            ..Self::default()
        }
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, BoostStats> {
        &self.player_stats
    }

    pub fn team_zero_stats(&self) -> &BoostStats {
        &self.team_zero_stats
    }

    pub fn team_one_stats(&self) -> &BoostStats {
        &self.team_one_stats
    }

    pub fn pickup_comparison_events(&self) -> &[BoostPickupComparisonEvent] {
        &self.pickup_comparison_events
    }

    pub fn ledger_events(&self) -> &[BoostLedgerEvent] {
        &self.ledger_events
    }

    pub fn state_events(&self) -> &[BoostStateEvent] {
        &self.state_events
    }

    pub(super) fn record_ledger_event(&mut self, event: BoostLedgerEvent) {
        if event.amount <= 0.0 && event.count == 0 {
            return;
        }

        self.ledger_events.push(event);
    }

    pub(super) fn record_state_event(&mut self, event: BoostStateEvent) {
        self.state_events.push(event);
    }
}
