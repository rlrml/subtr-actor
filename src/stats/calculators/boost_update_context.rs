use super::*;

pub(super) struct BoostUpdateContext {
    pub(super) boost_levels_live: bool,
    pub(super) track_boost_levels: bool,
    pub(super) track_boost_pickups: bool,
    pub(super) boost_levels_resumed_this_sample: bool,
    pub(super) kickoff_phase_active: bool,
    pub(super) live_play: bool,
}

impl BoostUpdateContext {
    pub(super) fn new(
        calculator: &BoostCalculator,
        gameplay: &GameplayState,
        live_play: bool,
    ) -> Self {
        let boost_levels_live = BoostCalculator::boost_levels_live(live_play);
        Self {
            boost_levels_live,
            track_boost_levels: BoostCalculator::tracks_boost_levels(boost_levels_live),
            track_boost_pickups: BoostCalculator::tracks_boost_pickups(gameplay, live_play),
            boost_levels_resumed_this_sample: boost_levels_live
                && !calculator.previous_boost_levels_live.unwrap_or(false),
            kickoff_phase_active: Self::kickoff_phase_active(gameplay),
            live_play,
        }
    }

    pub(super) fn kickoff_phase_started(&self, calculator: &BoostCalculator) -> bool {
        self.kickoff_phase_active && !calculator.kickoff_phase_active_last_frame
    }

    fn kickoff_phase_active(gameplay: &GameplayState) -> bool {
        gameplay.game_state == Some(GAME_STATE_KICKOFF_COUNTDOWN)
            || gameplay.kickoff_countdown_time.is_some_and(|t| t > 0)
            || gameplay.ball_has_been_hit == Some(false)
    }
}
