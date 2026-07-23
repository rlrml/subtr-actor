use super::*;

const KICKOFF_CENTER_MAX_ABS_X: f32 = 350.0;
const KICKOFF_CENTER_MIN_ABS_Y: f32 = 4300.0;
const KICKOFF_OFF_CENTER_MAX_ABS_X: f32 = 900.0;
const KICKOFF_OFF_CENTER_MIN_ABS_Y: f32 = 3300.0;
const KICKOFF_DIAGONAL_MIN_ABS_X: f32 = 1500.0;
const KICKOFF_DIAGONAL_MAX_ABS_Y: f32 = 3300.0;
const KICKOFF_RESOLUTION_AFTER_FIRST_TOUCH_SECONDS: f32 = 1.25;
const KICKOFF_FOLLOW_UP_AFTER_FIRST_TOUCH_SECONDS: f32 = 2.0;
const KICKOFF_GOAL_MAX_SECONDS: f32 = 12.0;
const KICKOFF_GOAL_MAX_DEFENSIVE_BALL_Y: f32 = 1280.0;
const KICKOFF_WIN_PROJECTION_SECONDS: f32 = 0.5;
const KICKOFF_FIELD_HALF_LENGTH: f32 = 5120.0;
const KICKOFF_WIN_MIN_PROJECTED_BALL_Y: f32 = 256.0;
const KICKOFF_BALL_DIRECTION_MIN_ABS_X: f32 = 180.0;
const KICKOFF_BALL_DIRECTION_MIN_ABS_SPEED_X: f32 = 220.0;
const KICKOFF_CLEAR_WIN_STRENGTH: f32 = 0.25;
const KICKOFF_STRONG_WIN_STRENGTH: f32 = 0.5;
const KICKOFF_TAKER_DISTANCE_TIE_EPSILON: f32 = 150.0;
const KICKOFF_POSSESSION_IMMEDIATE_CONTEST_SECONDS: f32 = 0.35;
const KICKOFF_TOUCH_CLUSTER_MAX_GAP_SECONDS: f32 = 0.35;
const KICKOFF_APPROACH_MIN_BOOST_USED: f32 = 3.0;
const KICKOFF_APPROACH_MIN_FAKE_MOVE_DISTANCE: f32 = 350.0;
const KICKOFF_APPROACH_FLIP_MIN_SECONDS_BEFORE_TOUCH: f32 = 0.5;
const KICKOFF_APPROACH_FRONT_FLIP_FORWARD_COMPONENT: f32 = 0.45;
const KICKOFF_APPROACH_DIAGONAL_FLIP_SIDE_COMPONENT: f32 = 0.35;
/// The dodge/flip impulse is delivered over several frames (the car drags
/// through the dodge), and boxcars often does not re-send the car velocity on
/// the exact frame the dodge becomes active, so a single-frame velocity delta at
/// dodge onset is unreliable (it is frequently exactly zero). Decompose the
/// *peak* velocity change across this short window after the dodge instead.
const KICKOFF_APPROACH_DODGE_DIRECTION_WINDOW_SECONDS: f32 = 0.20;
/// Forward acceleration contributed by boost, subtracted across the dodge window
/// so the recovered dodge direction reflects the flip impulse rather than the
/// boost the taker is holding through it. Mirrors the speed-flip detector.
const KICKOFF_APPROACH_BOOST_ACCELERATION_UU_PER_SECOND_SQUARED: f32 = 991.6667;
const KICKOFF_SUPPORT_CHEAT_MIN_CENTER_PROGRESS: f32 = 400.0;
const KICKOFF_ADVANTAGE_POSSESSION_MIN_RUN_SECONDS: f32 = 1.25;
const KICKOFF_PRESSURE_NEUTRAL_ZONE_HALF_WIDTH_Y: f32 = 200.0;
const KICKOFF_PRESSURE_MIN_ESTABLISH_SECONDS: f32 = 2.0;
const KICKOFF_PRESSURE_MIN_ESTABLISH_THIRD_SECONDS: f32 = 0.75;
const KICKOFF_SUPPORT_GO_FOR_BOOST_MIN_LATERAL_MOVE: f32 = 600.0;
const KICKOFF_SUPPORT_GO_FOR_BOOST_MIN_BOOST_GAIN: f32 = 10.0;
const KICKOFF_SUPPORT_BACK_BIG_MAX_SECONDS_AFTER_GO: f32 = 3.0;

#[derive(Debug, Clone, PartialEq)]
struct KickoffPlayerSnapshot {
    player: PlayerId,
    is_team_0: bool,
    start_position: [f32; 3],
    spawn_position: KickoffSpawnPosition,
    start_boost: Option<f32>,
    first_touch_boost: Option<f32>,
    first_touch_time: Option<f32>,
    first_touch_frame: Option<usize>,
    first_touch_contact: Option<KickoffContactSnapshot>,
    approach_trace: KickoffApproachTrace,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct KickoffApproachTrace {
    boost_active_sample_count: u32,
    first_dodge_time: Option<f32>,
    first_dodge_frame: Option<usize>,
    first_dodge_forward_component: Option<f32>,
    first_dodge_side_component: Option<f32>,
    /// State for the windowed dodge-direction recovery (see
    /// `KICKOFF_APPROACH_DODGE_DIRECTION_WINDOW_SECONDS`). Captured at dodge
    /// onset and consumed for a short window afterward.
    dodge_direction_baseline_velocity: Option<glam::Vec3>,
    dodge_onset_forward: Option<glam::Vec3>,
    dodge_onset_right: Option<glam::Vec3>,
    dodge_direction_window_deadline: Option<f32>,
    dodge_direction_boost_compensation: glam::Vec3,
    best_dodge_direction_delta: f32,
    max_speed: f32,
    min_boost: Option<f32>,
    previous_boost: Option<f32>,
    sampled_boost_used: f32,
    /// Boost collected from pad pickups (including steals; overfill excluded)
    /// during the approach window, summed from the `BoostCalculator`'s
    /// deduplicated pickup events. This deliberately does NOT come from gross
    /// frame-to-frame `boost_amount` deltas: a single pad pickup can show up as
    /// a `+jump` one frame and a `-correction` the next as the inferred and
    /// reported boost balances reconcile, so summing positive deltas
    /// double-counts and can push `boost_used` past a full tank.
    pickup_boost_collected: f32,
    picked_up_immediate_own_back_big_boost: bool,
    last_position: Option<[f32; 3]>,
    previous_velocity: Option<glam::Vec3>,
    previous_dodge_active: bool,
}

#[derive(Debug, Clone, PartialEq)]
struct KickoffTouchSnapshot {
    time: f32,
    frame: usize,
    team_is_team_0: bool,
    player: Option<PlayerId>,
}

#[derive(Debug, Clone, PartialEq)]
struct KickoffContactSnapshot {
    player_position: [f32; 3],
    player_velocity: Option<[f32; 3]>,
    car_forward: Option<[f32; 3]>,
    local_ball_position: Option<[f32; 3]>,
    local_contact_point: Option<[f32; 3]>,
    contact_gap: Option<f32>,
    behind_ball_depth: f32,
    lateral_offset: f32,
    lateral_abs_offset: f32,
    velocity_attack_alignment: Option<f32>,
    velocity_ball_alignment: Option<f32>,
    nose_attack_alignment: Option<f32>,
    ball_exit_attack_alignment: Option<f32>,
}

#[derive(Debug, Clone)]
struct KickoffResolutionSnapshot {
    ball: BallFrameState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KickoffAdvantageKind {
    Possession,
    Pressure,
    Goal,
}

#[derive(Debug, Clone)]
struct EstablishedKickoffAdvantage {
    kind: KickoffAdvantageKind,
    team_is_team_0: bool,
    time: f32,
    frame: usize,
    player: Option<PlayerId>,
}

/// Watches the play that follows a kickoff to decide who the kickoff was
/// actually good for (see [`KickoffAdvantage`]), running through the same
/// post-conclusion in-flight window used for late goal attribution.
///
/// Two competing detectors race; the first to fire settles the kickoff:
///
/// - **Possession run**: consecutive touches by one team — no opposing touch
///   in between — spanning at least
///   [`KICKOFF_ADVANTAGE_POSSESSION_MIN_RUN_SECONDS`]. This is what credits the
///   team that *lost* the opening exchange but cleanly collected the ball,
///   even deep in its own half.
/// - **Anchored pressure**: the ball held beyond the neutral band in the
///   opponent's half long enough to establish territorial pressure. The
///   clocks only run once the attacking team has touched the ball inside that
///   zone (the anchor); a hard poke sailing deep while the defense calmly
///   retrieves it never anchors and so never reads as pressure. Defensive
///   panic touches do not reset the clocks — a defense that strings real
///   possession together wins the race through the possession run instead.
#[derive(Debug, Clone, Default)]
struct KickoffAdvantageWatcher {
    established: Option<EstablishedKickoffAdvantage>,
    touches_seen: usize,
    run_team_is_team_0: Option<bool>,
    run_start_time: f32,
    pressure_team_is_team_0: Option<bool>,
    pressure_anchored: bool,
    pressure_zone_seconds: f32,
    pressure_third_seconds: f32,
}

impl KickoffAdvantageWatcher {
    fn zone_side(ball: &BallFrameState) -> Option<bool> {
        let ball_y = ball.sample()?.position().y;
        if ball_y > KICKOFF_PRESSURE_NEUTRAL_ZONE_HALF_WIDTH_Y {
            Some(true)
        } else if ball_y < -KICKOFF_PRESSURE_NEUTRAL_ZONE_HALF_WIDTH_Y {
            Some(false)
        } else {
            None
        }
    }

    fn reset_pressure(&mut self) {
        self.pressure_team_is_team_0 = None;
        self.pressure_anchored = false;
        self.pressure_zone_seconds = 0.0;
        self.pressure_third_seconds = 0.0;
    }

    fn establish_goal(&mut self, goal: &GoalEvent) {
        if self.established.is_some() {
            return;
        }
        self.established = Some(EstablishedKickoffAdvantage {
            kind: KickoffAdvantageKind::Goal,
            team_is_team_0: goal.scoring_team_is_team_0,
            time: goal.time,
            frame: goal.frame,
            player: goal.player.clone(),
        });
    }

    fn observe(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        touches: &[KickoffTouchSnapshot],
    ) {
        if self.established.is_some() {
            self.touches_seen = touches.len();
            return;
        }
        let zone_side = Self::zone_side(ball);
        if self.pressure_team_is_team_0 != zone_side {
            self.reset_pressure();
            self.pressure_team_is_team_0 = zone_side;
        }

        for touch in &touches[self.touches_seen..] {
            if self.run_team_is_team_0 == Some(touch.team_is_team_0) {
                if touch.time - self.run_start_time >= KICKOFF_ADVANTAGE_POSSESSION_MIN_RUN_SECONDS
                {
                    self.established = Some(EstablishedKickoffAdvantage {
                        kind: KickoffAdvantageKind::Possession,
                        team_is_team_0: touch.team_is_team_0,
                        time: touch.time,
                        frame: touch.frame,
                        player: touch.player.clone(),
                    });
                    break;
                }
            } else {
                self.run_team_is_team_0 = Some(touch.team_is_team_0);
                self.run_start_time = touch.time;
            }
            if zone_side == Some(touch.team_is_team_0) {
                self.pressure_anchored = true;
            }
        }
        self.touches_seen = touches.len();
        if self.established.is_some() {
            return;
        }

        let Some(attacking_team_is_team_0) = zone_side else {
            return;
        };
        if !self.pressure_anchored {
            return;
        }
        self.pressure_zone_seconds += frame.dt;
        let normalized_ball_y = ball
            .sample()
            .map(|sample| {
                if attacking_team_is_team_0 {
                    sample.position().y
                } else {
                    -sample.position().y
                }
            })
            .unwrap_or(0.0);
        if normalized_ball_y > FIELD_ZONE_BOUNDARY_Y {
            self.pressure_third_seconds += frame.dt;
        }
        if self.pressure_zone_seconds >= KICKOFF_PRESSURE_MIN_ESTABLISH_SECONDS
            || self.pressure_third_seconds >= KICKOFF_PRESSURE_MIN_ESTABLISH_THIRD_SECONDS
        {
            self.established = Some(EstablishedKickoffAdvantage {
                kind: KickoffAdvantageKind::Pressure,
                team_is_team_0: attacking_team_is_team_0,
                time: frame.time,
                frame: frame.frame_number,
                player: None,
            });
        }
    }
}

#[derive(Debug, Clone)]
struct ActiveKickoff {
    start_time: f32,
    start_frame: usize,
    live_action_start_time: Option<f32>,
    live_action_start_frame: Option<usize>,
    movement_start_time: Option<f32>,
    movement_start_frame: Option<usize>,
    players: Vec<KickoffPlayerSnapshot>,
    first_touch_time: Option<f32>,
    first_touch_frame: Option<usize>,
    first_touch_team_is_team_0: Option<bool>,
    first_touch_id: Option<u64>,
    first_touch_ball_position: Option<[f32; 3]>,
    first_touch_ball_velocity: Option<[f32; 3]>,
    touches: Vec<KickoffTouchSnapshot>,
    speed_flip_directions: HashMap<PlayerId, KickoffFlipDirection>,
    resolution: Option<KickoffResolutionSnapshot>,
    /// Running ball-y extremes observed since the kickoff's first touch,
    /// including frames after the kickoff's logical close. Used by the
    /// kickoff-goal field-position gate: if the ball retreated meaningfully
    /// into the eventual scoring team's own half, the goal came from a reset
    /// play rather than the kickoff exchange.
    min_ball_y_after_first_touch: Option<f32>,
    max_ball_y_after_first_touch: Option<f32>,
    /// Keeps watching after the kickoff's logical close (alongside goal
    /// attribution) to decide who came out of the kickoff with the advantage;
    /// its result is written onto the concluded event at emission.
    advantage: KickoffAdvantageWatcher,
    /// The fully built event, frozen at the kickoff's logical close
    /// (`should_finish`). The item then stays in flight, awaiting goal
    /// attribution: a goal scored within [`KICKOFF_GOAL_MAX_SECONDS`] of the
    /// first touch still counts as a kickoff goal even though it lands after
    /// the kickoff itself has closed.
    concluded: Option<Box<KickoffEvent>>,
}

impl InFlightItem for ActiveKickoff {
    fn recognition(&self) -> Recognition {
        // A kickoff phase is always a real kickoff, so it is committed from the
        // moment it is recognized.
        Recognition::committed(self.start_time, self.start_frame)
    }

    fn on_boundary(&mut self, boundary: Boundary) -> Disposition {
        // A concluded kickoff is a complete event that is merely waiting for
        // late goal attribution; the stream ending just means no further goal
        // can arrive, so it is emitted as-is. A kickoff that never reached its
        // logical close has no resolution; emitting a truncated event would be
        // misleading, so it is discarded (preserving the previous "drop the
        // unfinished kickoff" behavior, now handled structurally rather than
        // by omission).
        if self.concluded.is_some() {
            Disposition::Finalize(FinalizeReason::Boundary(boundary))
        } else {
            Disposition::Discard
        }
    }
}

/// Detects and classifies kickoffs and their outcomes.
#[derive(Debug, Clone, Default)]
pub struct KickoffCalculator {
    active: InFlightLedger<ActiveKickoff>,
    events: EventStream<KickoffEvent>,
}

pub(crate) struct KickoffUpdateContext<'a> {
    pub frame: &'a FrameInfo,
    pub gameplay: &'a GameplayState,
    pub ball: &'a BallFrameState,
    pub players: &'a PlayerFrameState,
    pub touch_state: &'a TouchState,
    pub events: &'a FrameEventsState,
    pub speed_flip_events: &'a [SpeedFlipEvent],
    /// Boost pickup events newly emitted this frame by the `BoostCalculator`.
    /// Used to attribute deduplicated boost collected to in-flight kickoff
    /// players during their approach window.
    pub boost_pickups: &'a [BoostPickupEvent],
}

pub(crate) const KICKOFF_SPAWN_LABELS: [StatLabel; 6] = [
    StatLabel::new("kickoff_spawn", "center"),
    StatLabel::new("kickoff_spawn", "off_center_left"),
    StatLabel::new("kickoff_spawn", "off_center_right"),
    StatLabel::new("kickoff_spawn", "diagonal_left"),
    StatLabel::new("kickoff_spawn", "diagonal_right"),
    StatLabel::new("kickoff_spawn", "unknown"),
];
pub(crate) const KICKOFF_TYPE_LABELS: [StatLabel; 4] = [
    StatLabel::new("kickoff_type", "diagonal"),
    StatLabel::new("kickoff_type", "center_offset"),
    StatLabel::new("kickoff_type", "center"),
    StatLabel::new("kickoff_type", "unknown"),
];
pub(crate) const KICKOFF_DIRECTION_LABELS: [StatLabel; 4] = [
    StatLabel::new("kickoff_direction", "left"),
    StatLabel::new("kickoff_direction", "right"),
    StatLabel::new("kickoff_direction", "center"),
    StatLabel::new("kickoff_direction", "unknown"),
];
pub(crate) const KICKOFF_TAKER_OUTCOME_LABELS: [StatLabel; 4] = [
    StatLabel::new("taker_outcome", "touched"),
    StatLabel::new("taker_outcome", "fake"),
    StatLabel::new("taker_outcome", "missed"),
    StatLabel::new("taker_outcome", "unknown"),
];
pub(crate) const KICKOFF_APPROACH_LABELS: [StatLabel; 6] = [
    StatLabel::new("kickoff_approach", "speed_flip"),
    StatLabel::new("kickoff_approach", "boost_into_ball"),
    StatLabel::new("kickoff_approach", "fake_go_for_boost"),
    StatLabel::new("kickoff_approach", "front_flip"),
    StatLabel::new("kickoff_approach", "diagonal_flip"),
    StatLabel::new("kickoff_approach", "other"),
];
pub(crate) const KICKOFF_FLIP_DIRECTION_LABELS: [StatLabel; 3] = [
    StatLabel::new("approach_flip_direction", "left"),
    StatLabel::new("approach_flip_direction", "right"),
    StatLabel::new("approach_flip_direction", "not_applicable"),
];
pub(crate) const KICKOFF_SUPPORT_BEHAVIOR_LABELS: [StatLabel; 4] = [
    StatLabel::new("support_behavior", "go_for_boost"),
    StatLabel::new("support_behavior", "cheat"),
    StatLabel::new("support_behavior", "other"),
    StatLabel::new("support_behavior", "unknown"),
];
pub(crate) const KICKOFF_BALL_DIRECTION_LABELS: [StatLabel; 4] = [
    StatLabel::new("ball_direction", "left"),
    StatLabel::new("ball_direction", "right"),
    StatLabel::new("ball_direction", "center"),
    StatLabel::new("ball_direction", "unknown"),
];
pub(crate) const KICKOFF_OUTCOME_LABELS: [StatLabel; 4] = [
    StatLabel::new("outcome", "team_zero_win"),
    StatLabel::new("outcome", "team_one_win"),
    StatLabel::new("outcome", "neutral"),
    StatLabel::new("outcome", "unknown"),
];
pub(crate) const KICKOFF_WIN_STRENGTH_LABELS: [StatLabel; 4] = [
    StatLabel::new("win_strength", "narrow"),
    StatLabel::new("win_strength", "clear"),
    StatLabel::new("win_strength", "strong"),
    StatLabel::new("win_strength", "unknown"),
];
pub(crate) const KICKOFF_POSSESSION_OUTCOME_LABELS: [StatLabel; 5] = [
    StatLabel::new("kickoff_possession_outcome", "team_zero_possession"),
    StatLabel::new("kickoff_possession_outcome", "team_one_possession"),
    StatLabel::new("kickoff_possession_outcome", "team_zero_advantage"),
    StatLabel::new("kickoff_possession_outcome", "team_one_advantage"),
    StatLabel::new("kickoff_possession_outcome", "contested"),
];
pub(crate) const KICKOFF_GOAL_LABELS: [StatLabel; 2] = [
    StatLabel::new("kickoff_goal", "false"),
    StatLabel::new("kickoff_goal", "true"),
];
pub(crate) const KICKOFF_ADVANTAGE_LABELS: [StatLabel; 7] = [
    StatLabel::new("kickoff_advantage", "team_zero_possession"),
    StatLabel::new("kickoff_advantage", "team_one_possession"),
    StatLabel::new("kickoff_advantage", "team_zero_pressure"),
    StatLabel::new("kickoff_advantage", "team_one_pressure"),
    StatLabel::new("kickoff_advantage", "team_zero_goal"),
    StatLabel::new("kickoff_advantage", "team_one_goal"),
    StatLabel::new("kickoff_advantage", "no_advantage"),
];

pub(crate) fn kickoff_spawn_label(spawn: KickoffSpawnPosition) -> StatLabel {
    StatLabel::new("kickoff_spawn", spawn.as_label_value())
}

pub(crate) fn kickoff_type_label(kickoff_type: KickoffType) -> StatLabel {
    StatLabel::new("kickoff_type", kickoff_type.as_label_value())
}

pub(crate) fn kickoff_direction_label(kickoff_direction: KickoffDirection) -> StatLabel {
    StatLabel::new("kickoff_direction", kickoff_direction.as_label_value())
}

pub(crate) fn kickoff_taker_outcome_label(outcome: KickoffTakerOutcome) -> StatLabel {
    StatLabel::new("taker_outcome", outcome.as_label_value())
}

pub(crate) fn kickoff_outcome_label(outcome: KickoffOutcome) -> StatLabel {
    StatLabel::new("outcome", outcome.as_label_value())
}

pub(crate) fn kickoff_win_strength_label(band: KickoffWinStrengthBand) -> StatLabel {
    StatLabel::new("win_strength", band.as_label_value())
}

pub(crate) fn kickoff_possession_outcome_label(outcome: KickoffPossessionOutcome) -> StatLabel {
    StatLabel::new("kickoff_possession_outcome", outcome.as_label_value())
}

pub(crate) fn kickoff_goal_label(kickoff_goal: bool) -> StatLabel {
    StatLabel::new("kickoff_goal", if kickoff_goal { "true" } else { "false" })
}

pub(crate) fn kickoff_advantage_label(advantage: KickoffAdvantage) -> StatLabel {
    StatLabel::new("kickoff_advantage", advantage.as_label_value())
}

pub(crate) fn kickoff_approach_label(approach: KickoffApproach) -> StatLabel {
    StatLabel::new("kickoff_approach", approach.as_label_value())
}

pub(crate) fn kickoff_flip_direction_label(direction: KickoffFlipDirection) -> StatLabel {
    StatLabel::new("approach_flip_direction", direction.as_label_value())
}

pub(crate) fn kickoff_support_behavior_label(behavior: KickoffSupportBehavior) -> StatLabel {
    StatLabel::new("support_behavior", behavior.as_label_value())
}

pub(crate) fn kickoff_ball_direction_label(direction: KickoffBallDirection) -> StatLabel {
    StatLabel::new("ball_direction", direction.as_label_value())
}

impl KickoffTakerEvent {
    pub(crate) fn labels(&self) -> Vec<StatLabel> {
        vec![
            kickoff_spawn_label(self.spawn_position),
            kickoff_taker_outcome_label(self.outcome),
            kickoff_approach_label(self.approach),
            kickoff_flip_direction_label(self.approach_flip_direction),
            kickoff_ball_direction_label(self.ball_direction),
        ]
    }
}

impl KickoffSupportEvent {
    pub(crate) fn labels(&self) -> Vec<StatLabel> {
        vec![
            kickoff_spawn_label(self.spawn_position),
            kickoff_support_behavior_label(self.support_behavior),
        ]
    }
}

pub(crate) enum KickoffPlayerEventRef<'a> {
    Taker(&'a KickoffTakerEvent),
    Support(&'a KickoffSupportEvent),
}

impl KickoffPlayerEventRef<'_> {
    pub(crate) fn player(&self) -> &PlayerId {
        match self {
            Self::Taker(event) => &event.player,
            Self::Support(event) => &event.player,
        }
    }

    pub(crate) fn is_team_0(&self) -> bool {
        match self {
            Self::Taker(event) => event.is_team_0,
            Self::Support(event) => event.is_team_0,
        }
    }

    pub(crate) fn boost_after(&self) -> Option<f32> {
        match self {
            Self::Taker(event) => event.boost_after,
            Self::Support(event) => event.boost_after,
        }
    }

    pub(crate) fn labels(&self) -> Vec<StatLabel> {
        match self {
            Self::Taker(event) => event.labels(),
            Self::Support(event) => event.labels(),
        }
    }

    pub(crate) fn as_taker(&self) -> Option<&KickoffTakerEvent> {
        match self {
            Self::Taker(event) => Some(event),
            Self::Support(_) => None,
        }
    }

    pub(crate) fn as_support(&self) -> Option<&KickoffSupportEvent> {
        match self {
            Self::Taker(_) => None,
            Self::Support(event) => Some(event),
        }
    }
}

impl KickoffEvent {
    pub(crate) fn labels(&self) -> [StatLabel; 7] {
        [
            kickoff_type_label(self.kickoff_type),
            kickoff_direction_label(self.kickoff_direction),
            kickoff_outcome_label(self.outcome),
            kickoff_win_strength_label(self.win_strength_band),
            kickoff_possession_outcome_label(self.kickoff_possession_outcome),
            kickoff_goal_label(self.kickoff_goal),
            kickoff_advantage_label(self.advantage),
        ]
    }

    pub(crate) fn player_events(&self) -> impl Iterator<Item = KickoffPlayerEventRef<'_>> {
        self.team_zero_taker
            .iter()
            .map(KickoffPlayerEventRef::Taker)
            .chain(self.team_one_taker.iter().map(KickoffPlayerEventRef::Taker))
            .chain(
                self.team_zero_non_takers
                    .iter()
                    .map(KickoffPlayerEventRef::Support),
            )
            .chain(
                self.team_one_non_takers
                    .iter()
                    .map(KickoffPlayerEventRef::Support),
            )
    }
}

impl KickoffCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn events(&self) -> &[KickoffEvent] {
        self.events.all()
    }

    pub fn new_events(&self) -> &[KickoffEvent] {
        self.events.new_events()
    }

    pub(crate) fn kickoff_spawn_position(
        position: glam::Vec3,
        is_team_0: bool,
    ) -> KickoffSpawnPosition {
        let abs_x = position.x.abs();
        let abs_y = position.y.abs();
        let relative_x = if is_team_0 { position.x } else { -position.x };

        if abs_x <= KICKOFF_CENTER_MAX_ABS_X && abs_y >= KICKOFF_CENTER_MIN_ABS_Y {
            return KickoffSpawnPosition::Center;
        }
        if abs_x <= KICKOFF_OFF_CENTER_MAX_ABS_X && abs_y >= KICKOFF_OFF_CENTER_MIN_ABS_Y {
            return if relative_x < 0.0 {
                KickoffSpawnPosition::OffCenterLeft
            } else {
                KickoffSpawnPosition::OffCenterRight
            };
        }
        if abs_x >= KICKOFF_DIAGONAL_MIN_ABS_X && abs_y <= KICKOFF_DIAGONAL_MAX_ABS_Y {
            return if relative_x < 0.0 {
                KickoffSpawnPosition::DiagonalLeft
            } else {
                KickoffSpawnPosition::DiagonalRight
            };
        }
        KickoffSpawnPosition::Unknown
    }

    fn kickoff_player_snapshot(player: &PlayerSample) -> Option<KickoffPlayerSnapshot> {
        let position = player.position()?;
        Some(KickoffPlayerSnapshot {
            player: player.player_id.clone(),
            is_team_0: player.is_team_0,
            start_position: position.to_array(),
            spawn_position: Self::kickoff_spawn_position(position, player.is_team_0),
            start_boost: player.boost_amount.or(player.last_boost_amount),
            first_touch_boost: None,
            first_touch_time: None,
            first_touch_frame: None,
            first_touch_contact: None,
            approach_trace: KickoffApproachTrace::default(),
        })
    }

    fn start_kickoff(&mut self, frame: &FrameInfo, players: &PlayerFrameState) {
        self.active.arm(ActiveKickoff {
            start_time: frame.time,
            start_frame: frame.frame_number,
            live_action_start_time: None,
            live_action_start_frame: None,
            movement_start_time: None,
            movement_start_frame: None,
            players: players
                .players
                .iter()
                .filter_map(Self::kickoff_player_snapshot)
                .collect(),
            first_touch_time: None,
            first_touch_frame: None,
            first_touch_team_is_team_0: None,
            first_touch_id: None,
            first_touch_ball_position: None,
            first_touch_ball_velocity: None,
            touches: Vec::new(),
            speed_flip_directions: HashMap::new(),
            resolution: None,
            min_ball_y_after_first_touch: None,
            max_ball_y_after_first_touch: None,
            advantage: KickoffAdvantageWatcher::default(),
            concluded: None,
        });
    }

    /// Finalize any in-flight kickoff at end of stream. Routed through the
    /// ledger so the boundary is handled uniformly; a concluded kickoff that
    /// was only waiting for late goal attribution is emitted, while an
    /// unresolved kickoff is discarded (see `ActiveKickoff::on_boundary`).
    pub fn finish(&mut self) {
        for (active, _reason) in self.active.finish() {
            Self::emit_concluded(&mut self.events, active);
        }
    }

    /// Emit a concluded kickoff, stamping the advantage watcher's verdict
    /// onto the frozen event. The advantage usually lands after the kickoff's
    /// logical close, so it is applied here — at emission — rather than in
    /// `finish_event`.
    fn emit_concluded(events: &mut EventStream<KickoffEvent>, active: ActiveKickoff) {
        let ActiveKickoff {
            concluded,
            advantage,
            ..
        } = active;
        let Some(mut event) = concluded else {
            return;
        };
        Self::apply_advantage(&mut event, &advantage);
        events.push(*event);
    }

    fn apply_advantage(event: &mut KickoffEvent, watcher: &KickoffAdvantageWatcher) {
        let Some(established) = watcher.established.as_ref() else {
            return;
        };
        event.advantage = match (established.kind, established.team_is_team_0) {
            (KickoffAdvantageKind::Possession, true) => KickoffAdvantage::TeamZeroPossession,
            (KickoffAdvantageKind::Possession, false) => KickoffAdvantage::TeamOnePossession,
            (KickoffAdvantageKind::Pressure, true) => KickoffAdvantage::TeamZeroPressure,
            (KickoffAdvantageKind::Pressure, false) => KickoffAdvantage::TeamOnePressure,
            (KickoffAdvantageKind::Goal, true) => KickoffAdvantage::TeamZeroGoal,
            (KickoffAdvantageKind::Goal, false) => KickoffAdvantage::TeamOneGoal,
        };
        event.advantage_team_is_team_0 = Some(established.team_is_team_0);
        event.advantage_time = Some(established.time);
        event.advantage_frame = Some(established.frame);
        event.advantage_seconds_after_first_touch = event
            .first_touch_time
            .map(|first_touch_time| established.time - first_touch_time);
        event.advantage_player = established.player.clone();
    }

    fn observe_movement_start(
        active: &mut ActiveKickoff,
        frame: &FrameInfo,
        gameplay: &GameplayState,
    ) {
        // A kickoff is always armed while the countdown is still active, so the
        // first armed frame on which the countdown is no longer active marks the
        // "GO" — the moment players are released toward the ball.
        //
        // We deliberately do not also require `kickoff_phase_active()` here. On
        // the opening kickoff of a match the engine reports
        // `ball_has_been_hit == None` (rather than `Some(false)`) until the ball
        // is first touched, so the kickoff phase never registers as active in the
        // window between the countdown ending and the first touch. Requiring it
        // left `movement_start` unset on the first kickoff, falling back to the
        // countdown's *start* and inflating every taker's `time_to_ball` by the
        // full countdown (~3s). Gating on the countdown alone keeps movement
        // start aligned with "GO" on the opening kickoff and matches the
        // behavior on every subsequent kickoff.
        if active.movement_start_time.is_none() && !gameplay.kickoff_countdown_active() {
            active.movement_start_time = Some(frame.time);
            active.movement_start_frame = Some(frame.frame_number);
        }
    }

    fn observe_live_action_start(active: &mut ActiveKickoff, frame: &FrameInfo) {
        if active.live_action_start_time.is_none() {
            active.live_action_start_time = Some(frame.time);
            active.live_action_start_frame = Some(frame.frame_number);
        }
    }

    fn boost_amount(player: &PlayerSample) -> Option<f32> {
        player.boost_amount.or(player.last_boost_amount)
    }

    fn observe_player_approach(
        trace: &mut KickoffApproachTrace,
        frame: &FrameInfo,
        player: &PlayerSample,
    ) {
        if player.boost_active {
            trace.boost_active_sample_count += 1;
        }
        if let Some(boost_amount) = Self::boost_amount(player) {
            if let Some(previous_boost) = trace.previous_boost {
                // Collection is tracked from pad-pickup events, not deltas; only
                // the spent side is sampled here, as a fallback for boost_used
                // when start/contact boost is unavailable.
                let delta = boost_amount - previous_boost;
                if delta < 0.0 {
                    // Only the depletion side is sampled here, as a fallback for
                    // `boost_used` when a player's first-touch boost is unknown.
                    // Collected boost is sourced from deduplicated pickup events
                    // (see `apply_boost_pickups`), not gross positive deltas.
                    trace.sampled_boost_used += -delta;
                }
            }
            trace.previous_boost = Some(boost_amount);
            trace.min_boost = Some(
                trace
                    .min_boost
                    .map(|current| current.min(boost_amount))
                    .unwrap_or(boost_amount),
            );
        }
        if let Some(position) = player.position() {
            trace.last_position = Some(position.to_array());
        }
        if let Some(speed) = player.speed() {
            trace.max_speed = trace.max_speed.max(speed);
        }

        let dodge_rising = player.dodge_active && !trace.previous_dodge_active;
        let had_first_dodge = trace.first_dodge_time.is_some();
        if dodge_rising && !had_first_dodge {
            trace.first_dodge_time = Some(frame.time);
            trace.first_dodge_frame = Some(frame.frame_number);
            if let Some(rigid_body) = player.rigid_body.as_ref() {
                let rotation = quat_to_glam(&rigid_body.rotation);
                trace.dodge_onset_forward = Some(rotation * glam::Vec3::X);
                trace.dodge_onset_right = Some(rotation * glam::Vec3::Y);
                // Baseline is the velocity just before the flip impulse lands.
                // Prefer the previous frame's velocity; the onset frame itself
                // often repeats it (boxcars did not re-send it), which is exactly
                // why the old single-frame delta read zero.
                trace.dodge_direction_baseline_velocity =
                    trace.previous_velocity.or_else(|| player.velocity());
                trace.dodge_direction_window_deadline =
                    Some(frame.time + KICKOFF_APPROACH_DODGE_DIRECTION_WINDOW_SECONDS);
            }
        }

        // A later, separate dodge (e.g. flipping forward into the ball at first
        // touch) must not overwrite the first dodge's recovered direction, so
        // close the window as soon as a new dodge begins.
        if dodge_rising && had_first_dodge {
            trace.dodge_direction_window_deadline = None;
        }

        // Recover the dodge direction from the peak velocity change over the
        // window after onset, decomposed in the car's onset frame, with the
        // forward boost contribution removed so a boosting taker's flip still
        // reads as diagonal rather than straight-forward.
        if let (Some(deadline), Some(baseline), Some(forward), Some(right)) = (
            trace.dodge_direction_window_deadline,
            trace.dodge_direction_baseline_velocity,
            trace.dodge_onset_forward,
            trace.dodge_onset_right,
        ) {
            if frame.time <= deadline {
                if player.boost_active {
                    trace.dodge_direction_boost_compensation += forward
                        * KICKOFF_APPROACH_BOOST_ACCELERATION_UU_PER_SECOND_SQUARED
                        * frame.dt;
                }
                if let Some(velocity) = player.velocity() {
                    let delta = velocity - baseline - trace.dodge_direction_boost_compensation;
                    let horizontal = delta.truncate().length();
                    if horizontal > trace.best_dodge_direction_delta
                        && delta.length_squared() > f32::EPSILON
                    {
                        trace.best_dodge_direction_delta = horizontal;
                        let direction = delta.normalize();
                        trace.first_dodge_forward_component = Some(direction.dot(forward));
                        trace.first_dodge_side_component = Some(direction.dot(right));
                    }
                }
            } else {
                trace.dodge_direction_window_deadline = None;
            }
        }

        trace.previous_velocity = player.velocity();
        trace.previous_dodge_active = player.dodge_active;
    }

    fn apply_player_samples(
        active: &mut ActiveKickoff,
        frame: &FrameInfo,
        players: &PlayerFrameState,
    ) {
        for snapshot in &mut active.players {
            if snapshot.first_touch_time.is_some() {
                continue;
            }
            let Some(player) = players.player(&snapshot.player) else {
                continue;
            };
            Self::observe_player_approach(&mut snapshot.approach_trace, frame, player);
        }
    }

    fn team_attack_direction(is_team_0: bool) -> glam::Vec2 {
        glam::Vec2::new(0.0, if is_team_0 { 1.0 } else { -1.0 })
    }

    fn team_right_direction(is_team_0: bool) -> glam::Vec2 {
        glam::Vec2::new(if is_team_0 { 1.0 } else { -1.0 }, 0.0)
    }

    fn normalize_xy(vector: glam::Vec3) -> Option<glam::Vec2> {
        let xy = vector.truncate();
        if xy.length_squared() > f32::EPSILON {
            Some(xy.normalize())
        } else {
            None
        }
    }

    fn kickoff_contact_snapshot(
        player: Option<&PlayerSample>,
        touch: &TouchEvent,
        ball: &BallFrameState,
    ) -> Option<KickoffContactSnapshot> {
        let player_body = player.and_then(|player| player.rigid_body.as_ref());
        let player_position = player_body
            .map(|body| vec_to_glam(&body.location))
            .or_else(|| touch.player_position.as_ref().map(vec_to_glam))?;
        let ball_position = ball.position()?;
        let attack_direction = Self::team_attack_direction(touch.team_is_team_0);
        let right_direction = Self::team_right_direction(touch.team_is_team_0);
        let ball_from_player = ball_position - player_position;
        let player_velocity = player.and_then(PlayerSample::velocity);
        let car_forward = player_body.map(|body| quat_to_glam(&body.rotation) * glam::Vec3::X);
        let contact_estimate = player_body.zip(player).and_then(|(body, player)| {
            car_hitbox_contact_estimate(ball_position, body, player.hitbox)
        });
        let ball_velocity = ball.velocity();

        Some(KickoffContactSnapshot {
            player_position: player_position.to_array(),
            player_velocity: player_velocity.map(|velocity| velocity.to_array()),
            car_forward: car_forward.map(|forward| forward.to_array()),
            local_ball_position: contact_estimate
                .as_ref()
                .map(|estimate| estimate.local_ball_position.to_array()),
            local_contact_point: contact_estimate
                .as_ref()
                .map(|estimate| estimate.local_contact_point.to_array()),
            contact_gap: contact_estimate
                .as_ref()
                .map(|estimate| (estimate.distance - BALL_COLLISION_RADIUS).max(0.0)),
            behind_ball_depth: ball_from_player.truncate().dot(attack_direction),
            lateral_offset: (player_position - ball_position)
                .truncate()
                .dot(right_direction),
            lateral_abs_offset: (player_position - ball_position)
                .truncate()
                .dot(right_direction)
                .abs(),
            velocity_attack_alignment: player_velocity
                .and_then(Self::normalize_xy)
                .map(|velocity| velocity.dot(attack_direction)),
            velocity_ball_alignment: player_velocity
                .and_then(Self::normalize_xy)
                .zip(Self::normalize_xy(ball_from_player))
                .map(|(velocity, ball_direction)| velocity.dot(ball_direction)),
            nose_attack_alignment: car_forward
                .and_then(Self::normalize_xy)
                .map(|forward| forward.dot(attack_direction)),
            ball_exit_attack_alignment: ball_velocity
                .and_then(Self::normalize_xy)
                .map(|velocity| velocity.dot(attack_direction)),
        })
    }

    fn apply_touches(
        active: &mut ActiveKickoff,
        touch_state: &TouchState,
        ball: &BallFrameState,
        players: &PlayerFrameState,
    ) {
        for touch in chronological_touch_events(&touch_state.touch_events) {
            active.touches.push(KickoffTouchSnapshot {
                time: touch.time,
                frame: touch.frame,
                team_is_team_0: touch.team_is_team_0,
                player: touch.player.clone(),
            });
            if active.first_touch_time.is_none() {
                active.first_touch_time = Some(touch.time);
                active.first_touch_frame = Some(touch.frame);
                active.first_touch_team_is_team_0 = Some(touch.team_is_team_0);
                active.first_touch_id = touch.touch_id;
                active.first_touch_ball_position =
                    ball.position().map(|position| position.to_array());
                active.first_touch_ball_velocity =
                    ball.velocity().map(|velocity| velocity.to_array());
            }
            let Some(player_id) = touch.player.as_ref() else {
                continue;
            };
            let Some(player) = active
                .players
                .iter_mut()
                .find(|player| &player.player == player_id)
            else {
                continue;
            };
            if player.first_touch_time.is_none() {
                player.first_touch_boost =
                    player.approach_trace.previous_boost.or(player.start_boost);
                player.first_touch_time = Some(touch.time);
                player.first_touch_frame = Some(touch.frame);
                player.first_touch_contact =
                    Self::kickoff_contact_snapshot(players.player(player_id), touch, ball);
            }
        }
    }

    /// Attribute deduplicated boost pickups to in-flight kickoff players during
    /// their approach window (from movement start through their first touch).
    ///
    /// `pickups` are the events newly emitted this frame by the
    /// `BoostCalculator`, whose `collected_amount` already folds the former
    /// Collected/Stolen ledger transactions into one accurate, capped figure
    /// (overfill and respawn excluded). Summing these is what keeps
    /// `boost_used = start_boost + collected - first_touch_boost` from
    /// overcounting the way gross frame-to-frame `boost_amount` deltas did.
    fn apply_boost_pickups(active: &mut ActiveKickoff, pickups: &[BoostPickupEvent]) {
        if pickups.is_empty() {
            return;
        }
        let lower_bound = active.movement_start_time.unwrap_or(active.start_time);
        for pickup in pickups {
            if pickup.time < lower_bound {
                continue;
            }
            let Some(snapshot) = active
                .players
                .iter_mut()
                .find(|player| player.player == pickup.player_id)
            else {
                continue;
            };
            if snapshot
                .first_touch_time
                .is_some_and(|touch_time| pickup.time > touch_time)
            {
                continue;
            }
            snapshot.approach_trace.pickup_boost_collected += pickup.collected_amount;
            if Self::is_immediate_own_back_big_pickup(snapshot, pickup, lower_bound) {
                snapshot
                    .approach_trace
                    .picked_up_immediate_own_back_big_boost = true;
            }
        }
    }

    fn is_immediate_own_back_big_pickup(
        player: &KickoffPlayerSnapshot,
        pickup: &BoostPickupEvent,
        movement_start_time: f32,
    ) -> bool {
        if pickup.pad_type != BoostPickupPadType::Big {
            return false;
        }
        if pickup.time < movement_start_time {
            return false;
        }
        if pickup.time - movement_start_time > KICKOFF_SUPPORT_BACK_BIG_MAX_SECONDS_AFTER_GO {
            return false;
        }
        let Some(position) = pickup.player_position else {
            return false;
        };
        let pickup_position = glam::Vec2::new(position[0], position[1]);
        let own_back_y = if player.is_team_0 {
            -BOOST_PAD_BACK_CORNER_Y
        } else {
            BOOST_PAD_BACK_CORNER_Y
        };
        [-BOOST_PAD_BACK_CORNER_X, BOOST_PAD_BACK_CORNER_X]
            .iter()
            .any(|x| {
                pickup_position.distance(glam::Vec2::new(*x, own_back_y))
                    <= STANDARD_PAD_MATCH_RADIUS_BIG
            })
    }

    fn apply_speed_flip_events(
        active: &mut ActiveKickoff,
        frame: &FrameInfo,
        speed_flip_events: &[SpeedFlipEvent],
    ) {
        for event in speed_flip_events {
            if event.time < active.start_time || event.resolved_time > frame.time {
                continue;
            }
            if active
                .players
                .iter()
                .any(|player| player.player == event.player)
            {
                active.speed_flip_directions.insert(
                    event.player.clone(),
                    KickoffFlipDirection::from_local_side_component(event.dodge_side_component),
                );
            }
        }
    }

    fn kickoff_start_distance(player: &KickoffPlayerSnapshot) -> f32 {
        glam::Vec2::new(player.start_position[0], player.start_position[1]).length()
    }

    fn relative_left_value(player: &KickoffPlayerSnapshot) -> f32 {
        if player.is_team_0 {
            player.start_position[0]
        } else {
            -player.start_position[0]
        }
    }

    fn expected_taker_by_team(players: &[KickoffPlayerSnapshot], is_team_0: bool) -> Option<usize> {
        let closest_distance = players
            .iter()
            .filter(|player| player.is_team_0 == is_team_0)
            .map(Self::kickoff_start_distance)
            .min_by(|left, right| left.total_cmp(right))?;

        let tied_candidates = players.iter().enumerate().filter(|(_, player)| {
            player.is_team_0 == is_team_0
                && (Self::kickoff_start_distance(player) - closest_distance).abs()
                    <= KICKOFF_TAKER_DISTANCE_TIE_EPSILON
        });

        tied_candidates
            .clone()
            .filter(|(_, player)| player.first_touch_time.is_some())
            .min_by(|(_, left), (_, right)| {
                left.first_touch_time
                    .unwrap_or(f32::INFINITY)
                    .total_cmp(&right.first_touch_time.unwrap_or(f32::INFINITY))
                    .then_with(|| {
                        left.first_touch_frame
                            .unwrap_or(usize::MAX)
                            .cmp(&right.first_touch_frame.unwrap_or(usize::MAX))
                    })
            })
            .or_else(|| {
                // No tied candidate touched the ball (the team was beaten to the
                // kickoff). Distance and first-touch can't disambiguate, so prefer
                // the player who actually committed to the ball: greatest advance
                // toward center, then most boost burned. The static left-side
                // tiebreak is only a last resort for genuinely identical approaches.
                tied_candidates.min_by(|(_, left), (_, right)| {
                    Self::center_progress(right)
                        .total_cmp(&Self::center_progress(left))
                        .then_with(|| {
                            Self::boost_committed(right).total_cmp(&Self::boost_committed(left))
                        })
                        .then_with(|| {
                            Self::relative_left_value(left)
                                .total_cmp(&Self::relative_left_value(right))
                        })
                })
            })
            .map(|(index, _)| index)
    }

    /// Boost spent during the kickoff approach (`start_boost - min_boost`). A
    /// player charging the ball burns boost; a teammate peeling off for a pad
    /// does not, so this separates the true taker from support when neither
    /// player touched the ball.
    fn boost_committed(player: &KickoffPlayerSnapshot) -> f32 {
        match (player.start_boost, player.approach_trace.min_boost) {
            (Some(start_boost), Some(min_boost)) => (start_boost - min_boost).max(0.0),
            _ => 0.0,
        }
    }

    fn taker_outcome(
        player: &KickoffPlayerSnapshot,
        expected_taker_index: Option<usize>,
        player_index: usize,
        team_touched: bool,
    ) -> KickoffTakerOutcome {
        if player.first_touch_time.is_some() {
            KickoffTakerOutcome::Touched
        } else if expected_taker_index == Some(player_index) && team_touched {
            KickoffTakerOutcome::Fake
        } else if expected_taker_index == Some(player_index) {
            KickoffTakerOutcome::Missed
        } else {
            KickoffTakerOutcome::Unknown
        }
    }

    fn is_taker(player_index: usize, expected_taker_index: Option<usize>) -> bool {
        expected_taker_index == Some(player_index)
    }

    fn boost_after(players: &PlayerFrameState, player_id: &PlayerId) -> Option<f32> {
        players.player(player_id).and_then(Self::boost_amount)
    }

    fn boost_used(player: &KickoffPlayerSnapshot, boost_after: Option<f32>) -> f32 {
        let Some(start_boost) = player.start_boost else {
            return 0.0;
        };
        let lowest_boost = player
            .approach_trace
            .min_boost
            .or(boost_after)
            .unwrap_or(start_boost);
        (start_boost - lowest_boost).max(0.0)
    }

    fn taker_time_to_ball(player: &KickoffPlayerSnapshot, movement_start_time: f32) -> Option<f32> {
        player
            .first_touch_time
            .map(|touch_time| (touch_time - movement_start_time).max(0.0))
    }

    fn taker_boost_collected(player: &KickoffPlayerSnapshot) -> f32 {
        player.approach_trace.pickup_boost_collected
    }

    /// Boost the taker has left *at the moment of contact* with the ball.
    ///
    /// This is the counterpart to [`taker_boost_used`], which measures boost
    /// spent over the `start -> first touch` window. The plain finish-frame
    /// sample (`boost_after`) is taken ~1.25s after the touch, by which point
    /// the taker has driven on and possibly re-collected boost, so pairing it
    /// with `boost_used` produces nonsensical combinations (e.g. used 94 /
    /// after 80). Using `first_touch_boost` keeps the two consistent:
    /// `start_boost + boost_collected == boost_used + boost_after`.
    ///
    /// Falls back to the finish-frame sample for takers who never touched the
    /// ball (fake / missed outcomes), where there is no contact moment.
    fn taker_boost_after(player: &KickoffPlayerSnapshot, boost_after: Option<f32>) -> Option<f32> {
        player.first_touch_boost.or(boost_after)
    }

    fn taker_boost_used(player: &KickoffPlayerSnapshot) -> f32 {
        match (player.start_boost, player.first_touch_boost) {
            (Some(start_boost), Some(first_touch_boost)) => {
                (start_boost + Self::taker_boost_collected(player) - first_touch_boost).max(0.0)
            }
            _ => player.approach_trace.sampled_boost_used,
        }
    }

    fn moved_distance(player: &KickoffPlayerSnapshot) -> f32 {
        let Some(last_position) = player.approach_trace.last_position else {
            return 0.0;
        };
        glam::Vec3::from_array(last_position)
            .distance(glam::Vec3::from_array(player.start_position))
    }

    fn approach_dodge_happened_before_contact(player: &KickoffPlayerSnapshot) -> bool {
        let Some(first_dodge_time) = player.approach_trace.first_dodge_time else {
            return false;
        };
        player.first_touch_time.is_none_or(|first_touch_time| {
            first_touch_time - first_dodge_time >= KICKOFF_APPROACH_FLIP_MIN_SECONDS_BEFORE_TOUCH
        })
    }

    fn classify_approach(
        player: &KickoffPlayerSnapshot,
        outcome: KickoffTakerOutcome,
        boost_after: Option<f32>,
        has_speed_flip: bool,
    ) -> KickoffApproach {
        if has_speed_flip {
            return KickoffApproach::SpeedFlip;
        }

        let boost_used = Self::boost_used(player, boost_after);
        let used_boost = player.approach_trace.boost_active_sample_count > 0
            || boost_used >= KICKOFF_APPROACH_MIN_BOOST_USED;
        let forward_component = player
            .approach_trace
            .first_dodge_forward_component
            .unwrap_or(0.0);
        let side_component = player
            .approach_trace
            .first_dodge_side_component
            .unwrap_or(0.0);
        if Self::approach_dodge_happened_before_contact(player) {
            if side_component.abs() >= KICKOFF_APPROACH_DIAGONAL_FLIP_SIDE_COMPONENT {
                return KickoffApproach::DiagonalFlip;
            }
            if forward_component >= KICKOFF_APPROACH_FRONT_FLIP_FORWARD_COMPONENT {
                return KickoffApproach::FrontFlip;
            }
        }

        if player.first_touch_time.is_none() {
            let center_progress = Self::center_progress(player);
            let low_center_progress = center_progress < KICKOFF_SUPPORT_CHEAT_MIN_CENTER_PROGRESS;
            let moved_away_with_boost = used_boost
                && low_center_progress
                && Self::moved_distance(player) >= KICKOFF_APPROACH_MIN_FAKE_MOVE_DISTANCE;
            if matches!(
                outcome,
                KickoffTakerOutcome::Fake | KickoffTakerOutcome::Missed
            ) && low_center_progress
                && (Self::boost_gain(player, boost_after)
                    >= KICKOFF_SUPPORT_GO_FOR_BOOST_MIN_BOOST_GAIN
                    || Self::lateral_movement(player)
                        >= KICKOFF_SUPPORT_GO_FOR_BOOST_MIN_LATERAL_MOVE
                    || moved_away_with_boost)
            {
                return KickoffApproach::FakeGoForBoost;
            }
            if used_boost && center_progress > 0.0 {
                return KickoffApproach::BoostIntoBall;
            }
            return KickoffApproach::Other;
        }

        if used_boost {
            return KickoffApproach::BoostIntoBall;
        }

        KickoffApproach::Other
    }

    fn approach_flip_direction(
        player: &KickoffPlayerSnapshot,
        approach: KickoffApproach,
        speed_flip_direction: Option<KickoffFlipDirection>,
    ) -> KickoffFlipDirection {
        match approach {
            KickoffApproach::SpeedFlip => speed_flip_direction.unwrap_or_default(),
            KickoffApproach::DiagonalFlip => player
                .approach_trace
                .first_dodge_side_component
                .map(KickoffFlipDirection::from_local_side_component)
                .unwrap_or_default(),
            _ => KickoffFlipDirection::NotApplicable,
        }
    }

    fn center_progress(player: &KickoffPlayerSnapshot) -> f32 {
        let Some(last_position) = player.approach_trace.last_position else {
            return 0.0;
        };
        let start_distance =
            glam::Vec2::new(player.start_position[0], player.start_position[1]).length();
        let end_distance = glam::Vec2::new(last_position[0], last_position[1]).length();
        (start_distance - end_distance).max(0.0)
    }

    fn lateral_movement(player: &KickoffPlayerSnapshot) -> f32 {
        let Some(last_position) = player.approach_trace.last_position else {
            return 0.0;
        };
        (last_position[0].abs() - player.start_position[0].abs()).max(0.0)
    }

    fn boost_gain(player: &KickoffPlayerSnapshot, boost_after: Option<f32>) -> f32 {
        match (player.start_boost, boost_after) {
            (Some(start_boost), Some(boost_after)) => (boost_after - start_boost).max(0.0),
            _ => 0.0,
        }
    }

    fn classify_support_behavior(
        player: &KickoffPlayerSnapshot,
        is_taker: bool,
    ) -> Option<KickoffSupportBehavior> {
        if is_taker {
            return None;
        }
        if player.first_touch_time.is_some()
            || Self::center_progress(player) >= KICKOFF_SUPPORT_CHEAT_MIN_CENTER_PROGRESS
        {
            return Some(KickoffSupportBehavior::Cheat);
        }
        if player.approach_trace.picked_up_immediate_own_back_big_boost {
            return Some(KickoffSupportBehavior::GoForBoost);
        }
        Some(KickoffSupportBehavior::Other)
    }

    fn win_strength_band(strength: f32) -> KickoffWinStrengthBand {
        if strength >= KICKOFF_STRONG_WIN_STRENGTH {
            KickoffWinStrengthBand::Strong
        } else if strength >= KICKOFF_CLEAR_WIN_STRENGTH {
            KickoffWinStrengthBand::Clear
        } else {
            KickoffWinStrengthBand::Narrow
        }
    }

    /// Decide the kickoff outcome from the resolution ball sample by projecting
    /// the ball's y position [`KICKOFF_WIN_PROJECTION_SECONDS`] ahead so the
    /// direction of travel counts: a ball barely across the center line but
    /// flying back toward it is not a win for the side it currently sits in.
    /// Strength is the projected depth as a fraction of the half-field length,
    /// so it lands in `0.0..=1.0`.
    fn win_from_ball(ball: &BallFrameState) -> (KickoffOutcome, Option<bool>, Option<f32>) {
        let Some(ball) = ball.sample() else {
            return (KickoffOutcome::Unknown, None, None);
        };
        let projected_y = (ball.position().y + ball.velocity().y * KICKOFF_WIN_PROJECTION_SECONDS)
            .clamp(-KICKOFF_FIELD_HALF_LENGTH, KICKOFF_FIELD_HALF_LENGTH);
        if projected_y.abs() < KICKOFF_WIN_MIN_PROJECTED_BALL_Y {
            return (KickoffOutcome::Neutral, None, None);
        }
        let toward_team_zero_win = projected_y > 0.0;
        let strength = projected_y.abs() / KICKOFF_FIELD_HALF_LENGTH;
        (
            if toward_team_zero_win {
                KickoffOutcome::TeamZeroWin
            } else {
                KickoffOutcome::TeamOneWin
            },
            Some(toward_team_zero_win),
            Some(strength),
        )
    }

    fn ball_direction(ball: &BallFrameState, is_team_0: bool) -> KickoffBallDirection {
        let Some(ball) = ball.sample() else {
            return KickoffBallDirection::Unknown;
        };
        let position_x = ball.position().x;
        if position_x.abs() >= KICKOFF_BALL_DIRECTION_MIN_ABS_X {
            return Self::ball_direction_from_global_x(position_x, is_team_0);
        }
        let velocity_x = ball.velocity().x;
        if velocity_x.abs() >= KICKOFF_BALL_DIRECTION_MIN_ABS_SPEED_X {
            return Self::ball_direction_from_global_x(velocity_x, is_team_0);
        }
        KickoffBallDirection::Center
    }

    fn ball_direction_from_global_x(value: f32, is_team_0: bool) -> KickoffBallDirection {
        if value > 0.0 {
            if is_team_0 {
                KickoffBallDirection::Right
            } else {
                KickoffBallDirection::Left
            }
        } else if is_team_0 {
            KickoffBallDirection::Left
        } else {
            KickoffBallDirection::Right
        }
    }

    fn exit_velocity(ball: &BallFrameState) -> Option<[f32; 3]> {
        ball.sample().map(|ball| ball.velocity().to_array())
    }

    fn exit_speed(exit_velocity: Option<[f32; 3]>) -> Option<f32> {
        exit_velocity
            .map(|velocity| glam::Vec3::new(velocity[0], velocity[1], velocity[2]).length())
    }

    fn first_follow_up_touch<'a>(
        touches: &'a [KickoffTouchSnapshot],
        first_touch_time: Option<f32>,
        first_touch_frame: Option<usize>,
        team_zero_taker_player: Option<&PlayerId>,
        team_one_taker_player: Option<&PlayerId>,
    ) -> Option<&'a KickoffTouchSnapshot> {
        let (Some(first_touch_time), Some(first_touch_frame)) =
            (first_touch_time, first_touch_frame)
        else {
            return None;
        };
        let mut previous_touch_time = first_touch_time;
        for touch in touches
            .iter()
            .filter(|touch| Self::touch_after(touch, first_touch_time, first_touch_frame))
        {
            if Self::is_non_taker_touch(touch, team_zero_taker_player, team_one_taker_player) {
                return Some(touch);
            }
            if touch.time - previous_touch_time > KICKOFF_TOUCH_CLUSTER_MAX_GAP_SECONDS {
                return Some(touch);
            }
            previous_touch_time = touch.time;
        }
        None
    }

    fn first_follow_up_touch_for_active(active: &ActiveKickoff) -> Option<&KickoffTouchSnapshot> {
        let team_zero_taker = Self::expected_taker_by_team(&active.players, true);
        let team_one_taker = Self::expected_taker_by_team(&active.players, false);
        let team_zero_taker_player = team_zero_taker.map(|index| &active.players[index].player);
        let team_one_taker_player = team_one_taker.map(|index| &active.players[index].player);
        Self::first_follow_up_touch(
            &active.touches,
            active.first_touch_time,
            active.first_touch_frame,
            team_zero_taker_player,
            team_one_taker_player,
        )
    }

    fn is_non_taker_touch(
        touch: &KickoffTouchSnapshot,
        team_zero_taker_player: Option<&PlayerId>,
        team_one_taker_player: Option<&PlayerId>,
    ) -> bool {
        let Some(player) = touch.player.as_ref() else {
            return false;
        };
        let expected_taker = if touch.team_is_team_0 {
            team_zero_taker_player
        } else {
            team_one_taker_player
        };
        expected_taker.is_some_and(|taker| taker != player)
    }

    fn touch_after(touch: &KickoffTouchSnapshot, time: f32, frame: usize) -> bool {
        touch.time > time || (touch.time == time && touch.frame > frame)
    }

    fn kickoff_possession_outcome(
        touches: &[KickoffTouchSnapshot],
        first_follow_up_touch: Option<&KickoffTouchSnapshot>,
        winning_team_is_team_0: Option<bool>,
    ) -> (KickoffPossessionOutcome, Option<bool>) {
        let Some(first_follow_up_touch) = first_follow_up_touch else {
            return match winning_team_is_team_0 {
                Some(true) => (KickoffPossessionOutcome::TeamZeroPossession, Some(true)),
                Some(false) => (KickoffPossessionOutcome::TeamOnePossession, Some(false)),
                None => (KickoffPossessionOutcome::Contested, None),
            };
        };
        let possession = match touches.iter().find(|touch| {
            Self::touch_after(
                touch,
                first_follow_up_touch.time,
                first_follow_up_touch.frame,
            )
        }) {
            Some(next_touch)
                if next_touch.team_is_team_0 != first_follow_up_touch.team_is_team_0
                    && next_touch.time - first_follow_up_touch.time
                        <= KICKOFF_POSSESSION_IMMEDIATE_CONTEST_SECONDS =>
            {
                KickoffPossessionOutcome::Contested
            }
            Some(next_touch)
                if next_touch.team_is_team_0 != first_follow_up_touch.team_is_team_0
                    && first_follow_up_touch.team_is_team_0 =>
            {
                KickoffPossessionOutcome::TeamZeroAdvantage
            }
            Some(next_touch)
                if next_touch.team_is_team_0 != first_follow_up_touch.team_is_team_0 =>
            {
                KickoffPossessionOutcome::TeamOneAdvantage
            }
            _ if first_follow_up_touch.team_is_team_0 => {
                KickoffPossessionOutcome::TeamZeroPossession
            }
            _ => KickoffPossessionOutcome::TeamOnePossession,
        };
        let possession_team = match possession {
            KickoffPossessionOutcome::TeamZeroPossession
            | KickoffPossessionOutcome::TeamZeroAdvantage => Some(true),
            KickoffPossessionOutcome::TeamOnePossession
            | KickoffPossessionOutcome::TeamOneAdvantage => Some(false),
            _ => None,
        };
        (possession, possession_team)
    }

    fn should_finish(
        active: &ActiveKickoff,
        frame: &FrameInfo,
        gameplay: &GameplayState,
        events: &FrameEventsState,
    ) -> bool {
        if !events.goal_events.is_empty() {
            return true;
        }
        let Some(first_touch_time) = active.first_touch_time else {
            return gameplay.game_state == Some(GAME_STATE_GOAL_SCORED_REPLAY);
        };
        (active.resolution.is_some() && Self::first_follow_up_touch_for_active(active).is_some())
            || frame.time - first_touch_time >= KICKOFF_FOLLOW_UP_AFTER_FIRST_TOUCH_SECONDS
            || gameplay.game_state == Some(GAME_STATE_GOAL_SCORED_REPLAY)
    }

    fn should_capture_resolution(active: &ActiveKickoff, frame: &FrameInfo) -> bool {
        active.resolution.is_none()
            && active.first_touch_time.is_some_and(|first_touch_time| {
                frame.time - first_touch_time >= KICKOFF_RESOLUTION_AFTER_FIRST_TOUCH_SECONDS
            })
    }

    fn earliest_goal(events: &FrameEventsState) -> Option<&GoalEvent> {
        events.goal_events.iter().min_by(|left, right| {
            left.time
                .total_cmp(&right.time)
                .then_with(|| left.frame.cmp(&right.frame))
        })
    }

    /// Track the ball's y extremes from the kickoff's first touch onward,
    /// including frames after the kickoff's logical close while it awaits
    /// goal attribution.
    fn observe_ball_extent(active: &mut ActiveKickoff, ball: &BallFrameState) {
        if active.first_touch_time.is_none() {
            return;
        }
        let Some(sample) = ball.sample() else {
            return;
        };
        let y = sample.position().y;
        active.min_ball_y_after_first_touch = Some(
            active
                .min_ball_y_after_first_touch
                .map_or(y, |current| current.min(y)),
        );
        active.max_ball_y_after_first_touch = Some(
            active
                .max_ball_y_after_first_touch
                .map_or(y, |current| current.max(y)),
        );
    }

    /// Whether a goal qualifies as a kickoff goal. The goal must land within
    /// [`KICKOFF_GOAL_MAX_SECONDS`] of the first touch, but proximity in time
    /// alone is not enough: the goal also has to flow from the kickoff
    /// exchange itself, so the conceding team must never have settled the ball
    /// in between, and the play must not have reset through the scoring
    /// team's own half.
    fn kickoff_goal_qualifies(active: &ActiveKickoff, goal: &GoalEvent) -> bool {
        let Some(first_touch_time) = active.first_touch_time else {
            return false;
        };
        let time_to_goal = goal.time - first_touch_time;
        (0.0..KICKOFF_GOAL_MAX_SECONDS).contains(&time_to_goal)
            && !Self::conceding_team_established_possession(&active.touches, goal)
            && !Self::ball_reset_into_scoring_half(active, goal)
    }

    /// The conceding team "established possession" when it recorded two
    /// touches separated by more than the immediate-contest window with no
    /// scoring-team touch in between — they settled the ball rather than
    /// merely deflecting it during the kickoff scramble. A single conceding
    /// touch (a failed clear or deflection straight into punishment) does not
    /// break the kickoff-goal chain.
    fn conceding_team_established_possession(
        touches: &[KickoffTouchSnapshot],
        goal: &GoalEvent,
    ) -> bool {
        let conceding_is_team_0 = !goal.scoring_team_is_team_0;
        let mut first_conceding_touch_time: Option<f32> = None;
        for touch in touches.iter().filter(|touch| touch.time <= goal.time) {
            if touch.team_is_team_0 == conceding_is_team_0 {
                match first_conceding_touch_time {
                    Some(anchor)
                        if touch.time - anchor > KICKOFF_POSSESSION_IMMEDIATE_CONTEST_SECONDS =>
                    {
                        return true;
                    }
                    Some(_) => {}
                    None => first_conceding_touch_time = Some(touch.time),
                }
            } else {
                first_conceding_touch_time = None;
            }
        }
        false
    }

    /// Whether the ball retreated past [`KICKOFF_GOAL_MAX_DEFENSIVE_BALL_Y`]
    /// into the scoring team's own half between the kickoff's first touch and
    /// the goal. Team zero attacks positive y, so its defensive half is
    /// negative y.
    fn ball_reset_into_scoring_half(active: &ActiveKickoff, goal: &GoalEvent) -> bool {
        if goal.scoring_team_is_team_0 {
            active
                .min_ball_y_after_first_touch
                .is_some_and(|y| y < -KICKOFF_GOAL_MAX_DEFENSIVE_BALL_Y)
        } else {
            active
                .max_ball_y_after_first_touch
                .is_some_and(|y| y > KICKOFF_GOAL_MAX_DEFENSIVE_BALL_Y)
        }
    }

    /// Attribute a qualifying goal (see [`Self::kickoff_goal_qualifies`]) that
    /// landed after the kickoff's logical close to the concluded kickoff
    /// event. Mirrors the attribution `finish_event` performs when the goal
    /// arrives while the kickoff is still open.
    fn attribute_goal(event: &mut KickoffEvent, goal: &GoalEvent) {
        let Some(first_touch_time) = event.first_touch_time else {
            return;
        };
        event.time_to_goal = Some(goal.time - first_touch_time);
        event.kickoff_goal = true;
        event.scoring_team_is_team_0 = Some(goal.scoring_team_is_team_0);
        if event.first_follow_up_touch_time.is_none() {
            event.kickoff_possession_outcome = if goal.scoring_team_is_team_0 {
                KickoffPossessionOutcome::TeamZeroPossession
            } else {
                KickoffPossessionOutcome::TeamOnePossession
            };
            event.kickoff_possession_team_is_team_0 = Some(goal.scoring_team_is_team_0);
        }
    }

    fn finish_event(
        active: ActiveKickoff,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        events: &FrameEventsState,
        speed_flip_events: &[SpeedFlipEvent],
    ) -> KickoffEvent {
        let mut active = active;
        Self::apply_speed_flip_events(&mut active, frame, speed_flip_events);
        let resolution_ball = active
            .resolution
            .as_ref()
            .map(|resolution| &resolution.ball)
            .unwrap_or(ball);
        let (outcome, winning_team_is_team_0, win_strength) = Self::win_from_ball(resolution_ball);
        let scoring_goal = events.goal_events.iter().min_by(|left, right| {
            left.time
                .total_cmp(&right.time)
                .then_with(|| left.frame.cmp(&right.frame))
        });
        let time_to_goal = scoring_goal.and_then(|goal| {
            active
                .first_touch_time
                .map(|first_touch| goal.time - first_touch)
        });
        let kickoff_goal =
            scoring_goal.is_some_and(|goal| Self::kickoff_goal_qualifies(&active, goal));
        let win_strength_band = win_strength
            .map(Self::win_strength_band)
            .unwrap_or_default();
        let team_zero_taker = Self::expected_taker_by_team(&active.players, true);
        let team_one_taker = Self::expected_taker_by_team(&active.players, false);
        let kickoff_type = KickoffType::from_taker_spawns(
            team_zero_taker.map(|index| active.players[index].spawn_position),
            team_one_taker.map(|index| active.players[index].spawn_position),
        );
        let kickoff_direction = KickoffDirection::from_taker_spawns(
            team_zero_taker.map(|index| active.players[index].spawn_position),
            team_one_taker.map(|index| active.players[index].spawn_position),
        );
        let first_touch = active.touches.first();
        let first_touch_player = first_touch.and_then(|touch| touch.player.clone());
        let team_zero_taker_touch_time =
            team_zero_taker.and_then(|index| active.players[index].first_touch_time);
        let team_zero_taker_touch_frame =
            team_zero_taker.and_then(|index| active.players[index].first_touch_frame);
        let team_one_taker_touch_time =
            team_one_taker.and_then(|index| active.players[index].first_touch_time);
        let team_one_taker_touch_frame =
            team_one_taker.and_then(|index| active.players[index].first_touch_frame);
        let taker_touch_delay_seconds =
            match (team_zero_taker_touch_time, team_one_taker_touch_time) {
                (Some(team_zero_time), Some(team_one_time)) => {
                    Some((team_one_time - team_zero_time).abs())
                }
                _ => None,
            };
        let first_touch_ball_position = active.first_touch_ball_position;
        let first_touch_ball_abs_x = first_touch_ball_position.map(|position| position[0].abs());
        let first_touch_ball_height = first_touch_ball_position.map(|position| position[2]);
        let exit_velocity = Self::exit_velocity(resolution_ball);
        let exit_speed = Self::exit_speed(exit_velocity);
        let exit_y_velocity = exit_velocity.map(|velocity| velocity[1]);
        let team_zero_taker_player = team_zero_taker.map(|index| &active.players[index].player);
        let team_one_taker_player = team_one_taker.map(|index| &active.players[index].player);
        let first_follow_up_touch = Self::first_follow_up_touch(
            &active.touches,
            active.first_touch_time,
            active.first_touch_frame,
            team_zero_taker_player,
            team_one_taker_player,
        );
        let first_follow_up_touch_team_is_team_0 =
            first_follow_up_touch.map(|touch| touch.team_is_team_0);
        let (mut kickoff_possession_outcome, mut kickoff_possession_team_is_team_0) =
            Self::kickoff_possession_outcome(
                &active.touches,
                first_follow_up_touch,
                winning_team_is_team_0,
            );
        if kickoff_goal && first_follow_up_touch.is_none() {
            if let Some(goal) = scoring_goal {
                kickoff_possession_outcome = if goal.scoring_team_is_team_0 {
                    KickoffPossessionOutcome::TeamZeroPossession
                } else {
                    KickoffPossessionOutcome::TeamOnePossession
                };
                kickoff_possession_team_is_team_0 = Some(goal.scoring_team_is_team_0);
            }
        }
        let team_zero_touched = active
            .players
            .iter()
            .any(|player| player.is_team_0 && player.first_touch_time.is_some());
        let team_one_touched = active
            .players
            .iter()
            .any(|player| !player.is_team_0 && player.first_touch_time.is_some());
        let mut team_zero_taker_event = None;
        let mut team_one_taker_event = None;
        let mut team_zero_non_takers = Vec::new();
        let mut team_one_non_takers = Vec::new();
        let movement_start_time = active.movement_start_time.unwrap_or(active.start_time);
        for (index, player) in active.players.iter().enumerate() {
            let expected_taker = if player.is_team_0 {
                team_zero_taker
            } else {
                team_one_taker
            };
            let boost_after = Self::boost_after(players, &player.player);
            let is_taker = Self::is_taker(index, expected_taker);
            if is_taker {
                let outcome = Self::taker_outcome(
                    player,
                    expected_taker,
                    index,
                    if player.is_team_0 {
                        team_zero_touched
                    } else {
                        team_one_touched
                    },
                );
                let taker_boost_after = Self::taker_boost_after(player, boost_after);
                let contact = player.first_touch_contact.as_ref();
                let speed_flip_direction =
                    active.speed_flip_directions.get(&player.player).copied();
                let approach = Self::classify_approach(
                    player,
                    outcome,
                    taker_boost_after,
                    speed_flip_direction.is_some(),
                );
                let player_event = KickoffTakerEvent {
                    player: player.player.clone(),
                    is_team_0: player.is_team_0,
                    start_position: player.start_position,
                    spawn_position: player.spawn_position,
                    start_boost: player.start_boost,
                    boost_after: taker_boost_after,
                    time_to_ball: Self::taker_time_to_ball(player, movement_start_time),
                    boost_collected: Self::taker_boost_collected(player),
                    boost_used: Self::taker_boost_used(player),
                    ball_direction: Self::ball_direction(ball, player.is_team_0),
                    first_touch_time: player.first_touch_time,
                    first_touch_frame: player.first_touch_frame,
                    contact_player_position: contact.map(|contact| contact.player_position),
                    contact_player_velocity: contact.and_then(|contact| contact.player_velocity),
                    contact_car_forward: contact.and_then(|contact| contact.car_forward),
                    contact_local_ball_position: contact
                        .and_then(|contact| contact.local_ball_position),
                    contact_local_contact_point: contact
                        .and_then(|contact| contact.local_contact_point),
                    contact_gap: contact.and_then(|contact| contact.contact_gap),
                    contact_behind_ball_depth: contact.map(|contact| contact.behind_ball_depth),
                    contact_lateral_offset: contact.map(|contact| contact.lateral_offset),
                    contact_lateral_abs_offset: contact.map(|contact| contact.lateral_abs_offset),
                    contact_velocity_attack_alignment: contact
                        .and_then(|contact| contact.velocity_attack_alignment),
                    contact_velocity_ball_alignment: contact
                        .and_then(|contact| contact.velocity_ball_alignment),
                    contact_nose_attack_alignment: contact
                        .and_then(|contact| contact.nose_attack_alignment),
                    contact_ball_exit_attack_alignment: contact
                        .and_then(|contact| contact.ball_exit_attack_alignment),
                    outcome,
                    approach,
                    approach_flip_direction: Self::approach_flip_direction(
                        player,
                        approach,
                        speed_flip_direction,
                    ),
                };
                if player_event.is_team_0 {
                    team_zero_taker_event = Some(player_event);
                } else {
                    team_one_taker_event = Some(player_event);
                }
            } else {
                let player_event = KickoffSupportEvent {
                    player: player.player.clone(),
                    is_team_0: player.is_team_0,
                    start_position: player.start_position,
                    start_distance_from_center: Self::kickoff_start_distance(player),
                    spawn_position: player.spawn_position,
                    start_boost: player.start_boost,
                    boost_after,
                    first_touch_time: player.first_touch_time,
                    first_touch_frame: player.first_touch_frame,
                    support_behavior: Self::classify_support_behavior(player, false)
                        .unwrap_or_default(),
                };
                if player_event.is_team_0 {
                    team_zero_non_takers.push(player_event);
                } else {
                    team_one_non_takers.push(player_event);
                }
            }
        }

        KickoffEvent {
            start_time: active.start_time,
            start_frame: active.start_frame,
            end_time: frame.time,
            end_frame: frame.frame_number,
            live_action_start_time: active.live_action_start_time,
            live_action_start_frame: active.live_action_start_frame,
            movement_start_time,
            movement_start_frame: active.movement_start_frame.unwrap_or(active.start_frame),
            kickoff_type,
            kickoff_direction,
            first_touch_time: active.first_touch_time,
            first_touch_frame: active.first_touch_frame,
            first_touch_team_is_team_0: active.first_touch_team_is_team_0,
            first_touch_player,
            first_touch_id: active.first_touch_id,
            first_touch_ball_position,
            first_touch_ball_abs_x,
            first_touch_ball_height,
            first_touch_ball_velocity: active.first_touch_ball_velocity,
            team_zero_taker_touch_time,
            team_zero_taker_touch_frame,
            team_one_taker_touch_time,
            team_one_taker_touch_frame,
            taker_touch_delay_seconds,
            exit_velocity,
            exit_speed,
            exit_y_velocity,
            first_follow_up_touch_time: first_follow_up_touch.map(|touch| touch.time),
            first_follow_up_touch_frame: first_follow_up_touch.map(|touch| touch.frame),
            first_follow_up_touch_team_is_team_0,
            first_follow_up_touch_player: first_follow_up_touch
                .and_then(|touch| touch.player.clone()),
            outcome,
            winning_team_is_team_0,
            win_strength,
            win_strength_band,
            kickoff_possession_outcome,
            kickoff_possession_team_is_team_0,
            kickoff_goal,
            scoring_team_is_team_0: scoring_goal.map(|goal| goal.scoring_team_is_team_0),
            time_to_goal,
            // The advantage usually resolves after the kickoff's logical
            // close; the watcher's verdict is stamped on at emission
            // (`emit_concluded`).
            advantage: KickoffAdvantage::NoAdvantage,
            advantage_team_is_team_0: None,
            advantage_time: None,
            advantage_frame: None,
            advantage_seconds_after_first_touch: None,
            advantage_player: None,
            team_zero_taker: team_zero_taker_event,
            team_one_taker: team_one_taker_event,
            team_zero_non_takers,
            team_one_non_takers,
        }
    }

    pub fn update(
        &mut self,
        frame: &FrameInfo,
        gameplay: &GameplayState,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        touch_state: &TouchState,
        events: &FrameEventsState,
    ) -> SubtrActorResult<()> {
        self.update_with_speed_flips(KickoffUpdateContext {
            frame,
            gameplay,
            ball,
            players,
            touch_state,
            events,
            speed_flip_events: &[],
            boost_pickups: &[],
        })
    }

    pub(crate) fn update_with_speed_flips(
        &mut self,
        ctx: KickoffUpdateContext<'_>,
    ) -> SubtrActorResult<()> {
        self.events.begin_update();
        if ctx.gameplay.kickoff_phase_active() {
            // Once the next kickoff phase begins, no further goal can belong
            // to a previous kickoff: flush any concluded kickoff still held
            // for goal attribution before arming the new one.
            let flushed = self.active.advance(ctx.frame.time, |active| {
                if active.concluded.is_some() {
                    Disposition::Finalize(FinalizeReason::Completed)
                } else {
                    Disposition::Keep
                }
            });
            for (active, _reason) in flushed {
                Self::emit_concluded(&mut self.events, active);
            }
            if self.active.is_empty() {
                self.start_kickoff(ctx.frame, ctx.players);
            }
        }

        let Some(active) = self.active.in_flight_mut().first_mut() else {
            return Ok(());
        };
        if active.concluded.is_none() {
            Self::observe_movement_start(active, ctx.frame, ctx.gameplay);
            if !ctx.gameplay.kickoff_countdown_active() {
                Self::observe_live_action_start(active, ctx.frame);
            }
            Self::apply_player_samples(active, ctx.frame, ctx.players);
            Self::apply_touches(active, ctx.touch_state, ctx.ball, ctx.players);
            Self::apply_boost_pickups(active, ctx.boost_pickups);
            Self::apply_speed_flip_events(active, ctx.frame, ctx.speed_flip_events);
            if Self::should_capture_resolution(active, ctx.frame) {
                active.resolution = Some(KickoffResolutionSnapshot {
                    ball: ctx.ball.clone(),
                });
            }
        } else {
            // The frozen event no longer changes, but the kickoff-goal gates
            // still need the touch chain and ball-position history through the
            // attribution window.
            Self::apply_touches(active, ctx.touch_state, ctx.ball, ctx.players);
        }
        Self::observe_ball_extent(active, ctx.ball);
        if let Some(goal) = Self::earliest_goal(ctx.events) {
            if Self::kickoff_goal_qualifies(active, goal) {
                active.advantage.establish_goal(goal);
            }
        }
        active
            .advantage
            .observe(ctx.frame, ctx.ball, &active.touches);

        // Natural finalization happens in two stages. The kickoff *concludes*
        // once `should_finish` is met: its event content is frozen there
        // (touches, possession, exit ball state). It then stays in flight,
        // awaiting goal attribution, until a goal arrives, the attribution
        // window closes, or the next kickoff begins. This lets a goal scored
        // after the kickoff's logical close — but still within
        // `KICKOFF_GOAL_MAX_SECONDS` of the first touch — count as a kickoff
        // goal. The ledger keeps the in-flight kickoff queryable and
        // guarantees it is resolved at the end of the stream.
        let finished = self.active.advance(ctx.frame.time, |active| {
            if active.concluded.is_some() {
                if let Some(goal) = Self::earliest_goal(ctx.events) {
                    if Self::kickoff_goal_qualifies(active, goal) {
                        let event = active
                            .concluded
                            .as_deref_mut()
                            .expect("concluded checked above");
                        Self::attribute_goal(event, goal);
                    }
                    // Whether or not the goal qualified, play stops here; no
                    // later goal can belong to this kickoff.
                    return Disposition::Finalize(FinalizeReason::Completed);
                }
                let attribution_window_closed = active
                    .first_touch_time
                    .map(|first_touch_time| {
                        ctx.frame.time - first_touch_time >= KICKOFF_GOAL_MAX_SECONDS
                    })
                    .unwrap_or(true);
                if attribution_window_closed
                    || ctx.gameplay.game_state == Some(GAME_STATE_GOAL_SCORED_REPLAY)
                {
                    return Disposition::Finalize(FinalizeReason::Completed);
                }
                return Disposition::Keep;
            }
            if Self::should_finish(active, ctx.frame, ctx.gameplay, ctx.events) {
                let event = Self::finish_event(
                    active.clone(),
                    ctx.frame,
                    ctx.ball,
                    ctx.players,
                    ctx.events,
                    ctx.speed_flip_events,
                );
                active.concluded = Some(Box::new(event));
                // A goal (or goal replay) at the close frame is already
                // attributed by `finish_event`; nothing further can change the
                // event, so emit immediately.
                if !ctx.events.goal_events.is_empty()
                    || ctx.gameplay.game_state == Some(GAME_STATE_GOAL_SCORED_REPLAY)
                {
                    return Disposition::Finalize(FinalizeReason::Completed);
                }
                return Disposition::Keep;
            }
            Disposition::Keep
        });
        for (active, _reason) in finished {
            Self::emit_concluded(&mut self.events, active);
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "kickoff_tests.rs"]
mod tests;
