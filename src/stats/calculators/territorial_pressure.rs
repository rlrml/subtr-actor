use super::*;

const DEFAULT_TERRITORIAL_PRESSURE_NEUTRAL_ZONE_HALF_WIDTH_Y: f32 = 200.0;
const DEFAULT_TERRITORIAL_PRESSURE_MIN_ESTABLISH_SECONDS: f32 = 2.0;
const DEFAULT_TERRITORIAL_PRESSURE_MIN_ESTABLISH_THIRD_SECONDS: f32 = 0.75;
const DEFAULT_TERRITORIAL_PRESSURE_RELIEF_GRACE_SECONDS: f32 = 3.0;
const DEFAULT_TERRITORIAL_PRESSURE_CONFIRMED_RELIEF_GRACE_SECONDS: f32 = 1.25;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum TerritorialPressureEndReason {
    Relieved,
    Stoppage,
    BallMissing,
    ReplayEnd,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TerritorialPressureStats {
    pub tracked_time: f32,
    pub team_zero_session_count: u32,
    pub team_one_session_count: u32,
    pub team_zero_session_time: f32,
    pub team_one_session_time: f32,
    pub team_zero_offensive_half_time: f32,
    pub team_one_offensive_half_time: f32,
    pub team_zero_offensive_third_time: f32,
    pub team_one_offensive_third_time: f32,
    pub team_zero_longest_session_time: f32,
    pub team_one_longest_session_time: f32,
    #[serde(default, skip_serializing_if = "LabeledCounts::is_empty")]
    pub labeled_session_counts: LabeledCounts,
    #[serde(default, skip_serializing_if = "LabeledFloatSums::is_empty")]
    pub labeled_time: LabeledFloatSums,
}

impl TerritorialPressureStats {
    pub fn for_team(&self, is_team_zero: bool) -> TerritorialPressureTeamStats {
        let (
            session_count,
            opponent_session_count,
            session_time,
            opponent_session_time,
            offensive_half_time,
            offensive_third_time,
            longest_session_time,
            opponent_longest_session_time,
        ) = if is_team_zero {
            (
                self.team_zero_session_count,
                self.team_one_session_count,
                self.team_zero_session_time,
                self.team_one_session_time,
                self.team_zero_offensive_half_time,
                self.team_zero_offensive_third_time,
                self.team_zero_longest_session_time,
                self.team_one_longest_session_time,
            )
        } else {
            (
                self.team_one_session_count,
                self.team_zero_session_count,
                self.team_one_session_time,
                self.team_zero_session_time,
                self.team_one_offensive_half_time,
                self.team_one_offensive_third_time,
                self.team_one_longest_session_time,
                self.team_zero_longest_session_time,
            )
        };

        let average_session_time = if session_count == 0 {
            0.0
        } else {
            session_time / session_count as f32
        };

        TerritorialPressureTeamStats {
            tracked_time: self.tracked_time,
            session_count,
            opponent_session_count,
            session_time,
            opponent_session_time,
            offensive_half_time,
            offensive_third_time,
            longest_session_time,
            opponent_longest_session_time,
            average_session_time,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct TerritorialPressureTeamStats {
    pub tracked_time: f32,
    pub session_count: u32,
    pub opponent_session_count: u32,
    pub session_time: f32,
    pub opponent_session_time: f32,
    pub offensive_half_time: f32,
    pub offensive_third_time: f32,
    pub longest_session_time: f32,
    pub opponent_longest_session_time: f32,
    pub average_session_time: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct TerritorialPressureEvent {
    pub start_time: f32,
    pub start_frame: usize,
    pub end_time: f32,
    pub end_frame: usize,
    pub team_is_team_0: bool,
    pub duration: f32,
    pub offensive_half_time: f32,
    pub offensive_third_time: f32,
    pub end_reason: TerritorialPressureEndReason,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TerritorialPressureCalculatorConfig {
    pub neutral_zone_half_width_y: f32,
    pub min_establish_seconds: f32,
    pub min_establish_third_seconds: f32,
    pub relief_grace_seconds: f32,
    pub confirmed_relief_grace_seconds: f32,
}

impl Default for TerritorialPressureCalculatorConfig {
    fn default() -> Self {
        Self {
            neutral_zone_half_width_y: DEFAULT_TERRITORIAL_PRESSURE_NEUTRAL_ZONE_HALF_WIDTH_Y,
            min_establish_seconds: DEFAULT_TERRITORIAL_PRESSURE_MIN_ESTABLISH_SECONDS,
            min_establish_third_seconds: DEFAULT_TERRITORIAL_PRESSURE_MIN_ESTABLISH_THIRD_SECONDS,
            relief_grace_seconds: DEFAULT_TERRITORIAL_PRESSURE_RELIEF_GRACE_SECONDS,
            confirmed_relief_grace_seconds:
                DEFAULT_TERRITORIAL_PRESSURE_CONFIRMED_RELIEF_GRACE_SECONDS,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TerritorialPressureCalculator {
    config: TerritorialPressureCalculatorConfig,
    stats: TerritorialPressureStats,
    events: EventStream<TerritorialPressureEvent>,
    candidate: Option<CandidateTerritorialPressureSession>,
    active: Option<ActiveTerritorialPressureSession>,
    last_frame: Option<TerritorialPressureFrameMarker>,
}

#[derive(Debug, Clone, PartialEq)]
struct CandidateTerritorialPressureSession {
    team_is_team_0: bool,
    start_time: f32,
    start_frame: usize,
    duration: f32,
    offensive_half_time: f32,
    offensive_third_time: f32,
}

#[derive(Debug, Clone, PartialEq)]
struct ActiveTerritorialPressureSession {
    team_is_team_0: bool,
    start_time: f32,
    start_frame: usize,
    duration: f32,
    offensive_half_time: f32,
    offensive_third_time: f32,
    relief_time: f32,
    confirmed_relief_time: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct TerritorialPressureFrameMarker {
    frame_number: usize,
    time: f32,
}

impl From<&FrameInfo> for TerritorialPressureFrameMarker {
    fn from(frame: &FrameInfo) -> Self {
        Self {
            frame_number: frame.frame_number,
            time: frame.time,
        }
    }
}

impl TerritorialPressureCalculator {
    pub fn new() -> Self {
        Self::with_config(TerritorialPressureCalculatorConfig::default())
    }

    pub fn with_config(config: TerritorialPressureCalculatorConfig) -> Self {
        Self {
            config,
            ..Self::default()
        }
    }

    pub fn stats(&self) -> &TerritorialPressureStats {
        &self.stats
    }

    pub fn events(&self) -> &[TerritorialPressureEvent] {
        self.events.all()
    }

    pub fn new_events(&self) -> &[TerritorialPressureEvent] {
        self.events.new_events()
    }

    pub fn config(&self) -> &TerritorialPressureCalculatorConfig {
        &self.config
    }

    pub fn finish(&mut self) -> SubtrActorResult<()> {
        if let Some(frame) = self.last_frame {
            self.end_active_session_parts(
                frame.frame_number,
                frame.time,
                TerritorialPressureEndReason::ReplayEnd,
            );
        }
        Ok(())
    }

    fn pressure_team_for_ball_y(&self, ball_y: f32) -> Option<bool> {
        if ball_y > self.config.neutral_zone_half_width_y {
            Some(true)
        } else if ball_y < -self.config.neutral_zone_half_width_y {
            Some(false)
        } else {
            None
        }
    }

    fn normalized_ball_y(team_is_team_0: bool, ball_y: f32) -> f32 {
        if team_is_team_0 {
            ball_y
        } else {
            -ball_y
        }
    }

    fn pressure_team_label(team_is_team_0: bool) -> StatLabel {
        StatLabel::new(
            "pressure_team",
            if team_is_team_0 {
                "team_zero"
            } else {
                "team_one"
            },
        )
    }

    fn territory_label(normalized_ball_y: f32) -> StatLabel {
        if normalized_ball_y > FIELD_ZONE_BOUNDARY_Y {
            StatLabel::new("territory", "offensive_third")
        } else if normalized_ball_y > 0.0 {
            StatLabel::new("territory", "offensive_half")
        } else {
            StatLabel::new("territory", "relief")
        }
    }

    fn add_session_count(&mut self, team_is_team_0: bool) {
        if team_is_team_0 {
            self.stats.team_zero_session_count += 1;
        } else {
            self.stats.team_one_session_count += 1;
        }
        self.stats
            .labeled_session_counts
            .increment([Self::pressure_team_label(team_is_team_0)]);
    }

    fn add_session_time(&mut self, team_is_team_0: bool, normalized_ball_y: f32, dt: f32) {
        if team_is_team_0 {
            self.stats.team_zero_session_time += dt;
            if normalized_ball_y > 0.0 {
                self.stats.team_zero_offensive_half_time += dt;
            }
            if normalized_ball_y > FIELD_ZONE_BOUNDARY_Y {
                self.stats.team_zero_offensive_third_time += dt;
            }
        } else {
            self.stats.team_one_session_time += dt;
            if normalized_ball_y > 0.0 {
                self.stats.team_one_offensive_half_time += dt;
            }
            if normalized_ball_y > FIELD_ZONE_BOUNDARY_Y {
                self.stats.team_one_offensive_third_time += dt;
            }
        }

        self.stats.labeled_time.add(
            [
                Self::pressure_team_label(team_is_team_0),
                Self::territory_label(normalized_ball_y),
            ],
            dt,
        );
    }

    fn update_longest_session_time(&mut self, team_is_team_0: bool, duration: f32) {
        if team_is_team_0 {
            self.stats.team_zero_longest_session_time =
                self.stats.team_zero_longest_session_time.max(duration);
        } else {
            self.stats.team_one_longest_session_time =
                self.stats.team_one_longest_session_time.max(duration);
        }
    }

    fn candidate_sample(
        team_is_team_0: bool,
        frame: &FrameInfo,
        normalized_ball_y: f32,
    ) -> CandidateTerritorialPressureSession {
        CandidateTerritorialPressureSession {
            team_is_team_0,
            start_time: frame.time,
            start_frame: frame.frame_number,
            duration: frame.dt,
            offensive_half_time: if normalized_ball_y > 0.0 {
                frame.dt
            } else {
                0.0
            },
            offensive_third_time: if normalized_ball_y > FIELD_ZONE_BOUNDARY_Y {
                frame.dt
            } else {
                0.0
            },
        }
    }

    fn update_candidate(&mut self, frame: &FrameInfo, ball_y: f32) {
        let Some(team_is_team_0) = self.pressure_team_for_ball_y(ball_y) else {
            self.candidate = None;
            return;
        };
        let normalized_ball_y = Self::normalized_ball_y(team_is_team_0, ball_y);

        if self
            .candidate
            .as_ref()
            .is_none_or(|candidate| candidate.team_is_team_0 != team_is_team_0)
        {
            self.candidate = Some(Self::candidate_sample(
                team_is_team_0,
                frame,
                normalized_ball_y,
            ));
        } else if let Some(candidate) = &mut self.candidate {
            candidate.duration += frame.dt;
            if normalized_ball_y > 0.0 {
                candidate.offensive_half_time += frame.dt;
            }
            if normalized_ball_y > FIELD_ZONE_BOUNDARY_Y {
                candidate.offensive_third_time += frame.dt;
            }
        }

        let should_start = self.candidate.as_ref().is_some_and(|candidate| {
            candidate.duration >= self.config.min_establish_seconds
                || candidate.offensive_third_time >= self.config.min_establish_third_seconds
        });
        if should_start {
            let candidate = self
                .candidate
                .take()
                .expect("candidate exists when pressure should start");
            self.start_session(candidate);
        }
    }

    fn start_session(&mut self, candidate: CandidateTerritorialPressureSession) {
        self.add_session_count(candidate.team_is_team_0);
        self.add_session_time(
            candidate.team_is_team_0,
            1.0,
            candidate.offensive_half_time - candidate.offensive_third_time,
        );
        self.add_session_time(
            candidate.team_is_team_0,
            FIELD_ZONE_BOUNDARY_Y + 1.0,
            candidate.offensive_third_time,
        );
        self.update_longest_session_time(candidate.team_is_team_0, candidate.duration);
        self.active = Some(ActiveTerritorialPressureSession {
            team_is_team_0: candidate.team_is_team_0,
            start_time: candidate.start_time,
            start_frame: candidate.start_frame,
            duration: candidate.duration,
            offensive_half_time: candidate.offensive_half_time,
            offensive_third_time: candidate.offensive_third_time,
            relief_time: 0.0,
            confirmed_relief_time: 0.0,
        });
    }

    fn update_active_session(
        &mut self,
        frame: &FrameInfo,
        ball_y: f32,
        possession_state: &PossessionState,
    ) {
        let Some(mut active) = self.active.take() else {
            return;
        };

        let normalized_ball_y = Self::normalized_ball_y(active.team_is_team_0, ball_y);
        active.duration += frame.dt;
        if normalized_ball_y > 0.0 {
            active.offensive_half_time += frame.dt;
        }
        if normalized_ball_y > FIELD_ZONE_BOUNDARY_Y {
            active.offensive_third_time += frame.dt;
        }
        self.add_session_time(active.team_is_team_0, normalized_ball_y, frame.dt);
        self.update_longest_session_time(active.team_is_team_0, active.duration);

        if normalized_ball_y > self.config.neutral_zone_half_width_y {
            active.relief_time = 0.0;
            active.confirmed_relief_time = 0.0;
        } else {
            active.relief_time += frame.dt;
            if possession_state.active_team_before_sample == Some(!active.team_is_team_0) {
                active.confirmed_relief_time += frame.dt;
            } else {
                active.confirmed_relief_time = 0.0;
            }
        }

        let relieved = active.confirmed_relief_time >= self.config.confirmed_relief_grace_seconds
            || active.relief_time >= self.config.relief_grace_seconds;

        self.active = Some(active);
        if relieved {
            self.end_active_session(frame, TerritorialPressureEndReason::Relieved);
        }
    }

    fn end_active_session(&mut self, frame: &FrameInfo, end_reason: TerritorialPressureEndReason) {
        self.end_active_session_parts(frame.frame_number, frame.time, end_reason);
    }

    fn end_active_session_parts(
        &mut self,
        end_frame: usize,
        end_time: f32,
        end_reason: TerritorialPressureEndReason,
    ) {
        let Some(active) = self.active.take() else {
            return;
        };
        self.events.push(TerritorialPressureEvent {
            start_time: active.start_time,
            start_frame: active.start_frame,
            end_time,
            end_frame,
            team_is_team_0: active.team_is_team_0,
            duration: active.duration,
            offensive_half_time: active.offensive_half_time,
            offensive_third_time: active.offensive_third_time,
            end_reason,
        });
    }

    pub fn update(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        possession_state: &PossessionState,
        live_play_state: &LivePlayState,
    ) -> SubtrActorResult<()> {
        self.events.begin_update();
        self.last_frame = Some(frame.into());
        if !live_play_state.is_live_play {
            self.candidate = None;
            self.end_active_session(frame, TerritorialPressureEndReason::Stoppage);
            return Ok(());
        }

        let Some(ball) = ball.sample() else {
            self.candidate = None;
            self.end_active_session(frame, TerritorialPressureEndReason::BallMissing);
            return Ok(());
        };

        self.stats.tracked_time += frame.dt;
        if self.active.is_some() {
            self.update_active_session(frame, ball.position().y, possession_state);
        } else {
            self.update_candidate(frame, ball.position().y);
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "territorial_pressure_tests.rs"]
mod tests;
