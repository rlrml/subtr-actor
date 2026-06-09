use super::*;

const FLIP_RESET_MIN_DODGE_TOUCH_DELAY_SECONDS: f32 = 0.05;
const FLIP_RESET_MAX_DODGE_TOUCH_DELAY_SECONDS: f32 = 2.0;
const FLIP_RESET_GROUNDED_Z: f32 = 80.0;

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct DodgeResetEvent {
    pub time: f32,
    pub frame: usize,
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_position: Option<[f32; 3]>,
    pub is_team_0: bool,
    pub counter_value: i32,
    /// Whether the dodge refresh happened on the ball (i.e. this reset is a flip reset).
    pub on_ball: bool,
    /// Whether an on-ball reset (flip reset) was later converted by a dodge-powered
    /// touch. Always `false` for non-`on_ball` resets. Set retroactively once the
    /// confirming touch is observed, so it is meaningful at finish time.
    #[serde(default)]
    pub used: bool,
}

/// Internal bookkeeping for an on-ball dodge reset awaiting confirmation, including
/// the index of the emitted [`DodgeResetEvent`] so it can be marked `used` later.
#[derive(Debug, Clone, PartialEq)]
struct PendingOnBallReset {
    reset: DodgeRefreshedEvent,
    event_index: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ConfirmedFlipResetEvent {
    pub time: f32,
    pub frame: usize,
    pub reset_time: f32,
    pub reset_frame: usize,
    pub player: PlayerId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_position: Option<[f32; 3]>,
    pub is_team_0: bool,
    pub counter_value: i32,
    pub time_since_reset: f32,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DodgeResetCalculator {
    events: EventStream<DodgeResetEvent>,
    confirmed_flip_reset_events: EventStream<ConfirmedFlipResetEvent>,
    pending_on_ball_resets: HashMap<PlayerId, PendingOnBallReset>,
    pending_reset_dodge_started: HashSet<PlayerId>,
    previous_dodge_active: HashMap<PlayerId, bool>,
}

impl DodgeResetCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn events(&self) -> &[DodgeResetEvent] {
        self.events.all()
    }

    pub fn new_events(&self) -> &[DodgeResetEvent] {
        self.events.new_events()
    }

    pub fn confirmed_flip_reset_events(&self) -> &[ConfirmedFlipResetEvent] {
        self.confirmed_flip_reset_events.all()
    }

    pub fn new_confirmed_flip_reset_events(&self) -> &[ConfirmedFlipResetEvent] {
        self.confirmed_flip_reset_events.new_events()
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

        let Some(pending) = self.pending_on_ball_resets.get(player_id).cloned() else {
            return;
        };
        let reset_event = &pending.reset;
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
                player_position: touch_event
                    .player_position
                    .map(|position| vec_to_glam(&position).to_array())
                    .or_else(|| players.player_position(player_id)),
                is_team_0: touch_event.team_is_team_0,
                counter_value: reset_event.counter_value,
                time_since_reset,
            });
        if let Some(reset) = self.events.get_mut(pending.event_index) {
            reset.used = true;
        }
        self.pending_on_ball_resets.remove(player_id);
        self.pending_reset_dodge_started.remove(player_id);
    }

    pub fn update(
        &mut self,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        events: &FrameEventsState,
        touch_state: &TouchState,
    ) -> SubtrActorResult<()> {
        self.events.begin_update();
        self.confirmed_flip_reset_events.begin_update();
        self.prune_pending_resets(players);
        for event in &events.dodge_refreshed_events {
            let on_ball = Self::on_ball_dodge_reset(ball, players, &event.player);
            let reset_event = event.clone();
            let event = DodgeResetEvent {
                time: event.time,
                frame: event.frame,
                player: event.player.clone(),
                player_position: players.player_position(&event.player),
                is_team_0: event.is_team_0,
                counter_value: event.counter_value,
                on_ball,
                used: false,
            };
            if on_ball {
                // Index this event will occupy after the push below, so a later
                // confirming touch can mark it `used`.
                let event_index = self.events.all().len();
                self.pending_on_ball_resets.insert(
                    event.player.clone(),
                    PendingOnBallReset {
                        reset: reset_event,
                        event_index,
                    },
                );
                self.pending_reset_dodge_started.remove(&event.player);
            }
            self.events.push(event);
        }
        self.update_pending_reset_dodges(players);
        for touch_event in chronological_touch_events(&touch_state.touch_events) {
            self.apply_confirmed_flip_reset_touch(players, touch_event);
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "dodge_reset_tests.rs"]
mod tests;
