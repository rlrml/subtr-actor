use super::*;

const CENTER_MAX_DURATION_SECONDS: f32 = 3.0;
const CENTER_MIN_BALL_TRAVEL_DISTANCE: f32 = 500.0;
const CENTER_MIN_LATERAL_DISTANCE: f32 = 500.0;
const CENTER_MIN_START_ABS_X: f32 = 1600.0;
const CENTER_MAX_END_ABS_X: f32 = 1400.0;
const CENTER_MIN_START_ATTACKING_Y: f32 = BOOST_PAD_MIDFIELD_TOLERANCE_Y;
const CENTER_MIN_END_ATTACKING_Y: f32 = FIELD_ZONE_BOUNDARY_Y;

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct CenterEvent {
    pub time: f32,
    pub frame: usize,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_position: Option<[f32; 3]>,
    pub is_team_0: bool,
    pub start_time: f32,
    pub start_frame: usize,
    pub duration: f32,
    pub start_ball_position: [f32; 3],
    pub end_ball_position: [f32; 3],
    pub ball_travel_distance: f32,
    pub ball_advance_distance: f32,
    pub lateral_centering_distance: f32,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct CenterPlayerStats {
    pub count: u32,
    pub total_ball_travel_distance: f32,
    pub total_ball_advance_distance: f32,
    pub total_lateral_centering_distance: f32,
    pub longest_center_distance: f32,
    pub is_last_center: bool,
    pub last_center_time: Option<f32>,
    pub last_center_frame: Option<usize>,
    pub time_since_last_center: Option<f32>,
    pub frames_since_last_center: Option<usize>,
}

impl CenterPlayerStats {
    pub fn average_ball_travel_distance(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            self.total_ball_travel_distance / self.count as f32
        }
    }

    pub fn average_ball_advance_distance(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            self.total_ball_advance_distance / self.count as f32
        }
    }

    pub fn average_lateral_centering_distance(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            self.total_lateral_centering_distance / self.count as f32
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct CenterTeamStats {
    pub count: u32,
    pub total_ball_travel_distance: f32,
    pub total_ball_advance_distance: f32,
    pub total_lateral_centering_distance: f32,
    pub longest_center_distance: f32,
}

impl CenterTeamStats {
    pub fn average_ball_travel_distance(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            self.total_ball_travel_distance / self.count as f32
        }
    }

    pub fn average_ball_advance_distance(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            self.total_ball_advance_distance / self.count as f32
        }
    }

    pub fn average_lateral_centering_distance(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            self.total_lateral_centering_distance / self.count as f32
        }
    }
}

#[derive(Debug, Clone)]
struct PendingCenterTouch {
    player: PlayerId,
    player_position: Option<[f32; 3]>,
    is_team_0: bool,
    time: f32,
    frame: usize,
    ball_position: glam::Vec3,
}

#[derive(Debug, Clone, Default)]
pub struct CenterCalculator {
    player_stats: HashMap<PlayerId, CenterPlayerStats>,
    team_zero_stats: CenterTeamStats,
    team_one_stats: CenterTeamStats,
    events: EventStream<CenterEvent>,
    pending_touch: Option<PendingCenterTouch>,
    current_last_center_player: Option<PlayerId>,
}

impl CenterCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, CenterPlayerStats> {
        &self.player_stats
    }

    pub fn team_zero_stats(&self) -> &CenterTeamStats {
        &self.team_zero_stats
    }

    pub fn team_one_stats(&self) -> &CenterTeamStats {
        &self.team_one_stats
    }

    pub fn events(&self) -> &[CenterEvent] {
        self.events.all()
    }

    pub fn new_events(&self) -> &[CenterEvent] {
        self.events.new_events()
    }

    fn begin_sample(&mut self, frame: &FrameInfo) {
        for stats in self.player_stats.values_mut() {
            stats.is_last_center = false;
            stats.time_since_last_center = stats
                .last_center_time
                .map(|time| (frame.time - time).max(0.0));
            stats.frames_since_last_center = stats
                .last_center_frame
                .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
        }
    }

    fn center_event_for_position(
        pending: &PendingCenterTouch,
        frame: &FrameInfo,
        ball_position: glam::Vec3,
    ) -> Option<CenterEvent> {
        let duration = frame.time - pending.time;
        if !(0.0..=CENTER_MAX_DURATION_SECONDS).contains(&duration) {
            return None;
        }

        let start_normalized_y = normalized_y(pending.is_team_0, pending.ball_position);
        let end_normalized_y = normalized_y(pending.is_team_0, ball_position);
        if start_normalized_y < CENTER_MIN_START_ATTACKING_Y
            || end_normalized_y < CENTER_MIN_END_ATTACKING_Y
        {
            return None;
        }

        let start_abs_x = pending.ball_position.x.abs();
        let end_abs_x = ball_position.x.abs();
        let lateral_centering_distance = start_abs_x - end_abs_x;
        if start_abs_x < CENTER_MIN_START_ABS_X
            || end_abs_x > CENTER_MAX_END_ABS_X
            || lateral_centering_distance < CENTER_MIN_LATERAL_DISTANCE
        {
            return None;
        }

        let ball_delta = ball_position - pending.ball_position;
        let ball_travel_distance = ball_delta.length();
        if ball_travel_distance < CENTER_MIN_BALL_TRAVEL_DISTANCE {
            return None;
        }

        let team_forward_sign = if pending.is_team_0 { 1.0 } else { -1.0 };
        Some(CenterEvent {
            time: frame.time,
            frame: frame.frame_number,
            player: pending.player.clone(),
            player_position: pending.player_position,
            is_team_0: pending.is_team_0,
            start_time: pending.time,
            start_frame: pending.frame,
            duration,
            start_ball_position: pending.ball_position.to_array(),
            end_ball_position: ball_position.to_array(),
            ball_travel_distance,
            ball_advance_distance: ball_delta.y * team_forward_sign,
            lateral_centering_distance,
        })
    }

    fn record_center(&mut self, frame: &FrameInfo, event: CenterEvent) {
        let player_stats = self.player_stats.entry(event.player.clone()).or_default();
        player_stats.count += 1;
        player_stats.total_ball_travel_distance += event.ball_travel_distance;
        player_stats.total_ball_advance_distance += event.ball_advance_distance;
        player_stats.total_lateral_centering_distance += event.lateral_centering_distance;
        player_stats.longest_center_distance = player_stats
            .longest_center_distance
            .max(event.ball_travel_distance);
        player_stats.last_center_time = Some(event.time);
        player_stats.last_center_frame = Some(event.frame);
        player_stats.time_since_last_center = Some((frame.time - event.time).max(0.0));
        player_stats.frames_since_last_center =
            Some(frame.frame_number.saturating_sub(event.frame));

        let team_stats = if event.is_team_0 {
            &mut self.team_zero_stats
        } else {
            &mut self.team_one_stats
        };
        team_stats.count += 1;
        team_stats.total_ball_travel_distance += event.ball_travel_distance;
        team_stats.total_ball_advance_distance += event.ball_advance_distance;
        team_stats.total_lateral_centering_distance += event.lateral_centering_distance;
        team_stats.longest_center_distance = team_stats
            .longest_center_distance
            .max(event.ball_travel_distance);

        self.current_last_center_player = Some(event.player.clone());
        self.events.push(event);
        self.pending_touch = None;
    }

    fn update_pending_center(&mut self, frame: &FrameInfo, ball_position: glam::Vec3) {
        let Some(pending) = self.pending_touch.as_ref() else {
            return;
        };
        let duration = frame.time - pending.time;
        if duration > CENTER_MAX_DURATION_SECONDS {
            self.pending_touch = None;
            return;
        }

        if let Some(event) = Self::center_event_for_position(pending, frame, ball_position) {
            self.record_center(frame, event);
        }
    }

    fn player_has_disqualifying_event(
        events: &FrameEventsState,
        player: &PlayerId,
        is_team_0: bool,
    ) -> bool {
        events.player_stat_events.iter().any(|event| {
            event.kind == PlayerStatEventKind::Shot
                && event.player == *player
                && event.is_team_0 == is_team_0
        }) || events
            .goal_events
            .iter()
            .any(|event| match event.player.as_ref() {
                Some(scorer) => scorer == player,
                None => event.scoring_team_is_team_0 == is_team_0,
            })
    }

    fn clear_disqualified_pending_center(&mut self, events: &FrameEventsState) {
        let should_clear = self.pending_touch.as_ref().is_some_and(|pending| {
            Self::player_has_disqualifying_event(events, &pending.player, pending.is_team_0)
        });
        if should_clear {
            self.pending_touch = None;
        }
    }

    pub fn update(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        touch_state: &TouchState,
        frame_events: &FrameEventsState,
        live_play: bool,
    ) -> SubtrActorResult<()> {
        self.events.begin_update();
        self.begin_sample(frame);
        if !live_play {
            self.pending_touch = None;
            self.current_last_center_player = None;
            return Ok(());
        }

        let Some(ball_position) = ball.position() else {
            return Ok(());
        };

        self.clear_disqualified_pending_center(frame_events);
        self.update_pending_center(frame, ball_position);

        for touch in &touch_state.touch_events {
            let Some(player) = touch.player.clone() else {
                self.pending_touch = None;
                continue;
            };

            if Self::player_has_disqualifying_event(frame_events, &player, touch.team_is_team_0) {
                self.pending_touch = None;
                continue;
            }

            self.pending_touch = Some(PendingCenterTouch {
                player,
                player_position: touch
                    .player_position
                    .map(|position| vec_to_glam(&position).to_array()),
                is_team_0: touch.team_is_team_0,
                time: touch.time,
                frame: touch.frame,
                ball_position,
            });
        }

        if let Some(player_id) = self.current_last_center_player.as_ref() {
            if let Some(stats) = self.player_stats.get_mut(player_id) {
                stats.is_last_center = true;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
#[path = "center_tests.rs"]
mod tests;
