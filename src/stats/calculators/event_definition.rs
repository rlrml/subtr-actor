#![cfg_attr(target_arch = "wasm32", allow(dead_code))]

use serde::Serialize;
use ts_rs::TS;

#[cfg(not(target_arch = "wasm32"))]
use linkme::distributed_slice;

use super::{
    BackboardBounceEvent, BallCarryEvent, BallDepthEvent, BallHalfEvent, BallProximityEvent,
    BoostPickupEvent, BumpEvent, CeilingShotEvent, CenterEvent, ControlledPlayEvent,
    CorePlayerScoreboardEvent, DepthRoleEvent, DodgeEvent, DodgeResetEvent, DoubleTapEvent,
    FieldHalfEvent, FieldThirdEvent, FiftyFiftyEvent, FirstManChangeEvent, FlickEvent,
    FlipResetEvent, HalfFlipEvent, HalfVolleyEvent, MovementEvent, MustyFlickEvent, OneTimerEvent,
    PassEvent, PlayerActivityEvent, PlayerPossessionEvent, PossessionEvent, PowerslideEvent,
    RespawnEvent, RotationRoleEvent, RushEvent, SpeedFlipEvent, TerritorialPressureEvent,
    TimelineEvent, TouchClassificationEvent, WallAerialEvent, WallAerialShotEvent, WavedashEvent,
    WhiffEvent,
};
use crate::stats::timeline::Event;

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
    /// When true this definition is a label-like or expansion-parent row that
    /// should not be offered as a selectable event type in the review UI.
    pub hidden_from_review: bool,
    /// Concrete event-type keys this definition expands into at serialization
    /// time (e.g. `boost_ledger` -> `boost_ledger_collected`). Expansion parents
    /// are typically also `hidden_from_review`; their variants are surfaced
    /// instead. Empty for ordinary events.
    pub variants: &'static [EventVariant],
}

impl EventDefinition {
    /// Set whether this definition is hidden from the review picker. Named to
    /// double as a `define_stats_event!` modifier (`hidden = true`).
    pub const fn hidden(self, hidden: bool) -> Self {
        let mut def = self;
        def.hidden_from_review = hidden;
        def
    }

    /// Attach the concrete variant keys this definition expands into. Named to
    /// double as a `define_stats_event!` modifier (`variants = SLICE`).
    pub const fn variants(self, variants: &'static [EventVariant]) -> Self {
        let mut def = self;
        def.variants = variants;
        def
    }
}

/// A concrete event-type key produced by expanding a parent [`EventDefinition`]
/// (for example a boost-ledger transaction or a rotation role/depth state).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct EventVariant {
    pub key: &'static str,
    pub label: &'static str,
    pub category: EventCategory,
}

impl EventVariant {
    pub const fn new(key: &'static str, label: &'static str, category: EventCategory) -> Self {
        Self {
            key,
            label,
            category,
        }
    }
}

/// Coarse product/domain grouping for an event definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum EventCategory {
    Core,
    Mechanic,
    Positioning,
    Annotation,
    Other,
    /// Label-like metadata rows (e.g. goal context). These are hidden from the
    /// review picker by default via [`EventDefinition::hidden_from_review`].
    Context,
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
        hidden_from_review: false,
        variants: &[],
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

/// Distributed catalog of every [`EventDefinition`].
///
/// `define_stats_event!` (and `register_event_definition!` for payload-less
/// rows) register into this slice automatically, so defining an event is the
/// only step required for it to appear everywhere definitions are consumed —
/// there is no separate central list to keep in sync. Read it through
/// [`all_event_definitions`], which sorts and de-duplicates by `id`.
#[cfg(not(target_arch = "wasm32"))]
#[distributed_slice]
pub static EVENT_DEFINITIONS: [EventDefinition];

/// All registered event definitions, sorted by `id` and de-duplicated.
///
/// `linkme` does not guarantee registration order, so this sorts for stable
/// output and panics if two registrations share an `id` but disagree on
/// contents (a real double-registration bug rather than something to hide).
#[cfg(not(target_arch = "wasm32"))]
pub fn all_event_definitions() -> &'static [EventDefinition] {
    use std::sync::OnceLock;
    static SORTED: OnceLock<Vec<EventDefinition>> = OnceLock::new();
    SORTED.get_or_init(|| {
        let mut defs: Vec<EventDefinition> = EVENT_DEFINITIONS.iter().copied().collect();
        defs.sort_by(|left, right| left.id.cmp(right.id));
        let mut deduped: Vec<EventDefinition> = Vec::with_capacity(defs.len());
        for def in defs {
            match deduped.last() {
                Some(last) if last.id == def.id => {
                    assert!(
                        *last == def,
                        "conflicting EventDefinition registrations for id {:?}",
                        def.id
                    );
                }
                _ => deduped.push(def),
            }
        }
        deduped
    })
}

// `linkme` is unavailable on wasm32, so the registry and `all_event_definitions()`
// are host-only — there is intentionally no wasm fallback. The catalog is consumed
// only by host/server tooling and by the build-time TypeScript codegen
// (`event_definition_catalog()` + its export test); wasm/browser consumers use the
// generated TS catalog instead. A wasm caller referencing it is a compile error
// rather than a silently-empty list.

/// A variant entry in the TypeScript event catalog (owned, ts-rs-exportable).
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct EventVariantTs {
    pub key: String,
    pub label: String,
    pub category: EventCategory,
}

/// One entry in the TypeScript event catalog: the slim, viewer-relevant view of
/// an [`EventDefinition`] (id/label/category/hidden + expansion variants). The
/// browser viewer derives its event list from a generated array of these so it
/// can never drift from the Rust registry. Confidence/approach metadata is
/// intentionally omitted — it is host/docs-only.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct EventDefinitionCatalogEntry {
    pub key: String,
    pub label: String,
    pub category: EventCategory,
    pub hidden_from_review: bool,
    pub variants: Vec<EventVariantTs>,
}

/// Build the TypeScript-facing catalog from the registry. Sorted/de-duplicated by
/// id (inherited from [`all_event_definitions`]) so codegen output is stable.
#[cfg(not(target_arch = "wasm32"))]
pub fn event_definition_catalog() -> Vec<EventDefinitionCatalogEntry> {
    all_event_definitions()
        .iter()
        .map(|definition| EventDefinitionCatalogEntry {
            key: definition.id.to_owned(),
            label: definition.label.to_owned(),
            category: definition.category,
            hidden_from_review: definition.hidden_from_review,
            variants: definition
                .variants
                .iter()
                .map(|variant| EventVariantTs {
                    key: variant.key.to_owned(),
                    label: variant.label.to_owned(),
                    category: variant.category,
                })
                .collect(),
        })
        .collect()
}

/// Build-time codegen for the TypeScript event catalog data file. ts-rs only
/// generates *types*; this writes the *data* array next to them. Runs as part of
/// the `generate:stats-types` npm script via the `export_bindings` test filter,
/// writing to `$TS_RS_EXPORT_DIR` when set. Without the env var it still validates
/// serialization but writes nothing, so a plain `cargo test` never touches the tree.
#[cfg(test)]
#[test]
fn export_bindings_event_definition_catalog() {
    let catalog = event_definition_catalog();
    let json = serde_json::to_string_pretty(&catalog).expect("serialize event catalog");
    let contents = format!(
        "// This file was generated from the subtr-actor event-definition registry. \
Do not edit this file manually.\n\
import type {{ EventDefinitionCatalogEntry }} from \"./EventDefinitionCatalogEntry.ts\";\n\
\n\
export const EVENT_DEFINITION_CATALOG: EventDefinitionCatalogEntry[] = {json};\n"
    );

    if let Ok(dir) = std::env::var("TS_RS_EXPORT_DIR") {
        let path = std::path::Path::new(&dir).join("eventDefinitionCatalog.generated.ts");
        std::fs::write(&path, contents).expect("write event catalog data file");
    }
}

/// Register an already-declared `EventDefinition` const into the
/// [`EVENT_DEFINITIONS`] catalog. Used for payload-less rows (core scoreboard
/// stats, goal context, expansion fallbacks) that have no [`StatsEvent`] type.
macro_rules! register_stats_event_definition {
    ($definition:ident) => {
        paste::paste! {
            #[cfg(not(target_arch = "wasm32"))]
            #[distributed_slice(EVENT_DEFINITIONS)]
            static [<$definition _REGISTRATION>]: EventDefinition = $definition;
        }
    };
}

macro_rules! define_stats_event {
    (
        $event_type:ty,
        $definition:ident,
        $id:literal,
        $label:literal,
        $category:expr_2021,
        summary = $summary:literal,
        approach = [$($approach:literal),* $(,)?]
        $(, $modifier:ident = $modval:expr_2021)* $(,)?
    ) => {
        pub const $definition: EventDefinition =
            event_definition($id, $label, $category, $summary, &[$($approach),*])
                $(.$modifier($modval))*;

        impl StatsEvent for $event_type {
            const DEFINITION: EventDefinition = $definition;
        }

        register_stats_event_definition!($definition);
    };

    (
        $event_type:ty,
        $definition:ident,
        $id:literal,
        $label:literal,
        $category:expr_2021
        $(, $modifier:ident = $modval:expr_2021)* $(,)?
    ) => {
        pub const $definition: EventDefinition =
            pending_event_definition($id, $label, $category)
                $(.$modifier($modval))*;

        impl StatsEvent for $event_type {
            const DEFINITION: EventDefinition = $definition;
        }

        register_stats_event_definition!($definition);
    };
}

// Variant tables for expansion-parent definitions. Each parent is
// `hidden_from_review` and surfaces these concrete keys instead. The keys must
// match the ones serialized at runtime in the server's timeline expansion.
// All pickups surface under one key; the `detection` payload field
// (`both` | `inferred_only` | `reported_only`) records corroboration provenance and is a
// filter facet, not an event-type split.
const BOOST_PICKUP_VARIANTS: &[EventVariant] = &[EventVariant::new(
    "boost_pickup",
    "Boost Pickup",
    EventCategory::Other,
)];

// Payload-less event definitions: native Rocket League scoreboard stats, goal
// context labels, and the air-dribble mechanic kind. These have no `StatsEvent`
// payload type but still belong in the catalog so they surface in the review
// picker (or are explicitly hidden) without a separate hand-maintained list.
pub const ASSIST_EVENT_DEFINITION: EventDefinition =
    pending_event_definition("assist", "Assist", EventCategory::Core);
register_stats_event_definition!(ASSIST_EVENT_DEFINITION);

pub const DEATH_EVENT_DEFINITION: EventDefinition =
    pending_event_definition("death", "Death", EventCategory::Core);
register_stats_event_definition!(DEATH_EVENT_DEFINITION);

pub const GOAL_EVENT_DEFINITION: EventDefinition =
    pending_event_definition("goal", "Goal", EventCategory::Core);
register_stats_event_definition!(GOAL_EVENT_DEFINITION);

pub const KILL_EVENT_DEFINITION: EventDefinition =
    pending_event_definition("kill", "Demolition", EventCategory::Core);
register_stats_event_definition!(KILL_EVENT_DEFINITION);

pub const SAVE_EVENT_DEFINITION: EventDefinition =
    pending_event_definition("save", "Save", EventCategory::Core);
register_stats_event_definition!(SAVE_EVENT_DEFINITION);

pub const SHOT_EVENT_DEFINITION: EventDefinition =
    pending_event_definition("shot", "Shot", EventCategory::Core);
register_stats_event_definition!(SHOT_EVENT_DEFINITION);

pub const KICKOFF_EVENT_DEFINITION: EventDefinition =
    pending_event_definition("kickoff", "Kickoff", EventCategory::Core);
register_stats_event_definition!(KICKOFF_EVENT_DEFINITION);

pub const GOAL_CONTEXT_EVENT_DEFINITION: EventDefinition =
    pending_event_definition("goal_context", "Goal Context", EventCategory::Context).hidden(true);
register_stats_event_definition!(GOAL_CONTEXT_EVENT_DEFINITION);

pub const AIR_DRIBBLE_EVENT_DEFINITION: EventDefinition = event_definition(
    "air_dribble",
    "Air Dribble",
    EventCategory::Mechanic,
    "An airborne ball-control sequence where a player keeps the ball under control off the ground.",
    &[
        "Reuse the ball-carry sequence sampler's air-dribble carry kind, which tracks player-owned ball control while airborne.",
        "Surface the span when a completed ball-carry sequence is classified as an air dribble rather than a grounded carry.",
    ],
);
register_stats_event_definition!(AIR_DRIBBLE_EVENT_DEFINITION);

define_stats_event!(
    TimelineEvent,
    TIMELINE_EVENT_DEFINITION,
    "timeline",
    "Replay Timeline Event",
    EventCategory::Core,
    hidden = true
);
define_stats_event!(
    CorePlayerScoreboardEvent,
    CORE_PLAYER_SCOREBOARD_EVENT_DEFINITION,
    "core_player_scoreboard",
    "Core Player Scoreboard",
    EventCategory::Core,
    hidden = true
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
        "Measure signed horizontal setup rotation so reverse flicks can be labeled as left or right based on the direction the car rotated before the flick.",
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
    summary = "A frame-level dodge refresh observed from replay state, marked as occurring on the ball (a flip reset) and as used when later converted by a dodge-powered touch.",
    approach = [
        "Consume dodge-refreshed replay events and preserve the player, team, frame, time, and counter value.",
        "Classify the refresh as on-ball (a flip reset) when the player and ball are both airborne enough, close together, and the ball is positioned under the car in local space.",
        "Keep on-ball resets pending in an in-flight ledger; if the player dodges into the ball within the reset-to-touch window, mark the originating reset event `used` with its reset-to-use latency.",
        "Resolve every pending reset into an outcome: used, landed, superseded by a newer reset, expired, or cut off by a goal, live play ending, or the replay ending.",
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
    summary =
        "A fast receiver touch from a completed pass that is immediately directed toward goal.",
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
    BallCarryEvent,
    BALL_CARRY_EVENT_DEFINITION,
    "ball_carry",
    "Ball Carry",
    EventCategory::Mechanic,
    summary =
        "A sustained player-ball control sequence, covering grounded carries and air dribbles.",
    approach = [
        "Use continuous ball-control tracking to build player-owned sequences while live play is active.",
        "Sample grounded carries from close horizontal/vertical ball gaps over the car, excluding wall contact.",
        "Sample air dribbles with the air-dribble policy, then emit completed sequences that meet the duration and validity rules for their carry kind.",
    ]
);
define_stats_event!(
    ControlledPlayEvent,
    CONTROLLED_PLAY_EVENT_DEFINITION,
    "controlled_play",
    "Controlled Play",
    EventCategory::Mechanic,
    summary =
        "A same-player possession episode with multiple touches and sustained close-ball time.",
    approach = [
        "Start a player-owned candidate from an attributed touch during live play.",
        "Require at least two distinct touches by the same player with at least one second between the first and last touch.",
        "Require sustained proximity to the ball and finish the candidate when another player touches, live play ends, or the touch chain times out.",
    ]
);
define_stats_event!(
    FiftyFiftyEvent,
    FIFTY_FIFTY_EVENT_DEFINITION,
    "fifty_fifty",
    "50/50",
    EventCategory::Other,
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
    EventCategory::Other,
    summary = "A quick possession transition where the attacking team has numbers moving out of its defensive half.",
    approach = [
        "Start from a possession change when the ball is still in the new attacking team's defensive half.",
        "Count non-demoed attackers near or ahead of the ball and defenders between the ball and their own goal.",
        "Emit once the new attacking team retains possession long enough with at least two attackers and at least one defender in the rush shape.",
    ]
);
define_stats_event!(
    DodgeEvent,
    DODGE_EVENT_DEFINITION,
    "dodge",
    "Dodge",
    EventCategory::Mechanic,
    summary = "A dodge-start event, optionally carrying a rough estimated dodge impulse when the velocity change is measurable.",
    approach = [
        "Start on the replay's dodge-active rising edge for each player.",
        "Sample the player's velocity change over the early dodge window and subtract an approximate forward boost contribution when boost is active.",
        "Store the impulse estimate as dodge_impulse, including car-local direction classification plus raw and compensated world-space vectors for visualization and downstream mechanic detectors.",
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
    EventCategory::Other,
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
    EventCategory::Mechanic,
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
    EventCategory::Other,
    summary = "A classified ball touch with strength kind, surface/height context, and an inferred intention.",
    approach = [
        "Classify each touch's strength kind (control, medium hit, hard hit) from the ball speed change it produces.",
        "Record surface, height band, and dodge context for the touching player at contact time.",
        "Resolve a single mutually-exclusive intention by precedence: replay-confirmed saves and shots first, then contested challenges, then geometric save/shot trajectory projections, then clears out of the defensive third, then passes led toward a teammate, falling back to neutral.",
        "Retroactively upgrade pass/neutral touches to a control intention by outcome: the toucher stayed close to the ball while matching its velocity for most of a short follow window, or earned the follow-up touch themselves.",
        "Mark a touch as a first touch when it starts a new reception: the previous global touch was by a different player or far enough in the past.",
    ]
);
define_stats_event!(
    BoostPickupEvent,
    BOOST_PICKUP_EVENT_DEFINITION,
    "boost_pickups",
    "Boost Pickup",
    EventCategory::Other,
    hidden = true,
    variants = BOOST_PICKUP_VARIANTS
);
define_stats_event!(
    RespawnEvent,
    BOOST_RESPAWN_EVENT_DEFINITION,
    "boost_respawn",
    "Respawn",
    EventCategory::Other
);
define_stats_event!(
    BumpEvent,
    BUMP_EVENT_DEFINITION,
    "bump",
    "Bump",
    EventCategory::Other
);
define_stats_event!(
    PossessionEvent,
    POSSESSION_EVENT_DEFINITION,
    "possession",
    "Possession",
    EventCategory::Other
);
define_stats_event!(
    PlayerPossessionEvent,
    PLAYER_POSSESSION_EVENT_DEFINITION,
    "player_possession",
    "Player Possession",
    EventCategory::Other,
    summary = "A contiguous single-player possession span enriched with touch, ball-progress, and sustained-control activity.",
    approach = [
        "Follow the shared possession tracker's controlling player and open a span when a player establishes control.",
        "Bridge contested or pending-turnover interruptions shorter than the merge gap when the same player re-establishes control, excluding the gap from possessed duration.",
        "Accumulate distinct touches (with aerial/wall classification), signed ball travel toward the opponent goal, and per-frame carry/air-dribble samples while the span is active.",
    ]
);
define_stats_event!(
    BallHalfEvent,
    PRESSURE_EVENT_DEFINITION,
    "ball_half",
    "Ball Half",
    EventCategory::Other
);
define_stats_event!(
    TerritorialPressureEvent,
    TERRITORIAL_PRESSURE_EVENT_DEFINITION,
    "territorial_pressure",
    "Territorial Pressure",
    EventCategory::Other
);
define_stats_event!(
    MovementEvent,
    MOVEMENT_EVENT_DEFINITION,
    "movement",
    "Movement",
    EventCategory::Other
);
define_stats_event!(
    PlayerActivityEvent,
    PLAYER_ACTIVITY_EVENT_DEFINITION,
    "player_activity",
    "Player Activity",
    EventCategory::Positioning
);
define_stats_event!(
    FieldThirdEvent,
    FIELD_THIRD_EVENT_DEFINITION,
    "field_third",
    "Field Third",
    EventCategory::Positioning
);
define_stats_event!(
    FieldHalfEvent,
    FIELD_HALF_EVENT_DEFINITION,
    "field_half",
    "Field Half",
    EventCategory::Positioning
);
define_stats_event!(
    BallDepthEvent,
    BALL_DEPTH_EVENT_DEFINITION,
    "ball_depth",
    "Ball Depth",
    EventCategory::Positioning
);
define_stats_event!(
    DepthRoleEvent,
    DEPTH_ROLE_EVENT_DEFINITION,
    "depth_role",
    "Depth Role",
    EventCategory::Positioning
);
define_stats_event!(
    BallProximityEvent,
    BALL_PROXIMITY_EVENT_DEFINITION,
    "ball_proximity",
    "Ball Proximity",
    EventCategory::Positioning
);
define_stats_event!(
    RotationRoleEvent,
    ROTATION_ROLE_EVENT_DEFINITION,
    "rotation_role",
    "Rotation Role",
    EventCategory::Positioning
);
define_stats_event!(
    FirstManChangeEvent,
    FIRST_MAN_CHANGE_EVENT_DEFINITION,
    "first_man_change",
    "First-Man Change",
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
    Event,
    TIMELINE_ENVELOPE_EVENT_DEFINITION,
    "event",
    "Event",
    EventCategory::Mechanic,
    summary = "A shared event envelope with common metadata and a typed event payload.",
    approach = [
        "Collect completed events from the analysis graph at finish time.",
        "Wrap each typed event payload with common timing, participant, team, position, confidence, and stream metadata.",
        "Serialize timeline events as a single heterogeneous event list for playback and analysis consumers.",
    ]
);

// The former hand-maintained `ALL_EVENT_DEFINITIONS` array has been replaced by
// the auto-populated `EVENT_DEFINITIONS` distributed slice; read it through
// `all_event_definitions()`. Defining an event via `define_stats_event!` (or
// `register_stats_event_definition!`) is now the only registration step.

pub(crate) const MATCH_STATS_EMITTED_EVENTS: &[EmittedEvent] = &[
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
];

pub(crate) const DEMO_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &TIMELINE_EVENT_DEFINITION,
    "demo",
    "DemoNode",
    "DemoCalculator",
)];

pub(crate) const BACKBOARD_BOUNCE_STATE_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &BACKBOARD_BOUNCE_EVENT_DEFINITION,
    "backboard_bounce_state",
    "BackboardBounceStateNode",
    "BackboardBounceCalculator",
)];

pub(crate) const CEILING_SHOT_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &CEILING_SHOT_EVENT_DEFINITION,
    "ceiling_shot",
    "CeilingShotNode",
    "CeilingShotCalculator",
)];

pub(crate) const WALL_AERIAL_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &WALL_AERIAL_EVENT_DEFINITION,
    "wall_aerial",
    "WallAerialNode",
    "WallAerialCalculator",
)];

pub(crate) const WALL_AERIAL_SHOT_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &WALL_AERIAL_SHOT_EVENT_DEFINITION,
    "wall_aerial_shot",
    "WallAerialShotNode",
    "WallAerialShotCalculator",
)];

pub(crate) const CENTER_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &CENTER_EVENT_DEFINITION,
    "center",
    "CenterNode",
    "CenterCalculator",
)];

pub(crate) const FLICK_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &FLICK_EVENT_DEFINITION,
    "flick",
    "FlickNode",
    "FlickCalculator",
)];

pub(crate) const MUSTY_FLICK_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &MUSTY_FLICK_EVENT_DEFINITION,
    "musty_flick",
    "MustyFlickNode",
    "MustyFlickCalculator",
)];

pub(crate) const DODGE_RESET_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &DODGE_RESET_EVENT_DEFINITION,
    "dodge_reset",
    "DodgeResetNode",
    "DodgeResetCalculator",
)];

pub(crate) const DOUBLE_TAP_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &DOUBLE_TAP_EVENT_DEFINITION,
    "double_tap",
    "DoubleTapNode",
    "DoubleTapCalculator",
)];

pub(crate) const ONE_TIMER_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &ONE_TIMER_EVENT_DEFINITION,
    "one_timer",
    "OneTimerNode",
    "OneTimerCalculator",
)];

pub(crate) const PASS_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &PASS_EVENT_DEFINITION,
    "pass",
    "PassNode",
    "PassCalculator",
)];

pub(crate) const BALL_CARRY_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &BALL_CARRY_EVENT_DEFINITION,
    "ball_carry",
    "BallCarryNode",
    "BallCarryCalculator",
)];

pub(crate) const CONTROLLED_PLAY_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &CONTROLLED_PLAY_EVENT_DEFINITION,
    "controlled_play",
    "ControlledPlayNode",
    "ControlledPlayCalculator",
)];

pub(crate) const FIFTY_FIFTY_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &FIFTY_FIFTY_EVENT_DEFINITION,
    "fifty_fifty",
    "FiftyFiftyNode",
    "FiftyFiftyCalculator",
)];

pub(crate) const RUSH_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &RUSH_EVENT_DEFINITION,
    "rush",
    "RushNode",
    "RushCalculator",
)];

pub(crate) const DODGE_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &DODGE_EVENT_DEFINITION,
    "dodge",
    "FlipImpulseNode",
    "FlipImpulseCalculator",
)];

pub(crate) const SPEED_FLIP_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &SPEED_FLIP_EVENT_DEFINITION,
    "speed_flip",
    "SpeedFlipNode",
    "SpeedFlipCalculator",
)];

pub(crate) const HALF_FLIP_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &HALF_FLIP_EVENT_DEFINITION,
    "half_flip",
    "HalfFlipNode",
    "HalfFlipCalculator",
)];

pub(crate) const HALF_VOLLEY_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &HALF_VOLLEY_EVENT_DEFINITION,
    "half_volley",
    "HalfVolleyNode",
    "HalfVolleyCalculator",
)];

pub(crate) const WAVEDASH_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &WAVEDASH_EVENT_DEFINITION,
    "wavedash",
    "WavedashNode",
    "WavedashCalculator",
)];

pub(crate) const WHIFF_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &WHIFF_EVENT_DEFINITION,
    "whiff",
    "WhiffNode",
    "WhiffCalculator",
)];

pub(crate) const POWERSLIDE_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &POWERSLIDE_EVENT_DEFINITION,
    "powerslide",
    "PowerslideNode",
    "PowerslideCalculator",
)];

pub(crate) const TOUCH_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &TOUCH_CLASSIFICATION_EVENT_DEFINITION,
    "touch",
    "TouchNode",
    "TouchCalculator",
)];

pub(crate) const BOOST_EMITTED_EVENTS: &[EmittedEvent] = &[
    produced_event(
        &BOOST_PICKUP_EVENT_DEFINITION,
        "boost",
        "BoostNode",
        "BoostCalculator",
    ),
    produced_event(
        &BOOST_RESPAWN_EVENT_DEFINITION,
        "boost",
        "BoostNode",
        "BoostCalculator",
    ),
];

pub(crate) const BUMP_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &BUMP_EVENT_DEFINITION,
    "bump",
    "BumpNode",
    "BumpCalculator",
)];

pub(crate) const POSSESSION_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &POSSESSION_EVENT_DEFINITION,
    "possession",
    "PossessionNode",
    "PossessionCalculator",
)];

pub(crate) const PLAYER_POSSESSION_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &PLAYER_POSSESSION_EVENT_DEFINITION,
    "player_possession",
    "PlayerPossessionNode",
    "PlayerPossessionCalculator",
)];

pub(crate) const BALL_HALF_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &PRESSURE_EVENT_DEFINITION,
    "ball_half",
    "BallHalfNode",
    "BallHalfCalculator",
)];

pub(crate) const TERRITORIAL_BALL_HALF_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &TERRITORIAL_PRESSURE_EVENT_DEFINITION,
    "territorial_pressure",
    "TerritorialPressureNode",
    "TerritorialPressureCalculator",
)];

pub(crate) const MOVEMENT_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &MOVEMENT_EVENT_DEFINITION,
    "movement",
    "MovementNode",
    "MovementCalculator",
)];

pub(crate) const POSITIONING_EMITTED_EVENTS: &[EmittedEvent] = &[
    produced_event(
        &PLAYER_ACTIVITY_EVENT_DEFINITION,
        "positioning",
        "PositioningNode",
        "PositioningCalculator",
    ),
    produced_event(
        &FIELD_THIRD_EVENT_DEFINITION,
        "positioning",
        "PositioningNode",
        "PositioningCalculator",
    ),
    produced_event(
        &FIELD_HALF_EVENT_DEFINITION,
        "positioning",
        "PositioningNode",
        "PositioningCalculator",
    ),
    produced_event(
        &BALL_DEPTH_EVENT_DEFINITION,
        "positioning",
        "PositioningNode",
        "PositioningCalculator",
    ),
    produced_event(
        &DEPTH_ROLE_EVENT_DEFINITION,
        "positioning",
        "PositioningNode",
        "PositioningCalculator",
    ),
    produced_event(
        &BALL_PROXIMITY_EVENT_DEFINITION,
        "positioning",
        "PositioningNode",
        "PositioningCalculator",
    ),
];

pub(crate) const ROTATION_EMITTED_EVENTS: &[EmittedEvent] = &[
    produced_event(
        &ROTATION_ROLE_EVENT_DEFINITION,
        "rotation",
        "RotationNode",
        "RotationCalculator",
    ),
    produced_event(
        &FIRST_MAN_CHANGE_EVENT_DEFINITION,
        "rotation",
        "RotationNode",
        "RotationCalculator",
    ),
];

pub(crate) const STATS_TIMELINE_EVENTS_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &TIMELINE_ENVELOPE_EVENT_DEFINITION,
    "stats_timeline_events",
    "StatsTimelineEventsNode",
    "StatsTimelineEventsState",
)];

#[cfg(test)]
#[path = "event_definition_tests.rs"]
mod tests;
