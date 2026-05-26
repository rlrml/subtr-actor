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
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub receiver: PlayerId,
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
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct PassPlayerStats {
    pub completed_pass_count: u32,
    pub received_pass_count: u32,
    pub total_pass_distance: f32,
    pub total_pass_advance: f32,
    pub longest_pass_distance: f32,
    pub is_last_completed_pass: bool,
    pub last_completed_pass_time: Option<f32>,
    pub last_completed_pass_frame: Option<usize>,
    pub time_since_last_completed_pass: Option<f32>,
    pub frames_since_last_completed_pass: Option<usize>,
}

impl PassPlayerStats {
    pub fn average_pass_distance(&self) -> f32 {
        if self.completed_pass_count == 0 {
            0.0
        } else {
            self.total_pass_distance / self.completed_pass_count as f32
        }
    }

    pub fn average_pass_advance(&self) -> f32 {
        if self.completed_pass_count == 0 {
            0.0
        } else {
            self.total_pass_advance / self.completed_pass_count as f32
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct PassTeamStats {
    pub completed_pass_count: u32,
    pub total_pass_distance: f32,
    pub total_pass_advance: f32,
    pub longest_pass_distance: f32,
}

impl PassTeamStats {
    pub fn average_pass_distance(&self) -> f32 {
        if self.completed_pass_count == 0 {
            0.0
        } else {
            self.total_pass_distance / self.completed_pass_count as f32
        }
    }

    pub fn average_pass_advance(&self) -> f32 {
        if self.completed_pass_count == 0 {
            0.0
        } else {
            self.total_pass_advance / self.completed_pass_count as f32
        }
    }
}

#[derive(Debug, Clone)]
struct PendingPassTouch {
    player: PlayerId,
    is_team_0: bool,
    time: f32,
    frame: usize,
    ball_position: glam::Vec3,
    from_fifty_fifty: bool,
}

#[derive(Debug, Clone, Default)]
pub struct PassCalculator {
    player_stats: HashMap<PlayerId, PassPlayerStats>,
    team_zero_stats: PassTeamStats,
    team_one_stats: PassTeamStats,
    events: Vec<PassEvent>,
    last_completed_events: Vec<PassLastCompletedEvent>,
    last_touch: Option<PendingPassTouch>,
    current_last_completed_pass_player: Option<PlayerId>,
    emitted_last_completed_pass_player: Option<PlayerId>,
}

impl PassCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, PassPlayerStats> {
        &self.player_stats
    }

    pub fn team_zero_stats(&self) -> &PassTeamStats {
        &self.team_zero_stats
    }

    pub fn team_one_stats(&self) -> &PassTeamStats {
        &self.team_one_stats
    }

    pub fn events(&self) -> &[PassEvent] {
        &self.events
    }

    pub fn last_completed_events(&self) -> &[PassLastCompletedEvent] {
        &self.last_completed_events
    }

    fn begin_sample(&mut self, frame: &FrameInfo) {
        for stats in self.player_stats.values_mut() {
            stats.is_last_completed_pass = false;
            stats.time_since_last_completed_pass = stats
                .last_completed_pass_time
                .map(|time| (frame.time - time).max(0.0));
            stats.frames_since_last_completed_pass = stats
                .last_completed_pass_frame
                .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
        }
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
            receiver: receiver.clone(),
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
        let passer_stats = self.player_stats.entry(event.passer.clone()).or_default();
        passer_stats.completed_pass_count += 1;
        passer_stats.total_pass_distance += event.ball_travel_distance;
        passer_stats.total_pass_advance += event.ball_advance_distance;
        passer_stats.longest_pass_distance = passer_stats
            .longest_pass_distance
            .max(event.ball_travel_distance);
        passer_stats.last_completed_pass_time = Some(event.time);
        passer_stats.last_completed_pass_frame = Some(event.frame);
        passer_stats.time_since_last_completed_pass = Some((frame.time - event.time).max(0.0));
        passer_stats.frames_since_last_completed_pass =
            Some(frame.frame_number.saturating_sub(event.frame));

        self.player_stats
            .entry(event.receiver.clone())
            .or_default()
            .received_pass_count += 1;

        let team_stats = if event.is_team_0 {
            &mut self.team_zero_stats
        } else {
            &mut self.team_one_stats
        };
        team_stats.completed_pass_count += 1;
        team_stats.total_pass_distance += event.ball_travel_distance;
        team_stats.total_pass_advance += event.ball_advance_distance;
        team_stats.longest_pass_distance = team_stats
            .longest_pass_distance
            .max(event.ball_travel_distance);

        self.current_last_completed_pass_player = Some(event.passer.clone());
        self.events.push(event);
    }

    fn emit_last_completed_event(&mut self, frame: &FrameInfo, player: Option<PlayerId>) {
        if self.emitted_last_completed_pass_player == player {
            return;
        }
        self.emitted_last_completed_pass_player = player.clone();
        self.last_completed_events.push(PassLastCompletedEvent {
            time: frame.time,
            frame: frame.frame_number,
            player,
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
        self.begin_sample(frame);
        if !live_play {
            self.last_touch = None;
            self.current_last_completed_pass_player = None;
            self.emit_last_completed_event(frame, None);
            return Ok(());
        }

        let Some(ball_position) = ball.position() else {
            self.emit_last_completed_event(frame, None);
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
                is_team_0: touch.team_is_team_0,
                time: touch.time,
                frame: touch.frame,
                ball_position,
                from_fifty_fifty: Self::touch_from_fifty_fifty(touch, fifty_fifty_state),
            });
        }

        if let Some(player_id) = self.current_last_completed_pass_player.as_ref() {
            if let Some(stats) = self.player_stats.get_mut(player_id) {
                stats.is_last_completed_pass = true;
            }
        }
        self.emit_last_completed_event(frame, self.current_last_completed_pass_player.clone());

        Ok(())
    }
}

#[cfg(test)]
#[path = "pass_tests.rs"]
mod tests;
