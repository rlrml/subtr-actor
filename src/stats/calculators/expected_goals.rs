//! Continuous threat / expected-goals state value.
//!
//! Evaluates a state-value function `V(state)` for BOTH teams on every
//! live-play frame: the probability (per the versioned model in
//! [`super::expected_goals_model`]) that the team scores within the next
//! [`THREAT_HORIZON_SECONDS`](super::expected_goals_model::THREAT_HORIZON_SECONDS)
//! seconds, computed from full ball + player physics state. Shots are *not* a
//! gating event -- threat is continuous. Derived observations:
//!
//! - [`ThreatTouchEvent`]: the detection-frame change in the touching team's V
//!   (detection-frame V minus the preceding live-frame V, both from the
//!   toucher's team's perspective). This is an observed one-frame delta, not a
//!   causal estimate of a touch's multi-frame impulse.
//! - [`ThreatEpisodeEvent`]: a contiguous span where one team's V exceeds
//!   [`THREAT_EPISODE_THRESHOLD`]; the episode's xG is the time integral
//!   `sum(V * dt) / tau` over the span (`tau` =
//!   [`THREAT_HORIZON_SECONDS`](super::expected_goals_model::THREAT_HORIZON_SECONDS)),
//!   credited to the attacking toucher associated with the episode's peak V.
//!   Dividing the time integral by the prediction horizon corrects for the
//!   overlapping windows evaluated on adjacent frames; summing episode peaks
//!   would count the same sustained chance repeatedly. The peak survives on
//!   the event as `peak_value` for display/intensity.
//! - The per-team full-match integral (over ALL evaluated live frames, not
//!   just above-threshold ones) is exposed via
//!   [`ExpectedGoalsCalculator::team_xg_integrals`] and is the team's
//!   accumulated xG.

use super::*;

/// Episode threshold on V. This keeps episodes focused on elevated scoring
/// probability rather than ordinary offensive-half possession.
pub const THREAT_EPISODE_THRESHOLD: f32 = 0.15;

/// A ballistic trajectory must cross the goal line within this many seconds
/// for the `on_target` feature to fire. Slightly looser than the shot
/// detector's 2.5s so slower-developing on-frame balls still register; beyond
/// this a wall/ground bounce almost certainly intervenes.
const ON_TARGET_MAX_SECONDS: f32 = 3.0;

/// Normalizer for player-to-ball / player-to-goal context distances. Distances
/// are clamped to this before dividing, so the features saturate at "too far
/// to matter" rather than scaling with arena diagonals.
const PLAYER_DISTANCE_NORM: f32 = 4000.0;

/// Maximum in-field distance to the goal center (far corner at ceiling
/// height), used to normalize ball/player goal distances into [0, 1].
const GOAL_DISTANCE_NORM: f32 = 11_200.0;

/// Ball-center height of a ball resting on the goal line; the aim point used
/// for goal-center distances and radial speed.
const GOAL_CENTER_Z: f32 = BALL_RADIUS_Z;

/// Net-region box for each player's `in_net` feature.
const NET_REGION_DEPTH_Y: f32 = 650.0;
const NET_REGION_MARGIN: f32 = 150.0;

/// How long after a stoppage closes an episode a goal may still resolve it as
/// a goal episode. Goal attribution (score change / goal event) can trail the
/// moment live play ends by the length of the goal-replay intro, so
/// stoppage-closed episodes are held pending until a goal arrives, the next
/// kickoff phase begins, or this grace expires.
const PENDING_EPISODE_GOAL_GRACE_SECONDS: f32 = 10.0;

/// Two goal records for the same team within this window are treated as one
/// goal (a replicated goal event plus the scoreboard increment).
const GOAL_RECORD_DEDUPE_SECONDS: f32 = 2.0;

pub const PLAYER_THREAT_FEATURE_COUNT: usize = 16;
pub const TEAM_THREAT_FEATURE_COUNT: usize = 2 * PLAYER_THREAT_FEATURE_COUNT;
pub const THREAT_FEATURE_COUNT: usize = 8 + 2 * TEAM_THREAT_FEATURE_COUNT;

/// Identically shaped state computed for every player before any team
/// aggregation. No player receives a positional role or a distinct schema.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize)]
pub struct PlayerThreatFeatures {
    pub position_x: f32,
    pub position_y: f32,
    pub position_z: f32,
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub velocity_z: f32,
    pub forward_x: f32,
    pub forward_y: f32,
    pub forward_z: f32,
    pub distance_to_ball: f32,
    pub distance_to_goal: f32,
    pub boost: f32,
    pub is_goalside: f32,
    pub in_net: f32,
    pub dodge_available: f32,
    pub demoed: f32,
}

impl PlayerThreatFeatures {
    pub const FEATURE_NAMES: [&'static str; PLAYER_THREAT_FEATURE_COUNT] = [
        "position_x",
        "position_y",
        "position_z",
        "velocity_x",
        "velocity_y",
        "velocity_z",
        "forward_x",
        "forward_y",
        "forward_z",
        "distance_to_ball",
        "distance_to_goal",
        "boost",
        "is_goalside",
        "in_net",
        "dodge_available",
        "demoed",
    ];

    pub fn to_array(self) -> [f32; PLAYER_THREAT_FEATURE_COUNT] {
        [
            self.position_x,
            self.position_y,
            self.position_z,
            self.velocity_x,
            self.velocity_y,
            self.velocity_z,
            self.forward_x,
            self.forward_y,
            self.forward_z,
            self.distance_to_ball,
            self.distance_to_goal,
            self.boost,
            self.is_goalside,
            self.in_net,
            self.dodge_available,
            self.demoed,
        ]
    }

    fn from_array(values: [f32; PLAYER_THREAT_FEATURE_COUNT]) -> Self {
        Self {
            position_x: values[0],
            position_y: values[1],
            position_z: values[2],
            velocity_x: values[3],
            velocity_y: values[4],
            velocity_z: values[5],
            forward_x: values[6],
            forward_y: values[7],
            forward_z: values[8],
            distance_to_ball: values[9],
            distance_to_goal: values[10],
            boost: values[11],
            is_goalside: values[12],
            in_net: values[13],
            dodge_available: values[14],
            demoed: values[15],
        }
    }
}

/// Permutation-invariant representation of one two-player team. `mean`
/// captures the team's center state and `spread` is the component-wise
/// absolute difference between its players. Both are unchanged when the two
/// players are swapped.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize)]
pub struct TeamThreatFeatures {
    pub mean: PlayerThreatFeatures,
    pub spread: PlayerThreatFeatures,
}

impl TeamThreatFeatures {
    fn from_players(first: PlayerThreatFeatures, second: PlayerThreatFeatures) -> Self {
        let first = first.to_array();
        let second = second.to_array();
        Self {
            mean: PlayerThreatFeatures::from_array(std::array::from_fn(|index| {
                (first[index] + second[index]) * 0.5
            })),
            spread: PlayerThreatFeatures::from_array(std::array::from_fn(|index| {
                (first[index] - second[index]).abs()
            })),
        }
    }

    fn to_array(self) -> [f32; TEAM_THREAT_FEATURE_COUNT] {
        let mut values = [0.0; TEAM_THREAT_FEATURE_COUNT];
        values[..PLAYER_THREAT_FEATURE_COUNT].copy_from_slice(&self.mean.to_array());
        values[PLAYER_THREAT_FEATURE_COUNT..].copy_from_slice(&self.spread.to_array());
        values
    }
}

/// Per-frame, per-team threat features, normalized so the team under
/// evaluation always attacks +Y (team one's world is rotated 180 degrees
/// about the z axis). Ball and per-player values are normalized into [-1, 1]
/// or [0, 1]; absolute pair spreads are therefore bounded by [0, 2].
///
/// [`ThreatFeatures::FEATURE_NAMES`] and [`ThreatFeatures::to_array`] share
/// one order -- that contract is what the offline training pipeline joins on.
/// This schema is defined only for 2v2. Every player passes through the same
/// feature transform, then each perspective-relative team pair is aggregated
/// without ordering. Team affiliation remains explicit because the output is
/// conditioned on which side is trying to score; there are no learned
/// near/far or first/second-player roles.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct ThreatFeatures {
    /// Ball y in the attacking frame / 5120: -1 at own goal line, +1 at the
    /// opponent goal line.
    pub ball_forward_y: f32,
    /// Ball distance to the opponent goal center (0, 5120, ball radius),
    /// normalized by [`GOAL_DISTANCE_NORM`].
    pub ball_dist_to_goal: f32,
    /// Ball height / ceiling height.
    pub ball_height: f32,
    /// Ball speed / the 6000 uu/s ball speed cap.
    pub ball_speed: f32,
    /// Radial ball speed toward the opponent goal center, / 6000 (negative
    /// when moving away).
    pub ball_speed_toward_goal: f32,
    /// Horizontal angle subtended by the goal mouth (posts at x = +/-893)
    /// from the ball, / pi.
    pub goal_open_angle: f32,
    /// 1.0 when the ballistic trajectory (gravity -650 uu/s^2) crosses the
    /// goal plane inside the mouth within [`ON_TARGET_MAX_SECONDS`].
    pub on_target: f32,
    /// 1 / (1 + seconds until the ball crosses the goal-line plane), or 0
    /// when it is not moving toward the plane. Higher = sooner.
    pub time_to_goal_line: f32,
    pub own_team: TeamThreatFeatures,
    pub opponent_team: TeamThreatFeatures,
}

/// Canonical per-frame threat feature rows published for ndarray extraction
/// and consumed by the expected-goals model. `current` is `None` outside live
/// play or when the ball state is unavailable.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ThreatFeaturesState {
    current: Option<[ThreatFeatures; 2]>,
}

impl ThreatFeaturesState {
    pub fn current(&self) -> Option<&[ThreatFeatures; 2]> {
        self.current.as_ref()
    }

    pub(crate) fn update(
        &mut self,
        is_doubles: bool,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        events: &FrameEventsState,
        dodge_available: &HashMap<PlayerId, bool>,
        live_play_state: &LivePlayState,
    ) {
        let Some(ball_sample) = ball
            .sample()
            .filter(|_| live_play_state.is_live_play && is_doubles)
        else {
            self.current = None;
            return;
        };
        let demoed_players: HashSet<PlayerId> = events
            .active_demos
            .iter()
            .map(|demo| demo.victim.clone())
            .collect();
        self.current = compute_threat_features(
            ball_sample.position(),
            ball_sample.velocity(),
            players,
            &demoed_players,
            dodge_available,
            true,
        )
        .zip(compute_threat_features(
            ball_sample.position(),
            ball_sample.velocity(),
            players,
            &demoed_players,
            dodge_available,
            false,
        ))
        .map(|(team_zero, team_one)| [team_zero, team_one]);
    }
}

impl ThreatFeatures {
    /// Column names for [`Self::to_array`], in the same order. The offline
    /// training pipeline joins on these names.
    pub const FEATURE_NAMES: [&'static str; THREAT_FEATURE_COUNT] = [
        "ball_forward_y",
        "ball_dist_to_goal",
        "ball_height",
        "ball_speed",
        "ball_speed_toward_goal",
        "goal_open_angle",
        "on_target",
        "time_to_goal_line",
        "own_team_mean_position_x",
        "own_team_mean_position_y",
        "own_team_mean_position_z",
        "own_team_mean_velocity_x",
        "own_team_mean_velocity_y",
        "own_team_mean_velocity_z",
        "own_team_mean_forward_x",
        "own_team_mean_forward_y",
        "own_team_mean_forward_z",
        "own_team_mean_distance_to_ball",
        "own_team_mean_distance_to_goal",
        "own_team_mean_boost",
        "own_team_mean_is_goalside",
        "own_team_mean_in_net",
        "own_team_mean_dodge_available",
        "own_team_mean_demoed",
        "own_team_spread_position_x",
        "own_team_spread_position_y",
        "own_team_spread_position_z",
        "own_team_spread_velocity_x",
        "own_team_spread_velocity_y",
        "own_team_spread_velocity_z",
        "own_team_spread_forward_x",
        "own_team_spread_forward_y",
        "own_team_spread_forward_z",
        "own_team_spread_distance_to_ball",
        "own_team_spread_distance_to_goal",
        "own_team_spread_boost",
        "own_team_spread_is_goalside",
        "own_team_spread_in_net",
        "own_team_spread_dodge_available",
        "own_team_spread_demoed",
        "opponent_team_mean_position_x",
        "opponent_team_mean_position_y",
        "opponent_team_mean_position_z",
        "opponent_team_mean_velocity_x",
        "opponent_team_mean_velocity_y",
        "opponent_team_mean_velocity_z",
        "opponent_team_mean_forward_x",
        "opponent_team_mean_forward_y",
        "opponent_team_mean_forward_z",
        "opponent_team_mean_distance_to_ball",
        "opponent_team_mean_distance_to_goal",
        "opponent_team_mean_boost",
        "opponent_team_mean_is_goalside",
        "opponent_team_mean_in_net",
        "opponent_team_mean_dodge_available",
        "opponent_team_mean_demoed",
        "opponent_team_spread_position_x",
        "opponent_team_spread_position_y",
        "opponent_team_spread_position_z",
        "opponent_team_spread_velocity_x",
        "opponent_team_spread_velocity_y",
        "opponent_team_spread_velocity_z",
        "opponent_team_spread_forward_x",
        "opponent_team_spread_forward_y",
        "opponent_team_spread_forward_z",
        "opponent_team_spread_distance_to_ball",
        "opponent_team_spread_distance_to_goal",
        "opponent_team_spread_boost",
        "opponent_team_spread_is_goalside",
        "opponent_team_spread_in_net",
        "opponent_team_spread_dodge_available",
        "opponent_team_spread_demoed",
    ];

    /// The feature vector, ordered exactly as [`Self::FEATURE_NAMES`].
    pub fn to_array(&self) -> [f32; THREAT_FEATURE_COUNT] {
        let mut values = [0.0; THREAT_FEATURE_COUNT];
        values[..8].copy_from_slice(&[
            self.ball_forward_y,
            self.ball_dist_to_goal,
            self.ball_height,
            self.ball_speed,
            self.ball_speed_toward_goal,
            self.goal_open_angle,
            self.on_target,
            self.time_to_goal_line,
        ]);
        values[8..8 + TEAM_THREAT_FEATURE_COUNT].copy_from_slice(&self.own_team.to_array());
        values[8 + TEAM_THREAT_FEATURE_COUNT..].copy_from_slice(&self.opponent_team.to_array());
        values
    }
}

/// Rotate a world-frame vector into the attacking frame for the given team:
/// team zero attacks +Y already; team one's world is rotated 180 degrees
/// about the z axis so it also attacks +Y.
fn attacking_frame(vector: glam::Vec3, attacking_team_is_team_0: bool) -> glam::Vec3 {
    if attacking_team_is_team_0 {
        vector
    } else {
        glam::Vec3::new(-vector.x, -vector.y, vector.z)
    }
}

/// Seconds until a ballistic trajectory reaches the goal-line plane
/// `y = 5120` in the attacking frame, or `None` when it never does. The
/// horizontal path is straight; only z is under gravity, so the crossing time
/// is the straight-line y solution.
fn seconds_to_goal_plane(position: glam::Vec3, velocity: glam::Vec3) -> Option<f32> {
    if velocity.y <= f32::EPSILON {
        return None;
    }
    let time = (STANDARD_GOAL_LINE_Y - position.y) / velocity.y;
    (time.is_finite() && time >= 0.0).then_some(time)
}

/// Whether the ballistic trajectory (gravity on z only) crosses the goal
/// plane inside the mouth (`|x| < 893`, `0 < z < 643`) within
/// [`ON_TARGET_MAX_SECONDS`].
fn ballistic_on_target(position: glam::Vec3, velocity: glam::Vec3) -> bool {
    let Some(time) = seconds_to_goal_plane(position, velocity) else {
        return false;
    };
    if time > ON_TARGET_MAX_SECONDS {
        return false;
    }
    let crossing_x = position.x + velocity.x * time;
    let crossing_z = position.z + velocity.z * time + 0.5 * STANDARD_BALL_GRAVITY_Z * time * time;
    crossing_x.abs() < STANDARD_GOAL_MOUTH_HALF_WIDTH_X
        && crossing_z > 0.0
        && crossing_z < STANDARD_GOAL_MOUTH_HEIGHT_Z
}

/// Horizontal angle subtended by the goal mouth from the ball, in radians.
/// The angle between the xy-plane rays from the ball to each post.
fn goal_open_angle_radians(position: glam::Vec3) -> f32 {
    let to_post = |post_x: f32| {
        glam::Vec2::new(post_x - position.x, STANDARD_GOAL_LINE_Y - position.y).normalize_or_zero()
    };
    let left = to_post(-STANDARD_GOAL_MOUTH_HALF_WIDTH_X);
    let right = to_post(STANDARD_GOAL_MOUTH_HALF_WIDTH_X);
    left.dot(right).clamp(-1.0, 1.0).acos()
}

fn normalized_distance(distance: f32, norm: f32) -> f32 {
    (distance / norm).clamp(0.0, 1.0)
}

/// Compute one team's [`ThreatFeatures`] from raw world-frame ball and player
/// samples. Pure function of its inputs; the export tooling calls this same
/// path through the calculator, so training rows and inference can never
/// diverge.
pub fn compute_threat_features(
    ball_position: glam::Vec3,
    ball_velocity: glam::Vec3,
    players: &PlayerFrameState,
    demoed_players: &HashSet<PlayerId>,
    dodge_available: &HashMap<PlayerId, bool>,
    attacking_team_is_team_0: bool,
) -> Option<ThreatFeatures> {
    let ball = attacking_frame(ball_position, attacking_team_is_team_0);
    let ball_vel = attacking_frame(ball_velocity, attacking_team_is_team_0);
    let goal_center = glam::Vec3::new(0.0, STANDARD_GOAL_LINE_Y, GOAL_CENTER_Z);
    let to_goal = goal_center - ball;
    let ball_dist_to_goal = to_goal.length();
    let ball_speed_toward_goal = ball_vel.dot(to_goal.normalize_or_zero());

    let team = |same_team: bool| {
        let team = players
            .players
            .iter()
            .filter(|player| (player.is_team_0 == attacking_team_is_team_0) == same_team)
            .collect::<Vec<_>>();
        (team.len() == 2).then_some(team)
    };
    let own_team = team(true)?;
    let opponent_team = team(false)?;

    let player_features = |player: &PlayerSample| {
        let demoed = demoed_players.contains(&player.player_id);
        let position = player
            .position()
            .map(|value| attacking_frame(value, attacking_team_is_team_0));
        let velocity = player
            .velocity()
            .map(|value| attacking_frame(value, attacking_team_is_team_0))
            .unwrap_or(glam::Vec3::ZERO);
        let forward = player
            .rigid_body
            .as_ref()
            .map(|body| quat_to_glam(&body.rotation) * glam::Vec3::X)
            .map(|value| attacking_frame(value, attacking_team_is_team_0))
            .unwrap_or(glam::Vec3::ZERO);
        let distance_to_ball = position
            .map(|value| normalized_distance((value - ball).length(), PLAYER_DISTANCE_NORM))
            .unwrap_or(1.0);
        let distance_to_goal = position
            .map(|value| normalized_distance((value - goal_center).length(), GOAL_DISTANCE_NORM))
            .unwrap_or(1.0);
        let position = position.unwrap_or(glam::Vec3::ZERO);
        let in_net = !demoed
            && position.y >= STANDARD_GOAL_LINE_Y - NET_REGION_DEPTH_Y
            && position.x.abs() <= STANDARD_GOAL_MOUTH_HALF_WIDTH_X + NET_REGION_MARGIN
            && position.z <= STANDARD_GOAL_MOUTH_HEIGHT_Z + NET_REGION_MARGIN;

        PlayerThreatFeatures {
            position_x: (position.x / 4096.0).clamp(-1.0, 1.0),
            position_y: (position.y / STANDARD_GOAL_LINE_Y).clamp(-1.0, 1.0),
            position_z: (position.z / SOCCAR_CEILING_Z).clamp(0.0, 1.0),
            velocity_x: (velocity.x / CAR_MAX_SPEED).clamp(-1.0, 1.0),
            velocity_y: (velocity.y / CAR_MAX_SPEED).clamp(-1.0, 1.0),
            velocity_z: (velocity.z / CAR_MAX_SPEED).clamp(-1.0, 1.0),
            forward_x: forward.x.clamp(-1.0, 1.0),
            forward_y: forward.y.clamp(-1.0, 1.0),
            forward_z: forward.z.clamp(-1.0, 1.0),
            distance_to_ball,
            distance_to_goal,
            boost: (player.boost_amount.unwrap_or(0.0) / BOOST_MAX_AMOUNT).clamp(0.0, 1.0),
            is_goalside: f32::from(u8::from(!demoed && position.y >= ball.y)),
            in_net: f32::from(u8::from(in_net)),
            dodge_available: f32::from(u8::from(
                dodge_available
                    .get(&player.player_id)
                    .copied()
                    .unwrap_or(false),
            )),
            demoed: f32::from(u8::from(demoed)),
        }
    };

    Some(ThreatFeatures {
        ball_forward_y: (ball.y / STANDARD_GOAL_LINE_Y).clamp(-1.0, 1.0),
        ball_dist_to_goal: normalized_distance(ball_dist_to_goal, GOAL_DISTANCE_NORM),
        ball_height: (ball.z / SOCCAR_CEILING_Z).clamp(0.0, 1.0),
        ball_speed: (ball_vel.length() / STANDARD_BALL_MAX_SPEED).clamp(0.0, 1.0),
        ball_speed_toward_goal: (ball_speed_toward_goal / STANDARD_BALL_MAX_SPEED).clamp(-1.0, 1.0),
        goal_open_angle: (goal_open_angle_radians(ball) / std::f32::consts::PI).clamp(0.0, 1.0),
        on_target: f32::from(u8::from(ballistic_on_target(ball, ball_vel))),
        time_to_goal_line: seconds_to_goal_plane(ball, ball_vel)
            .map(|time| 1.0 / (1.0 + time))
            .unwrap_or(0.0),
        own_team: TeamThreatFeatures::from_players(
            player_features(own_team[0]),
            player_features(own_team[1]),
        ),
        opponent_team: TeamThreatFeatures::from_players(
            player_features(opponent_team[0]),
            player_features(opponent_team[1]),
        ),
    })
}

/// The detection-frame change in the touching team's V for one touch.
///
/// A touch recovered from the candidate cache can be *backdated*: its contact
/// moment precedes the frame the stats pipeline detected it on. Fields
/// therefore split into two groups:
///
/// - Contact-time fields, copied from the underlying [`TouchEvent`]: `time`,
///   `frame`, and `touch_id` (the touch's stable join key, when assigned).
/// - Detection-time fields: `detection_frame` / `detection_time` are the live
///   processing frame the touch surfaced on. `value_before` is the toucher's
///   team's V on the live frame *before* detection and `value_after` is the V
///   on the detection frame itself. The ΔV bracketing deliberately anchors on
///   detection rather than contact: V is only evaluated on processed live
///   frames, and the detection frame is the first frame whose ball/player
///   state reflects the touch. Consequently, [`Self::delta`] is an observed
///   one-frame state-value change, not a causal estimate of the touch's full
///   multi-frame impulse.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ThreatTouchEvent {
    /// Contact time of the underlying touch (can precede `detection_time`).
    pub time: f32,
    /// Contact frame of the underlying touch (can precede `detection_frame`).
    pub frame: usize,
    /// Stable id of the underlying attributed touch, `None` when the touch
    /// pipeline had not assigned one.
    pub touch_id: Option<u64>,
    /// Frame the touch was detected on; `value_before`/`value_after` bracket
    /// this frame.
    pub detection_frame: usize,
    /// Time of the detection frame.
    pub detection_time: f32,
    pub team_is_team_0: bool,
    pub player: Option<PlayerId>,
    pub value_before: f32,
    pub value_after: f32,
}

impl ThreatTouchEvent {
    pub fn delta(&self) -> f32 {
        self.value_after - self.value_before
    }
}

/// Why a threat episode closed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ThreatEpisodeEndReason {
    /// V dropped back to/under the threshold during live play.
    ValueDropped,
    /// The attacking team scored while the episode was open (or during the
    /// post-close goal grace).
    Goal,
    /// Live play ended (goal replay, other stoppage, or missing ball data)
    /// and no goal for the attacking team followed.
    Stoppage,
    ReplayEnd,
}

/// A contiguous span where one team's V exceeded the episode threshold.
/// `credited_player` is the attacking team's most recent toucher when the
/// episode reached `peak_value`, `None` when the team had not touched during
/// the live stretch by that point -- team-only credit. Later touches at lower
/// V do not take credit for threat accumulated before them.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ThreatEpisodeEvent {
    pub start_time: f32,
    pub start_frame: usize,
    pub end_time: f32,
    pub end_frame: usize,
    pub team_is_team_0: bool,
    /// The episode's xG: the time integral `sum(V * dt) / tau` over the
    /// span, where `tau` is
    /// [`THREAT_HORIZON_SECONDS`](super::expected_goals_model::THREAT_HORIZON_SECONDS).
    /// Frames that count: every evaluated live-play frame from the frame that
    /// opens the episode through the frame that closes it (for value-drop
    /// closes the final sub-threshold frame contributes too; stoppage /
    /// replay-end closes end at the last evaluated live frame). This is the
    /// calibrated goal-scale estimator -- summing episode peaks over-counts
    /// goals ~2.7x.
    pub xg: f32,
    /// Peak V over the span (the pre-calibration `xg`), kept for
    /// display/intensity ranking.
    pub peak_value: f32,
    pub credited_player: Option<PlayerId>,
    pub ended_in_goal: bool,
    pub end_reason: ThreatEpisodeEndReason,
}

/// A goal observed while processing (from replicated goal events, with a
/// scoreboard-increment fallback), kept for episode outcomes and dataset
/// labeling.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ThreatGoalRecord {
    pub time: f32,
    pub frame: usize,
    pub scoring_team_is_team_0: bool,
}

/// Configuration for [`ExpectedGoalsCalculator`].
#[derive(Debug, Clone, PartialEq)]
pub struct ExpectedGoalsCalculatorConfig {
    pub episode_threshold: f32,
}

impl Default for ExpectedGoalsCalculatorConfig {
    fn default() -> Self {
        Self {
            episode_threshold: THREAT_EPISODE_THRESHOLD,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct ActiveThreatEpisode {
    start_time: f32,
    start_frame: usize,
    peak_value: f32,
    /// Running `sum(V * dt) / tau` over the episode's evaluated live frames.
    xg_integral: f64,
    /// Most recent attacking toucher when `peak_value` was established.
    credited_player: Option<PlayerId>,
}

/// A stoppage-closed episode held until its goal outcome is known (a goal for
/// the team resolves it as a goal episode; the next kickoff phase or the goal
/// grace expiring resolves it as a plain stoppage).
#[derive(Debug, Clone, PartialEq)]
struct PendingThreatEpisode {
    event: ThreatEpisodeEvent,
    closed_at: f32,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct TeamThreatState {
    active_episode: Option<ActiveThreatEpisode>,
    pending_episode: Option<PendingThreatEpisode>,
    /// Most recent toucher on this team within the current live stretch.
    last_toucher: Option<PlayerId>,
}

/// Evaluates the continuous threat value for both teams each live-play frame
/// and derives touch threat deltas and threat episodes.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ExpectedGoalsCalculator {
    config: ExpectedGoalsCalculatorConfig,
    touch_events: EventStream<ThreatTouchEvent>,
    episode_events: EventStream<ThreatEpisodeEvent>,
    goal_records: Vec<ThreatGoalRecord>,
    team_states: [TeamThreatState; 2],
    /// Per-team full-match `sum(V * dt) / tau`, accumulated over EVERY
    /// evaluated live-play frame (sub-threshold frames included), indexed
    /// `[team zero, team one]`. This is the calibrated team xG.
    team_xg_integrals: [f64; 2],
    /// Both teams' V on the previous live frame, if it was live.
    previous_values: Option<[f32; 2]>,
    last_score: Option<(i32, i32)>,
    last_frame: Option<(usize, f32)>,
    was_live: bool,
}

fn team_index(is_team_0: bool) -> usize {
    usize::from(!is_team_0)
}

impl ExpectedGoalsCalculator {
    pub fn new() -> Self {
        Self::with_config(ExpectedGoalsCalculatorConfig::default())
    }

    pub fn with_config(config: ExpectedGoalsCalculatorConfig) -> Self {
        Self {
            config,
            ..Self::default()
        }
    }

    pub fn config(&self) -> &ExpectedGoalsCalculatorConfig {
        &self.config
    }

    pub fn touch_events(&self) -> &[ThreatTouchEvent] {
        self.touch_events.all()
    }

    pub fn new_touch_events(&self) -> &[ThreatTouchEvent] {
        self.touch_events.new_events()
    }

    pub fn episode_events(&self) -> &[ThreatEpisodeEvent] {
        self.episode_events.all()
    }

    pub fn new_episode_events(&self) -> &[ThreatEpisodeEvent] {
        self.episode_events.new_events()
    }

    pub fn goal_records(&self) -> &[ThreatGoalRecord] {
        &self.goal_records
    }

    /// Per-team full-match `sum(V * dt) / tau` over every evaluated live-play
    /// frame (`[team zero, team one]`). Corpus-calibrated to actual goals per
    /// team-game within ~1%; this is the team's accumulated xG. Episodes
    /// capture only the above-threshold portion of it (empirically ~62%).
    pub fn team_xg_integrals(&self) -> [f64; 2] {
        self.team_xg_integrals
    }

    /// Both teams' V on the most recent live frame (`[team zero, team one]`),
    /// `None` outside live play.
    pub fn current_values(&self) -> Option<[f32; 2]> {
        self.previous_values
    }

    /// Time of the last processed frame (any phase), for downstream
    /// censoring: seconds-to-replay-end labels.
    pub fn last_frame_time(&self) -> Option<f32> {
        self.last_frame.map(|(_, time)| time)
    }

    fn record_goal(&mut self, frame: &FrameInfo, time: f32, scoring_team_is_team_0: bool) {
        let duplicate = self.goal_records.iter().any(|record| {
            record.scoring_team_is_team_0 == scoring_team_is_team_0
                && (time - record.time).abs() <= GOAL_RECORD_DEDUPE_SECONDS
        });
        if duplicate {
            return;
        }
        self.goal_records.push(ThreatGoalRecord {
            time,
            frame: frame.frame_number,
            scoring_team_is_team_0,
        });
        self.close_episode_as_goal(frame, time, scoring_team_is_team_0);
    }

    fn close_episode_as_goal(
        &mut self,
        frame: &FrameInfo,
        time: f32,
        scoring_team_is_team_0: bool,
    ) {
        let state = &mut self.team_states[team_index(scoring_team_is_team_0)];
        if let Some(active) = state.active_episode.take() {
            let event = Self::event_from_active(
                &active,
                frame.frame_number,
                frame.time,
                scoring_team_is_team_0,
                true,
                ThreatEpisodeEndReason::Goal,
            );
            self.episode_events.push(event);
            return;
        }
        if let Some(pending) = state.pending_episode.take() {
            let mut event = pending.event;
            // Enforce the goal grace inside attribution too: goal detection
            // runs before stale-pending expiry within a frame, so without
            // this bound a goal arriving long after the stoppage (a later,
            // unrelated attack on a quiet scoreboard) would still upgrade the
            // stale episode. A goal past the grace resolves it as the plain
            // stoppage it already was.
            if time - pending.closed_at <= PENDING_EPISODE_GOAL_GRACE_SECONDS {
                event.ended_in_goal = true;
                event.end_reason = ThreatEpisodeEndReason::Goal;
            }
            self.episode_events.push(event);
        }
    }

    fn event_from_active(
        active: &ActiveThreatEpisode,
        end_frame: usize,
        end_time: f32,
        team_is_team_0: bool,
        ended_in_goal: bool,
        end_reason: ThreatEpisodeEndReason,
    ) -> ThreatEpisodeEvent {
        ThreatEpisodeEvent {
            start_time: active.start_time,
            start_frame: active.start_frame,
            end_time,
            end_frame,
            team_is_team_0,
            xg: active.xg_integral as f32,
            peak_value: active.peak_value,
            credited_player: active.credited_player.clone(),
            ended_in_goal,
            end_reason,
        }
    }

    /// One frame's contribution to an xG time integral: `V * dt / tau`.
    fn integral_contribution(value: f32, dt: f32) -> f64 {
        f64::from(value) * f64::from(dt) / f64::from(expected_goals_model::THREAT_HORIZON_SECONDS)
    }

    fn detect_goals(
        &mut self,
        frame: &FrameInfo,
        gameplay: &GameplayState,
        events: &FrameEventsState,
    ) {
        for goal in events.goal_events.clone() {
            self.record_goal(frame, goal.time, goal.scoring_team_is_team_0);
        }
        // Scoreboard fallback: some replays attribute goals only through the
        // team score, mirroring the live-play tracker's score-change path.
        if let (Some((team_zero, team_one)), Some((last_zero, last_one))) =
            (gameplay.current_score(), self.last_score)
        {
            if team_zero > last_zero {
                self.record_goal(frame, frame.time, true);
            }
            if team_one > last_one {
                self.record_goal(frame, frame.time, false);
            }
        }
        if let Some(score) = gameplay.current_score() {
            self.last_score = Some(score);
        }
    }

    /// Resolve stoppage-closed episodes whose goal grace has passed (or once
    /// the next kickoff phase begins) as plain stoppages.
    fn resolve_stale_pending_episodes(&mut self, frame: &FrameInfo, kickoff_phase_active: bool) {
        for team_index in 0..2 {
            let expired = self.team_states[team_index]
                .pending_episode
                .as_ref()
                .is_some_and(|pending| {
                    kickoff_phase_active
                        || frame.time - pending.closed_at > PENDING_EPISODE_GOAL_GRACE_SECONDS
                });
            if expired {
                let pending = self.team_states[team_index]
                    .pending_episode
                    .take()
                    .expect("pending episode exists when expired");
                self.episode_events.push(pending.event);
            }
        }
    }

    /// Close any active episodes because live play ended. The closed episode
    /// is held pending: the goal that caused the stoppage may only be
    /// attributed a few frames later.
    fn suspend_active_episodes(&mut self, frame: &FrameInfo) {
        for (team_index, state) in self.team_states.iter_mut().enumerate() {
            let Some(active) = state.active_episode.take() else {
                continue;
            };
            let event = Self::event_from_active(
                &active,
                frame.frame_number,
                frame.time,
                team_index == 0,
                false,
                ThreatEpisodeEndReason::Stoppage,
            );
            // A newer stoppage-closed episode supersedes an unresolved older
            // one; flush the older one un-goaled first.
            if let Some(previous) = state.pending_episode.take() {
                self.episode_events.push(previous.event);
            }
            state.pending_episode = Some(PendingThreatEpisode {
                event,
                closed_at: frame.time,
            });
        }
    }

    /// Emit at most one [`ThreatTouchEvent`] per team per frame.
    ///
    /// [`TouchState`] can report several simultaneous contacts on one frame
    /// (contested 50/50s, same-team double commits), but the change from the
    /// previous live frame's V to this frame's V is a single transition:
    /// crediting it to every same-team touch would count it once per toucher
    /// in the accumulator. The team's *primary* touch -- the latest,
    /// best-evidence contact per
    /// [`TouchState::primary_touch_event_for_team`], the same notion of "the"
    /// decisive touch that `TouchState` already encodes for the rest of the
    /// stats pipeline -- receives the whole transition.
    fn emit_touch_events(&mut self, frame: &FrameInfo, touch_state: &TouchState, values: [f32; 2]) {
        for is_team_0 in [true, false] {
            let Some(touch) = touch_state.primary_touch_event_for_team(is_team_0) else {
                continue;
            };
            let index = team_index(is_team_0);
            let value_before = self
                .previous_values
                .map(|previous| previous[index])
                .unwrap_or(values[index]);
            self.touch_events.push(ThreatTouchEvent {
                time: touch.time,
                frame: touch.frame,
                touch_id: touch.touch_id,
                detection_frame: frame.frame_number,
                detection_time: frame.time,
                team_is_team_0: is_team_0,
                player: touch.player.clone(),
                value_before,
                value_after: values[index],
            });
            if let Some(player) = touch.player.clone() {
                let state = &mut self.team_states[index];
                state.last_toucher = Some(player);
            }
        }
    }

    /// Track episode state for one evaluated live frame. Episodes integrate
    /// `V * dt / tau` over exactly these frames -- the same live-play frames
    /// where V is evaluated -- from the frame that opens the episode through
    /// the frame that closes it (a value-drop close's final sub-threshold
    /// frame contributes too; stoppage/replay-end closes end at the last
    /// evaluated live frame, since non-live frames are never evaluated).
    fn update_episodes(&mut self, frame: &FrameInfo, values: [f32; 2]) {
        for (team_index, state) in self.team_states.iter_mut().enumerate() {
            let value = values[team_index];
            let last_toucher = state.last_toucher.clone();
            match state.active_episode.as_mut() {
                Some(active) => {
                    if value > active.peak_value {
                        active.peak_value = value;
                        active.credited_player = last_toucher;
                    }
                    active.xg_integral += Self::integral_contribution(value, frame.dt);
                    if value <= self.config.episode_threshold {
                        let active = state
                            .active_episode
                            .take()
                            .expect("active episode exists when closing");
                        self.episode_events.push(Self::event_from_active(
                            &active,
                            frame.frame_number,
                            frame.time,
                            team_index == 0,
                            false,
                            ThreatEpisodeEndReason::ValueDropped,
                        ));
                    }
                }
                None => {
                    if value > self.config.episode_threshold {
                        state.active_episode = Some(ActiveThreatEpisode {
                            start_time: frame.time,
                            start_frame: frame.frame_number,
                            peak_value: value,
                            xg_integral: Self::integral_contribution(value, frame.dt),
                            credited_player: last_toucher,
                        });
                    }
                }
            }
        }
    }

    pub fn update_parts(
        &mut self,
        frame: &FrameInfo,
        gameplay: &GameplayState,
        events: &FrameEventsState,
        touch_state: &TouchState,
        threat_features: &ThreatFeaturesState,
    ) -> SubtrActorResult<()> {
        self.touch_events.begin_update();
        self.episode_events.begin_update();
        self.last_frame = Some((frame.frame_number, frame.time));

        self.detect_goals(frame, gameplay, events);
        self.resolve_stale_pending_episodes(frame, gameplay.kickoff_phase_active());

        let Some(features) = threat_features.current().copied() else {
            self.suspend_active_episodes(frame);
            if self.was_live {
                for state in self.team_states.iter_mut() {
                    state.last_toucher = None;
                }
            }
            self.previous_values = None;
            self.was_live = false;
            return Ok(());
        };
        let values = [
            expected_goals_model::threat_value(&features[0]),
            expected_goals_model::threat_value(&features[1]),
        ];

        // The full-match team integral covers EVERY evaluated live frame,
        // sub-threshold ones included -- diffuse below-threshold threat is
        // ~38% of the calibrated total.
        for (team_index, value) in values.iter().enumerate() {
            self.team_xg_integrals[team_index] += Self::integral_contribution(*value, frame.dt);
        }

        self.emit_touch_events(frame, touch_state, values);
        self.update_episodes(frame, values);
        self.previous_values = Some(values);
        self.was_live = true;
        Ok(())
    }

    pub fn finish_calculation(&mut self) -> SubtrActorResult<()> {
        self.touch_events.begin_update();
        self.episode_events.begin_update();
        let (end_frame, end_time) = self.last_frame.unwrap_or((0, 0.0));
        for (team_index, state) in self.team_states.iter_mut().enumerate() {
            if let Some(active) = state.active_episode.take() {
                let event = Self::event_from_active(
                    &active,
                    end_frame,
                    end_time,
                    team_index == 0,
                    false,
                    ThreatEpisodeEndReason::ReplayEnd,
                );
                self.episode_events.push(event);
            }
            if let Some(pending) = state.pending_episode.take() {
                self.episode_events.push(pending.event);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "expected_goals_tests.rs"]
mod tests;
