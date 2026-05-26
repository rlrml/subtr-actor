use super::*;

const FLIP_RESET_MIN_DODGE_TOUCH_DELAY_SECONDS: f32 = 0.05;
const FLIP_RESET_MAX_DODGE_TOUCH_DELAY_SECONDS: f32 = 2.0;
const FLIP_RESET_GROUNDED_Z: f32 = 80.0;

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct DodgeResetEvent {
    pub time: f32,
    pub frame: usize,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    pub is_team_0: bool,
    pub counter_value: i32,
    pub on_ball: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ConfirmedFlipResetEvent {
    pub time: f32,
    pub frame: usize,
    pub reset_time: f32,
    pub reset_frame: usize,
    pub player: PlayerId,
    pub is_team_0: bool,
    pub counter_value: i32,
    pub time_since_reset: f32,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct DodgeResetStats {
    pub count: u32,
    pub on_ball_count: u32,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DodgeResetCalculator {
    player_stats: HashMap<PlayerId, DodgeResetStats>,
    events: Vec<DodgeResetEvent>,
    on_ball_events: Vec<DodgeRefreshedEvent>,
    confirmed_flip_reset_events: Vec<ConfirmedFlipResetEvent>,
    pending_on_ball_resets: HashMap<PlayerId, DodgeRefreshedEvent>,
    pending_reset_dodge_started: HashSet<PlayerId>,
    previous_dodge_active: HashMap<PlayerId, bool>,
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

    fn player<'a>(players: &'a PlayerFrameState, player_id: &PlayerId) -> Option<&'a PlayerSample> {
        players
            .players
            .iter()
            .find(|player| &player.player_id == player_id)
    }

    fn player_is_grounded(players: &PlayerFrameState, player_id: &PlayerId) -> bool {
        Self::player(players, player_id)
            .and_then(PlayerSample::position)
            .is_some_and(|position| position.z <= FLIP_RESET_GROUNDED_Z)
    }

    fn player_dodge_active(players: &PlayerFrameState, player_id: &PlayerId) -> bool {
        Self::player(players, player_id).is_some_and(|player| player.dodge_active)
    }

    fn on_ball_dodge_reset(
        ball: &BallFrameState,
        players: &PlayerFrameState,
        player_id: &PlayerId,
    ) -> bool {
        const MIN_PLAYER_HEIGHT: f32 = 95.0;
        const MIN_BALL_HEIGHT: f32 = 80.0;
        const MAX_CENTER_DISTANCE: f32 = 180.0;
        const MAX_LOCAL_VERTICAL_OFFSET: f32 = 140.0;

        let Some(ball) = ball.sample() else {
            return false;
        };
        let Some(player) = Self::player(players, player_id) else {
            return false;
        };
        let Some(player_rigid_body) = &player.rigid_body else {
            return false;
        };

        let ball_position = vec_to_glam(&ball.rigid_body.location);
        let player_position = vec_to_glam(&player_rigid_body.location);
        if player_position.z < MIN_PLAYER_HEIGHT || ball_position.z < MIN_BALL_HEIGHT {
            return false;
        }

        let relative_ball_position = ball_position - player_position;
        let center_distance = relative_ball_position.length();
        if !center_distance.is_finite() || center_distance > MAX_CENTER_DISTANCE {
            return false;
        }

        let player_rotation = quat_to_glam(&player_rigid_body.rotation);
        let local_ball_position = player_rotation.inverse() * relative_ball_position;
        local_ball_position.z <= MAX_LOCAL_VERTICAL_OFFSET
    }

    fn prune_pending_resets(&mut self, players: &PlayerFrameState) {
        let grounded_players = self
            .pending_on_ball_resets
            .keys()
            .filter(|player_id| Self::player_is_grounded(players, player_id))
            .cloned()
            .collect::<Vec<_>>();
        for player_id in grounded_players {
            self.pending_on_ball_resets.remove(&player_id);
            self.pending_reset_dodge_started.remove(&player_id);
        }
    }

    fn update_pending_reset_dodges(&mut self, players: &PlayerFrameState) {
        for player in &players.players {
            let was_dodge_active = self
                .previous_dodge_active
                .insert(player.player_id.clone(), player.dodge_active)
                .unwrap_or(false);
            if player.dodge_active
                && !was_dodge_active
                && self.pending_on_ball_resets.contains_key(&player.player_id)
            {
                self.pending_reset_dodge_started
                    .insert(player.player_id.clone());
            }
        }
    }

    fn apply_confirmed_flip_reset_touch(
        &mut self,
        players: &PlayerFrameState,
        touch_event: &TouchEvent,
    ) {
        let Some(player_id) = touch_event.player.as_ref() else {
            return;
        };
        if !self.pending_reset_dodge_started.contains(player_id)
            || !Self::player_dodge_active(players, player_id)
        {
            return;
        }

        let Some(reset_event) = self.pending_on_ball_resets.get(player_id).cloned() else {
            return;
        };
        let time_since_reset = touch_event.time - reset_event.time;
        if !(FLIP_RESET_MIN_DODGE_TOUCH_DELAY_SECONDS..=FLIP_RESET_MAX_DODGE_TOUCH_DELAY_SECONDS)
            .contains(&time_since_reset)
        {
            if time_since_reset > FLIP_RESET_MAX_DODGE_TOUCH_DELAY_SECONDS {
                self.pending_on_ball_resets.remove(player_id);
                self.pending_reset_dodge_started.remove(player_id);
            }
            return;
        }

        self.confirmed_flip_reset_events
            .push(ConfirmedFlipResetEvent {
                time: touch_event.time,
                frame: touch_event.frame,
                reset_time: reset_event.time,
                reset_frame: reset_event.frame,
                player: player_id.clone(),
                is_team_0: touch_event.team_is_team_0,
                counter_value: reset_event.counter_value,
                time_since_reset,
            });
        self.pending_on_ball_resets.remove(player_id);
        self.pending_reset_dodge_started.remove(player_id);
    }

    pub fn update(
        &mut self,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        events: &FrameEventsState,
    ) -> SubtrActorResult<()> {
        self.prune_pending_resets(players);
        for event in &events.dodge_refreshed_events {
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
        self.update_pending_reset_dodges(players);
        for touch_event in &events.touch_events {
            self.apply_confirmed_flip_reset_touch(players, touch_event);
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "dodge_reset_tests.rs"]
mod tests;
