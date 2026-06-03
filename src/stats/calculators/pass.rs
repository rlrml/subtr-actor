use super::*;

const PASS_MAX_DURATION_SECONDS: f32 = 3.5;
const PASS_MIN_BALL_TRAVEL_DISTANCE: f32 = 500.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum PassKind {
    Direct,
    Backboard,
    FiftyFifty,
    FiftyFiftyBackboard,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct PassEvent {
    pub time: f32,
    pub frame: usize,
    pub sample_time: f32,
    pub sample_frame: usize,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub passer: PlayerId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub passer_position: Option<[f32; 3]>,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub receiver: PlayerId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receiver_position: Option<[f32; 3]>,
    pub is_team_0: bool,
    pub start_time: f32,
    pub start_frame: usize,
    pub duration: f32,
    pub ball_travel_distance: f32,
    pub ball_advance_distance: f32,
    pub pass_kind: PassKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct PassLastCompletedEvent {
    pub time: f32,
    pub frame: usize,
    #[ts(as = "Option<crate::ts_bindings::RemoteIdTs>")]
    pub player: Option<PlayerId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_position: Option<[f32; 3]>,
}

#[derive(Debug, Clone)]
struct PendingPassTouch {
    player: PlayerId,
    player_position: Option<[f32; 3]>,
    is_team_0: bool,
    time: f32,
    frame: usize,
    ball_position: glam::Vec3,
    from_fifty_fifty: bool,
}

#[derive(Debug, Clone, Default)]
pub struct PassCalculator {
    events: EventStream<PassEvent>,
    last_completed_events: EventStream<PassLastCompletedEvent>,
    last_touch: Option<PendingPassTouch>,
    emitted_last_completed_pass_player: Option<PlayerId>,
}

impl PassCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn events(&self) -> &[PassEvent] {
        self.events.all()
    }

    pub fn new_events(&self) -> &[PassEvent] {
        self.events.new_events()
    }

    pub fn last_completed_events(&self) -> &[PassLastCompletedEvent] {
        self.last_completed_events.all()
    }

    pub fn new_last_completed_events(&self) -> &[PassLastCompletedEvent] {
        self.last_completed_events.new_events()
    }

    fn pass_event_for_touch(
        &self,
        touch: &TouchEvent,
        receiver: &PlayerId,
        ball_position: glam::Vec3,
        backboard_bounce_state: &BackboardBounceState,
    ) -> Option<PassEvent> {
        let previous = self.last_touch.as_ref()?;
        if previous.player == *receiver || previous.is_team_0 != touch.team_is_team_0 {
            return None;
        }

        let duration = touch.time - previous.time;
        if !(0.0..=PASS_MAX_DURATION_SECONDS).contains(&duration) {
            return None;
        }

        let ball_delta = ball_position - previous.ball_position;
        let ball_travel_distance = ball_delta.length();
        if ball_travel_distance < PASS_MIN_BALL_TRAVEL_DISTANCE {
            return None;
        }

        let team_forward_sign = if touch.team_is_team_0 { 1.0 } else { -1.0 };
        let went_off_backboard = Self::has_backboard_bounce_between(
            previous,
            touch,
            backboard_bounce_state.last_bounce_event.as_ref(),
        );
        Some(PassEvent {
            time: touch.time,
            frame: touch.frame,
            sample_time: touch.time,
            sample_frame: touch.frame,
            passer: previous.player.clone(),
            passer_position: previous.player_position,
            receiver: receiver.clone(),
            receiver_position: touch
                .player_position
                .map(|position| vec_to_glam(&position).to_array()),
            is_team_0: touch.team_is_team_0,
            start_time: previous.time,
            start_frame: previous.frame,
            duration,
            ball_travel_distance,
            ball_advance_distance: ball_delta.y * team_forward_sign,
            pass_kind: Self::pass_kind(previous.from_fifty_fifty, went_off_backboard),
        })
    }

    fn pass_kind(from_fifty_fifty: bool, went_off_backboard: bool) -> PassKind {
        match (from_fifty_fifty, went_off_backboard) {
            (true, true) => PassKind::FiftyFiftyBackboard,
            (true, false) => PassKind::FiftyFifty,
            (false, true) => PassKind::Backboard,
            (false, false) => PassKind::Direct,
        }
    }

    fn has_backboard_bounce_between(
        previous: &PendingPassTouch,
        touch: &TouchEvent,
        bounce_event: Option<&BackboardBounceEvent>,
    ) -> bool {
        bounce_event.is_some_and(|event| {
            event.player == previous.player
                && event.is_team_0 == previous.is_team_0
                && event.time >= previous.time
                && event.time <= touch.time
        })
    }

    fn touch_from_fifty_fifty(touch: &TouchEvent, fifty_fifty_state: &FiftyFiftyState) -> bool {
        fifty_fifty_state
            .active_event
            .as_ref()
            .is_some_and(|event| {
                Self::fifty_fifty_involves_touch(
                    event.start_time,
                    event.last_touch_time,
                    event.team_zero_player.as_ref(),
                    event.team_one_player.as_ref(),
                    touch,
                )
            })
            || fifty_fifty_state
                .last_resolved_event
                .as_ref()
                .is_some_and(|event| {
                    Self::fifty_fifty_involves_touch(
                        event.start_time,
                        event.resolve_time,
                        event.team_zero_player.as_ref(),
                        event.team_one_player.as_ref(),
                        touch,
                    )
                })
    }

    fn fifty_fifty_involves_touch(
        start_time: f32,
        end_time: f32,
        team_zero_player: Option<&PlayerId>,
        team_one_player: Option<&PlayerId>,
        touch: &TouchEvent,
    ) -> bool {
        if touch.time < start_time || touch.time > end_time {
            return false;
        }

        match (touch.team_is_team_0, touch.player.as_ref()) {
            (true, Some(player)) => team_zero_player == Some(player),
            (false, Some(player)) => team_one_player == Some(player),
            _ => false,
        }
    }

    fn record_pass(&mut self, frame: &FrameInfo, mut event: PassEvent) {
        event.sample_time = frame.time;
        event.sample_frame = frame.frame_number;
        self.events.push(event);
    }

    fn emit_last_completed_event(
        &mut self,
        frame: &FrameInfo,
        player: Option<PlayerId>,
        player_position: Option<[f32; 3]>,
    ) {
        if self.emitted_last_completed_pass_player == player {
            return;
        }
        self.emitted_last_completed_pass_player = player.clone();
        self.last_completed_events.push(PassLastCompletedEvent {
            time: frame.time,
            frame: frame.frame_number,
            player,
            player_position,
        });
    }

    pub fn update(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        touch_state: &TouchState,
        backboard_bounce_state: &BackboardBounceState,
        fifty_fifty_state: &FiftyFiftyState,
        live_play: bool,
    ) -> SubtrActorResult<()> {
        self.events.begin_update();
        self.last_completed_events.begin_update();
        if !live_play {
            self.last_touch = None;
            self.emit_last_completed_event(frame, None, None);
            return Ok(());
        }

        let Some(ball_position) = ball.position() else {
            self.emit_last_completed_event(frame, None, None);
            return Ok(());
        };

        for touch in &touch_state.touch_events {
            let Some(player) = touch.player.clone() else {
                self.last_touch = None;
                continue;
            };

            if let Some(pass_event) =
                self.pass_event_for_touch(touch, &player, ball_position, backboard_bounce_state)
            {
                self.record_pass(frame, pass_event);
            }

            self.last_touch = Some(PendingPassTouch {
                player,
                player_position: touch
                    .player_position
                    .map(|position| vec_to_glam(&position).to_array()),
                is_team_0: touch.team_is_team_0,
                time: touch.time,
                frame: touch.frame,
                ball_position,
                from_fifty_fifty: Self::touch_from_fifty_fifty(touch, fifty_fifty_state),
            });
        }
        let current_last_completed_pass_event = self.events.iter().next_back();
        let current_last_completed_pass_player =
            current_last_completed_pass_event.map(|event| event.passer.clone());
        let current_last_completed_pass_position =
            current_last_completed_pass_event.and_then(|event| event.passer_position);
        self.emit_last_completed_event(
            frame,
            current_last_completed_pass_player,
            current_last_completed_pass_position,
        );

        Ok(())
    }
}

#[cfg(test)]
impl PassCalculator {
    pub fn player_stats(&self) -> &HashMap<PlayerId, PassPlayerStats> {
        let mut stats = PassStatsAccumulator::default();
        for event in self.events() {
            let frame = stats_test_frame(event.sample_time, event.sample_frame);
            stats.apply_event(&frame, event);
        }
        leak_test_stats(stats.player_stats().clone())
    }

    pub fn team_zero_stats(&self) -> &PassTeamStats {
        let mut stats = PassStatsAccumulator::default();
        for event in self.events() {
            let frame = stats_test_frame(event.sample_time, event.sample_frame);
            stats.apply_event(&frame, event);
        }
        leak_test_stats(stats.team_zero_stats().clone())
    }

    pub fn team_one_stats(&self) -> &PassTeamStats {
        let mut stats = PassStatsAccumulator::default();
        for event in self.events() {
            let frame = stats_test_frame(event.sample_time, event.sample_frame);
            stats.apply_event(&frame, event);
        }
        leak_test_stats(stats.team_one_stats().clone())
    }
}

#[cfg(test)]
#[path = "pass_tests.rs"]
mod tests;
