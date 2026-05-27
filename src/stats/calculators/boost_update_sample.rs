use super::*;

#[derive(Default)]
pub(super) struct BoostUpdateSample {
    pub(super) current_boost_amounts: Vec<(PlayerId, f32)>,
    pub(super) pickup_counts_by_player: HashMap<PlayerId, usize>,
    pub(super) respawn_amounts_by_player: HashMap<PlayerId, f32>,
}

impl BoostUpdateSample {
    pub(super) fn from_events(events: &FrameEventsState) -> Self {
        let mut sample = Self::default();
        for event in &events.boost_pad_events {
            let BoostPadEventKind::PickedUp { .. } = event.kind else {
                continue;
            };
            let Some(player_id) = &event.player else {
                continue;
            };
            *sample
                .pickup_counts_by_player
                .entry(player_id.clone())
                .or_default() += 1;
        }
        sample
    }
}

impl BoostCalculator {
    pub(super) fn begin_boost_update_sample(
        &mut self,
        context: &boost_update_context::BoostUpdateContext,
        events: &FrameEventsState,
    ) {
        if context.kickoff_phase_started(self) {
            self.kickoff_respawn_awarded.clear();
        }
        for demo in &events.demo_events {
            let pre_demo_boost_amount = self.previous_boost_amounts.get(&demo.victim).copied();
            self.pending_demo_respawns
                .entry(demo.victim.clone())
                .or_insert(PendingDemoRespawn {
                    demo_time: demo.time,
                    pre_demo_boost_amount,
                });
        }
    }

    pub(super) fn finish_boost_update_sample(
        &mut self,
        frame: &FrameInfo,
        players: &PlayerFrameState,
        context: &boost_update_context::BoostUpdateContext,
        sample: BoostUpdateSample,
    ) {
        for (player_id, boost_amount) in sample.current_boost_amounts {
            self.previous_boost_amounts.insert(player_id, boost_amount);
        }
        for player in &players.players {
            if let Some(speed) = player.speed() {
                self.previous_player_speeds
                    .insert(player.player_id.clone(), speed);
            }
        }
        self.warn_for_sample_boost_invariants(frame, players);
        self.kickoff_phase_active_last_frame = context.kickoff_phase_active;
        self.previous_boost_levels_live = Some(context.boost_levels_live);
    }
}
