#![cfg_attr(target_arch = "wasm32", allow(dead_code))]

use serde::Serialize;

#[cfg(not(target_arch = "wasm32"))]
use linkme::distributed_slice;

use super::{
    BackboardBounceEvent, BallCarryEvent, BoostLedgerEvent, BoostPickupComparisonEvent,
    BoostStateEvent, BumpEvent, CeilingShotEvent, CenterEvent, ConfirmedFlipResetEvent,
    CorePlayerGoalContextEvent, CorePlayerScoreboardEvent, DodgeRefreshedEvent, DodgeResetEvent,
    DoubleTapEvent, FiftyFiftyEvent, FlickEvent, FlipResetEvent, FlipResetFollowupDodgeEvent,
    GoalContextEvent, HalfFlipEvent, HalfVolleyEvent, MovementEvent, MustyFlickEvent,
    OneTimerEvent, PassEvent, PassLastCompletedEvent, PositioningEvent, PossessionEvent,
    PostWallDodgeEvent, PowerslideEvent, PressureEvent, RotationDepthSpanEvent,
    RotationFirstManStintEvent, RotationPlayerEvent, RotationRoleSpanEvent, RotationTeamEvent,
    RushEvent, SpeedFlipEvent, TerritorialPressureEvent, TimelineEvent, TouchBallMovementEvent,
    TouchClassificationEvent, TouchLastTouchEvent, WallAerialEvent, WallAerialShotEvent,
    WavedashEvent, WhiffEvent,
};
use crate::stats::timeline::StatsTimelineTagEvent;

/// Static, English-language metadata for a stat event type.
///
/// Event structs own this definition through [`StatsEvent`]. Analysis nodes
/// then link event definitions to the calculator code that produces them via
/// [`EmittedEvent`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct EventDefinition {
    pub id: &'static str,
    pub label: &'static str,
    pub category: EventCategory,
    pub confidence: DetectionConfidence,
    pub summary: &'static str,
    pub approach: &'static [&'static str],
    pub limitations: &'static [&'static str],
}

/// Coarse product/domain grouping for an event definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EventCategory {
    Core,
    Mechanic,
    GoalContext,
    Possession,
    Positioning,
    Boost,
    Contact,
    Movement,
}

/// Multi-dimensional confidence metadata for an event detector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct DetectionConfidence {
    pub approach: ApproachConfidenceLevel,
    pub true_positive_evidence: TruePositiveEvidenceLevel,
    pub false_positive_evidence: DetectionIssueEvidenceLevel,
    pub false_negative_evidence: DetectionIssueEvidenceLevel,
    pub testing: TestingThoroughnessLevel,
    pub known_issues: &'static [KnownIssueRef],
}

/// How plausible and stable the current detector approach is by design.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ApproachConfidenceLevel {
    Unknown,
    High,
    Medium,
    Low,
    Experimental,
}

/// Whether the detector is known to produce correct detections.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TruePositiveEvidenceLevel {
    NotEvaluated,
    Plausible,
    ManuallyConfirmed,
    AutomatedTestCovered,
    RepeatedlyConfirmed,
}

/// Whether the detector is known to produce incorrect detections or misses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DetectionIssueEvidenceLevel {
    NotEvaluated,
    NoneKnown,
    Suspected,
    Observed,
}

/// Rough level of testing behind the detector definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TestingThoroughnessLevel {
    Untested,
    ManualSpotCheck,
    TargetedAutomatedTest,
    MultipleTargetedTests,
    CuratedSuite,
    CorpusSample,
}

/// Lightweight reference to a known detector issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct KnownIssueRef {
    pub id: &'static str,
    pub summary: &'static str,
    pub url: Option<&'static str>,
}

pub const UNKNOWN_DETECTION_CONFIDENCE: DetectionConfidence = DetectionConfidence {
    approach: ApproachConfidenceLevel::Unknown,
    true_positive_evidence: TruePositiveEvidenceLevel::NotEvaluated,
    false_positive_evidence: DetectionIssueEvidenceLevel::NotEvaluated,
    false_negative_evidence: DetectionIssueEvidenceLevel::NotEvaluated,
    testing: TestingThoroughnessLevel::Untested,
    known_issues: &[],
};

pub const fn pending_event_definition(
    id: &'static str,
    label: &'static str,
    category: EventCategory,
) -> EventDefinition {
    event_definition(id, label, category, "Definition pending.", &[])
}

pub const fn event_definition(
    id: &'static str,
    label: &'static str,
    category: EventCategory,
    summary: &'static str,
    approach: &'static [&'static str],
) -> EventDefinition {
    EventDefinition {
        id,
        label,
        category,
        confidence: UNKNOWN_DETECTION_CONFIDENCE,
        summary,
        approach,
        limitations: &[],
    }
}

pub const fn produced_event(
    event: &'static EventDefinition,
    node_name: &'static str,
    node_type: &'static str,
    calculator_type: &'static str,
) -> EmittedEvent {
    EmittedEvent {
        event,
        producer: ProducerDefinition {
            node_name,
            node_type,
            calculator_type,
            implementation_notes: &[],
        },
    }
}

pub const fn produced_event_for<E: StatsEvent>(
    node_name: &'static str,
    node_type: &'static str,
    calculator_type: &'static str,
) -> EmittedEvent {
    produced_event(&E::DEFINITION, node_name, node_type, calculator_type)
}

/// Trait implemented by typed stat event payloads.
pub trait StatsEvent {
    const DEFINITION: EventDefinition;
}

/// Static metadata for the analysis node and calculator that produce an event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ProducerDefinition {
    pub node_name: &'static str,
    pub node_type: &'static str,
    pub calculator_type: &'static str,
    pub implementation_notes: &'static [&'static str],
}

/// Link between an event definition and the graph node that emits it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct EmittedEvent {
    pub event: &'static EventDefinition,
    pub producer: ProducerDefinition,
}

/// Static registration for the events emitted by one analysis node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct EventProducerDefinition {
    pub node_name: &'static str,
    pub emitted_events: &'static [EmittedEvent],
}

/// Distributed static catalog of event producers.
///
/// Each analysis node can register its own emitted events next to the node or
/// calculator implementation. Documentation generation should walk this slice
/// instead of maintaining a second central catalog.
#[cfg(not(target_arch = "wasm32"))]
#[distributed_slice]
pub static EVENT_PRODUCERS: [EventProducerDefinition];

#[cfg(not(target_arch = "wasm32"))]
pub fn event_producers() -> &'static [EventProducerDefinition] {
    &EVENT_PRODUCERS
}

/// `linkme` does not support `wasm32-unknown-unknown`, so the static
/// documentation registry is available on host/tooling builds only for now.
#[cfg(target_arch = "wasm32")]
pub fn event_producers() -> &'static [EventProducerDefinition] {
    &[]
}

macro_rules! define_stats_event {
    (
        $event_type:ty,
        $definition:ident,
        $id:literal,
        $label:literal,
        $category:expr,
        summary = $summary:literal,
        approach = [$($approach:literal),* $(,)?]
    ) => {
        pub const $definition: EventDefinition =
            event_definition($id, $label, $category, $summary, &[$($approach),*]);

        impl StatsEvent for $event_type {
            const DEFINITION: EventDefinition = $definition;
        }
    };

    ($event_type:ty, $definition:ident, $id:literal, $label:literal, $category:expr) => {
        pub const $definition: EventDefinition = pending_event_definition($id, $label, $category);

        impl StatsEvent for $event_type {
            const DEFINITION: EventDefinition = $definition;
        }
    };
}

macro_rules! register_event_producer {
    ($static_name:ident, $node_name:literal, $emitted_events:expr) => {
        #[cfg(not(target_arch = "wasm32"))]
        #[distributed_slice(EVENT_PRODUCERS)]
        static $static_name: EventProducerDefinition = EventProducerDefinition {
            node_name: $node_name,
            emitted_events: $emitted_events,
        };
    };
}

define_stats_event!(
    TimelineEvent,
    TIMELINE_EVENT_DEFINITION,
    "timeline",
    "Replay Timeline Event",
    EventCategory::Core
);
define_stats_event!(
    CorePlayerScoreboardEvent,
    CORE_PLAYER_SCOREBOARD_EVENT_DEFINITION,
    "core_player_scoreboard",
    "Core Player Scoreboard",
    EventCategory::Core
);
define_stats_event!(
    CorePlayerGoalContextEvent,
    CORE_PLAYER_GOAL_CONTEXT_EVENT_DEFINITION,
    "core_player_goal_context",
    "Core Player Goal Context",
    EventCategory::GoalContext
);
define_stats_event!(
    GoalContextEvent,
    GOAL_CONTEXT_EVENT_DEFINITION,
    "goal_context",
    "Goal Context",
    EventCategory::GoalContext
);
define_stats_event!(
    BackboardBounceEvent,
    BACKBOARD_BOUNCE_EVENT_DEFINITION,
    "backboard_bounce",
    "Backboard Bounce",
    EventCategory::Mechanic,
    summary = "A ball rebound off the opponent backboard attributed to the player who sent the ball there.",
    approach = [
        "Track the last touch during live play and attribute a later backboard rebound to that touch when it occurs within the configured attribution window.",
        "Require the ball to be high, near the backboard face, moving toward the backboard before the rebound, and moving away after the rebound.",
        "Ignore frames with a simultaneous touch so the rebound is not confused with a player-ball contact.",
    ]
);
define_stats_event!(
    CeilingShotEvent,
    CEILING_SHOT_EVENT_DEFINITION,
    "ceiling_shot",
    "Ceiling Shot",
    EventCategory::Mechanic,
    summary = "A shot touch shortly after the player contacts the ceiling and drops back toward the ball.",
    approach = [
        "Record recent ceiling contacts when the car is near the ceiling and oriented roof-first against it.",
        "Match a later touch by the same player within the ceiling-contact window after the player has separated from the ceiling.",
        "Score the candidate from contact timing, height, separation, forward alignment, approach speed, ball impulse, and ceiling-contact alignment.",
    ]
);
define_stats_event!(
    WallAerialEvent,
    WALL_AERIAL_EVENT_DEFINITION,
    "wall_aerial",
    "Wall Aerial",
    EventCategory::Mechanic,
    summary = "An aerial play that starts from controlled ball movement on a side or back wall.",
    approach = [
        "Track wall-control sequences where the last toucher keeps the ball close while positioned on a side or back wall.",
        "Arm a wall-aerial candidate when the player leaves the wall soon after a qualifying wall-control setup.",
        "Emit on a later aerial touch by the same player when the player and ball are high enough, the setup/takeoff windows hold, and the confidence score clears the threshold.",
    ]
);
define_stats_event!(
    WallAerialShotEvent,
    WALL_AERIAL_SHOT_EVENT_DEFINITION,
    "wall_aerial_shot",
    "Wall Aerial Shot",
    EventCategory::Mechanic,
    summary = "A shot credited to a player shortly after taking off from a wall.",
    approach = [
        "Track recent wall contact for each player and arm a candidate when the player leaves the wall while still above the ground threshold.",
        "Match a subsequent shot stat event by that player within the takeoff-to-shot window.",
        "Require the shot touch to occur off the wall with sufficient player and ball height, then score confidence from timing, height, goal alignment, and ball speed.",
    ]
);
define_stats_event!(
    CenterEvent,
    CENTER_EVENT_DEFINITION,
    "center",
    "Center",
    EventCategory::Mechanic,
    summary = "A touch that moves the ball from a wide attacking position toward the central attacking area.",
    approach = [
        "Start a pending center from a live-play touch, unless that player immediately has a shot or goal event.",
        "Watch the ball for a short window after the touch and require meaningful travel from a wide x-position toward a more central x-position in the attacking half.",
        "Clear the candidate when it ages out, loses attribution, or becomes a shot/goal by the same player instead of a center.",
    ]
);
define_stats_event!(
    FlickEvent,
    FLICK_EVENT_DEFINITION,
    "flick",
    "Flick",
    EventCategory::Mechanic,
    summary = "A dodge-powered touch following a short controlled carry setup.",
    approach = [
        "Track controlled setup windows where the current controlling player keeps the ball close above the car within local-position and gap thresholds.",
        "Record dodge starts that happen immediately after, or during, a qualifying setup.",
        "Emit on a same-player touch shortly after the dodge when the ball impulse is large and directed away from the player, with confidence from setup duration, timing, impulse, and separation.",
    ]
);
define_stats_event!(
    MustyFlickEvent,
    MUSTY_FLICK_EVENT_DEFINITION,
    "musty_flick",
    "Musty Flick",
    EventCategory::Mechanic,
    summary = "A back-flip style flick where the ball is contacted behind/on top of the car during a dominant pitch rotation.",
    approach = [
        "Track dodge starts and keep only recent candidates whose car orientation is compatible with a musty-style setup.",
        "On a same-player touch, require the ball to be behind and above the car in local space, with rear/top alignment and forward approach speed.",
        "Require a meaningful ball speed change and pitch-dominant angular velocity, then score confidence from timing, alignment, approach, pitch, impulse, and setup orientation.",
    ]
);
define_stats_event!(
    DodgeResetEvent,
    DODGE_RESET_EVENT_DEFINITION,
    "dodge_reset",
    "Dodge Reset",
    EventCategory::Mechanic,
    summary = "A frame-level dodge refresh observed from replay state, optionally marked as occurring on the ball.",
    approach = [
        "Consume dodge-refreshed replay events and preserve the player, team, frame, time, and counter value.",
        "Classify the refresh as on-ball when the player and ball are both airborne enough, close together, and the ball is positioned under the car in local space.",
        "Keep on-ball resets pending until the player lands or uses the reset in a later confirmed flip-reset sequence.",
    ]
);
define_stats_event!(
    DodgeRefreshedEvent,
    DODGE_REFRESHED_EVENT_DEFINITION,
    "dodge_refreshed",
    "Dodge Refreshed",
    EventCategory::Mechanic,
    summary = "A raw replay dodge-refresh signal for a player.",
    approach = [
        "Forward the replay's dodge-refreshed event stream with player, team, time, frame, and counter value.",
        "Use this lower-level event as evidence for higher-level reset mechanics, including on-ball dodge resets and confirmed flip resets.",
    ]
);
define_stats_event!(
    ConfirmedFlipResetEvent,
    CONFIRMED_FLIP_RESET_EVENT_DEFINITION,
    "confirmed_flip_reset",
    "Confirmed Flip Reset",
    EventCategory::Mechanic,
    summary = "A flip reset that is confirmed by a later dodge-powered touch after an on-ball dodge refresh.",
    approach = [
        "Start from a pending on-ball dodge reset detected by the dodge reset calculator.",
        "Require the player to start a dodge after that reset and then touch the ball while the dodge is active.",
        "Accept only touches within the configured reset-to-touch window, then clear the pending reset so each reset confirms at most once.",
    ]
);
define_stats_event!(
    DoubleTapEvent,
    DOUBLE_TAP_EVENT_DEFINITION,
    "double_tap",
    "Double Tap",
    EventCategory::Mechanic,
    summary = "A same-player follow-up touch after an attributed backboard bounce that creates a shot-like trajectory.",
    approach = [
        "Arm a pending double tap from a backboard-bounce event attributed to the player who sent the ball to the backboard.",
        "Require the same player and team to touch the ball again during live play within the follow-up window.",
        "Accept the follow-up only when the post-touch straight-line ball trajectory projects into or close to the opponent goal mouth.",
    ]
);
define_stats_event!(
    OneTimerEvent,
    ONE_TIMER_EVENT_DEFINITION,
    "one_timer",
    "One Timer",
    EventCategory::Mechanic,
    summary = "A fast receiver touch from a completed pass that is immediately directed toward goal.",
    approach = [
        "Consume newly completed pass events on the frame they are recorded.",
        "Require the current ball speed after the receiver's touch to exceed the one-timer speed threshold.",
        "Require the post-touch ball velocity to align with the opponent goal center direction.",
    ]
);
define_stats_event!(
    PassEvent,
    PASS_EVENT_DEFINITION,
    "pass",
    "Pass",
    EventCategory::Mechanic,
    summary = "A same-team touch sequence where one player sends the ball to a different teammate.",
    approach = [
        "Track the last attributed touch in live play and compare it to each new touch.",
        "Emit when a different teammate touches the ball within the pass window after the ball has traveled far enough.",
        "Classify the pass as direct, backboard, fifty-fifty, or fifty-fifty backboard using intervening backboard-bounce and fifty-fifty state.",
    ]
);
define_stats_event!(
    PassLastCompletedEvent,
    PASS_LAST_COMPLETED_EVENT_DEFINITION,
    "pass_last_completed",
    "Pass Last Completed",
    EventCategory::Mechanic,
    summary = "A state-change marker for the most recent player to complete a pass reception.",
    approach = [
        "Emit when the pass calculator's last completed receiver changes.",
        "Reset to no player when play is not live or when ball/player attribution is unavailable.",
        "Use this as a compact timeline/state event derived from completed pass detections.",
    ]
);
define_stats_event!(
    BallCarryEvent,
    BALL_CARRY_EVENT_DEFINITION,
    "ball_carry",
    "Ball Carry",
    EventCategory::Mechanic,
    summary = "A sustained player-ball control sequence, covering grounded carries and air dribbles.",
    approach = [
        "Use continuous ball-control tracking to build player-owned sequences while live play is active.",
        "Sample grounded carries from close horizontal/vertical ball gaps over the car, excluding wall contact.",
        "Sample air dribbles with the air-dribble policy, then emit completed sequences that meet the duration and validity rules for their carry kind.",
    ]
);
define_stats_event!(
    FiftyFiftyEvent,
    FIFTY_FIFTY_EVENT_DEFINITION,
    "fifty_fifty",
    "50/50",
    EventCategory::Contact,
    summary = "A contested ball interaction involving touches or pressure from both teams in a short window.",
    approach = [
        "Start an active 50/50 when a frame contains touches from both teams, including kickoff-specific tracking.",
        "Continue the contest for short follow-up touch windows while either involved team remains in contact.",
        "Resolve after a delay once ball movement, possession state, or max duration gives a winner, possession outcome, or neutral result.",
    ]
);
define_stats_event!(
    RushEvent,
    RUSH_EVENT_DEFINITION,
    "rush",
    "Rush",
    EventCategory::Possession,
    summary = "A quick possession transition where the attacking team has numbers moving out of its defensive half.",
    approach = [
        "Start from a possession change when the ball is still in the new attacking team's defensive half.",
        "Count non-demoed attackers near or ahead of the ball and defenders between the ball and their own goal.",
        "Emit once the new attacking team retains possession long enough with at least two attackers and at least one defender in the rush shape.",
    ]
);
define_stats_event!(
    SpeedFlipEvent,
    SPEED_FLIP_EVENT_DEFINITION,
    "speed_flip",
    "Speed Flip",
    EventCategory::Mechanic,
    summary = "A ground-started diagonal dodge/cancel acceleration pattern, primarily intended for kickoff speed flips.",
    approach = [
        "Start candidates on dodge rising edges while the player is grounded, moving in the car's forward direction, and, for kickoff cases, within the kickoff-start window.",
        "Track speed, forward alignment, boost alignment, diagonal angular-velocity balance, and early forward acceleration during a short evaluation window.",
        "Emit when the combined diagonal, cancel, speed, and alignment confidence score clears the speed-flip threshold before the candidate expires.",
    ]
);
define_stats_event!(
    HalfFlipEvent,
    HALF_FLIP_EVENT_DEFINITION,
    "half_flip",
    "Half Flip",
    EventCategory::Mechanic,
    summary = "A dodge sequence that starts while driving backward and reorients the car to move forward.",
    approach = [
        "Start candidates on grounded dodge rising edges when the car is moving backward relative to its facing direction.",
        "Track reorientation during the evaluation window, including forward-vector reversal, alignment with the resulting velocity, and vertical flip evidence.",
        "Emit when the candidate shows enough reversal, reorientation, flip motion, and speed evidence to clear the confidence threshold.",
    ]
);
define_stats_event!(
    HalfVolleyEvent,
    HALF_VOLLEY_EVENT_DEFINITION,
    "half_volley",
    "Half Volley",
    EventCategory::Mechanic,
    summary = "A fast touch shortly after the ball bounces off the floor, paired with a recent player dodge.",
    approach = [
        "Detect floor bounces from ball height and vertical velocity reversal when no touch occurs on the bounce frame.",
        "Track each player's recent ground contact and dodge start.",
        "Emit on a same-player touch shortly after the floor bounce and dodge when the post-touch ball speed clears the configured threshold.",
    ]
);
define_stats_event!(
    WavedashEvent,
    WAVEDASH_EVENT_DEFINITION,
    "wavedash",
    "Wavedash",
    EventCategory::Mechanic,
    summary = "A low airborne dodge that lands quickly and converts the dodge into ground speed.",
    approach = [
        "Start candidates on dodge rising edges from a low but airborne height.",
        "Watch for a landing within the wavedash window while the car is sufficiently upright.",
        "Score confidence from dodge-to-landing timing, starting height, speed gain or landing speed, and landing uprightness.",
    ]
);
define_stats_event!(
    WhiffEvent,
    WHIFF_EVENT_DEFINITION,
    "whiff",
    "Whiff",
    EventCategory::Mechanic,
    summary = "A committed attempt near the ball that does not result in that player touching it.",
    approach = [
        "Start candidates when a player gets within hitbox distance of the ball while moving or dodging toward it with sufficient alignment and closing speed.",
        "Track the closest approach while the candidate remains near the ball.",
        "Resolve as a whiff when the player exits the candidate window without touching, or as beaten-to-ball when an opponent touches first.",
    ]
);
define_stats_event!(
    PowerslideEvent,
    POWERSLIDE_EVENT_DEFINITION,
    "powerslide",
    "Powerslide",
    EventCategory::Movement,
    summary = "A state-change event for effective grounded powerslide use.",
    approach = [
        "Read each player's powerslide-active input/state on every frame.",
        "Treat powerslide as effective only while the player is close enough to the ground.",
        "Emit when a player's effective powerslide state changes between active and inactive.",
    ]
);
define_stats_event!(
    TouchClassificationEvent,
    TOUCH_CLASSIFICATION_EVENT_DEFINITION,
    "touch",
    "Touch",
    EventCategory::Contact
);
define_stats_event!(
    TouchBallMovementEvent,
    TOUCH_BALL_MOVEMENT_EVENT_DEFINITION,
    "touch_ball_movement",
    "Touch Ball Movement",
    EventCategory::Contact
);
define_stats_event!(
    TouchLastTouchEvent,
    TOUCH_LAST_TOUCH_EVENT_DEFINITION,
    "touch_last_touch",
    "Touch Last Touch",
    EventCategory::Contact
);
define_stats_event!(
    BoostPickupComparisonEvent,
    BOOST_PICKUP_COMPARISON_EVENT_DEFINITION,
    "boost_pickups",
    "Boost Pickup",
    EventCategory::Boost
);
define_stats_event!(
    BoostLedgerEvent,
    BOOST_LEDGER_EVENT_DEFINITION,
    "boost_ledger",
    "Boost Ledger",
    EventCategory::Boost
);
define_stats_event!(
    BoostStateEvent,
    BOOST_STATE_EVENT_DEFINITION,
    "boost_state",
    "Boost State",
    EventCategory::Boost
);
define_stats_event!(
    BumpEvent,
    BUMP_EVENT_DEFINITION,
    "bump",
    "Bump",
    EventCategory::Contact
);
define_stats_event!(
    PossessionEvent,
    POSSESSION_EVENT_DEFINITION,
    "possession",
    "Possession",
    EventCategory::Possession
);
define_stats_event!(
    PressureEvent,
    PRESSURE_EVENT_DEFINITION,
    "pressure",
    "Pressure",
    EventCategory::Possession
);
define_stats_event!(
    TerritorialPressureEvent,
    TERRITORIAL_PRESSURE_EVENT_DEFINITION,
    "territorial_pressure",
    "Territorial Pressure",
    EventCategory::Possession
);
define_stats_event!(
    MovementEvent,
    MOVEMENT_EVENT_DEFINITION,
    "movement",
    "Movement",
    EventCategory::Movement
);
define_stats_event!(
    PositioningEvent,
    POSITIONING_EVENT_DEFINITION,
    "positioning",
    "Positioning",
    EventCategory::Positioning
);
define_stats_event!(
    RotationPlayerEvent,
    ROTATION_PLAYER_EVENT_DEFINITION,
    "rotation_player",
    "Player Rotation",
    EventCategory::Positioning
);
define_stats_event!(
    RotationRoleSpanEvent,
    ROTATION_ROLE_SPAN_EVENT_DEFINITION,
    "rotation_role_span",
    "Rotation Role Span",
    EventCategory::Positioning
);
define_stats_event!(
    RotationDepthSpanEvent,
    ROTATION_DEPTH_SPAN_EVENT_DEFINITION,
    "rotation_depth_span",
    "Rotation Depth Span",
    EventCategory::Positioning
);
define_stats_event!(
    RotationFirstManStintEvent,
    ROTATION_FIRST_MAN_STINT_EVENT_DEFINITION,
    "rotation_first_man_stint",
    "First Man Stint",
    EventCategory::Positioning
);
define_stats_event!(
    RotationTeamEvent,
    ROTATION_TEAM_EVENT_DEFINITION,
    "rotation_team",
    "Team Rotation",
    EventCategory::Positioning
);
define_stats_event!(
    FlipResetEvent,
    FLIP_RESET_EVENT_DEFINITION,
    "flip_reset",
    "Flip Reset",
    EventCategory::Mechanic,
    summary = "A touch candidate where the ball contacts the underside of an airborne car in flip-reset-like geometry.",
    approach = [
        "Evaluate touch events using normalized ball and player rigid bodies plus hitbox contact gap.",
        "Require airborne player/ball height, underside alignment, local ball position under the car, and a bounded hitbox contact gap.",
        "Also emit proximity-based candidates when replay touch attribution is missing but the ball/car geometry strongly matches a reset contact.",
    ]
);
define_stats_event!(
    PostWallDodgeEvent,
    POST_WALL_DODGE_EVENT_DEFINITION,
    "post_wall_dodge",
    "Post-Wall Dodge",
    EventCategory::Mechanic,
    summary = "A dodge that starts shortly after the player contacts a wall.",
    approach = [
        "Track recent wall contact times for each player from player position.",
        "Record dodge rising edges and match them to recent wall contacts for the same player.",
        "Emit when the dodge begins inside the configured wall-contact-to-dodge window.",
    ]
);
define_stats_event!(
    FlipResetFollowupDodgeEvent,
    FLIP_RESET_FOLLOWUP_DODGE_EVENT_DEFINITION,
    "flip_reset_followup_dodge",
    "Flip Reset Follow-Up Dodge",
    EventCategory::Mechanic,
    summary = "A dodge after a loose flip-reset touch candidate, used as supporting evidence that the candidate produced a usable reset.",
    approach = [
        "Track lower-confidence underside touch candidates that are plausible but not strong enough to stand alone.",
        "Record dodge rising edges for the same player after the candidate touch.",
        "Emit when the dodge occurs within the follow-up window, carrying through the candidate touch confidence.",
    ]
);
define_stats_event!(
    StatsTimelineTagEvent,
    STATS_TIMELINE_TAG_EVENT_DEFINITION,
    "mechanics",
    "Mechanic Timeline Tag",
    EventCategory::Mechanic,
    summary = "A normalized timeline representation of mechanic detections for playback and visualization.",
    approach = [
        "Collect completed mechanic events from the analysis graph at finish time.",
        "Convert point mechanics into moment tags and span mechanics into duration tags with stable IDs.",
        "Attach selected mechanic-specific properties, such as air-dribble origin and touch count, for timeline consumers.",
    ]
);

pub const ALL_EVENT_DEFINITIONS: &[&EventDefinition] = &[
    &TIMELINE_EVENT_DEFINITION,
    &CORE_PLAYER_SCOREBOARD_EVENT_DEFINITION,
    &CORE_PLAYER_GOAL_CONTEXT_EVENT_DEFINITION,
    &GOAL_CONTEXT_EVENT_DEFINITION,
    &BACKBOARD_BOUNCE_EVENT_DEFINITION,
    &CEILING_SHOT_EVENT_DEFINITION,
    &WALL_AERIAL_EVENT_DEFINITION,
    &WALL_AERIAL_SHOT_EVENT_DEFINITION,
    &CENTER_EVENT_DEFINITION,
    &FLICK_EVENT_DEFINITION,
    &MUSTY_FLICK_EVENT_DEFINITION,
    &DODGE_RESET_EVENT_DEFINITION,
    &DODGE_REFRESHED_EVENT_DEFINITION,
    &CONFIRMED_FLIP_RESET_EVENT_DEFINITION,
    &DOUBLE_TAP_EVENT_DEFINITION,
    &ONE_TIMER_EVENT_DEFINITION,
    &PASS_EVENT_DEFINITION,
    &PASS_LAST_COMPLETED_EVENT_DEFINITION,
    &BALL_CARRY_EVENT_DEFINITION,
    &FIFTY_FIFTY_EVENT_DEFINITION,
    &RUSH_EVENT_DEFINITION,
    &SPEED_FLIP_EVENT_DEFINITION,
    &HALF_FLIP_EVENT_DEFINITION,
    &HALF_VOLLEY_EVENT_DEFINITION,
    &WAVEDASH_EVENT_DEFINITION,
    &WHIFF_EVENT_DEFINITION,
    &POWERSLIDE_EVENT_DEFINITION,
    &TOUCH_CLASSIFICATION_EVENT_DEFINITION,
    &TOUCH_BALL_MOVEMENT_EVENT_DEFINITION,
    &TOUCH_LAST_TOUCH_EVENT_DEFINITION,
    &BOOST_PICKUP_COMPARISON_EVENT_DEFINITION,
    &BOOST_LEDGER_EVENT_DEFINITION,
    &BOOST_STATE_EVENT_DEFINITION,
    &BUMP_EVENT_DEFINITION,
    &POSSESSION_EVENT_DEFINITION,
    &PRESSURE_EVENT_DEFINITION,
    &TERRITORIAL_PRESSURE_EVENT_DEFINITION,
    &MOVEMENT_EVENT_DEFINITION,
    &POSITIONING_EVENT_DEFINITION,
    &ROTATION_PLAYER_EVENT_DEFINITION,
    &ROTATION_TEAM_EVENT_DEFINITION,
    &FLIP_RESET_EVENT_DEFINITION,
    &POST_WALL_DODGE_EVENT_DEFINITION,
    &FLIP_RESET_FOLLOWUP_DODGE_EVENT_DEFINITION,
    &STATS_TIMELINE_TAG_EVENT_DEFINITION,
];

const MATCH_STATS_EMITTED_EVENTS: &[EmittedEvent] = &[
    produced_event(
        &TIMELINE_EVENT_DEFINITION,
        "match_stats",
        "MatchStatsNode",
        "MatchStatsCalculator",
    ),
    produced_event(
        &CORE_PLAYER_SCOREBOARD_EVENT_DEFINITION,
        "match_stats",
        "MatchStatsNode",
        "MatchStatsCalculator",
    ),
    produced_event(
        &CORE_PLAYER_GOAL_CONTEXT_EVENT_DEFINITION,
        "match_stats",
        "MatchStatsNode",
        "MatchStatsCalculator",
    ),
    produced_event(
        &GOAL_CONTEXT_EVENT_DEFINITION,
        "match_stats",
        "MatchStatsNode",
        "MatchStatsCalculator",
    ),
];

const DEMO_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &TIMELINE_EVENT_DEFINITION,
    "demo",
    "DemoNode",
    "DemoCalculator",
)];

const BACKBOARD_BOUNCE_STATE_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &BACKBOARD_BOUNCE_EVENT_DEFINITION,
    "backboard_bounce_state",
    "BackboardBounceStateNode",
    "BackboardBounceCalculator",
)];

const CEILING_SHOT_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &CEILING_SHOT_EVENT_DEFINITION,
    "ceiling_shot",
    "CeilingShotNode",
    "CeilingShotCalculator",
)];

const WALL_AERIAL_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &WALL_AERIAL_EVENT_DEFINITION,
    "wall_aerial",
    "WallAerialNode",
    "WallAerialCalculator",
)];

const WALL_AERIAL_SHOT_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &WALL_AERIAL_SHOT_EVENT_DEFINITION,
    "wall_aerial_shot",
    "WallAerialShotNode",
    "WallAerialShotCalculator",
)];

const CENTER_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &CENTER_EVENT_DEFINITION,
    "center",
    "CenterNode",
    "CenterCalculator",
)];

const FLICK_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &FLICK_EVENT_DEFINITION,
    "flick",
    "FlickNode",
    "FlickCalculator",
)];

const MUSTY_FLICK_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &MUSTY_FLICK_EVENT_DEFINITION,
    "musty_flick",
    "MustyFlickNode",
    "MustyFlickCalculator",
)];

const DODGE_RESET_EMITTED_EVENTS: &[EmittedEvent] = &[
    produced_event(
        &DODGE_RESET_EVENT_DEFINITION,
        "dodge_reset",
        "DodgeResetNode",
        "DodgeResetCalculator",
    ),
    produced_event(
        &DODGE_REFRESHED_EVENT_DEFINITION,
        "dodge_reset",
        "DodgeResetNode",
        "DodgeResetCalculator",
    ),
    produced_event(
        &CONFIRMED_FLIP_RESET_EVENT_DEFINITION,
        "dodge_reset",
        "DodgeResetNode",
        "DodgeResetCalculator",
    ),
];

const DOUBLE_TAP_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &DOUBLE_TAP_EVENT_DEFINITION,
    "double_tap",
    "DoubleTapNode",
    "DoubleTapCalculator",
)];

const ONE_TIMER_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &ONE_TIMER_EVENT_DEFINITION,
    "one_timer",
    "OneTimerNode",
    "OneTimerCalculator",
)];

const PASS_EMITTED_EVENTS: &[EmittedEvent] = &[
    produced_event(&PASS_EVENT_DEFINITION, "pass", "PassNode", "PassCalculator"),
    produced_event(
        &PASS_LAST_COMPLETED_EVENT_DEFINITION,
        "pass",
        "PassNode",
        "PassCalculator",
    ),
];

const BALL_CARRY_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &BALL_CARRY_EVENT_DEFINITION,
    "ball_carry",
    "BallCarryNode",
    "BallCarryCalculator",
)];

const FIFTY_FIFTY_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &FIFTY_FIFTY_EVENT_DEFINITION,
    "fifty_fifty",
    "FiftyFiftyNode",
    "FiftyFiftyCalculator",
)];

const RUSH_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &RUSH_EVENT_DEFINITION,
    "rush",
    "RushNode",
    "RushCalculator",
)];

const SPEED_FLIP_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &SPEED_FLIP_EVENT_DEFINITION,
    "speed_flip",
    "SpeedFlipNode",
    "SpeedFlipCalculator",
)];

const HALF_FLIP_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &HALF_FLIP_EVENT_DEFINITION,
    "half_flip",
    "HalfFlipNode",
    "HalfFlipCalculator",
)];

const HALF_VOLLEY_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &HALF_VOLLEY_EVENT_DEFINITION,
    "half_volley",
    "HalfVolleyNode",
    "HalfVolleyCalculator",
)];

const WAVEDASH_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &WAVEDASH_EVENT_DEFINITION,
    "wavedash",
    "WavedashNode",
    "WavedashCalculator",
)];

const WHIFF_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &WHIFF_EVENT_DEFINITION,
    "whiff",
    "WhiffNode",
    "WhiffCalculator",
)];

const POWERSLIDE_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &POWERSLIDE_EVENT_DEFINITION,
    "powerslide",
    "PowerslideNode",
    "PowerslideCalculator",
)];

const TOUCH_EMITTED_EVENTS: &[EmittedEvent] = &[
    produced_event(
        &TOUCH_CLASSIFICATION_EVENT_DEFINITION,
        "touch",
        "TouchNode",
        "TouchCalculator",
    ),
    produced_event(
        &TOUCH_BALL_MOVEMENT_EVENT_DEFINITION,
        "touch",
        "TouchNode",
        "TouchCalculator",
    ),
    produced_event(
        &TOUCH_LAST_TOUCH_EVENT_DEFINITION,
        "touch",
        "TouchNode",
        "TouchCalculator",
    ),
];

const BOOST_EMITTED_EVENTS: &[EmittedEvent] = &[
    produced_event(
        &BOOST_PICKUP_COMPARISON_EVENT_DEFINITION,
        "boost",
        "BoostNode",
        "BoostCalculator",
    ),
    produced_event(
        &BOOST_LEDGER_EVENT_DEFINITION,
        "boost",
        "BoostNode",
        "BoostCalculator",
    ),
    produced_event(
        &BOOST_STATE_EVENT_DEFINITION,
        "boost",
        "BoostNode",
        "BoostCalculator",
    ),
];

const BUMP_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &BUMP_EVENT_DEFINITION,
    "bump",
    "BumpNode",
    "BumpCalculator",
)];

const POSSESSION_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &POSSESSION_EVENT_DEFINITION,
    "possession",
    "PossessionNode",
    "PossessionCalculator",
)];

const PRESSURE_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &PRESSURE_EVENT_DEFINITION,
    "pressure",
    "PressureNode",
    "PressureCalculator",
)];

const TERRITORIAL_PRESSURE_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &TERRITORIAL_PRESSURE_EVENT_DEFINITION,
    "territorial_pressure",
    "TerritorialPressureNode",
    "TerritorialPressureCalculator",
)];

const MOVEMENT_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &MOVEMENT_EVENT_DEFINITION,
    "movement",
    "MovementNode",
    "MovementCalculator",
)];

const POSITIONING_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &POSITIONING_EVENT_DEFINITION,
    "positioning",
    "PositioningNode",
    "PositioningCalculator",
)];

const ROTATION_EMITTED_EVENTS: &[EmittedEvent] = &[
    produced_event(
        &ROTATION_PLAYER_EVENT_DEFINITION,
        "rotation",
        "RotationNode",
        "RotationCalculator",
    ),
    produced_event(
        &ROTATION_TEAM_EVENT_DEFINITION,
        "rotation",
        "RotationNode",
        "RotationCalculator",
    ),
];

const STATS_TIMELINE_EVENTS_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &STATS_TIMELINE_TAG_EVENT_DEFINITION,
    "stats_timeline_events",
    "StatsTimelineEventsNode",
    "StatsTimelineEventsState",
)];

register_event_producer!(
    MATCH_STATS_EVENT_PRODUCER,
    "match_stats",
    MATCH_STATS_EMITTED_EVENTS
);
register_event_producer!(DEMO_EVENT_PRODUCER, "demo", DEMO_EMITTED_EVENTS);
register_event_producer!(
    BACKBOARD_BOUNCE_STATE_EVENT_PRODUCER,
    "backboard_bounce_state",
    BACKBOARD_BOUNCE_STATE_EMITTED_EVENTS
);
register_event_producer!(
    CEILING_SHOT_EVENT_PRODUCER,
    "ceiling_shot",
    CEILING_SHOT_EMITTED_EVENTS
);
register_event_producer!(
    WALL_AERIAL_EVENT_PRODUCER,
    "wall_aerial",
    WALL_AERIAL_EMITTED_EVENTS
);
register_event_producer!(
    WALL_AERIAL_SHOT_EVENT_PRODUCER,
    "wall_aerial_shot",
    WALL_AERIAL_SHOT_EMITTED_EVENTS
);
register_event_producer!(CENTER_EVENT_PRODUCER, "center", CENTER_EMITTED_EVENTS);
register_event_producer!(FLICK_EVENT_PRODUCER, "flick", FLICK_EMITTED_EVENTS);
register_event_producer!(
    MUSTY_FLICK_EVENT_PRODUCER,
    "musty_flick",
    MUSTY_FLICK_EMITTED_EVENTS
);
register_event_producer!(
    DODGE_RESET_EVENT_PRODUCER,
    "dodge_reset",
    DODGE_RESET_EMITTED_EVENTS
);
register_event_producer!(
    DOUBLE_TAP_EVENT_PRODUCER,
    "double_tap",
    DOUBLE_TAP_EMITTED_EVENTS
);
register_event_producer!(
    ONE_TIMER_EVENT_PRODUCER,
    "one_timer",
    ONE_TIMER_EMITTED_EVENTS
);
register_event_producer!(PASS_EVENT_PRODUCER, "pass", PASS_EMITTED_EVENTS);
register_event_producer!(
    BALL_CARRY_EVENT_PRODUCER,
    "ball_carry",
    BALL_CARRY_EMITTED_EVENTS
);
register_event_producer!(
    FIFTY_FIFTY_EVENT_PRODUCER,
    "fifty_fifty",
    FIFTY_FIFTY_EMITTED_EVENTS
);
register_event_producer!(RUSH_EVENT_PRODUCER, "rush", RUSH_EMITTED_EVENTS);
register_event_producer!(
    SPEED_FLIP_EVENT_PRODUCER,
    "speed_flip",
    SPEED_FLIP_EMITTED_EVENTS
);
register_event_producer!(
    HALF_FLIP_EVENT_PRODUCER,
    "half_flip",
    HALF_FLIP_EMITTED_EVENTS
);
register_event_producer!(
    HALF_VOLLEY_EVENT_PRODUCER,
    "half_volley",
    HALF_VOLLEY_EMITTED_EVENTS
);
register_event_producer!(WAVEDASH_EVENT_PRODUCER, "wavedash", WAVEDASH_EMITTED_EVENTS);
register_event_producer!(WHIFF_EVENT_PRODUCER, "whiff", WHIFF_EMITTED_EVENTS);
register_event_producer!(
    POWERSLIDE_EVENT_PRODUCER,
    "powerslide",
    POWERSLIDE_EMITTED_EVENTS
);
register_event_producer!(TOUCH_EVENT_PRODUCER, "touch", TOUCH_EMITTED_EVENTS);
register_event_producer!(BOOST_EVENT_PRODUCER, "boost", BOOST_EMITTED_EVENTS);
register_event_producer!(BUMP_EVENT_PRODUCER, "bump", BUMP_EMITTED_EVENTS);
register_event_producer!(
    POSSESSION_EVENT_PRODUCER,
    "possession",
    POSSESSION_EMITTED_EVENTS
);
register_event_producer!(PRESSURE_EVENT_PRODUCER, "pressure", PRESSURE_EMITTED_EVENTS);
register_event_producer!(
    TERRITORIAL_PRESSURE_EVENT_PRODUCER,
    "territorial_pressure",
    TERRITORIAL_PRESSURE_EMITTED_EVENTS
);
register_event_producer!(MOVEMENT_EVENT_PRODUCER, "movement", MOVEMENT_EMITTED_EVENTS);
register_event_producer!(
    POSITIONING_EVENT_PRODUCER,
    "positioning",
    POSITIONING_EMITTED_EVENTS
);
register_event_producer!(ROTATION_EVENT_PRODUCER, "rotation", ROTATION_EMITTED_EVENTS);
register_event_producer!(
    STATS_TIMELINE_EVENTS_EVENT_PRODUCER,
    "stats_timeline_events",
    STATS_TIMELINE_EVENTS_EMITTED_EVENTS
);

#[cfg(test)]
#[path = "event_definition_tests.rs"]
mod tests;
