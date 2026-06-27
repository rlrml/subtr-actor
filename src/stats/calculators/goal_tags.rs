use super::*;

#[path = "goal_tag_helpers.rs"]
mod goal_tag_helpers;
use goal_tag_helpers::*;

const DEFAULT_AERIAL_GOAL_MIN_BALL_Z: f32 = 600.0;
const DEFAULT_HIGH_AERIAL_GOAL_MIN_BALL_Z: f32 = 700.0;
const DEFAULT_LONG_DISTANCE_GOAL_MAX_ATTACKING_Y: f32 = 1024.0;
const DEFAULT_OWN_HALF_GOAL_MAX_ATTACKING_Y: f32 = 0.0;
// Avoid labeling long delayed clears as own-half goals solely from replay goal credit.
const OWN_HALF_GOAL_MAX_TOUCH_TO_GOAL_SECONDS: f32 = 8.0;
const DEFAULT_EMPTY_NET_MIN_DEFENDER_Y_MARGIN: f32 = 700.0;
const DEFAULT_EMPTY_NET_MIN_DEFENDER_DISTANCE: f32 = 1000.0;
const DEFAULT_EMPTY_NET_MAX_TOUCH_ATTACKING_Y: f32 = 3600.0;
const DEFAULT_FLICK_GOAL_MAX_EVENT_TO_GOAL_SECONDS: f32 = 5.0;
const DEFAULT_CEILING_SHOT_GOAL_MAX_EVENT_TO_GOAL_SECONDS: f32 = 3.0;
const DEFAULT_DOUBLE_TAP_GOAL_MAX_EVENT_TO_GOAL_SECONDS: f32 = 3.0;
const DEFAULT_ONE_TIMER_GOAL_MAX_EVENT_TO_GOAL_SECONDS: f32 = 3.0;
const DEFAULT_PASSING_GOAL_MAX_PASS_TO_GOAL_SECONDS: f32 = 3.0;
const DEFAULT_AIR_DRIBBLE_GOAL_MAX_END_TO_GOAL_SECONDS: f32 = 3.0;
const DEFAULT_FLIP_RESET_GOAL_MAX_EVENT_TO_GOAL_SECONDS: f32 = 8.0;
const DEFAULT_FLIP_INTO_BALL_GOAL_MAX_TOUCH_TO_GOAL_SECONDS: f32 = 3.0;
// Matches `TouchDodgeState::Dodge.as_label_value()` in the touch calculator.
const FLIP_INTO_BALL_DODGE_STATE_LABEL: &str = "dodge";
const DEFAULT_BUMP_GOAL_MAX_EVENT_TO_GOAL_SECONDS: f32 = 3.0;
// Demos can create an open-net chance that takes a few seconds to cash in; the
// kickoff boundary check in `demo_event_matches_goal` prevents this wider
// window from reaching into a previous play.
const DEFAULT_DEMO_GOAL_MAX_EVENT_TO_GOAL_SECONDS: f32 = 5.25;
const DEFAULT_HALF_VOLLEY_GOAL_MAX_TOUCH_TO_GOAL_SECONDS: f32 = 3.0;
const DEFAULT_HALF_VOLLEY_GOAL_MIN_GOAL_ALIGNMENT: f32 = 0.55;
// Kickoff events record the attributed goal as first_touch_time +
// time_to_goal; this epsilon absorbs the float round-trip when matching that
// back to a goal context's timestamp.
const KICKOFF_GOAL_TAG_MATCH_EPSILON_SECONDS: f32 = 0.05;

/// Identifier for a kind of goal tag.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum GoalTagKind {
    AerialGoal,
    HighAerialGoal,
    LongDistanceGoal,
    OwnHalfGoal,
    EmptyNetGoal,
    CounterAttackGoal,
    SustainedPressureGoal,
    FlickGoal,
    CeilingShotGoal,
    DoubleTapGoal,
    OneTimerGoal,
    PassingGoal,
    AirDribbleGoal,
    FlipResetGoal,
    FlipIntoBallGoal,
    BumpGoal,
    DemoGoal,
    HalfVolleyGoal,
    KickoffGoal,
}

/// Which event stream a goal tag's evidence is drawn from.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum GoalTagEventStream {
    Flick,
    CeilingShot,
    DoubleTap,
    OneTimer,
    Pass,
    BallCarry,
    DodgeReset,
    FlipReset,
    Touch,
    Bump,
    Demo,
    HalfVolley,
}

/// The kind of evidence supporting a goal tag.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum GoalTagEvidenceKind {
    GoalContext,
    ScorerLastTouch,
    /// A touch in the goal's leadup (not necessarily the scorer's last touch)
    /// that satisfied a tag's criteria, e.g. the high-aerial build-up touch.
    LeadupTouch,
    DefenderPosition,
    GoalBuildup,
    Flick,
    CeilingShot,
    DoubleTap,
    OneTimer,
    Pass,
    AirDribble,
    FlipReset,
    FlipIntoBall,
    Bump,
    Demo,
    HalfVolley,
}

/// An optional modifier qualifying a goal tag.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum GoalTagModifier {
    ByScorer,
}

/// Who performed the tagged action relative to the goal.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum GoalTagPerformer {
    Scorer,
    Teammate,
}

/// Evidence linking an event to a tagged goal.
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct GoalTagEvidence {
    pub kind: GoalTagEvidenceKind,
    pub time: f32,
    pub frame: usize,
    #[ts(as = "Option<crate::interop::ts_bindings::RemoteIdTs>")]
    pub player: Option<PlayerId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_position: Option<GoalContextPosition>,
}

/// Reference to the event backing a goal tag.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct GoalTagEventRef {
    pub stream: GoalTagEventStream,
    pub index: usize,
}

/// A small categorical descriptor copied from the mechanic event that
/// produced a goal tag (e.g. a flick goal's flick `kind`), so consumers can
/// surface mechanic flavor without dereferencing `related_events`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct GoalTagDetail {
    pub key: String,
    pub value: String,
}

/// Metadata describing a goal-tag definition.
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct GoalTagMetadata {
    pub confidence: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub performer: Option<GoalTagPerformer>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub modifiers: Vec<GoalTagModifier>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related_events: Vec<GoalTagEventRef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub details: Vec<GoalTagDetail>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<GoalTagEvidence>,
}

/// An assignable tag describing how a goal was scored.
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[serde(tag = "kind", content = "metadata", rename_all = "snake_case")]
#[ts(export)]
pub enum GoalTag {
    AerialGoal(GoalTagMetadata),
    HighAerialGoal(GoalTagMetadata),
    LongDistanceGoal(GoalTagMetadata),
    OwnHalfGoal(GoalTagMetadata),
    EmptyNetGoal(GoalTagMetadata),
    CounterAttackGoal(GoalTagMetadata),
    SustainedPressureGoal(GoalTagMetadata),
    FlickGoal(GoalTagMetadata),
    CeilingShotGoal(GoalTagMetadata),
    DoubleTapGoal(GoalTagMetadata),
    OneTimerGoal(GoalTagMetadata),
    PassingGoal(GoalTagMetadata),
    AirDribbleGoal(GoalTagMetadata),
    FlipResetGoal(GoalTagMetadata),
    FlipIntoBallGoal(GoalTagMetadata),
    BumpGoal(GoalTagMetadata),
    DemoGoal(GoalTagMetadata),
    HalfVolleyGoal(GoalTagMetadata),
    KickoffGoal(GoalTagMetadata),
}

/// Definition of a goal tag and how it is matched against a goal.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct GoalTagDefinition {
    pub kind: GoalTagKind,
    pub id: &'static str,
    pub label: &'static str,
    pub summary: &'static str,
    pub approach: &'static [&'static str],
}

pub const fn goal_tag_definition(
    kind: GoalTagKind,
    id: &'static str,
    label: &'static str,
    summary: &'static str,
    approach: &'static [&'static str],
) -> GoalTagDefinition {
    GoalTagDefinition {
        kind,
        id,
        label,
        summary,
        approach,
    }
}

pub const ALL_GOAL_TAG_DEFINITIONS: &[GoalTagDefinition] = &[
    goal_tag_definition(
        GoalTagKind::AerialGoal,
        "aerial_goal",
        "Aerial Goal",
        "A goal whose scorer last touched the ball while it was high in the air.",
        &[
            "Inspect each goal context and its scorer-last-touch evidence.",
            "Require the last-touch ball height to meet the aerial-goal threshold.",
            "Attach goal-context and last-touch evidence to the goal tag metadata.",
        ],
    ),
    goal_tag_definition(
        GoalTagKind::HighAerialGoal,
        "high_aerial_goal",
        "High Aerial Goal",
        "A goal whose scoring possession includes a touch taken from a high ball height, even when the finishing touch itself was lower.",
        &[
            "Scan the scoring team's touches within the possession that led to the goal (back to the last turnover or neutral loose ball).",
            "Require at least one such touch to meet the high-aerial ball-height threshold.",
            "Tag the goal from the highest qualifying touch, attaching it as leadup-touch evidence.",
        ],
    ),
    goal_tag_definition(
        GoalTagKind::LongDistanceGoal,
        "long_distance_goal",
        "Long-Distance Goal",
        "A goal where the scorer's last touch started from deep enough in the attacking team's half-space.",
        &[
            "Use the scorer-last-touch ball position from goal context.",
            "Normalize field direction by scoring team and compare the touch y-position to the long-distance threshold.",
            "Attach goal-context and last-touch evidence to the goal tag metadata.",
        ],
    ),
    goal_tag_definition(
        GoalTagKind::OwnHalfGoal,
        "own_half_goal",
        "Own-Half Goal",
        "A long-distance goal where the scorer's last touch came from their own half and close enough in time to the goal.",
        &[
            "Use the scorer-last-touch ball position and time from goal context.",
            "Require the touch to be in the scoring team's own half and within the own-half touch-to-goal window.",
            "Allow the long-distance goal tag to also apply when both distance thresholds are met.",
        ],
    ),
    goal_tag_definition(
        GoalTagKind::EmptyNetGoal,
        "empty_net_goal",
        "Empty Net Goal",
        "A goal where defenders are judged too far or too poorly positioned to cover the net.",
        &[
            "Inspect defending-player positions in the goal context.",
            "Compare defender depth and distance against the empty-net thresholds.",
            "Avoid tagging very deep attacking touches as empty nets when the touch position is outside the configured range.",
        ],
    ),
    goal_tag_definition(
        GoalTagKind::CounterAttackGoal,
        "counter_attack_goal",
        "Counter-Attack Goal",
        "A goal whose buildup was classified as a counterattack.",
        &[
            "Use the goal-buildup classification computed in goal context.",
            "Tag goals whose buildup kind is counterattack.",
            "Attach goal-buildup evidence to the goal tag metadata.",
        ],
    ),
    goal_tag_definition(
        GoalTagKind::SustainedPressureGoal,
        "sustained_pressure_goal",
        "Sustained Pressure Goal",
        "A goal whose buildup was classified as sustained offensive pressure.",
        &[
            "Use the goal-buildup classification computed in goal context.",
            "Tag goals whose buildup kind is sustained pressure.",
            "Attach goal-buildup evidence to the goal tag metadata.",
        ],
    ),
    goal_tag_definition(
        GoalTagKind::FlickGoal,
        "flick_goal",
        "Flick Goal",
        "A goal linked to a recent flick event.",
        &[
            "Compare recent flick events against each goal's scorer-last-touch context.",
            "Require the flick to fall within the configured event-to-goal window.",
            "Prefer by-scorer evidence when the flick player matches the scorer's last touch.",
        ],
    ),
    goal_tag_definition(
        GoalTagKind::CeilingShotGoal,
        "ceiling_shot_goal",
        "Ceiling-Shot Goal",
        "A goal linked to a recent ceiling-shot event.",
        &[
            "Compare recent ceiling-shot events against each goal's scorer-last-touch context.",
            "Require the ceiling shot to fall within the configured event-to-goal window.",
            "Attach a related ceiling-shot event reference and ceiling-shot evidence to the goal tag metadata.",
        ],
    ),
    goal_tag_definition(
        GoalTagKind::DoubleTapGoal,
        "double_tap_goal",
        "Double-Tap Goal",
        "A goal linked to a recent double-tap event.",
        &[
            "Compare recent double-tap events against each goal's scorer-last-touch context.",
            "Require the double tap to fall within the configured event-to-goal window.",
            "Attach a related-event reference and mechanic evidence to the goal tag metadata.",
        ],
    ),
    goal_tag_definition(
        GoalTagKind::OneTimerGoal,
        "one_timer_goal",
        "One-Timer Goal",
        "A goal linked to a recent one-timer event.",
        &[
            "Compare recent one-timer events against each goal's scorer-last-touch context.",
            "Require the one timer to fall within the configured event-to-goal window.",
            "Prefer by-scorer evidence when the one-timer receiver matches the scorer's last touch.",
        ],
    ),
    goal_tag_definition(
        GoalTagKind::PassingGoal,
        "passing_goal",
        "Passing Goal",
        "A goal where a completed pass is linked to the scoring touch.",
        &[
            "Compare pass events against each goal's scorer-last-touch context.",
            "Require the pass receiver to match the scorer's last touch within the pass-to-goal window.",
            "Attach a related pass-event reference and pass evidence to the goal tag metadata.",
        ],
    ),
    goal_tag_definition(
        GoalTagKind::AirDribbleGoal,
        "air_dribble_goal",
        "Air-Dribble Goal",
        "A goal linked to an air-dribble ball-carry sequence that reaches the scoring touch.",
        &[
            "Inspect completed ball-carry events whose kind is air dribble.",
            "Match air-dribble sequences to goals by timing and scorer-last-touch context.",
            "Attach a related ball-carry event reference and air-dribble evidence to the goal tag metadata.",
        ],
    ),
    goal_tag_definition(
        GoalTagKind::FlipResetGoal,
        "flip_reset_goal",
        "Flip-Reset Goal",
        "A goal linked to a recent on-ball dodge reset or flip-reset event.",
        &[
            "Compare reset-related mechanic events against each goal's scorer-last-touch context.",
            "Require the reset evidence to fall within the configured event-to-goal window.",
            "Prefer by-scorer evidence when the reset player matches the scorer's last touch.",
        ],
    ),
    goal_tag_definition(
        GoalTagKind::FlipIntoBallGoal,
        "flip_into_ball_goal",
        "Flip-Into-Ball Goal",
        "A goal where the scorer flipped (dodged) into the ball on the scoring touch.",
        &[
            "Match the scorer's last touch to its touch-classification event by touch id (player and frame for data predating touch ids).",
            "Require the scoring touch's dodge state to be active and the touch to fall within the touch-to-goal window.",
            "Limitation: the dodge state covers any active dodge overlapping the touch, so incidental flips that happen to contact the ball can also qualify; dodge direction toward the ball is not yet verified.",
        ],
    ),
    goal_tag_definition(
        GoalTagKind::BumpGoal,
        "bump_goal",
        "Bump Goal",
        "A goal linked to a recent scoring-team bump on an opponent.",
        &[
            "Compare non-team bump events against each goal's timing and scoring team.",
            "Require the bump initiator to be on the scoring team and within the configured event-to-goal window.",
            "Attach a related bump-event reference and bump evidence, even when the initiator is not the scorer.",
        ],
    ),
    goal_tag_definition(
        GoalTagKind::DemoGoal,
        "demo_goal",
        "Demo Goal",
        "A goal linked to a recent scoring-team demolition.",
        &[
            "Compare demolition kill events against each goal's timing and scoring team.",
            "Require the demo attacker to be on the scoring team and within the configured event-to-goal window.",
            "Attach a related demo-event reference and demo evidence, even when the attacker is not the scorer.",
        ],
    ),
    goal_tag_definition(
        GoalTagKind::HalfVolleyGoal,
        "half_volley_goal",
        "Half-Volley Goal",
        "A goal where the scorer's last touch matches a recent half-volley candidate.",
        &[
            "Compare half-volley events against each goal's scorer-last-touch context.",
            "Require the half-volley touch to be close enough to the goal and sufficiently aligned toward goal.",
            "Attach a related half-volley event reference and half-volley evidence to the goal tag metadata.",
        ],
    ),
    goal_tag_definition(
        GoalTagKind::KickoffGoal,
        "kickoff_goal",
        "Kickoff Goal",
        "A goal flowing directly from the kickoff exchange.",
        &[
            "Use the kickoff calculator's goal attribution as the source of truth.",
            "Require the goal to land within the kickoff-goal timing window of the first touch.",
            "Reject goals where the conceding team settled possession or the play reset through the scoring team's own half.",
            "Attach goal-context evidence so the tag appears with the goal label.",
        ],
    ),
];

impl GoalTag {
    pub fn from_parts(kind: GoalTagKind, metadata: GoalTagMetadata) -> Self {
        match kind {
            GoalTagKind::AerialGoal => Self::AerialGoal(metadata),
            GoalTagKind::HighAerialGoal => Self::HighAerialGoal(metadata),
            GoalTagKind::LongDistanceGoal => Self::LongDistanceGoal(metadata),
            GoalTagKind::OwnHalfGoal => Self::OwnHalfGoal(metadata),
            GoalTagKind::EmptyNetGoal => Self::EmptyNetGoal(metadata),
            GoalTagKind::CounterAttackGoal => Self::CounterAttackGoal(metadata),
            GoalTagKind::SustainedPressureGoal => Self::SustainedPressureGoal(metadata),
            GoalTagKind::FlickGoal => Self::FlickGoal(metadata),
            GoalTagKind::CeilingShotGoal => Self::CeilingShotGoal(metadata),
            GoalTagKind::DoubleTapGoal => Self::DoubleTapGoal(metadata),
            GoalTagKind::OneTimerGoal => Self::OneTimerGoal(metadata),
            GoalTagKind::PassingGoal => Self::PassingGoal(metadata),
            GoalTagKind::AirDribbleGoal => Self::AirDribbleGoal(metadata),
            GoalTagKind::FlipResetGoal => Self::FlipResetGoal(metadata),
            GoalTagKind::FlipIntoBallGoal => Self::FlipIntoBallGoal(metadata),
            GoalTagKind::BumpGoal => Self::BumpGoal(metadata),
            GoalTagKind::DemoGoal => Self::DemoGoal(metadata),
            GoalTagKind::HalfVolleyGoal => Self::HalfVolleyGoal(metadata),
            GoalTagKind::KickoffGoal => Self::KickoffGoal(metadata),
        }
    }

    pub fn kind(&self) -> GoalTagKind {
        match self {
            Self::AerialGoal(_) => GoalTagKind::AerialGoal,
            Self::HighAerialGoal(_) => GoalTagKind::HighAerialGoal,
            Self::LongDistanceGoal(_) => GoalTagKind::LongDistanceGoal,
            Self::OwnHalfGoal(_) => GoalTagKind::OwnHalfGoal,
            Self::EmptyNetGoal(_) => GoalTagKind::EmptyNetGoal,
            Self::CounterAttackGoal(_) => GoalTagKind::CounterAttackGoal,
            Self::SustainedPressureGoal(_) => GoalTagKind::SustainedPressureGoal,
            Self::FlickGoal(_) => GoalTagKind::FlickGoal,
            Self::CeilingShotGoal(_) => GoalTagKind::CeilingShotGoal,
            Self::DoubleTapGoal(_) => GoalTagKind::DoubleTapGoal,
            Self::OneTimerGoal(_) => GoalTagKind::OneTimerGoal,
            Self::PassingGoal(_) => GoalTagKind::PassingGoal,
            Self::AirDribbleGoal(_) => GoalTagKind::AirDribbleGoal,
            Self::FlipResetGoal(_) => GoalTagKind::FlipResetGoal,
            Self::FlipIntoBallGoal(_) => GoalTagKind::FlipIntoBallGoal,
            Self::BumpGoal(_) => GoalTagKind::BumpGoal,
            Self::DemoGoal(_) => GoalTagKind::DemoGoal,
            Self::HalfVolleyGoal(_) => GoalTagKind::HalfVolleyGoal,
            Self::KickoffGoal(_) => GoalTagKind::KickoffGoal,
        }
    }

    pub fn metadata(&self) -> &GoalTagMetadata {
        match self {
            Self::AerialGoal(metadata)
            | Self::HighAerialGoal(metadata)
            | Self::LongDistanceGoal(metadata)
            | Self::OwnHalfGoal(metadata)
            | Self::EmptyNetGoal(metadata)
            | Self::CounterAttackGoal(metadata)
            | Self::SustainedPressureGoal(metadata)
            | Self::FlickGoal(metadata)
            | Self::CeilingShotGoal(metadata)
            | Self::DoubleTapGoal(metadata)
            | Self::OneTimerGoal(metadata)
            | Self::PassingGoal(metadata)
            | Self::AirDribbleGoal(metadata)
            | Self::FlipResetGoal(metadata)
            | Self::FlipIntoBallGoal(metadata)
            | Self::BumpGoal(metadata)
            | Self::DemoGoal(metadata)
            | Self::HalfVolleyGoal(metadata)
            | Self::KickoffGoal(metadata) => metadata,
        }
    }
}

/// A goal tag assigned to a specific goal.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GoalTagAssignment {
    pub goal_index: usize,
    pub tag: GoalTag,
}

/// Configuration thresholds for the aerial goal-tag calculator.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct AerialGoalCalculatorConfig {
    pub min_ball_z: f32,
}

impl Default for AerialGoalCalculatorConfig {
    fn default() -> Self {
        Self {
            min_ball_z: DEFAULT_AERIAL_GOAL_MIN_BALL_Z,
        }
    }
}

/// Configuration thresholds for the high-aerial goal-tag calculator.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct HighAerialGoalCalculatorConfig {
    pub min_ball_z: f32,
}

impl Default for HighAerialGoalCalculatorConfig {
    fn default() -> Self {
        Self {
            min_ball_z: DEFAULT_HIGH_AERIAL_GOAL_MIN_BALL_Z,
        }
    }
}

/// Configuration thresholds for the long-distance goal-tag calculator.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct LongDistanceGoalCalculatorConfig {
    pub max_attacking_y: f32,
}

impl Default for LongDistanceGoalCalculatorConfig {
    fn default() -> Self {
        Self {
            max_attacking_y: DEFAULT_LONG_DISTANCE_GOAL_MAX_ATTACKING_Y,
        }
    }
}

/// Configuration thresholds for the own-half goal-tag calculator.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct OwnHalfGoalCalculatorConfig {
    pub max_attacking_y: f32,
}

impl Default for OwnHalfGoalCalculatorConfig {
    fn default() -> Self {
        Self {
            max_attacking_y: DEFAULT_OWN_HALF_GOAL_MAX_ATTACKING_Y,
        }
    }
}

/// Configuration thresholds for the empty-net goal-tag calculator.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct EmptyNetGoalCalculatorConfig {
    pub min_defender_y_margin: f32,
    pub min_defender_distance: f32,
    pub max_touch_attacking_y: f32,
}

impl Default for EmptyNetGoalCalculatorConfig {
    fn default() -> Self {
        Self {
            min_defender_y_margin: DEFAULT_EMPTY_NET_MIN_DEFENDER_Y_MARGIN,
            min_defender_distance: DEFAULT_EMPTY_NET_MIN_DEFENDER_DISTANCE,
            max_touch_attacking_y: DEFAULT_EMPTY_NET_MAX_TOUCH_ATTACKING_Y,
        }
    }
}

/// Configuration thresholds for the flick goal-tag calculator.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct FlickGoalCalculatorConfig {
    pub max_event_to_goal_seconds: f32,
}

impl Default for FlickGoalCalculatorConfig {
    fn default() -> Self {
        Self {
            max_event_to_goal_seconds: DEFAULT_FLICK_GOAL_MAX_EVENT_TO_GOAL_SECONDS,
        }
    }
}

/// Configuration thresholds for the ceiling-shot goal-tag calculator.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct CeilingShotGoalCalculatorConfig {
    pub max_event_to_goal_seconds: f32,
}

impl Default for CeilingShotGoalCalculatorConfig {
    fn default() -> Self {
        Self {
            max_event_to_goal_seconds: DEFAULT_CEILING_SHOT_GOAL_MAX_EVENT_TO_GOAL_SECONDS,
        }
    }
}

/// Configuration thresholds for the double-tap goal-tag calculator.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct DoubleTapGoalCalculatorConfig {
    pub max_event_to_goal_seconds: f32,
}

impl Default for DoubleTapGoalCalculatorConfig {
    fn default() -> Self {
        Self {
            max_event_to_goal_seconds: DEFAULT_DOUBLE_TAP_GOAL_MAX_EVENT_TO_GOAL_SECONDS,
        }
    }
}

/// Configuration thresholds for the one-timer goal-tag calculator.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct OneTimerGoalCalculatorConfig {
    pub max_event_to_goal_seconds: f32,
}

impl Default for OneTimerGoalCalculatorConfig {
    fn default() -> Self {
        Self {
            max_event_to_goal_seconds: DEFAULT_ONE_TIMER_GOAL_MAX_EVENT_TO_GOAL_SECONDS,
        }
    }
}

/// Configuration thresholds for the passing goal-tag calculator.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct PassingGoalCalculatorConfig {
    pub max_pass_to_goal_seconds: f32,
}

impl Default for PassingGoalCalculatorConfig {
    fn default() -> Self {
        Self {
            max_pass_to_goal_seconds: DEFAULT_PASSING_GOAL_MAX_PASS_TO_GOAL_SECONDS,
        }
    }
}

/// Configuration thresholds for the air-dribble goal-tag calculator.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct AirDribbleGoalCalculatorConfig {
    pub max_end_to_goal_seconds: f32,
}

impl Default for AirDribbleGoalCalculatorConfig {
    fn default() -> Self {
        Self {
            max_end_to_goal_seconds: DEFAULT_AIR_DRIBBLE_GOAL_MAX_END_TO_GOAL_SECONDS,
        }
    }
}

/// Configuration thresholds for the flip-reset goal-tag calculator.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct FlipResetGoalCalculatorConfig {
    pub max_event_to_goal_seconds: f32,
}

impl Default for FlipResetGoalCalculatorConfig {
    fn default() -> Self {
        Self {
            max_event_to_goal_seconds: DEFAULT_FLIP_RESET_GOAL_MAX_EVENT_TO_GOAL_SECONDS,
        }
    }
}

/// Configuration thresholds for the flip-into-ball goal-tag calculator.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct FlipIntoBallGoalCalculatorConfig {
    pub max_touch_to_goal_seconds: f32,
}

impl Default for FlipIntoBallGoalCalculatorConfig {
    fn default() -> Self {
        Self {
            max_touch_to_goal_seconds: DEFAULT_FLIP_INTO_BALL_GOAL_MAX_TOUCH_TO_GOAL_SECONDS,
        }
    }
}

/// Configuration thresholds for the bump goal-tag calculator.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct BumpGoalCalculatorConfig {
    pub max_event_to_goal_seconds: f32,
}

impl Default for BumpGoalCalculatorConfig {
    fn default() -> Self {
        Self {
            max_event_to_goal_seconds: DEFAULT_BUMP_GOAL_MAX_EVENT_TO_GOAL_SECONDS,
        }
    }
}

/// Configuration thresholds for the demo goal-tag calculator.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct DemoGoalCalculatorConfig {
    pub max_event_to_goal_seconds: f32,
}

impl Default for DemoGoalCalculatorConfig {
    fn default() -> Self {
        Self {
            max_event_to_goal_seconds: DEFAULT_DEMO_GOAL_MAX_EVENT_TO_GOAL_SECONDS,
        }
    }
}

/// Configuration thresholds for the half-volley goal-tag calculator.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct HalfVolleyGoalCalculatorConfig {
    pub max_touch_to_goal_seconds: f32,
    pub min_goal_alignment: f32,
}

impl Default for HalfVolleyGoalCalculatorConfig {
    fn default() -> Self {
        Self {
            max_touch_to_goal_seconds: DEFAULT_HALF_VOLLEY_GOAL_MAX_TOUCH_TO_GOAL_SECONDS,
            min_goal_alignment: DEFAULT_HALF_VOLLEY_GOAL_MIN_GOAL_ALIGNMENT,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct GoalTaggingContext {
    goal_index: usize,
}

/// Tags goals scored from an aerial.
#[derive(Debug, Clone, PartialEq)]
pub struct AerialGoalCalculator {
    config: AerialGoalCalculatorConfig,
    events: EventStream<GoalTagAssignment>,
}

/// Tags goals scored from a high aerial.
#[derive(Debug, Clone, PartialEq)]
pub struct HighAerialGoalCalculator {
    config: HighAerialGoalCalculatorConfig,
    events: EventStream<GoalTagAssignment>,
}

/// Tags goals scored from long distance.
#[derive(Debug, Clone, PartialEq)]
pub struct LongDistanceGoalCalculator {
    config: LongDistanceGoalCalculatorConfig,
    events: EventStream<GoalTagAssignment>,
}

/// Tags goals scored from the shooter's own half.
#[derive(Debug, Clone, PartialEq)]
pub struct OwnHalfGoalCalculator {
    config: OwnHalfGoalCalculatorConfig,
    events: EventStream<GoalTagAssignment>,
}

/// Tags goals scored into an empty net.
#[derive(Debug, Clone, PartialEq)]
pub struct EmptyNetGoalCalculator {
    config: EmptyNetGoalCalculatorConfig,
    events: EventStream<GoalTagAssignment>,
}

/// Tags goals scored on a counter-attack.
#[derive(Debug, Clone, PartialEq)]
pub struct CounterAttackGoalCalculator {
    events: EventStream<GoalTagAssignment>,
}

/// Tags goals scored after sustained offensive pressure.
#[derive(Debug, Clone, PartialEq)]
pub struct SustainedPressureGoalCalculator {
    events: EventStream<GoalTagAssignment>,
}

/// Tags goals scored from a flick.
#[derive(Debug, Clone, PartialEq)]
pub struct FlickGoalCalculator {
    config: FlickGoalCalculatorConfig,
    events: EventStream<GoalTagAssignment>,
}

/// Tags goals scored from a ceiling shot.
#[derive(Debug, Clone, PartialEq)]
pub struct CeilingShotGoalCalculator {
    config: CeilingShotGoalCalculatorConfig,
    events: EventStream<GoalTagAssignment>,
}

/// Tags goals scored from a double tap.
#[derive(Debug, Clone, PartialEq)]
pub struct DoubleTapGoalCalculator {
    config: DoubleTapGoalCalculatorConfig,
    events: EventStream<GoalTagAssignment>,
}

/// Tags goals scored from a one-timer.
#[derive(Debug, Clone, PartialEq)]
pub struct OneTimerGoalCalculator {
    config: OneTimerGoalCalculatorConfig,
    events: EventStream<GoalTagAssignment>,
}

/// Tags goals scored off a pass.
#[derive(Debug, Clone, PartialEq)]
pub struct PassingGoalCalculator {
    config: PassingGoalCalculatorConfig,
    events: EventStream<GoalTagAssignment>,
}

/// Tags goals scored from an air dribble.
#[derive(Debug, Clone, PartialEq)]
pub struct AirDribbleGoalCalculator {
    config: AirDribbleGoalCalculatorConfig,
    events: EventStream<GoalTagAssignment>,
}

/// Tags goals scored after a flip reset.
#[derive(Debug, Clone, PartialEq)]
pub struct FlipResetGoalCalculator {
    config: FlipResetGoalCalculatorConfig,
    events: EventStream<GoalTagAssignment>,
}

/// Tags goals scored by flipping into the ball.
#[derive(Debug, Clone, PartialEq)]
pub struct FlipIntoBallGoalCalculator {
    config: FlipIntoBallGoalCalculatorConfig,
    events: EventStream<GoalTagAssignment>,
}

/// Tags goals involving a bump.
#[derive(Debug, Clone, PartialEq)]
pub struct BumpGoalCalculator {
    config: BumpGoalCalculatorConfig,
    events: EventStream<GoalTagAssignment>,
}

/// Tags goals involving a demolition.
#[derive(Debug, Clone, PartialEq)]
pub struct DemoGoalCalculator {
    config: DemoGoalCalculatorConfig,
    events: EventStream<GoalTagAssignment>,
}

/// Tags goals scored from a half-volley.
#[derive(Debug, Clone, PartialEq)]
pub struct HalfVolleyGoalCalculator {
    config: HalfVolleyGoalCalculatorConfig,
    events: EventStream<GoalTagAssignment>,
}

/// Tags goals scored directly off a kickoff.
#[derive(Debug, Clone, PartialEq)]
pub struct KickoffGoalCalculator {
    events: EventStream<GoalTagAssignment>,
}

macro_rules! impl_goal_tag_calculator {
    ($calculator:ident, $config:ident) => {
        impl Default for $calculator {
            fn default() -> Self {
                Self::new()
            }
        }

        impl $calculator {
            pub fn new() -> Self {
                Self::with_config($config::default())
            }

            pub fn with_config(config: $config) -> Self {
                Self {
                    config,
                    events: EventStream::new(),
                }
            }

            pub fn config(&self) -> &$config {
                &self.config
            }

            pub fn events(&self) -> &[GoalTagAssignment] {
                self.events.all()
            }

            pub fn new_events(&self) -> &[GoalTagAssignment] {
                self.events.new_events()
            }
        }
    };
}

impl_goal_tag_calculator!(AerialGoalCalculator, AerialGoalCalculatorConfig);
impl_goal_tag_calculator!(HighAerialGoalCalculator, HighAerialGoalCalculatorConfig);
impl_goal_tag_calculator!(LongDistanceGoalCalculator, LongDistanceGoalCalculatorConfig);
impl_goal_tag_calculator!(OwnHalfGoalCalculator, OwnHalfGoalCalculatorConfig);
impl_goal_tag_calculator!(EmptyNetGoalCalculator, EmptyNetGoalCalculatorConfig);
impl_goal_tag_calculator!(FlickGoalCalculator, FlickGoalCalculatorConfig);
impl_goal_tag_calculator!(CeilingShotGoalCalculator, CeilingShotGoalCalculatorConfig);
impl_goal_tag_calculator!(DoubleTapGoalCalculator, DoubleTapGoalCalculatorConfig);
impl_goal_tag_calculator!(OneTimerGoalCalculator, OneTimerGoalCalculatorConfig);
impl_goal_tag_calculator!(PassingGoalCalculator, PassingGoalCalculatorConfig);
impl_goal_tag_calculator!(AirDribbleGoalCalculator, AirDribbleGoalCalculatorConfig);
impl_goal_tag_calculator!(FlipResetGoalCalculator, FlipResetGoalCalculatorConfig);
impl_goal_tag_calculator!(FlipIntoBallGoalCalculator, FlipIntoBallGoalCalculatorConfig);
impl_goal_tag_calculator!(BumpGoalCalculator, BumpGoalCalculatorConfig);
impl_goal_tag_calculator!(DemoGoalCalculator, DemoGoalCalculatorConfig);

impl Default for CounterAttackGoalCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl CounterAttackGoalCalculator {
    pub fn new() -> Self {
        Self {
            events: EventStream::new(),
        }
    }

    pub fn events(&self) -> &[GoalTagAssignment] {
        self.events.all()
    }

    pub fn new_events(&self) -> &[GoalTagAssignment] {
        self.events.new_events()
    }
}

impl Default for SustainedPressureGoalCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl SustainedPressureGoalCalculator {
    pub fn new() -> Self {
        Self {
            events: EventStream::new(),
        }
    }

    pub fn events(&self) -> &[GoalTagAssignment] {
        self.events.all()
    }

    pub fn new_events(&self) -> &[GoalTagAssignment] {
        self.events.new_events()
    }
}

impl Default for HalfVolleyGoalCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl HalfVolleyGoalCalculator {
    pub fn new() -> Self {
        Self::with_config(HalfVolleyGoalCalculatorConfig::default())
    }

    pub fn with_config(config: HalfVolleyGoalCalculatorConfig) -> Self {
        Self {
            config,
            events: EventStream::new(),
        }
    }

    pub fn config(&self) -> &HalfVolleyGoalCalculatorConfig {
        &self.config
    }

    pub fn events(&self) -> &[GoalTagAssignment] {
        self.events.all()
    }

    pub fn new_events(&self) -> &[GoalTagAssignment] {
        self.events.new_events()
    }
}

impl Default for KickoffGoalCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl KickoffGoalCalculator {
    pub fn new() -> Self {
        Self {
            events: EventStream::new(),
        }
    }

    pub fn events(&self) -> &[GoalTagAssignment] {
        self.events.all()
    }

    pub fn new_events(&self) -> &[GoalTagAssignment] {
        self.events.new_events()
    }
}

impl AerialGoalCalculator {
    pub fn update(&mut self, match_stats: &MatchStatsCalculator) -> SubtrActorResult<()> {
        self.events
            .replace_all_assuming_append_only(self.tag_goals(match_stats.goal_context_events()));
        Ok(())
    }

    fn tag_goals(&self, goals: &[GoalContextEvent]) -> Vec<GoalTagAssignment> {
        tag_goals_by_height(goals, GoalTagKind::AerialGoal, self.config.min_ball_z)
    }
}

impl HighAerialGoalCalculator {
    pub fn update(
        &mut self,
        match_stats: &MatchStatsCalculator,
        touch: &TouchCalculator,
        possession: &PossessionCalculator,
    ) -> SubtrActorResult<()> {
        self.events.replace_all_assuming_append_only(self.tag_goals(
            match_stats.goal_context_events(),
            touch.events(),
            &possession.projected_events(),
        ));
        Ok(())
    }

    fn tag_goals(
        &self,
        goals: &[GoalContextEvent],
        touch_events: &[TouchClassificationEvent],
        possession_events: &[PossessionEvent],
    ) -> Vec<GoalTagAssignment> {
        tag_goals_by_possession_touch_height(
            goals,
            touch_events,
            possession_events,
            GoalTagKind::HighAerialGoal,
            self.config.min_ball_z,
        )
    }
}

impl LongDistanceGoalCalculator {
    pub fn update(&mut self, match_stats: &MatchStatsCalculator) -> SubtrActorResult<()> {
        self.events
            .replace_all_assuming_append_only(self.tag_goals(match_stats.goal_context_events()));
        Ok(())
    }

    fn tag_goals(&self, goals: &[GoalContextEvent]) -> Vec<GoalTagAssignment> {
        tag_goals_by_attacking_y(
            goals,
            GoalTagKind::LongDistanceGoal,
            self.config.max_attacking_y,
        )
    }
}

impl OwnHalfGoalCalculator {
    pub fn update(&mut self, match_stats: &MatchStatsCalculator) -> SubtrActorResult<()> {
        self.events
            .replace_all_assuming_append_only(self.tag_goals(match_stats.goal_context_events()));
        Ok(())
    }

    fn tag_goals(&self, goals: &[GoalContextEvent]) -> Vec<GoalTagAssignment> {
        tag_goals_by_recent_attacking_y(
            goals,
            GoalTagKind::OwnHalfGoal,
            self.config.max_attacking_y,
            OWN_HALF_GOAL_MAX_TOUCH_TO_GOAL_SECONDS,
        )
    }
}

impl EmptyNetGoalCalculator {
    pub fn update(&mut self, match_stats: &MatchStatsCalculator) -> SubtrActorResult<()> {
        self.events
            .replace_all_assuming_append_only(self.tag_goals(match_stats.goal_context_events()));
        Ok(())
    }

    fn tag_goals(&self, goals: &[GoalContextEvent]) -> Vec<GoalTagAssignment> {
        let mut tags = Vec::new();
        for (goal_index, goal) in goals.iter().enumerate() {
            let ctx = GoalTaggingContext { goal_index };
            let Some(touch) = goal.scorer_last_touch.as_ref() else {
                continue;
            };
            let Some(ball_position) = touch.ball_position else {
                continue;
            };
            let ball = position_to_vec(ball_position);
            let touch_attacking_y = normalized_y(goal.scoring_team_is_team_0, ball);
            if touch_attacking_y > self.config.max_touch_attacking_y {
                continue;
            }

            let player_contexts = if touch.players.is_empty() {
                &goal.players
            } else {
                &touch.players
            };

            let defenders: Vec<_> = player_contexts
                .iter()
                .filter(|player| player.is_team_0 != goal.scoring_team_is_team_0)
                .filter_map(|player| {
                    player
                        .position
                        .map(|position| (player, position_to_vec(position)))
                })
                .collect();
            if defenders.is_empty() {
                continue;
            }

            let mut closest_defender_distance = f32::INFINITY;
            let mut smallest_y_margin = f32::INFINITY;
            let mut evidence = vec![last_touch_evidence(touch), goal_context_evidence(goal)];

            for (defender, position) in defenders {
                closest_defender_distance = closest_defender_distance.min(position.distance(ball));
                let defender_attacking_y = normalized_y(goal.scoring_team_is_team_0, position);
                let y_margin = touch_attacking_y - defender_attacking_y;
                smallest_y_margin = smallest_y_margin.min(y_margin);
                evidence.push(defender_evidence(defender, goal));
            }

            if smallest_y_margin < self.config.min_defender_y_margin {
                continue;
            }
            if closest_defender_distance < self.config.min_defender_distance {
                continue;
            }

            tags.push(goal_tag(ctx, GoalTagKind::EmptyNetGoal, 1.0, evidence));
        }
        tags
    }
}

impl CounterAttackGoalCalculator {
    pub fn update(&mut self, match_stats: &MatchStatsCalculator) -> SubtrActorResult<()> {
        self.events
            .replace_all_assuming_append_only(self.tag_goals(match_stats.goal_context_events()));
        Ok(())
    }

    fn tag_goals(&self, goals: &[GoalContextEvent]) -> Vec<GoalTagAssignment> {
        goals
            .iter()
            .enumerate()
            .filter(|(_, goal)| goal.goal_buildup == GoalBuildupKind::CounterAttack)
            .map(|(goal_index, goal)| {
                goal_tag(
                    GoalTaggingContext { goal_index },
                    GoalTagKind::CounterAttackGoal,
                    1.0,
                    vec![goal_buildup_evidence(goal), goal_context_evidence(goal)],
                )
            })
            .collect()
    }
}

impl SustainedPressureGoalCalculator {
    pub fn update(&mut self, match_stats: &MatchStatsCalculator) -> SubtrActorResult<()> {
        self.events
            .replace_all_assuming_append_only(self.tag_goals(match_stats.goal_context_events()));
        Ok(())
    }

    fn tag_goals(&self, goals: &[GoalContextEvent]) -> Vec<GoalTagAssignment> {
        goals
            .iter()
            .enumerate()
            .filter(|(_, goal)| goal.goal_buildup == GoalBuildupKind::SustainedPressure)
            .map(|(goal_index, goal)| {
                goal_tag(
                    GoalTaggingContext { goal_index },
                    GoalTagKind::SustainedPressureGoal,
                    1.0,
                    vec![goal_buildup_evidence(goal), goal_context_evidence(goal)],
                )
            })
            .collect()
    }
}

impl FlickGoalCalculator {
    pub fn update(
        &mut self,
        match_stats: &MatchStatsCalculator,
        flick: &FlickCalculator,
    ) -> SubtrActorResult<()> {
        self.events.replace_all_assuming_append_only(
            self.tag_goals(match_stats.goal_context_events(), flick.events()),
        );
        Ok(())
    }

    fn tag_goals(
        &self,
        goals: &[GoalContextEvent],
        events: &[FlickEvent],
    ) -> Vec<GoalTagAssignment> {
        tag_goals_by_point_mechanic_event(
            goals,
            events,
            GoalTagKind::FlickGoal,
            self.config.max_event_to_goal_seconds,
        )
    }
}

impl CeilingShotGoalCalculator {
    pub fn update(
        &mut self,
        match_stats: &MatchStatsCalculator,
        ceiling_shot: &CeilingShotCalculator,
    ) -> SubtrActorResult<()> {
        self.events.replace_all_assuming_append_only(
            self.tag_goals(match_stats.goal_context_events(), ceiling_shot.events()),
        );
        Ok(())
    }

    fn tag_goals(
        &self,
        goals: &[GoalContextEvent],
        events: &[CeilingShotEvent],
    ) -> Vec<GoalTagAssignment> {
        tag_goals_by_point_mechanic_event(
            goals,
            events,
            GoalTagKind::CeilingShotGoal,
            self.config.max_event_to_goal_seconds,
        )
    }
}

impl OneTimerGoalCalculator {
    pub fn update(
        &mut self,
        match_stats: &MatchStatsCalculator,
        one_timer: &OneTimerCalculator,
    ) -> SubtrActorResult<()> {
        self.events.replace_all_assuming_append_only(
            self.tag_goals(match_stats.goal_context_events(), one_timer.events()),
        );
        Ok(())
    }

    fn tag_goals(
        &self,
        goals: &[GoalContextEvent],
        events: &[OneTimerEvent],
    ) -> Vec<GoalTagAssignment> {
        tag_goals_by_point_mechanic_event(
            goals,
            events,
            GoalTagKind::OneTimerGoal,
            self.config.max_event_to_goal_seconds,
        )
    }
}

impl PassingGoalCalculator {
    pub fn update(
        &mut self,
        match_stats: &MatchStatsCalculator,
        pass: &PassCalculator,
    ) -> SubtrActorResult<()> {
        self.events.replace_all_assuming_append_only(
            self.tag_goals(match_stats.goal_context_events(), pass.events()),
        );
        Ok(())
    }

    fn tag_goals(
        &self,
        goals: &[GoalContextEvent],
        events: &[PassEvent],
    ) -> Vec<GoalTagAssignment> {
        let mut tags = Vec::new();
        for (goal_index, goal) in goals.iter().enumerate() {
            let ctx = GoalTaggingContext { goal_index };
            let Some((event_index, event)) = events
                .iter()
                .enumerate()
                .filter(|(_, event)| pass_event_matches_goal(event, goal))
                .filter(|(_, event)| goal.time - event.time <= self.config.max_pass_to_goal_seconds)
                .max_by(|left, right| {
                    left.1
                        .time
                        .total_cmp(&right.1.time)
                        .then_with(|| left.1.frame.cmp(&right.1.frame))
                })
            else {
                continue;
            };

            tags.push(mechanic_goal_tag(
                ctx,
                GoalTagKind::PassingGoal,
                1.0,
                mechanic_goal_performer(goal, &event.receiver),
                mechanic_goal_modifiers(goal, &event.receiver),
                mechanic_goal_evidence(goal, pass_evidence(event)),
                vec![GoalTagEventRef {
                    stream: GoalTagEventStream::Pass,
                    index: event_index,
                }],
                Vec::new(),
            ));
        }
        tags
    }
}

impl DoubleTapGoalCalculator {
    pub fn update(
        &mut self,
        match_stats: &MatchStatsCalculator,
        double_tap: &DoubleTapCalculator,
    ) -> SubtrActorResult<()> {
        self.events.replace_all_assuming_append_only(
            self.tag_goals(match_stats.goal_context_events(), double_tap.events()),
        );
        Ok(())
    }

    fn tag_goals(
        &self,
        goals: &[GoalContextEvent],
        events: &[DoubleTapEvent],
    ) -> Vec<GoalTagAssignment> {
        tag_goals_by_point_mechanic_event(
            goals,
            events,
            GoalTagKind::DoubleTapGoal,
            self.config.max_event_to_goal_seconds,
        )
    }
}

impl AirDribbleGoalCalculator {
    pub fn update(
        &mut self,
        match_stats: &MatchStatsCalculator,
        ball_carry: &BallCarryCalculator,
    ) -> SubtrActorResult<()> {
        self.events.replace_all_assuming_append_only(
            self.tag_goals(match_stats.goal_context_events(), ball_carry.carry_events()),
        );
        Ok(())
    }

    fn tag_goals(
        &self,
        goals: &[GoalContextEvent],
        events: &[BallCarryEvent],
    ) -> Vec<GoalTagAssignment> {
        tag_goals_by_air_dribble_event(goals, events, self.config.max_end_to_goal_seconds)
    }
}

impl FlipResetGoalCalculator {
    pub fn update(
        &mut self,
        match_stats: &MatchStatsCalculator,
        dodge_reset: &DodgeResetCalculator,
        touch: &TouchCalculator,
    ) -> SubtrActorResult<()> {
        self.events.replace_all_assuming_append_only(self.tag_goals(
            match_stats.goal_context_events(),
            dodge_reset.confirmed_flip_reset_events(),
            dodge_reset.events(),
            touch.events(),
        ));
        Ok(())
    }

    fn tag_goals(
        &self,
        goals: &[GoalContextEvent],
        flip_reset_events: &[FlipResetEvent],
        dodge_reset_events: &[DodgeResetEvent],
        touch_events: &[TouchClassificationEvent],
    ) -> Vec<GoalTagAssignment> {
        let confirmed_tags = tag_goals_by_point_mechanic_event(
            goals,
            flip_reset_events,
            GoalTagKind::FlipResetGoal,
            self.config.max_event_to_goal_seconds,
        );
        let direct_reset_tags =
            self.tag_goals_by_scoring_reset_touch(goals, dodge_reset_events, touch_events);
        latest_goal_tag_assignments(&[&confirmed_tags, &direct_reset_tags])
    }

    fn tag_goals_by_scoring_reset_touch(
        &self,
        goals: &[GoalContextEvent],
        dodge_reset_events: &[DodgeResetEvent],
        touch_events: &[TouchClassificationEvent],
    ) -> Vec<GoalTagAssignment> {
        let mut tags = Vec::new();
        for (goal_index, goal) in goals.iter().enumerate() {
            let ctx = GoalTaggingContext { goal_index };
            let Some((event_index, event, touch_index, touch_event)) = dodge_reset_events
                .iter()
                .enumerate()
                .filter_map(|(event_index, event)| {
                    let (touch_index, touch_event) = touch_events
                        .iter()
                        .enumerate()
                        .filter(|(_, touch_event)| {
                            scoring_dodge_touch_matches_on_ball_reset_goal(
                                touch_event,
                                event,
                                goal,
                                self.config.max_event_to_goal_seconds,
                            )
                        })
                        .max_by(|left, right| {
                            left.1
                                .time
                                .total_cmp(&right.1.time)
                                .then_with(|| left.1.frame.cmp(&right.1.frame))
                        })?;
                    Some((event_index, event, touch_index, touch_event))
                })
                .max_by(|left, right| {
                    left.1
                        .time
                        .total_cmp(&right.1.time)
                        .then_with(|| left.1.frame.cmp(&right.1.frame))
                })
            else {
                continue;
            };

            let mut evidence = mechanic_goal_evidence(goal, dodge_reset_evidence(event));
            evidence.push(flip_into_ball_evidence(touch_event));

            tags.push(mechanic_goal_tag(
                ctx,
                GoalTagKind::FlipResetGoal,
                1.0,
                mechanic_goal_performer(goal, &event.player),
                mechanic_goal_modifiers(goal, &event.player),
                evidence,
                vec![
                    GoalTagEventRef {
                        stream: GoalTagEventStream::DodgeReset,
                        index: event_index,
                    },
                    GoalTagEventRef {
                        stream: GoalTagEventStream::Touch,
                        index: touch_index,
                    },
                ],
                Vec::new(),
            ));
        }
        tags
    }
}

impl FlipIntoBallGoalCalculator {
    pub fn update(
        &mut self,
        match_stats: &MatchStatsCalculator,
        touch: &TouchCalculator,
    ) -> SubtrActorResult<()> {
        self.events.replace_all_assuming_append_only(
            self.tag_goals(match_stats.goal_context_events(), touch.events()),
        );
        Ok(())
    }

    fn tag_goals(
        &self,
        goals: &[GoalContextEvent],
        touch_events: &[TouchClassificationEvent],
    ) -> Vec<GoalTagAssignment> {
        let mut tags = Vec::new();
        for (goal_index, goal) in goals.iter().enumerate() {
            let ctx = GoalTaggingContext { goal_index };
            let Some((event_index, event)) =
                self.scoring_touch_dodge_classification(goal, touch_events)
            else {
                continue;
            };

            tags.push(mechanic_goal_tag(
                ctx,
                GoalTagKind::FlipIntoBallGoal,
                1.0,
                mechanic_goal_performer(goal, &event.player),
                mechanic_goal_modifiers(goal, &event.player),
                mechanic_goal_evidence(goal, flip_into_ball_evidence(event)),
                vec![GoalTagEventRef {
                    stream: GoalTagEventStream::Touch,
                    index: event_index,
                }],
                Vec::new(),
            ));
        }
        tags
    }

    fn scoring_touch_dodge_classification<'a>(
        &self,
        goal: &GoalContextEvent,
        touch_events: &'a [TouchClassificationEvent],
    ) -> Option<(usize, &'a TouchClassificationEvent)> {
        let touch = goal.scorer_last_touch.as_ref()?;
        if goal.time - touch.time > self.config.max_touch_to_goal_seconds {
            return None;
        }
        touch_events
            .iter()
            .enumerate()
            .filter(|(_, event)| scoring_touch_is_dodge(event, touch, goal))
            .max_by(|left, right| {
                left.1
                    .time
                    .total_cmp(&right.1.time)
                    .then_with(|| left.1.frame.cmp(&right.1.frame))
            })
    }
}

impl BumpGoalCalculator {
    pub fn update(
        &mut self,
        match_stats: &MatchStatsCalculator,
        bump: &BumpCalculator,
    ) -> SubtrActorResult<()> {
        self.events.replace_all_assuming_append_only(
            self.tag_goals(match_stats.goal_context_events(), bump.events()),
        );
        Ok(())
    }

    fn tag_goals(
        &self,
        goals: &[GoalContextEvent],
        bump_events: &[BumpEvent],
    ) -> Vec<GoalTagAssignment> {
        let mut tags = Vec::new();
        for (goal_index, goal) in goals.iter().enumerate() {
            let ctx = GoalTaggingContext { goal_index };
            let Some((event_index, event)) = bump_events
                .iter()
                .enumerate()
                .filter(|(_, event)| bump_event_matches_goal(event, goal))
                .filter(|(_, event)| {
                    goal.time - event.time <= self.config.max_event_to_goal_seconds
                })
                .max_by(|left, right| {
                    left.1
                        .time
                        .total_cmp(&right.1.time)
                        .then_with(|| left.1.frame.cmp(&right.1.frame))
                })
            else {
                continue;
            };

            tags.push(mechanic_goal_tag(
                ctx,
                GoalTagKind::BumpGoal,
                event.confidence,
                mechanic_goal_performer(goal, &event.initiator),
                mechanic_goal_modifiers(goal, &event.initiator),
                mechanic_goal_evidence(goal, bump_evidence(event)),
                vec![GoalTagEventRef {
                    stream: GoalTagEventStream::Bump,
                    index: event_index,
                }],
                Vec::new(),
            ));
        }
        tags
    }
}

impl DemoGoalCalculator {
    pub fn update(
        &mut self,
        match_stats: &MatchStatsCalculator,
        demo: &DemoCalculator,
    ) -> SubtrActorResult<()> {
        self.events.replace_all_assuming_append_only(
            self.tag_goals(match_stats.goal_context_events(), demo.events()),
        );
        Ok(())
    }

    fn tag_goals(
        &self,
        goals: &[GoalContextEvent],
        demo_events: &[DemolitionEvent],
    ) -> Vec<GoalTagAssignment> {
        let mut tags = Vec::new();
        for (goal_index, goal) in goals.iter().enumerate() {
            let ctx = GoalTaggingContext { goal_index };
            let Some((event_index, event)) = demo_events
                .iter()
                .enumerate()
                .filter(|(_, event)| demo_event_matches_goal(event, goal))
                .filter(|(_, event)| {
                    goal.time - event.time <= self.config.max_event_to_goal_seconds
                })
                .max_by(|left, right| {
                    left.1
                        .time
                        .total_cmp(&right.1.time)
                        .then_with(|| left.1.frame.cmp(&right.1.frame))
                })
            else {
                continue;
            };

            let attacker = &event.attacker;

            tags.push(mechanic_goal_tag(
                ctx,
                GoalTagKind::DemoGoal,
                1.0,
                mechanic_goal_performer(goal, attacker),
                mechanic_goal_modifiers(goal, attacker),
                mechanic_goal_evidence(goal, demo_evidence(event)),
                vec![GoalTagEventRef {
                    stream: GoalTagEventStream::Demo,
                    index: event_index,
                }],
                Vec::new(),
            ));
        }
        tags
    }
}

impl HalfVolleyGoalCalculator {
    pub fn update(
        &mut self,
        match_stats: &MatchStatsCalculator,
        half_volley: &HalfVolleyCalculator,
    ) -> SubtrActorResult<()> {
        self.events.replace_all_assuming_append_only(
            self.tag_goals(match_stats.goal_context_events(), half_volley.events()),
        );
        Ok(())
    }

    fn tag_goals(
        &self,
        goals: &[GoalContextEvent],
        half_volley_events: &[HalfVolleyEvent],
    ) -> Vec<GoalTagAssignment> {
        let mut tags = Vec::new();
        for (goal_index, goal) in goals.iter().enumerate() {
            let ctx = GoalTaggingContext { goal_index };
            let Some((candidate_index, candidate)) =
                self.tag_goals_by_half_volley_event(goal, half_volley_events)
            else {
                continue;
            };

            tags.push(mechanic_goal_tag(
                ctx,
                GoalTagKind::HalfVolleyGoal,
                1.0,
                mechanic_goal_performer(goal, &candidate.player),
                mechanic_goal_modifiers(goal, &candidate.player),
                mechanic_goal_evidence(goal, half_volley_evidence(candidate)),
                vec![GoalTagEventRef {
                    stream: GoalTagEventStream::HalfVolley,
                    index: candidate_index,
                }],
                Vec::new(),
            ));
        }
        tags
    }

    fn tag_goals_by_half_volley_event<'a>(
        &self,
        goal: &GoalContextEvent,
        half_volley_events: &'a [HalfVolleyEvent],
    ) -> Option<(usize, &'a HalfVolleyEvent)> {
        half_volley_events
            .iter()
            .enumerate()
            .filter(|(_, candidate)| self.candidate_matches_goal(candidate, goal))
            .max_by(|left, right| {
                left.1
                    .time
                    .total_cmp(&right.1.time)
                    .then_with(|| left.1.frame.cmp(&right.1.frame))
            })
    }

    fn candidate_matches_goal(&self, candidate: &HalfVolleyEvent, goal: &GoalContextEvent) -> bool {
        const MAX_EVENT_AFTER_GOAL_SECONDS: f32 = 0.05;

        if candidate.is_team_0 != goal.scoring_team_is_team_0
            || candidate.time > goal.time + MAX_EVENT_AFTER_GOAL_SECONDS
            || candidate.frame > goal.frame
            || goal.time - candidate.time > self.config.max_touch_to_goal_seconds
            || candidate.goal_alignment < self.config.min_goal_alignment
        {
            return false;
        }

        let Some(touch) = goal.scorer_last_touch.as_ref() else {
            return false;
        };
        touch.player == candidate.player && touch.frame == candidate.frame
    }
}

impl KickoffGoalCalculator {
    pub fn update(
        &mut self,
        match_stats: &MatchStatsCalculator,
        kickoff: &KickoffCalculator,
    ) -> SubtrActorResult<()> {
        self.events.replace_all_assuming_append_only(
            self.tag_goals(match_stats.goal_context_events(), kickoff.events()),
        );
        Ok(())
    }

    /// The kickoff calculator is the source of truth for what counts as a
    /// kickoff goal (timing window, unbroken possession chain, and
    /// field-position gates); this tags exactly the goals some kickoff event
    /// attributed to itself.
    fn tag_goals(
        &self,
        goals: &[GoalContextEvent],
        kickoff_events: &[KickoffEvent],
    ) -> Vec<GoalTagAssignment> {
        goals
            .iter()
            .enumerate()
            .filter(|(_, goal)| {
                kickoff_events
                    .iter()
                    .any(|event| kickoff_event_attributes_goal(event, goal))
            })
            .map(|(goal_index, goal)| {
                goal_tag(
                    GoalTaggingContext { goal_index },
                    GoalTagKind::KickoffGoal,
                    1.0,
                    vec![goal_context_evidence(goal)],
                )
            })
            .collect()
    }
}

fn kickoff_event_attributes_goal(event: &KickoffEvent, goal: &GoalContextEvent) -> bool {
    if !event.kickoff_goal || event.scoring_team_is_team_0 != Some(goal.scoring_team_is_team_0) {
        return false;
    }
    match (event.first_touch_time, event.time_to_goal) {
        (Some(first_touch_time), Some(time_to_goal)) => {
            (first_touch_time + time_to_goal - goal.time).abs()
                <= KICKOFF_GOAL_TAG_MATCH_EPSILON_SECONDS
        }
        _ => false,
    }
}

trait GoalMechanicPointEvent {
    fn event_time(&self) -> f32;
    fn event_frame(&self) -> usize;
    fn event_player(&self) -> &PlayerId;
    fn event_team_is_team_0(&self) -> bool;
    fn event_confidence(&self) -> f32;
    fn evidence_kind(&self) -> GoalTagEvidenceKind;
    fn event_stream(&self) -> GoalTagEventStream;

    fn event_ref(&self, index: usize) -> GoalTagEventRef {
        GoalTagEventRef {
            stream: self.event_stream(),
            index,
        }
    }

    /// Categorical descriptors copied onto the goal tag so consumers can show
    /// mechanic flavor (e.g. reverse flick) without resolving `related_events`.
    fn goal_tag_details(&self) -> Vec<GoalTagDetail> {
        Vec::new()
    }
}

impl GoalMechanicPointEvent for FlickEvent {
    fn event_time(&self) -> f32 {
        self.time
    }

    fn event_frame(&self) -> usize {
        self.frame
    }

    fn event_player(&self) -> &PlayerId {
        &self.player
    }

    fn event_team_is_team_0(&self) -> bool {
        self.is_team_0
    }

    fn event_confidence(&self) -> f32 {
        self.confidence
    }

    fn evidence_kind(&self) -> GoalTagEvidenceKind {
        GoalTagEvidenceKind::Flick
    }

    fn event_stream(&self) -> GoalTagEventStream {
        GoalTagEventStream::Flick
    }

    fn goal_tag_details(&self) -> Vec<GoalTagDetail> {
        let mut details = vec![GoalTagDetail {
            key: "kind".to_owned(),
            value: self.kind.clone(),
        }];
        if self.direction != FlickDirection::Center.as_label_value() {
            details.push(GoalTagDetail {
                key: "direction".to_owned(),
                value: self.direction.clone(),
            });
        }
        details
    }
}

impl GoalMechanicPointEvent for CeilingShotEvent {
    fn event_time(&self) -> f32 {
        self.time
    }

    fn event_frame(&self) -> usize {
        self.frame
    }

    fn event_player(&self) -> &PlayerId {
        &self.player
    }

    fn event_team_is_team_0(&self) -> bool {
        self.is_team_0
    }

    fn event_confidence(&self) -> f32 {
        self.confidence
    }

    fn evidence_kind(&self) -> GoalTagEvidenceKind {
        GoalTagEvidenceKind::CeilingShot
    }

    fn event_stream(&self) -> GoalTagEventStream {
        GoalTagEventStream::CeilingShot
    }
}

impl GoalMechanicPointEvent for OneTimerEvent {
    fn event_time(&self) -> f32 {
        self.time
    }

    fn event_frame(&self) -> usize {
        self.frame
    }

    fn event_player(&self) -> &PlayerId {
        &self.player
    }

    fn event_team_is_team_0(&self) -> bool {
        self.is_team_0
    }

    fn event_confidence(&self) -> f32 {
        1.0
    }

    fn evidence_kind(&self) -> GoalTagEvidenceKind {
        GoalTagEvidenceKind::OneTimer
    }

    fn event_stream(&self) -> GoalTagEventStream {
        GoalTagEventStream::OneTimer
    }
}

impl GoalMechanicPointEvent for DoubleTapEvent {
    fn event_time(&self) -> f32 {
        self.time
    }

    fn event_frame(&self) -> usize {
        self.frame
    }

    fn event_player(&self) -> &PlayerId {
        &self.player
    }

    fn event_team_is_team_0(&self) -> bool {
        self.is_team_0
    }

    fn event_confidence(&self) -> f32 {
        1.0
    }

    fn evidence_kind(&self) -> GoalTagEvidenceKind {
        GoalTagEvidenceKind::DoubleTap
    }

    fn event_stream(&self) -> GoalTagEventStream {
        GoalTagEventStream::DoubleTap
    }
}

impl GoalMechanicPointEvent for FlipResetEvent {
    fn event_time(&self) -> f32 {
        self.time
    }

    fn event_frame(&self) -> usize {
        self.frame
    }

    fn event_player(&self) -> &PlayerId {
        &self.player
    }

    fn event_team_is_team_0(&self) -> bool {
        self.is_team_0
    }

    fn event_confidence(&self) -> f32 {
        1.0
    }

    fn evidence_kind(&self) -> GoalTagEvidenceKind {
        GoalTagEvidenceKind::FlipReset
    }

    fn event_stream(&self) -> GoalTagEventStream {
        GoalTagEventStream::FlipReset
    }
}

pub fn combined_goal_tag_assignments(
    calculators: &[&[GoalTagAssignment]],
) -> Vec<GoalTagAssignment> {
    let mut assignments: Vec<_> = calculators
        .iter()
        .flat_map(|events| events.iter().cloned())
        .collect();
    assignments.sort_by(|left, right| {
        left.goal_index
            .cmp(&right.goal_index)
            .then_with(|| format!("{:?}", left.tag.kind()).cmp(&format!("{:?}", right.tag.kind())))
    });
    assignments
}

pub fn goal_context_events_with_tags(
    goals: &[GoalContextEvent],
    assignments: &[GoalTagAssignment],
) -> Vec<GoalContextEvent> {
    let mut goals_with_tags = goals.to_vec();
    for assignment in assignments {
        let Some(goal) = goals_with_tags.get_mut(assignment.goal_index) else {
            continue;
        };
        goal.tags.push(assignment.tag.clone());
    }
    for goal in &mut goals_with_tags {
        goal.tags.sort_by(|left, right| {
            format!("{:?}", left.kind())
                .cmp(&format!("{:?}", right.kind()))
                .then_with(|| {
                    right
                        .metadata()
                        .confidence
                        .total_cmp(&left.metadata().confidence)
                })
        });
    }
    goals_with_tags
}

#[cfg(test)]
#[path = "goal_tags_tests.rs"]
mod tests;
