use super::*;

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct BallCarryEvent {
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
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

#[derive(Debug, Clone, Default)]
pub struct BallCarryCalculator {
    carry_events: EventStream<BallCarryEvent>,
    processed_control_sequence_count: usize,
}

impl BallCarryCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn carry_events(&self) -> &[BallCarryEvent] {
        self.carry_events.all()
    }

    pub fn new_carry_events(&self) -> &[BallCarryEvent] {
        self.carry_events.new_events()
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
        self.carry_events.push(event);
    }

    pub fn update(&mut self, control_state: &ContinuousBallControlState) -> SubtrActorResult<()> {
        self.carry_events.begin_update();
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
