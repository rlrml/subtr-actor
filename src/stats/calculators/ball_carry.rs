use super::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
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
    #[serde(default, skip_serializing_if = "LabeledCounts::is_empty")]
    pub labeled_event_counts: LabeledCounts,
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

    fn record_event(&mut self, event: &BallCarryEvent) {
        self.labeled_event_counts
            .increment([ball_carry_kind_label(event.kind)]);
        self.carry_count = self.labeled_event_counts.total();
    }

    pub fn event_count_with_labels(&self, labels: &[StatLabel]) -> u32 {
        self.labeled_event_counts.count_matching(labels)
    }

    pub fn complete_labeled_event_counts(&self) -> LabeledCounts {
        LabeledCounts::complete_from_label_sets(
            &[&BALL_CARRY_KIND_LABELS],
            &self.labeled_event_counts,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct BallCarryEvent {
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub player_id: PlayerId,
    pub is_team_0: bool,
    pub kind: BallCarryKind,
    pub start_position: [f32; 3],
    pub end_position: [f32; 3],
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
    pub touch_count: u32,
    pub air_touch_count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub air_dribble_origin: Option<AirDribbleOrigin>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum BallCarryKind {
    Carry,
    AirDribble,
}

const BALL_CARRY_KIND_LABELS: [StatLabel; 2] = [
    StatLabel::new("kind", "carry"),
    StatLabel::new("kind", "air_dribble"),
];

fn ball_carry_kind_label(kind: BallCarryKind) -> StatLabel {
    match kind {
        BallCarryKind::Carry => StatLabel::new("kind", "carry"),
        BallCarryKind::AirDribble => StatLabel::new("kind", "air_dribble"),
    }
}

#[derive(Debug, Clone, Default)]
pub struct BallCarryCalculator {
    player_stats: HashMap<PlayerId, BallCarryStats>,
    player_air_dribble_stats: HashMap<PlayerId, AirDribbleStats>,
    team_zero_stats: BallCarryStats,
    team_one_stats: BallCarryStats,
    team_zero_air_dribble_stats: AirDribbleStats,
    team_one_air_dribble_stats: AirDribbleStats,
    carry_events: Vec<BallCarryEvent>,
    processed_control_sequence_count: usize,
}

impl BallCarryCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, BallCarryStats> {
        &self.player_stats
    }

    pub fn player_air_dribble_stats(&self) -> &HashMap<PlayerId, AirDribbleStats> {
        &self.player_air_dribble_stats
    }

    pub fn team_zero_stats(&self) -> &BallCarryStats {
        &self.team_zero_stats
    }

    pub fn team_one_stats(&self) -> &BallCarryStats {
        &self.team_one_stats
    }

    pub fn team_zero_air_dribble_stats(&self) -> &AirDribbleStats {
        &self.team_zero_air_dribble_stats
    }

    pub fn team_one_air_dribble_stats(&self) -> &AirDribbleStats {
        &self.team_one_air_dribble_stats
    }

    pub fn carry_events(&self) -> &[BallCarryEvent] {
        &self.carry_events
    }

    pub(crate) fn carry_frame_sample(
        player: &PlayerSample,
        ball: &BallSample,
    ) -> Option<ContinuousBallControlSample<BallCarryKind>> {
        let player_position = player.position()?;
        let ball_position = ball.position();
        let horizontal_gap = player_position
            .truncate()
            .distance(ball_position.truncate());
        let vertical_gap = ball_position.z - player_position.z;

        if AirDribblePolicy::is_sample(player_position, ball_position, horizontal_gap, vertical_gap)
        {
            return Some(ContinuousBallControlSample {
                player_position,
                kind: BallCarryKind::AirDribble,
                horizontal_gap,
                vertical_gap,
                speed: player.speed().unwrap_or(0.0),
            });
        }

        if player_is_on_wall(player_position) {
            return None;
        }

        if !(BALL_CARRY_MIN_BALL_Z..=BALL_CARRY_MAX_BALL_Z).contains(&ball_position.z) {
            return None;
        }

        if horizontal_gap > BALL_CARRY_MAX_HORIZONTAL_GAP {
            return None;
        }

        if !(0.0..=BALL_CARRY_MAX_VERTICAL_GAP).contains(&vertical_gap) {
            return None;
        }

        Some(ContinuousBallControlSample {
            player_position,
            kind: BallCarryKind::Carry,
            horizontal_gap,
            vertical_gap,
            speed: player.speed().unwrap_or(0.0),
        })
    }

    pub(crate) fn kind_requires_airborne(kind: BallCarryKind) -> bool {
        AirDribblePolicy::kind_requires_airborne(kind)
    }

    pub(crate) fn control_player_statuses(
        players: &PlayerFrameState,
    ) -> Vec<ContinuousBallControlPlayerStatus> {
        players
            .players
            .iter()
            .filter_map(|player| {
                Some(ContinuousBallControlPlayerStatus {
                    player_id: player.player_id.clone(),
                    is_airborne: AirDribblePolicy::is_air_touch_position(player.position()?),
                })
            })
            .collect()
    }

    pub(crate) fn control_touches(
        touch_state: &TouchState,
        players: &PlayerFrameState,
    ) -> Vec<ContinuousBallControlTouch> {
        touch_state
            .touch_events
            .iter()
            .filter_map(|touch| {
                let player_id = touch.player.clone()?;
                let player = players
                    .players
                    .iter()
                    .find(|player| player.player_id == player_id)?;
                Some(ContinuousBallControlTouch {
                    player_id,
                    is_airborne: AirDribblePolicy::is_air_touch_position(player.position()?),
                })
            })
            .collect()
    }

    pub(crate) fn min_duration_for_kind(kind: BallCarryKind) -> f32 {
        match kind {
            BallCarryKind::Carry => BALL_CARRY_MIN_DURATION,
            BallCarryKind::AirDribble => AIR_DRIBBLE_MIN_DURATION,
        }
    }

    pub(crate) fn control_candidate(
        ball: &BallFrameState,
        players: &PlayerFrameState,
        live_play: bool,
        touch_state: &TouchState,
    ) -> Option<ContinuousBallControlCandidate<BallCarryKind>> {
        if !live_play {
            return None;
        }
        let ball = ball.sample()?;
        let player_id = touch_state.last_touch_player.as_ref()?;
        let touch_count = touch_state
            .touch_events
            .iter()
            .filter(|event| event.player.as_ref() == Some(player_id))
            .count() as u32;
        players
            .players
            .iter()
            .find(|player| &player.player_id == player_id)
            .and_then(|player| {
                Self::carry_frame_sample(player, ball).map(|sample| {
                    let air_touch_count =
                        if AirDribblePolicy::is_air_touch_position(sample.player_position) {
                            touch_count
                        } else {
                            0
                        };
                    ContinuousBallControlCandidate {
                        player_id: player.player_id.clone(),
                        is_team_0: player.is_team_0,
                        touch_count,
                        air_touch_count,
                        sample,
                    }
                })
            })
    }

    fn event_from_sequence(
        sequence: CompletedBallControlSequence<BallCarryKind>,
    ) -> BallCarryEvent {
        let air_dribble_origin = (sequence.kind == BallCarryKind::AirDribble)
            .then(|| AirDribblePolicy::origin(sequence.start_position));
        BallCarryEvent {
            player_id: sequence.player_id,
            is_team_0: sequence.is_team_0,
            kind: sequence.kind,
            start_position: sequence.start_position.to_array(),
            end_position: sequence.end_position.to_array(),
            start_frame: sequence.start_frame,
            end_frame: sequence.end_frame,
            start_time: sequence.start_time,
            end_time: sequence.end_time,
            duration: sequence.duration,
            straight_line_distance: sequence.straight_line_distance,
            path_distance: sequence.path_distance,
            average_horizontal_gap: sequence.average_horizontal_gap,
            average_vertical_gap: sequence.average_vertical_gap,
            average_speed: sequence.average_speed,
            touch_count: sequence.touch_count,
            air_touch_count: sequence.air_touch_count,
            air_dribble_origin,
        }
    }

    fn record_carry_event(&mut self, event: BallCarryEvent) {
        match event.kind {
            BallCarryKind::Carry => {
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
            }
            BallCarryKind::AirDribble => {
                let player_stats = self
                    .player_air_dribble_stats
                    .entry(event.player_id.clone())
                    .or_default();
                AirDribblePolicy::apply_event(player_stats, &event);

                let team_stats = if event.is_team_0 {
                    &mut self.team_zero_air_dribble_stats
                } else {
                    &mut self.team_one_air_dribble_stats
                };
                AirDribblePolicy::apply_event(team_stats, &event);
            }
        }
        self.carry_events.push(event);
    }

    fn apply_carry_event(stats: &mut BallCarryStats, event: &BallCarryEvent) {
        stats.record_event(event);
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

    pub fn update(&mut self, control_state: &ContinuousBallControlState) -> SubtrActorResult<()> {
        for sequence in control_state
            .completed_sequences
            .iter()
            .skip(self.processed_control_sequence_count)
            .cloned()
        {
            if !AirDribblePolicy::is_valid_sequence(&sequence) {
                continue;
            }
            self.record_carry_event(Self::event_from_sequence(sequence));
        }
        self.processed_control_sequence_count = control_state.completed_sequences.len();
        Ok(())
    }
}

#[cfg(test)]
#[path = "ball_carry_tests.rs"]
mod tests;
