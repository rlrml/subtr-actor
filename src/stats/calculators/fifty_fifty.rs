use super::*;

pub(crate) const FIFTY_FIFTY_CONTINUATION_TOUCH_WINDOW_SECONDS: f32 = 0.2;
pub(crate) const FIFTY_FIFTY_RESOLUTION_DELAY_SECONDS: f32 = 0.35;
pub(crate) const FIFTY_FIFTY_MAX_DURATION_SECONDS: f32 = 1.25;
pub(crate) const FIFTY_FIFTY_MIN_EXIT_DISTANCE: f32 = 180.0;
pub(crate) const FIFTY_FIFTY_MIN_EXIT_SPEED: f32 = 220.0;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FiftyFiftyState {
    pub active_event: Option<ActiveFiftyFifty>,
    pub resolved_events: Vec<FiftyFiftyEvent>,
    pub last_resolved_event: Option<FiftyFiftyEvent>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActiveFiftyFifty {
    pub start_time: f32,
    pub start_frame: usize,
    pub last_touch_time: f32,
    pub last_touch_frame: usize,
    pub is_kickoff: bool,
    pub team_zero_player: Option<PlayerId>,
    pub team_one_player: Option<PlayerId>,
    pub team_zero_position: [f32; 3],
    pub team_one_position: [f32; 3],
    pub midpoint: [f32; 3],
    pub plane_normal: [f32; 3],
}

impl ActiveFiftyFifty {
    pub fn midpoint_vec(&self) -> glam::Vec3 {
        glam::Vec3::from_array(self.midpoint)
    }

    pub fn plane_normal_vec(&self) -> glam::Vec3 {
        glam::Vec3::from_array(self.plane_normal)
    }

    pub fn contains_team_touch(&self, touch_events: &[TouchEvent]) -> bool {
        touch_events.iter().any(|touch| {
            (touch.team_is_team_0 && self.team_zero_player.is_some())
                || (!touch.team_is_team_0 && self.team_one_player.is_some())
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct FiftyFiftyEvent {
    pub start_time: f32,
    pub start_frame: usize,
    pub resolve_time: f32,
    pub resolve_frame: usize,
    pub is_kickoff: bool,
    #[ts(as = "Option<crate::ts_bindings::RemoteIdTs>")]
    pub team_zero_player: Option<PlayerId>,
    #[ts(as = "Option<crate::ts_bindings::RemoteIdTs>")]
    pub team_one_player: Option<PlayerId>,
    pub team_zero_position: [f32; 3],
    pub team_one_position: [f32; 3],
    pub midpoint: [f32; 3],
    pub plane_normal: [f32; 3],
    pub winning_team_is_team_0: Option<bool>,
    pub possession_team_is_team_0: Option<bool>,
}

const FIFTY_FIFTY_PHASE_LABELS: [StatLabel; 2] = [
    StatLabel::new("phase", "open_play"),
    StatLabel::new("phase", "kickoff"),
];
const FIFTY_FIFTY_TEAM_OUTCOME_LABELS: [StatLabel; 3] = [
    StatLabel::new("winning_team", "team_zero"),
    StatLabel::new("winning_team", "team_one"),
    StatLabel::new("winning_team", "neutral"),
];
const FIFTY_FIFTY_POSSESSION_LABELS: [StatLabel; 3] = [
    StatLabel::new("possession_after", "team_zero"),
    StatLabel::new("possession_after", "team_one"),
    StatLabel::new("possession_after", "neutral"),
];
const FIFTY_FIFTY_PLAYER_OUTCOME_LABELS: [StatLabel; 3] = [
    StatLabel::new("outcome", "win"),
    StatLabel::new("outcome", "loss"),
    StatLabel::new("outcome", "neutral"),
];
const FIFTY_FIFTY_PLAYER_POSSESSION_LABELS: [StatLabel; 3] = [
    StatLabel::new("possession_after", "self"),
    StatLabel::new("possession_after", "opponent"),
    StatLabel::new("possession_after", "neutral"),
];

fn fifty_fifty_phase_label(is_kickoff: bool) -> StatLabel {
    if is_kickoff {
        StatLabel::new("phase", "kickoff")
    } else {
        StatLabel::new("phase", "open_play")
    }
}

fn fifty_fifty_team_outcome_label(team_is_team_0: Option<bool>) -> StatLabel {
    match team_is_team_0 {
        Some(true) => StatLabel::new("winning_team", "team_zero"),
        Some(false) => StatLabel::new("winning_team", "team_one"),
        None => StatLabel::new("winning_team", "neutral"),
    }
}

fn fifty_fifty_possession_label(team_is_team_0: Option<bool>) -> StatLabel {
    match team_is_team_0 {
        Some(true) => StatLabel::new("possession_after", "team_zero"),
        Some(false) => StatLabel::new("possession_after", "team_one"),
        None => StatLabel::new("possession_after", "neutral"),
    }
}

fn fifty_fifty_player_outcome_label(
    player_team_is_team_0: bool,
    winning_team_is_team_0: Option<bool>,
) -> StatLabel {
    match winning_team_is_team_0 {
        Some(team_is_team_0) if team_is_team_0 == player_team_is_team_0 => {
            StatLabel::new("outcome", "win")
        }
        Some(_) => StatLabel::new("outcome", "loss"),
        None => StatLabel::new("outcome", "neutral"),
    }
}

fn fifty_fifty_player_possession_label(
    player_team_is_team_0: bool,
    possession_team_is_team_0: Option<bool>,
) -> StatLabel {
    match possession_team_is_team_0 {
        Some(team_is_team_0) if team_is_team_0 == player_team_is_team_0 => {
            StatLabel::new("possession_after", "self")
        }
        Some(_) => StatLabel::new("possession_after", "opponent"),
        None => StatLabel::new("possession_after", "neutral"),
    }
}

impl FiftyFiftyEvent {
    fn labels(&self) -> [StatLabel; 3] {
        [
            fifty_fifty_phase_label(self.is_kickoff),
            fifty_fifty_team_outcome_label(self.winning_team_is_team_0),
            fifty_fifty_possession_label(self.possession_team_is_team_0),
        ]
    }

    fn player_labels(&self, player_team_is_team_0: bool) -> [StatLabel; 3] {
        [
            fifty_fifty_phase_label(self.is_kickoff),
            fifty_fifty_player_outcome_label(player_team_is_team_0, self.winning_team_is_team_0),
            fifty_fifty_player_possession_label(
                player_team_is_team_0,
                self.possession_team_is_team_0,
            ),
        ]
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct FiftyFiftyStats {
    pub count: u32,
    pub team_zero_wins: u32,
    pub team_one_wins: u32,
    pub neutral_outcomes: u32,
    pub kickoff_count: u32,
    pub kickoff_team_zero_wins: u32,
    pub kickoff_team_one_wins: u32,
    pub kickoff_neutral_outcomes: u32,
    pub team_zero_possession_after_count: u32,
    pub team_one_possession_after_count: u32,
    pub neutral_possession_after_count: u32,
    pub kickoff_team_zero_possession_after_count: u32,
    pub kickoff_team_one_possession_after_count: u32,
    pub kickoff_neutral_possession_after_count: u32,
    #[serde(default, skip_serializing_if = "LabeledCounts::is_empty")]
    pub labeled_event_counts: LabeledCounts,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct FiftyFiftyPlayerStats {
    pub count: u32,
    pub wins: u32,
    pub losses: u32,
    pub neutral_outcomes: u32,
    pub kickoff_count: u32,
    pub kickoff_wins: u32,
    pub kickoff_losses: u32,
    pub kickoff_neutral_outcomes: u32,
    pub possession_after_count: u32,
    pub kickoff_possession_after_count: u32,
    #[serde(default, skip_serializing_if = "LabeledCounts::is_empty")]
    pub labeled_event_counts: LabeledCounts,
}

impl FiftyFiftyStats {
    fn record_event(&mut self, event: &FiftyFiftyEvent) {
        self.labeled_event_counts.increment(event.labels());
        self.sync_legacy_counts();
    }

    pub fn event_count_with_labels(&self, labels: &[StatLabel]) -> u32 {
        self.labeled_event_counts.count_matching(labels)
    }

    pub fn complete_labeled_event_counts(&self) -> LabeledCounts {
        LabeledCounts::complete_from_label_sets(
            &[
                &FIFTY_FIFTY_PHASE_LABELS,
                &FIFTY_FIFTY_TEAM_OUTCOME_LABELS,
                &FIFTY_FIFTY_POSSESSION_LABELS,
            ],
            &self.labeled_event_counts,
        )
    }

    fn sync_legacy_counts(&mut self) {
        self.count = self.labeled_event_counts.total();
        self.team_zero_wins =
            self.event_count_with_labels(&[fifty_fifty_team_outcome_label(Some(true))]);
        self.team_one_wins =
            self.event_count_with_labels(&[fifty_fifty_team_outcome_label(Some(false))]);
        self.neutral_outcomes =
            self.event_count_with_labels(&[fifty_fifty_team_outcome_label(None)]);
        self.kickoff_count = self.event_count_with_labels(&[fifty_fifty_phase_label(true)]);
        self.kickoff_team_zero_wins = self.event_count_with_labels(&[
            fifty_fifty_phase_label(true),
            fifty_fifty_team_outcome_label(Some(true)),
        ]);
        self.kickoff_team_one_wins = self.event_count_with_labels(&[
            fifty_fifty_phase_label(true),
            fifty_fifty_team_outcome_label(Some(false)),
        ]);
        self.kickoff_neutral_outcomes = self.event_count_with_labels(&[
            fifty_fifty_phase_label(true),
            fifty_fifty_team_outcome_label(None),
        ]);
        self.team_zero_possession_after_count =
            self.event_count_with_labels(&[fifty_fifty_possession_label(Some(true))]);
        self.team_one_possession_after_count =
            self.event_count_with_labels(&[fifty_fifty_possession_label(Some(false))]);
        self.neutral_possession_after_count =
            self.event_count_with_labels(&[fifty_fifty_possession_label(None)]);
        self.kickoff_team_zero_possession_after_count = self.event_count_with_labels(&[
            fifty_fifty_phase_label(true),
            fifty_fifty_possession_label(Some(true)),
        ]);
        self.kickoff_team_one_possession_after_count = self.event_count_with_labels(&[
            fifty_fifty_phase_label(true),
            fifty_fifty_possession_label(Some(false)),
        ]);
        self.kickoff_neutral_possession_after_count = self.event_count_with_labels(&[
            fifty_fifty_phase_label(true),
            fifty_fifty_possession_label(None),
        ]);
    }

    pub fn team_zero_win_pct(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            self.team_zero_wins as f32 * 100.0 / self.count as f32
        }
    }

    pub fn team_one_win_pct(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            self.team_one_wins as f32 * 100.0 / self.count as f32
        }
    }

    pub fn kickoff_team_zero_win_pct(&self) -> f32 {
        if self.kickoff_count == 0 {
            0.0
        } else {
            self.kickoff_team_zero_wins as f32 * 100.0 / self.kickoff_count as f32
        }
    }

    pub fn kickoff_team_one_win_pct(&self) -> f32 {
        if self.kickoff_count == 0 {
            0.0
        } else {
            self.kickoff_team_one_wins as f32 * 100.0 / self.kickoff_count as f32
        }
    }
}

impl FiftyFiftyPlayerStats {
    fn record_event(&mut self, player_team_is_team_0: bool, event: &FiftyFiftyEvent) {
        self.labeled_event_counts
            .increment(event.player_labels(player_team_is_team_0));
        self.sync_legacy_counts();
    }

    pub fn event_count_with_labels(&self, labels: &[StatLabel]) -> u32 {
        self.labeled_event_counts.count_matching(labels)
    }

    pub fn complete_labeled_event_counts(&self) -> LabeledCounts {
        LabeledCounts::complete_from_label_sets(
            &[
                &FIFTY_FIFTY_PHASE_LABELS,
                &FIFTY_FIFTY_PLAYER_OUTCOME_LABELS,
                &FIFTY_FIFTY_PLAYER_POSSESSION_LABELS,
            ],
            &self.labeled_event_counts,
        )
    }

    fn sync_legacy_counts(&mut self) {
        self.count = self.labeled_event_counts.total();
        self.wins = self.event_count_with_labels(&[StatLabel::new("outcome", "win")]);
        self.losses = self.event_count_with_labels(&[StatLabel::new("outcome", "loss")]);
        self.neutral_outcomes =
            self.event_count_with_labels(&[StatLabel::new("outcome", "neutral")]);
        self.kickoff_count = self.event_count_with_labels(&[fifty_fifty_phase_label(true)]);
        self.kickoff_wins = self.event_count_with_labels(&[
            fifty_fifty_phase_label(true),
            StatLabel::new("outcome", "win"),
        ]);
        self.kickoff_losses = self.event_count_with_labels(&[
            fifty_fifty_phase_label(true),
            StatLabel::new("outcome", "loss"),
        ]);
        self.kickoff_neutral_outcomes = self.event_count_with_labels(&[
            fifty_fifty_phase_label(true),
            StatLabel::new("outcome", "neutral"),
        ]);
        self.possession_after_count =
            self.event_count_with_labels(&[StatLabel::new("possession_after", "self")]);
        self.kickoff_possession_after_count = self.event_count_with_labels(&[
            fifty_fifty_phase_label(true),
            StatLabel::new("possession_after", "self"),
        ]);
    }

    pub fn win_pct(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            self.wins as f32 * 100.0 / self.count as f32
        }
    }

    pub fn kickoff_win_pct(&self) -> f32 {
        if self.kickoff_count == 0 {
            0.0
        } else {
            self.kickoff_wins as f32 * 100.0 / self.kickoff_count as f32
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct FiftyFiftyTeamStats {
    pub count: u32,
    pub wins: u32,
    pub losses: u32,
    pub neutral_outcomes: u32,
    pub kickoff_count: u32,
    pub kickoff_wins: u32,
    pub kickoff_losses: u32,
    pub kickoff_neutral_outcomes: u32,
    pub possession_after_count: u32,
    pub opponent_possession_after_count: u32,
    pub neutral_possession_after_count: u32,
    pub kickoff_possession_after_count: u32,
    pub kickoff_opponent_possession_after_count: u32,
    pub kickoff_neutral_possession_after_count: u32,
}

impl FiftyFiftyStats {
    pub fn for_team(&self, is_team_zero: bool) -> FiftyFiftyTeamStats {
        let (
            wins,
            losses,
            kickoff_wins,
            kickoff_losses,
            possession_after_count,
            opponent_possession_after_count,
            kickoff_possession_after_count,
            kickoff_opponent_possession_after_count,
        ) = if is_team_zero {
            (
                self.team_zero_wins,
                self.team_one_wins,
                self.kickoff_team_zero_wins,
                self.kickoff_team_one_wins,
                self.team_zero_possession_after_count,
                self.team_one_possession_after_count,
                self.kickoff_team_zero_possession_after_count,
                self.kickoff_team_one_possession_after_count,
            )
        } else {
            (
                self.team_one_wins,
                self.team_zero_wins,
                self.kickoff_team_one_wins,
                self.kickoff_team_zero_wins,
                self.team_one_possession_after_count,
                self.team_zero_possession_after_count,
                self.kickoff_team_one_possession_after_count,
                self.kickoff_team_zero_possession_after_count,
            )
        };

        FiftyFiftyTeamStats {
            count: self.count,
            wins,
            losses,
            neutral_outcomes: self.neutral_outcomes,
            kickoff_count: self.kickoff_count,
            kickoff_wins,
            kickoff_losses,
            kickoff_neutral_outcomes: self.kickoff_neutral_outcomes,
            possession_after_count,
            opponent_possession_after_count,
            neutral_possession_after_count: self.neutral_possession_after_count,
            kickoff_possession_after_count,
            kickoff_opponent_possession_after_count,
            kickoff_neutral_possession_after_count: self.kickoff_neutral_possession_after_count,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FiftyFiftyCalculator {
    stats: FiftyFiftyStats,
    player_stats: HashMap<PlayerId, FiftyFiftyPlayerStats>,
    events: Vec<FiftyFiftyEvent>,
}

impl FiftyFiftyCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn stats(&self) -> &FiftyFiftyStats {
        &self.stats
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, FiftyFiftyPlayerStats> {
        &self.player_stats
    }

    pub fn events(&self) -> &[FiftyFiftyEvent] {
        &self.events
    }

    fn apply_event(&mut self, event: &FiftyFiftyEvent) {
        self.stats.record_event(event);

        if let Some(player_id) = event.team_zero_player.as_ref() {
            let stats = self.player_stats.entry(player_id.clone()).or_default();
            stats.record_event(true, event);
        }
        if let Some(player_id) = event.team_one_player.as_ref() {
            let stats = self.player_stats.entry(player_id.clone()).or_default();
            stats.record_event(false, event);
        }

        self.events.push(event.clone());
    }

    pub(crate) fn kickoff_phase_active(gameplay: &GameplayState) -> bool {
        gameplay.game_state == Some(GAME_STATE_KICKOFF_COUNTDOWN)
            || gameplay.kickoff_countdown_time.is_some_and(|time| time > 0)
            || gameplay.ball_has_been_hit == Some(false)
    }

    pub(crate) fn contested_touch(
        frame: &FrameInfo,
        players: &PlayerFrameState,
        touch_events: &[TouchEvent],
        is_kickoff: bool,
    ) -> Option<ActiveFiftyFifty> {
        let team_zero_touch = touch_events.iter().find(|touch| touch.team_is_team_0)?;
        let team_one_touch = touch_events.iter().find(|touch| !touch.team_is_team_0)?;
        let team_zero_position = team_zero_touch.player.as_ref().and_then(|player_id| {
            players
                .players
                .iter()
                .find(|player| &player.player_id == player_id)
                .and_then(PlayerSample::position)
        })?;
        let team_one_position = team_one_touch.player.as_ref().and_then(|player_id| {
            players
                .players
                .iter()
                .find(|player| &player.player_id == player_id)
                .and_then(PlayerSample::position)
        })?;
        let midpoint = (team_zero_position + team_one_position) * 0.5;
        let mut plane_normal = team_one_position - team_zero_position;
        plane_normal.z = 0.0;
        if plane_normal.length_squared() <= f32::EPSILON {
            plane_normal = glam::Vec3::Y;
        } else {
            plane_normal = plane_normal.normalize();
        }

        Some(ActiveFiftyFifty {
            start_time: frame.time,
            start_frame: frame.frame_number,
            last_touch_time: frame.time,
            last_touch_frame: frame.frame_number,
            is_kickoff,
            team_zero_player: team_zero_touch.player.clone(),
            team_one_player: team_one_touch.player.clone(),
            team_zero_position: team_zero_position.to_array(),
            team_one_position: team_one_position.to_array(),
            midpoint: midpoint.to_array(),
            plane_normal: plane_normal.to_array(),
        })
    }

    pub(crate) fn winning_team_from_ball(
        active: &ActiveFiftyFifty,
        ball: &BallFrameState,
    ) -> Option<bool> {
        let ball = ball.sample()?;
        let midpoint = active.midpoint_vec();
        let plane_normal = active.plane_normal_vec();
        let displacement = ball.position() - midpoint;
        let signed_distance = displacement.dot(plane_normal);
        if signed_distance.abs() >= FIFTY_FIFTY_MIN_EXIT_DISTANCE {
            return Some(signed_distance > 0.0);
        }

        let signed_speed = ball.velocity().dot(plane_normal);
        if signed_speed.abs() >= FIFTY_FIFTY_MIN_EXIT_SPEED {
            return Some(signed_speed > 0.0);
        }

        None
    }

    pub fn update(&mut self, fifty_fifty_state: &FiftyFiftyState) -> SubtrActorResult<()> {
        for event in &fifty_fifty_state.resolved_events {
            self.apply_event(event);
        }
        Ok(())
    }
}
