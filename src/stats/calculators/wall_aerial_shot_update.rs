use super::*;

impl WallAerialShotCalculator {
    pub fn update(
        &mut self,
        frame: &FrameInfo,
        players: &PlayerFrameState,
        frame_events: &FrameEventsState,
        live_play: bool,
    ) -> SubtrActorResult<()> {
        self.begin_sample(frame);
        if !live_play {
            self.recent_wall_contacts.clear();
            self.armed_shots.clear();
            self.current_last_wall_aerial_shot_player = None;
            return Ok(());
        }

        self.update_wall_contacts_and_takeoffs(frame, players);
        self.prune_armed_shots(frame.time);
        self.apply_player_stat_events(frame, players, frame_events);
        self.mark_current_last_wall_aerial_shot();

        Ok(())
    }

    fn apply_player_stat_events(
        &mut self,
        frame: &FrameInfo,
        players: &PlayerFrameState,
        frame_events: &FrameEventsState,
    ) {
        for stat_event in &frame_events.player_stat_events {
            if let Some(event) = self.shot_event(players, stat_event) {
                self.record_event(frame, event);
            }
        }
    }

    fn mark_current_last_wall_aerial_shot(&mut self) {
        if let Some(player_id) = self.current_last_wall_aerial_shot_player.as_ref() {
            if let Some(stats) = self.player_stats.get_mut(player_id) {
                stats.is_last_wall_aerial_shot = true;
            }
        }
    }
}
