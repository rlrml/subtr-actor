use super::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct BallCarryStats {
    pub carry_count: u32,
    pub total_carry_time: f32,
    pub total_straight_line_distance: f32,
    pub total_path_distance: f32,
    pub longest_carry_time: f32,
    pub furthest_carry_distance: f32,
    pub fastest_carry_speed: f32,
    pub carry_speed_sum: f32,
    pub average_horizontal_gap_sum: f32,
    pub average_vertical_gap_sum: f32,
}

impl BallCarryStats {
    fn pct_count_average(&self, value: f32) -> f32 {
        if self.carry_count == 0 {
            0.0
        } else {
            value / self.carry_count as f32
        }
    }

    pub fn average_carry_time(&self) -> f32 {
        self.pct_count_average(self.total_carry_time)
    }

    pub fn average_straight_line_distance(&self) -> f32 {
        self.pct_count_average(self.total_straight_line_distance)
    }

    pub fn average_path_distance(&self) -> f32 {
        self.pct_count_average(self.total_path_distance)
    }

    pub fn average_carry_speed(&self) -> f32 {
        self.pct_count_average(self.carry_speed_sum)
    }

    pub fn average_horizontal_gap(&self) -> f32 {
        self.pct_count_average(self.average_horizontal_gap_sum)
    }

    pub fn average_vertical_gap(&self) -> f32 {
        self.pct_count_average(self.average_vertical_gap_sum)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BallCarryEvent {
    pub player_id: PlayerId,
    pub is_team_0: bool,
    pub start_frame: usize,
    pub end_frame: usize,
    pub start_time: f32,
    pub end_time: f32,
    pub duration: f32,
    pub straight_line_distance: f32,
    pub path_distance: f32,
    pub average_horizontal_gap: f32,
    pub average_vertical_gap: f32,
    pub average_speed: f32,
}

#[derive(Debug, Clone)]
struct ActiveBallCarry {
    player_id: PlayerId,
    is_team_0: bool,
    start_frame: usize,
    last_frame: usize,
    start_time: f32,
    last_time: f32,
    start_position: glam::Vec3,
    last_position: glam::Vec3,
    duration: f32,
    path_distance: f32,
    horizontal_gap_integral: f32,
    vertical_gap_integral: f32,
    speed_integral: f32,
}

#[derive(Debug, Clone, Copy)]
struct BallCarryFrameSample {
    player_position: glam::Vec3,
    horizontal_gap: f32,
    vertical_gap: f32,
    speed: f32,
}

#[derive(Debug, Clone, Default)]
pub struct BallCarryCalculator {
    player_stats: HashMap<PlayerId, BallCarryStats>,
    team_zero_stats: BallCarryStats,
    team_one_stats: BallCarryStats,
    carry_events: Vec<BallCarryEvent>,
    active_carry: Option<ActiveBallCarry>,
    last_touch_player: Option<PlayerId>,
}

impl BallCarryCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, BallCarryStats> {
        &self.player_stats
    }

    pub fn team_zero_stats(&self) -> &BallCarryStats {
        &self.team_zero_stats
    }

    pub fn team_one_stats(&self) -> &BallCarryStats {
        &self.team_one_stats
    }

    pub fn carry_events(&self) -> &[BallCarryEvent] {
        &self.carry_events
    }

    fn carry_frame_sample(
        player: &PlayerSample,
        ball: &BallSample,
    ) -> Option<BallCarryFrameSample> {
        let player_position = player.position()?;
        let ball_position = ball.position();
        if !(BALL_CARRY_MIN_BALL_Z..=BALL_CARRY_MAX_BALL_Z).contains(&ball_position.z) {
            return None;
        }

        let horizontal_gap = player_position
            .truncate()
            .distance(ball_position.truncate());
        if horizontal_gap > BALL_CARRY_MAX_HORIZONTAL_GAP {
            return None;
        }

        let vertical_gap = ball_position.z - player_position.z;
        if !(0.0..=BALL_CARRY_MAX_VERTICAL_GAP).contains(&vertical_gap) {
            return None;
        }

        Some(BallCarryFrameSample {
            player_position,
            horizontal_gap,
            vertical_gap,
            speed: player.speed().unwrap_or(0.0),
        })
    }

    fn begin_carry(
        &self,
        frame: &FrameInfo,
        player: &PlayerSample,
        frame_sample: BallCarryFrameSample,
    ) -> ActiveBallCarry {
        let start_time = (frame.time - frame.dt).max(0.0);
        let start_frame = frame.frame_number.saturating_sub(1);
        ActiveBallCarry {
            player_id: player.player_id.clone(),
            is_team_0: player.is_team_0,
            start_frame,
            last_frame: frame.frame_number,
            start_time,
            last_time: frame.time,
            start_position: frame_sample.player_position,
            last_position: frame_sample.player_position,
            duration: frame.dt,
            path_distance: 0.0,
            horizontal_gap_integral: frame_sample.horizontal_gap * frame.dt,
            vertical_gap_integral: frame_sample.vertical_gap * frame.dt,
            speed_integral: frame_sample.speed * frame.dt,
        }
    }

    fn extend_carry(
        active_carry: &mut ActiveBallCarry,
        frame: &FrameInfo,
        frame_sample: BallCarryFrameSample,
    ) {
        active_carry.duration += frame.dt;
        active_carry.path_distance += frame_sample
            .player_position
            .distance(active_carry.last_position);
        active_carry.last_position = frame_sample.player_position;
        active_carry.last_time = frame.time;
        active_carry.last_frame = frame.frame_number;
        active_carry.horizontal_gap_integral += frame_sample.horizontal_gap * frame.dt;
        active_carry.vertical_gap_integral += frame_sample.vertical_gap * frame.dt;
        active_carry.speed_integral += frame_sample.speed * frame.dt;
    }

    fn finalize_active_carry(&mut self) {
        let Some(active_carry) = self.active_carry.take() else {
            return;
        };
        if active_carry.duration < BALL_CARRY_MIN_DURATION {
            return;
        }

        let event = BallCarryEvent {
            player_id: active_carry.player_id.clone(),
            is_team_0: active_carry.is_team_0,
            start_frame: active_carry.start_frame,
            end_frame: active_carry.last_frame,
            start_time: active_carry.start_time,
            end_time: active_carry.last_time,
            duration: active_carry.duration,
            straight_line_distance: active_carry
                .start_position
                .truncate()
                .distance(active_carry.last_position.truncate()),
            path_distance: active_carry.path_distance,
            average_horizontal_gap: active_carry.horizontal_gap_integral / active_carry.duration,
            average_vertical_gap: active_carry.vertical_gap_integral / active_carry.duration,
            average_speed: active_carry.speed_integral / active_carry.duration,
        };
        self.record_carry_event(event);
    }

    fn record_carry_event(&mut self, event: BallCarryEvent) {
        let player_stats = self
            .player_stats
            .entry(event.player_id.clone())
            .or_default();
        Self::apply_carry_event(player_stats, &event);

        let team_stats = if event.is_team_0 {
            &mut self.team_zero_stats
        } else {
            &mut self.team_one_stats
        };
        Self::apply_carry_event(team_stats, &event);
        self.carry_events.push(event);
    }

    fn apply_carry_event(stats: &mut BallCarryStats, event: &BallCarryEvent) {
        stats.carry_count += 1;
        stats.total_carry_time += event.duration;
        stats.total_straight_line_distance += event.straight_line_distance;
        stats.total_path_distance += event.path_distance;
        stats.longest_carry_time = stats.longest_carry_time.max(event.duration);
        stats.furthest_carry_distance = stats
            .furthest_carry_distance
            .max(event.straight_line_distance);
        stats.fastest_carry_speed = stats.fastest_carry_speed.max(event.average_speed);
        stats.carry_speed_sum += event.average_speed;
        stats.average_horizontal_gap_sum += event.average_horizontal_gap;
        stats.average_vertical_gap_sum += event.average_vertical_gap;
    }

    fn process_sample(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        live_play: bool,
        controlling_player: Option<PlayerId>,
    ) -> SubtrActorResult<()> {
        let carry_candidate = if live_play && frame.dt > 0.0 {
            if let (Some(ball), Some(player_id)) = (ball.sample(), controlling_player.as_ref()) {
                players
                    .players
                    .iter()
                    .find(|player| &player.player_id == player_id)
                    .and_then(|player| {
                        Self::carry_frame_sample(player, ball)
                            .map(|frame_sample| (player, frame_sample))
                    })
            } else {
                None
            }
        } else {
            None
        };

        match (self.active_carry.as_mut(), carry_candidate) {
            (Some(active_carry), Some((player, frame_sample)))
                if active_carry.player_id == player.player_id =>
            {
                Self::extend_carry(active_carry, frame, frame_sample);
            }
            (Some(_), Some((player, frame_sample))) => {
                self.finalize_active_carry();
                self.active_carry = Some(self.begin_carry(frame, player, frame_sample));
            }
            (Some(_), None) => {
                self.finalize_active_carry();
            }
            (None, Some((player, frame_sample))) => {
                self.active_carry = Some(self.begin_carry(frame, player, frame_sample));
            }
            (None, None) => {}
        }

        if let Some(active_carry) = &self.active_carry {
            if controlling_player.as_ref() != Some(&active_carry.player_id) {
                self.finalize_active_carry();
            }
        }

        self.last_touch_player = controlling_player;
        Ok(())
    }

    pub fn update(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        live_play: bool,
        controlling_player: Option<PlayerId>,
    ) -> SubtrActorResult<()> {
        self.process_sample(frame, ball, players, live_play, controlling_player)
    }
    pub fn finish_calculation(&mut self) -> SubtrActorResult<()> {
        self.finalize_active_carry();
        Ok(())
    }
}
