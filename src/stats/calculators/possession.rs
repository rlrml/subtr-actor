use super::*;

const PENDING_TURNOVER_CONFIRMATION_WINDOW_SECONDS: f32 = 1.25;
const LOOSE_BALL_TIMEOUT_SECONDS: f32 = 3.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PossessionStateLabel {
    TeamZero,
    TeamOne,
    Neutral,
}

impl PossessionStateLabel {
    fn as_label(self) -> StatLabel {
        let value = match self {
            Self::TeamZero => "team_zero",
            Self::TeamOne => "team_one",
            Self::Neutral => "neutral",
        };
        StatLabel::new("possession_state", value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
enum FieldThirdLabel {
    TeamZeroThird,
    NeutralThird,
    TeamOneThird,
}

impl FieldThirdLabel {
    fn from_ball(ball: &BallSample) -> Self {
        let ball_y = ball.position().y;
        if ball_y < -FIELD_ZONE_BOUNDARY_Y {
            Self::TeamZeroThird
        } else if ball_y > FIELD_ZONE_BOUNDARY_Y {
            Self::TeamOneThird
        } else {
            Self::NeutralThird
        }
    }

    fn as_label(self) -> StatLabel {
        let value = match self {
            Self::TeamZeroThird => "team_zero_third",
            Self::NeutralThird => "neutral_third",
            Self::TeamOneThird => "team_one_third",
        };
        StatLabel::new("field_third", value)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PossessionStats {
    pub tracked_time: f32,
    pub team_zero_time: f32,
    pub team_one_time: f32,
    pub neutral_time: f32,
    #[serde(default, skip_serializing_if = "LabeledFloatSums::is_empty")]
    pub labeled_time: LabeledFloatSums,
}

impl PossessionStats {
    pub fn team_zero_pct(&self) -> f32 {
        if self.tracked_time == 0.0 {
            0.0
        } else {
            self.team_zero_time * 100.0 / self.tracked_time
        }
    }

    pub fn team_one_pct(&self) -> f32 {
        if self.tracked_time == 0.0 {
            0.0
        } else {
            self.team_one_time * 100.0 / self.tracked_time
        }
    }

    pub fn neutral_pct(&self) -> f32 {
        if self.tracked_time == 0.0 {
            0.0
        } else {
            self.neutral_time * 100.0 / self.tracked_time
        }
    }

    pub fn time_with_labels(&self, labels: &[StatLabel]) -> f32 {
        self.labeled_time.sum_matching(labels)
    }

    pub fn for_team(&self, is_team_zero: bool) -> PossessionTeamStats {
        let (possession_time, opponent_possession_time) = if is_team_zero {
            (self.team_zero_time, self.team_one_time)
        } else {
            (self.team_one_time, self.team_zero_time)
        };

        let mut labeled_time = LabeledFloatSums::default();
        for entry in &self.labeled_time.entries {
            labeled_time.add(
                entry
                    .labels
                    .iter()
                    .map(|label| team_relative_possession_label(label, is_team_zero)),
                entry.value,
            );
        }

        PossessionTeamStats {
            tracked_time: self.tracked_time,
            possession_time,
            opponent_possession_time,
            neutral_time: self.neutral_time,
            labeled_time,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct PossessionTeamStats {
    pub tracked_time: f32,
    pub possession_time: f32,
    pub opponent_possession_time: f32,
    pub neutral_time: f32,
    #[serde(default, skip_serializing_if = "LabeledFloatSums::is_empty")]
    pub labeled_time: LabeledFloatSums,
}

fn team_relative_possession_label(label: &StatLabel, is_team_zero: bool) -> StatLabel {
    match (label.key, label.value) {
        ("possession_state", "team_zero") => StatLabel::new(
            "possession_state",
            if is_team_zero { "own" } else { "opponent" },
        ),
        ("possession_state", "team_one") => StatLabel::new(
            "possession_state",
            if is_team_zero { "opponent" } else { "own" },
        ),
        ("field_third", "team_zero_third") => StatLabel::new(
            "field_third",
            if is_team_zero {
                "defensive_third"
            } else {
                "offensive_third"
            },
        ),
        ("field_third", "team_one_third") => StatLabel::new(
            "field_third",
            if is_team_zero {
                "offensive_third"
            } else {
                "defensive_third"
            },
        ),
        _ => label.clone(),
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct PossessionTracker {
    current_team_is_team_0: Option<bool>,
    current_player: Option<PlayerId>,
    last_possession_touch_time: Option<f32>,
    pending_turnover_team_is_team_0: Option<bool>,
    pending_turnover_touch_time: Option<f32>,
}

impl PossessionTracker {
    fn clear_pending_turnover(&mut self) {
        self.pending_turnover_team_is_team_0 = None;
        self.pending_turnover_touch_time = None;
    }

    pub(crate) fn reset(&mut self) {
        self.current_team_is_team_0 = None;
        self.current_player = None;
        self.last_possession_touch_time = None;
        self.clear_pending_turnover();
    }

    fn expire_pending_turnover(&mut self, time: f32) {
        let Some(pending_time) = self.pending_turnover_touch_time else {
            return;
        };
        if time - pending_time < PENDING_TURNOVER_CONFIRMATION_WINDOW_SECONDS {
            return;
        }

        self.current_team_is_team_0 = None;
        self.current_player = None;
        self.last_possession_touch_time = None;
        self.clear_pending_turnover();
    }

    fn expire_loose_ball(&mut self, time: f32) {
        if self.pending_turnover_team_is_team_0.is_some() {
            return;
        }
        let Some(last_touch_time) = self.last_possession_touch_time else {
            return;
        };
        if time - last_touch_time < LOOSE_BALL_TIMEOUT_SECONDS {
            return;
        }

        self.current_team_is_team_0 = None;
        self.current_player = None;
        self.last_possession_touch_time = None;
    }

    fn register_single_team_touch(&mut self, team_is_team_0: bool, time: f32) {
        if self.current_team_is_team_0 == Some(team_is_team_0) {
            self.last_possession_touch_time = Some(time);
            self.clear_pending_turnover();
            return;
        }

        if self.current_team_is_team_0.is_none() {
            self.current_team_is_team_0 = Some(team_is_team_0);
            self.last_possession_touch_time = Some(time);
            self.clear_pending_turnover();
            return;
        }

        if self.pending_turnover_team_is_team_0 == Some(team_is_team_0) {
            self.current_team_is_team_0 = Some(team_is_team_0);
            self.last_possession_touch_time = Some(time);
            self.clear_pending_turnover();
            return;
        }

        self.pending_turnover_team_is_team_0 = Some(team_is_team_0);
        self.pending_turnover_touch_time = Some(time);
    }

    fn register_contested_touch(&mut self, time: f32) {
        let Some(current_team_is_team_0) = self.current_team_is_team_0 else {
            self.clear_pending_turnover();
            return;
        };

        self.last_possession_touch_time = Some(time);
        self.pending_turnover_team_is_team_0 = Some(!current_team_is_team_0);
        self.pending_turnover_touch_time = Some(time);
    }

    fn update_player_control(
        &mut self,
        active_team_before_sample: Option<bool>,
        touched_team_zero_player: Option<&PlayerId>,
        touched_team_one_player: Option<&PlayerId>,
    ) {
        let Some(current_team_is_team_0) = self.current_team_is_team_0 else {
            self.current_player = None;
            return;
        };

        if self.pending_turnover_team_is_team_0.is_some() {
            self.current_player = None;
            return;
        }

        let controlling_touch_player = if current_team_is_team_0 {
            touched_team_zero_player
        } else {
            touched_team_one_player
        };
        if let Some(player) = controlling_touch_player {
            self.current_player = Some(player.clone());
            return;
        }

        if active_team_before_sample != self.current_team_is_team_0 {
            self.current_player = None;
        }
    }

    pub(crate) fn update(&mut self, time: f32, touch_events: &[TouchEvent]) -> PossessionState {
        self.expire_pending_turnover(time);
        self.expire_loose_ball(time);

        let active_team_before_sample = self.current_team_is_team_0;
        let active_player_before_sample = self.current_player.clone();
        let touched_team_zero = touch_events.iter().any(|touch| touch.team_is_team_0);
        let touched_team_one = touch_events.iter().any(|touch| !touch.team_is_team_0);
        let touched_team_zero_player = touch_events
            .iter()
            .rev()
            .find(|touch| touch.team_is_team_0)
            .and_then(|touch| touch.player.clone());
        let touched_team_one_player = touch_events
            .iter()
            .rev()
            .find(|touch| !touch.team_is_team_0)
            .and_then(|touch| touch.player.clone());

        match (touched_team_zero, touched_team_one) {
            (true, true) => self.register_contested_touch(time),
            (true, false) => self.register_single_team_touch(true, time),
            (false, true) => self.register_single_team_touch(false, time),
            (false, false) => {}
        }
        self.update_player_control(
            active_team_before_sample,
            touched_team_zero_player.as_ref(),
            touched_team_one_player.as_ref(),
        );

        PossessionState {
            active_team_before_sample,
            current_team_is_team_0: self.current_team_is_team_0,
            active_player_before_sample,
            current_player: self.current_player.clone(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PossessionCalculator {
    stats: PossessionStats,
    tracker: PossessionTracker,
}

impl PossessionCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn stats(&self) -> &PossessionStats {
        &self.stats
    }

    fn apply_possession_time(
        stats: &mut PossessionStats,
        state: PossessionStateLabel,
        field_third: Option<FieldThirdLabel>,
        dt: f32,
    ) {
        match state {
            PossessionStateLabel::TeamZero => stats.team_zero_time += dt,
            PossessionStateLabel::TeamOne => stats.team_one_time += dt,
            PossessionStateLabel::Neutral => stats.neutral_time += dt,
        }
        if let Some(field_third) = field_third {
            stats
                .labeled_time
                .add([state.as_label(), field_third.as_label()], dt);
        } else {
            stats.labeled_time.add([state.as_label()], dt);
        }
    }

    pub fn update(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        possession_state: &PossessionState,
        live_play_state: &LivePlayState,
    ) -> SubtrActorResult<()> {
        if live_play_state.is_live_play {
            self.stats.tracked_time += frame.dt;
            let field_third = ball.sample().map(FieldThirdLabel::from_ball);
            if let Some(possession_team_is_team_0) = possession_state.active_team_before_sample {
                let state = if possession_team_is_team_0 {
                    PossessionStateLabel::TeamZero
                } else {
                    PossessionStateLabel::TeamOne
                };
                Self::apply_possession_time(&mut self.stats, state, field_third, frame.dt);
            } else {
                Self::apply_possession_time(
                    &mut self.stats,
                    PossessionStateLabel::Neutral,
                    field_third,
                    frame.dt,
                );
            }
        }
        Ok(())
    }
}
