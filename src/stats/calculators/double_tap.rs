use super::*;

const DOUBLE_TAP_TOUCH_WINDOW_SECONDS: f32 = 2.5;

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct DoubleTapEvent {
    pub time: f32,
    pub frame: usize,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    pub is_team_0: bool,
    pub backboard_time: f32,
    pub backboard_frame: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct DoubleTapPlayerStats {
    pub count: u32,
    pub is_last_double_tap: bool,
    pub last_double_tap_time: Option<f32>,
    pub last_double_tap_frame: Option<usize>,
    pub time_since_last_double_tap: Option<f32>,
    pub frames_since_last_double_tap: Option<usize>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct DoubleTapTeamStats {
    pub count: u32,
}

#[derive(Debug, Clone)]
struct PendingBackboardBounce {
    player_id: PlayerId,
    is_team_0: bool,
    time: f32,
    frame: usize,
}

#[derive(Debug, Clone, Default)]
pub struct DoubleTapCalculator {
    player_stats: HashMap<PlayerId, DoubleTapPlayerStats>,
    team_zero_stats: DoubleTapTeamStats,
    team_one_stats: DoubleTapTeamStats,
    events: Vec<DoubleTapEvent>,
    pending_backboard_bounces: Vec<PendingBackboardBounce>,
    current_last_double_tap_player: Option<PlayerId>,
}

impl DoubleTapCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, DoubleTapPlayerStats> {
        &self.player_stats
    }

    pub fn team_zero_stats(&self) -> &DoubleTapTeamStats {
        &self.team_zero_stats
    }

    pub fn team_one_stats(&self) -> &DoubleTapTeamStats {
        &self.team_one_stats
    }

    pub fn events(&self) -> &[DoubleTapEvent] {
        &self.events
    }

    fn begin_sample(&mut self, frame: &FrameInfo) {
        for stats in self.player_stats.values_mut() {
            stats.is_last_double_tap = false;
            stats.time_since_last_double_tap = stats
                .last_double_tap_time
                .map(|time| (frame.time - time).max(0.0));
            stats.frames_since_last_double_tap = stats
                .last_double_tap_frame
                .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
        }
    }

    fn prune_pending_backboard_bounces(&mut self, current_time: f32) {
        self.pending_backboard_bounces
            .retain(|entry| current_time - entry.time <= DOUBLE_TAP_TOUCH_WINDOW_SECONDS);
    }

    fn record_backboard_bounces(&mut self, state: &BackboardBounceState) {
        for event in &state.bounce_events {
            if let Some(existing) = self
                .pending_backboard_bounces
                .iter_mut()
                .find(|pending| pending.player_id == event.player)
            {
                *existing = PendingBackboardBounce {
                    player_id: event.player.clone(),
                    is_team_0: event.is_team_0,
                    time: event.time,
                    frame: event.frame,
                };
            } else {
                self.pending_backboard_bounces.push(PendingBackboardBounce {
                    player_id: event.player.clone(),
                    is_team_0: event.is_team_0,
                    time: event.time,
                    frame: event.frame,
                });
            }
        }
    }

    fn resolve_double_tap_touches(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        touch_events: &[TouchEvent],
    ) {
        if touch_events.is_empty() || self.pending_backboard_bounces.is_empty() {
            return;
        }

        let mut completed_events = Vec::new();
        self.pending_backboard_bounces.retain(|pending| {
            if frame.time <= pending.time {
                return true;
            }

            let matching_touch = touch_events.iter().any(|touch| {
                touch.team_is_team_0 == pending.is_team_0
                    && touch.player.as_ref() == Some(&pending.player_id)
            });
            let conflicting_touch = touch_events
                .iter()
                .any(|touch| touch.player.as_ref() != Some(&pending.player_id));

            if matching_touch
                && !conflicting_touch
                && Self::followup_touch_is_goal_directed(ball, pending.is_team_0)
            {
                completed_events.push(DoubleTapEvent {
                    time: frame.time,
                    frame: frame.frame_number,
                    player: pending.player_id.clone(),
                    is_team_0: pending.is_team_0,
                    backboard_time: pending.time,
                    backboard_frame: pending.frame,
                });
            }
            false
        });

        for event in completed_events {
            self.record_double_tap(frame, event);
        }
    }

    fn record_double_tap(&mut self, frame: &FrameInfo, event: DoubleTapEvent) {
        let stats = self.player_stats.entry(event.player.clone()).or_default();
        stats.count += 1;
        stats.last_double_tap_time = Some(event.time);
        stats.last_double_tap_frame = Some(event.frame);
        stats.time_since_last_double_tap = Some((frame.time - event.time).max(0.0));
        stats.frames_since_last_double_tap = Some(frame.frame_number.saturating_sub(event.frame));

        let team_stats = if event.is_team_0 {
            &mut self.team_zero_stats
        } else {
            &mut self.team_one_stats
        };
        team_stats.count += 1;
        self.current_last_double_tap_player = Some(event.player.clone());
        self.events.push(event);
    }

    fn followup_touch_is_goal_directed(ball: &BallFrameState, is_team_0: bool) -> bool {
        const GOAL_CENTER_Y: f32 = 5120.0;
        const MIN_GOAL_ALIGNMENT_COSINE: f32 = 0.6;

        let Some(ball) = ball.sample() else {
            return false;
        };

        let target_y = if is_team_0 {
            GOAL_CENTER_Y
        } else {
            -GOAL_CENTER_Y
        };
        let ball_velocity = ball.velocity();
        if ball_velocity.length_squared() <= f32::EPSILON {
            return false;
        }

        let goal_direction = glam::Vec3::new(0.0, target_y, ball.position().z) - ball.position();
        goal_direction
            .normalize_or_zero()
            .dot(ball_velocity.normalize_or_zero())
            >= MIN_GOAL_ALIGNMENT_COSINE
    }

    pub fn update(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        events: &FrameEventsState,
        backboard_bounce_state: &BackboardBounceState,
        live_play: bool,
    ) -> SubtrActorResult<()> {
        self.begin_sample(frame);
        if !live_play {
            self.pending_backboard_bounces.clear();
        }

        self.prune_pending_backboard_bounces(frame.time);
        self.record_backboard_bounces(backboard_bounce_state);
        self.resolve_double_tap_touches(frame, ball, &events.touch_events);

        if let Some(player_id) = self.current_last_double_tap_player.as_ref() {
            if let Some(stats) = self.player_stats.get_mut(player_id) {
                stats.is_last_double_tap = true;
            }
        }
        Ok(())
    }
}
