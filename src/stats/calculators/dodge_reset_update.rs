use super::*;

impl DodgeResetCalculator {
    pub fn update(
        &mut self,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        events: &FrameEventsState,
    ) -> SubtrActorResult<()> {
        self.prune_pending_resets(players);
        for event in &events.dodge_refreshed_events {
            self.apply_dodge_refreshed_event(ball, players, event);
        }
        self.update_pending_reset_dodges(players);
        for touch_event in &events.touch_events {
            self.apply_confirmed_flip_reset_touch(players, touch_event);
        }
        Ok(())
    }

    fn apply_dodge_refreshed_event(
        &mut self,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        event: &DodgeRefreshedEvent,
    ) {
        let on_ball = Self::on_ball_dodge_reset(ball, players, &event.player);
        let stats = self.player_stats.entry(event.player.clone()).or_default();
        stats.count += 1;
        if on_ball {
            stats.on_ball_count += 1;
            self.on_ball_events.push(event.clone());
            self.pending_on_ball_resets
                .insert(event.player.clone(), event.clone());
            self.pending_reset_dodge_started.remove(&event.player);
        }
        self.events.push(DodgeResetEvent {
            time: event.time,
            frame: event.frame,
            player: event.player.clone(),
            is_team_0: event.is_team_0,
            counter_value: event.counter_value,
            on_ball,
        });
    }
}
