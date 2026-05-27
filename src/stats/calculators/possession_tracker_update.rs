use super::*;

impl PossessionTracker {
    fn register_contested_touch(&mut self, time: f32) {
        let Some(current_team_is_team_0) = self.current_team_is_team_0 else {
            self.clear_pending_turnover();
            return;
        };
        self.last_possession_touch_time = Some(time);
        self.pending_turnover_team_is_team_0 = Some(!current_team_is_team_0);
        self.pending_turnover_touch_time = Some(time);
    }

    fn update_player_control(
        &mut self,
        active_team_before_sample: Option<bool>,
        touched_team_zero_player: Option<&PlayerId>,
        touched_team_one_player: Option<&PlayerId>,
    ) {
        let Some(current_team_is_team_0) = self.current_team_is_team_0 else {
            self.current_player = None;
            return;
        };
        if self.pending_turnover_team_is_team_0.is_some() {
            self.current_player = None;
            return;
        }
        let controlling_touch_player = if current_team_is_team_0 {
            touched_team_zero_player
        } else {
            touched_team_one_player
        };
        if let Some(player) = controlling_touch_player {
            self.current_player = Some(player.clone());
            return;
        }
        if active_team_before_sample != self.current_team_is_team_0 {
            self.current_player = None;
        }
    }

    pub(crate) fn update(&mut self, time: f32, touch_events: &[TouchEvent]) -> PossessionState {
        self.expire_pending_turnover(time);
        self.expire_loose_ball(time);
        let active_team_before_sample = self.current_team_is_team_0;
        let active_player_before_sample = self.current_player.clone();
        let touched_team_zero = touch_events.iter().any(|touch| touch.team_is_team_0);
        let touched_team_one = touch_events.iter().any(|touch| !touch.team_is_team_0);
        let touched_team_zero_player = last_touched_player(touch_events, true);
        let touched_team_one_player = last_touched_player(touch_events, false);
        match (touched_team_zero, touched_team_one) {
            (true, true) => self.register_contested_touch(time),
            (true, false) => self.register_single_team_touch(true, time),
            (false, true) => self.register_single_team_touch(false, time),
            (false, false) => {}
        }
        self.update_player_control(
            active_team_before_sample,
            touched_team_zero_player.as_ref(),
            touched_team_one_player.as_ref(),
        );
        PossessionState {
            active_team_before_sample,
            current_team_is_team_0: self.current_team_is_team_0,
            active_player_before_sample,
            current_player: self.current_player.clone(),
        }
    }
}

fn last_touched_player(touch_events: &[TouchEvent], team_is_team_0: bool) -> Option<PlayerId> {
    touch_events
        .iter()
        .rev()
        .find(|touch| touch.team_is_team_0 == team_is_team_0)
        .and_then(|touch| touch.player.clone())
}
