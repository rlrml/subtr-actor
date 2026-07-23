#![cfg_attr(target_arch = "wasm32", allow(dead_code))]

use serde::Serialize;
use ts_rs::TS;

#[cfg(not(target_arch = "wasm32"))]
use linkme::distributed_slice;

use super::{
    BackboardBounceEvent, BallCarryEvent, BallDepthEvent, BallHalfEvent, BallProximityEvent,
    BallThirdEvent, BeatenToBallEvent, BoostPickupEvent, BumpEvent, CeilingShotEvent, CenterEvent,
    ControlledPlayEvent, CorePlayerScoreboardEvent, DemolitionEvent, DepthRoleEvent, DodgeEvent,
    DodgeResetEvent, DoubleTapEvent, FieldHalfEvent, FieldThirdEvent, FiftyFiftyEvent,
    FirstManChangeEvent, FlickEvent, FlipResetEvent, HalfFlipEvent, HalfVolleyEvent,
    LoosePossessionEvent, MovementEvent, OneTimerEvent, PassEvent, PlayerActivityEvent,
    PlayerPossessionEvent, PossessionEvent, PowerslideEvent, RespawnEvent, RotationRoleEvent,
    RushEvent, SpeedFlipEvent, TerritorialPressureEvent, TimelineEvent, TouchClassificationEvent,
    WallAerialEvent, WallAerialShotEvent, WavedashEvent, WhiffEvent,
};
use crate::ShadowDefenseEvent;
use crate::stats::timeline::{Event, EventPayload, EventScope};

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
    /// How a timeline client fans this event's stream out into lanes. `Match`
    /// (the default) keeps everything on one shared row; `Team`/`Player` are the
    /// opt-in exceptions that split into one lane per team or per player. Declared
    /// here so fan-out travels with the event type rather than a side table; the
    /// shipped `EventMeta.scope` reads from this via [`EventPayload::scope`].
    pub scope: EventScope,
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

    /// Set how this event's stream fans out into timeline lanes. Named to double
    /// as a `define_stats_event!` modifier (`scope = EventScope::Player`). The
    /// default is [`EventScope::Match`] (single shared row); declare `Player` or
    /// `Team` to opt a stream into per-entity lanes.
    pub const fn scope(self, scope: EventScope) -> Self {
        let mut def = self;
        def.scope = scope;
        def
    }
}

impl EventPayload {
    /// The lane fan-out scope for this event, taken from the payload type's
    /// declared [`EventDefinition::scope`]. This is the single authoritative
    /// source for the `EventMeta.scope` shipped on every timeline event, so the
    /// `scope =` declared next to each `define_stats_event!` is what reaches the
    /// client. The match is exhaustive on purpose: a newly added payload variant
    /// will not compile until its scope is declared here.
    pub fn scope(&self) -> EventScope {
        match self {
            Self::Timeline(_) => TimelineEvent::DEFINITION.scope,
            Self::CorePlayer(_) => CorePlayerScoreboardEvent::DEFINITION.scope,
            Self::Possession(_) => PossessionEvent::DEFINITION.scope,
            Self::LoosePossession(_) => LoosePossessionEvent::DEFINITION.scope,
            Self::PlayerPossession(_) => PlayerPossessionEvent::DEFINITION.scope,
            Self::BallHalf(_) => BallHalfEvent::DEFINITION.scope,
            Self::BallThird(_) => BallThirdEvent::DEFINITION.scope,
            Self::TerritorialPressure(_) => TerritorialPressureEvent::DEFINITION.scope,
            Self::Movement(_) => MovementEvent::DEFINITION.scope,
            Self::PlayerActivity(_) => PlayerActivityEvent::DEFINITION.scope,
            Self::FieldThird(_) => FieldThirdEvent::DEFINITION.scope,
            Self::FieldHalf(_) => FieldHalfEvent::DEFINITION.scope,
            Self::BallDepth(_) => BallDepthEvent::DEFINITION.scope,
            Self::DepthRole(_) => DepthRoleEvent::DEFINITION.scope,
            Self::BallProximity(_) => BallProximityEvent::DEFINITION.scope,
            Self::ShadowDefense(_) => ShadowDefenseEvent::DEFINITION.scope,
            Self::RotationRole(_) => RotationRoleEvent::DEFINITION.scope,
            Self::FirstManChange(_) => FirstManChangeEvent::DEFINITION.scope,
            Self::GoalContext(_) => GOAL_CONTEXT_EVENT_DEFINITION.scope,
            Self::Backboard(_) => BackboardBounceEvent::DEFINITION.scope,
            Self::CeilingShot(_) => CeilingShotEvent::DEFINITION.scope,
            Self::WallAerial(_) => WallAerialEvent::DEFINITION.scope,
            Self::WallAerialShot(_) => WallAerialShotEvent::DEFINITION.scope,
            Self::Center(_) => CenterEvent::DEFINITION.scope,
            Self::Flick(_) => FlickEvent::DEFINITION.scope,
            Self::DodgeReset(_) => DodgeResetEvent::DEFINITION.scope,
            Self::FlipReset(_) => FlipResetEvent::DEFINITION.scope,
            Self::DoubleTap(_) => DoubleTapEvent::DEFINITION.scope,
            Self::FiftyFifty(_) => FiftyFiftyEvent::DEFINITION.scope,
            Self::Kickoff(_) => KICKOFF_EVENT_DEFINITION.scope,
            Self::OneTimer(_) => OneTimerEvent::DEFINITION.scope,
            Self::Pass(_) => PassEvent::DEFINITION.scope,
            Self::BallCarry(_) => BallCarryEvent::DEFINITION.scope,
            Self::ControlledPlay(_) => ControlledPlayEvent::DEFINITION.scope,
            Self::Rush(_) => RushEvent::DEFINITION.scope,
            Self::Dodge(_) => DodgeEvent::DEFINITION.scope,
            Self::SpeedFlip(_) => SpeedFlipEvent::DEFINITION.scope,
            Self::HalfFlip(_) => HalfFlipEvent::DEFINITION.scope,
            Self::HalfVolley(_) => HalfVolleyEvent::DEFINITION.scope,
            Self::Wavedash(_) => WavedashEvent::DEFINITION.scope,
            Self::Whiff(_) => WhiffEvent::DEFINITION.scope,
            Self::BeatenToBall(_) => BeatenToBallEvent::DEFINITION.scope,
            Self::Powerslide(_) => PowerslideEvent::DEFINITION.scope,
            Self::Touch(_) => TouchClassificationEvent::DEFINITION.scope,
            Self::BoostPickup(_) => BoostPickupEvent::DEFINITION.scope,
            Self::Respawn(_) => RespawnEvent::DEFINITION.scope,
            Self::Bump(_) => BumpEvent::DEFINITION.scope,
            Self::Demolition(_) => DemolitionEvent::DEFINITION.scope,
        }
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
    Basic,
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
        // Fan-out is opt-in: streams stay on one shared row unless a definition
        // declares `Player`/`Team` via the `scope =` modifier.
        scope: EventScope::Match,
        confidence: UNKNOWN_DETECTION_CONFIDENCE,
        summary,
        approach,
        limitations: &[],
        hidden_from_review: false,
        variants: &[],
    }
}

/// When a projected stream's events are guaranteed to be finalized, relative
/// to each event's end timing, under incremental (interim) projection.
///
/// Declared per stream next to the stream's [`EmittedEvent`] entry so the
/// promise is statically auditable; the real-replay corpus instrumentation
/// test enforces it empirically (observed finalization lag must stay within
/// the declared horizon, modulo one projection interval of slack).
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum FinalizationHorizon {
    /// Finalized within this many seconds of game time after the event's end
    /// (its commit-lag bound; `0.0` for streams whose events commit fully
    /// formed at their end frame).
    EndPlus(f32),
    /// Finalized by the time live play next resumes after the first stoppage
    /// (any non-`ActivePlay` gameplay phase) following the event's end — or by
    /// match end, if play never resumes.
    NextStoppage,
    /// Only the finish projection finalizes this stream. The documented
    /// exception: every `MatchEnd` declaration carries a justification
    /// comment at its declaration site.
    MatchEnd,
}

/// A timeline event stream as projected by exactly one analysis node, with
/// its declared finalization horizon.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct ProjectedStream {
    /// The `EventMeta::stream` name this node's projection emits.
    pub stream: &'static str,
    pub horizon: FinalizationHorizon,
}

pub const fn produced_event(
    event: &'static EventDefinition,
    stream: &'static str,
    horizon: FinalizationHorizon,
    node_name: &'static str,
    node_type: &'static str,
    calculator_type: &'static str,
) -> EmittedEvent {
    EmittedEvent {
        event,
        projected: Some(ProjectedStream { stream, horizon }),
        producer: ProducerDefinition {
            node_name,
            node_type,
            calculator_type,
            implementation_notes: &[],
        },
    }
}

/// An [`EmittedEvent`] entry for a node that contributes to an event's
/// detection state without projecting a timeline stream of its own.
pub const fn contributed_event(
    event: &'static EventDefinition,
    node_name: &'static str,
    node_type: &'static str,
    calculator_type: &'static str,
) -> EmittedEvent {
    EmittedEvent {
        event,
        projected: None,
        producer: ProducerDefinition {
            node_name,
            node_type,
            calculator_type,
            implementation_notes: &[],
        },
    }
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
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct EmittedEvent {
    pub event: &'static EventDefinition,
    /// The timeline stream this node projects for this event (with its
    /// finalization horizon), or `None` for detection-state contributions
    /// that project nothing themselves. Each stream is owned by exactly one
    /// node; the graph enforces that a node only projects streams it
    /// declares.
    pub projected: Option<ProjectedStream>,
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

#[cfg(test)]
#[path = "event_definition_scope_tests.rs"]
mod scope_tests;

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

pub const GOAL_EVENT_DEFINITION: EventDefinition =
    pending_event_definition("goal", "Goal", EventCategory::Core);
register_stats_event_definition!(GOAL_EVENT_DEFINITION);

pub const SAVE_EVENT_DEFINITION: EventDefinition =
    pending_event_definition("save", "Save", EventCategory::Core);
register_stats_event_definition!(SAVE_EVENT_DEFINITION);

pub const SHOT_EVENT_DEFINITION: EventDefinition =
    pending_event_definition("shot", "Shot", EventCategory::Core);
register_stats_event_definition!(SHOT_EVENT_DEFINITION);

pub const KICKOFF_EVENT_DEFINITION: EventDefinition =
    pending_event_definition("kickoff", "Kickoff", EventCategory::Core).scope(EventScope::Player);
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
    "Backboard Hit",
    EventCategory::Basic,
    summary = "A ball rebound off the opponent backboard attributed to the player who sent the ball there.",
    approach = [
        "Track the last touch during live play and attribute a later backboard rebound to that touch when it occurs within the configured attribution window.",
        "Require the ball to be high, near the backboard face, and moving toward the backboard before contact.",
        "Confirm the contact either by rebound velocity away from the backboard or by a same-player simultaneous touch at the backboard face.",
    ],
    scope = EventScope::Player
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
    ],
    scope = EventScope::Player
);
define_stats_event!(
    WallAerialEvent,
    WALL_AERIAL_EVENT_DEFINITION,
    "wall_aerial",
    "Wall Aerial",
    EventCategory::Mechanic,
    summary = "An aerial launched off a side, end, or corner wall, whether or not the player is carrying the ball.",
    approach = [
        "Track how long each player rides the wall surface (a side or end wall), regardless of whether they have the ball.",
        "Arm a wall-aerial candidate when a player who rode the wall long enough leaves it while airborne.",
        "Classify the takeoff wall relative to the player's attack direction (left/right side, front/back end, or a corner) from the car's surface normal at the last wall contact.",
        "Emit on a later aerial touch by the same player while the player and ball are high enough and the takeoff-to-touch window holds.",
    ],
    scope = EventScope::Player
);
define_stats_event!(
    WallAerialShotEvent,
    WALL_AERIAL_SHOT_EVENT_DEFINITION,
    "wall_aerial_shot",
    "Wall Shot",
    EventCategory::Mechanic,
    summary = "A shot credited to a player shortly after taking off from a wall.",
    approach = [
        "Track recent wall contact for each player and arm a candidate when the player leaves the wall while still above the ground threshold.",
        "Classify the takeoff wall relative to the player's attack direction (left/right side, front/back end, or a corner) from the car's surface normal at the last wall contact.",
        "Match a subsequent shot stat event by that player within the takeoff-to-shot window.",
        "Require the shot touch to occur off the wall with sufficient player and ball height, then score confidence from timing, height, goal alignment, and ball speed.",
    ],
    scope = EventScope::Player
);
define_stats_event!(
    CenterEvent,
    CENTER_EVENT_DEFINITION,
    "center",
    "Center",
    EventCategory::Basic,
    summary = "A touch that moves the ball from a wide attacking position toward the central attacking area.",
    approach = [
        "Start a pending center from a live-play touch, unless that player immediately has a shot or goal event.",
        "Watch the ball for a short window after the touch and require meaningful travel from a wide x-position toward a more central x-position in the attacking half.",
        "Clear the candidate when it ages out, loses attribution, or becomes a shot/goal by the same player instead of a center.",
    ],
    scope = EventScope::Player
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
        "Record dodge starts that happen immediately after, or during, a qualifying setup, capturing the dodge torque (the flip axis) and the run's travel direction.",
        "Classify the flick kind from the dodge direction: decompose the dodge torque in the travel frame into forward/back and side components, labeling a backflip that still launches the ball forward as a reverse flick, a sideways-dominant dodge as a side flick, and a forward dodge as a forward flick.",
        "Tag handedness left/right from the ball's lateral deflection relative to travel.",
        "Emit on a same-player touch shortly after the dodge when the ball impulse is large and directed away from the player, with confidence from setup duration, timing, impulse, and separation.",
    ],
    scope = EventScope::Player
);
define_stats_event!(
    DodgeResetEvent,
    DODGE_RESET_EVENT_DEFINITION,
    "dodge_reset",
    "Dodge Reset",
    EventCategory::Basic,
    summary = "A frame-level dodge refresh observed from replay state, marked as occurring on the ball (a flip reset) and as used when later converted by a dodge-powered touch.",
    approach = [
        "Consume dodge-refreshed replay events and preserve the player, team, frame, time, and counter value.",
        "Classify the refresh as on-ball (a flip reset) when the player and ball are both airborne enough, close together, and the ball is positioned under the car in local space.",
        "Keep on-ball resets pending in an in-flight ledger; if the player dodges into the ball within the reset-to-touch window, mark the originating reset event `used` with its reset-to-use latency.",
        "Resolve every pending reset into an outcome: used, landed, superseded by a newer reset, expired, or cut off by a goal, live play ending, or the replay ending.",
    ],
    scope = EventScope::Player
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
    ],
    scope = EventScope::Player
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
    ],
    scope = EventScope::Player
);
define_stats_event!(
    PassEvent,
    PASS_EVENT_DEFINITION,
    "pass",
    "Pass",
    EventCategory::Basic,
    summary = "A same-team touch sequence where one player sends the ball to a different teammate.",
    approach = [
        "Track the last attributed touch in live play and compare it to each new touch.",
        "Emit when a different teammate touches the ball within the pass window after the ball has traveled far enough.",
        "Classify the pass as direct, backboard, fifty-fifty, or fifty-fifty backboard using intervening backboard-bounce and fifty-fifty state.",
    ],
    scope = EventScope::Player
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
    ],
    scope = EventScope::Player
);
define_stats_event!(
    ControlledPlayEvent,
    CONTROLLED_PLAY_EVENT_DEFINITION,
    "controlled_play",
    "Controlled Play",
    EventCategory::Other,
    summary =
        "A same-player possession episode with multiple touches and sustained close-ball time.",
    approach = [
        "Start a player-owned candidate from an attributed touch during live play.",
        "Require at least two distinct touches by the same player with at least one second between the first and last touch.",
        "Require sustained proximity to the ball and finish the candidate when another player touches, live play ends, or the touch chain times out.",
    ],
    scope = EventScope::Team
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
    ],
    scope = EventScope::Team
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
    ],
    scope = EventScope::Team
);
define_stats_event!(
    DodgeEvent,
    DODGE_EVENT_DEFINITION,
    "dodge",
    "Dodge",
    EventCategory::Basic,
    summary = "A dodge-start event, optionally carrying a rough estimated dodge impulse when the velocity change is measurable.",
    approach = [
        "Start on the replay's dodge-active rising edge for each player.",
        "Sample the player's velocity change over the early dodge window and subtract an approximate forward boost contribution when boost is active.",
        "Store the impulse estimate as dodge_impulse, including car-local direction classification plus raw and compensated world-space vectors for visualization and downstream mechanic detectors.",
    ],
    scope = EventScope::Player
);
define_stats_event!(
    SpeedFlipEvent,
    SPEED_FLIP_EVENT_DEFINITION,
    "speed_flip",
    "Speed Flip",
    EventCategory::Mechanic,
    summary = "A forward diagonal dodge whose flip is cancelled into a roll-to-recover, keeping the car pointed along its travel direction.",
    approach = [
        "Start candidates on dodge rising edges shortly after the car leaves the ground.",
        "Track the full airborne arc through landing: final speed, travel alignment, nose sweep, roll-to-recover, and the forward-diagonal input when replicated dodge torque is available.",
        "Emit when the car lands quickly with speed, stays pointed along its travel direction without an end-over-end tumble, and rolls through a real recovery.",
    ],
    scope = EventScope::Player
);
define_stats_event!(
    HalfFlipEvent,
    HALF_FLIP_EVENT_DEFINITION,
    "half_flip",
    "Half Flip",
    EventCategory::Mechanic,
    summary = "A dodge sequence that cancels a flip into an opposite facing direction.",
    approach = [
        "Start candidates on low grounded or low-air dodge rising edges.",
        "Track the car's forward vector through the evaluation window, including vertical flip evidence and final horizontal facing direction.",
        "Emit when the candidate has pitched through a flip, reaches and retains roughly opposite facing instead of rotating through a full end-over-end flip, and finishes with a meaningful horizontal facing direction.",
    ],
    scope = EventScope::Player
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
    ],
    scope = EventScope::Player
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
    ],
    scope = EventScope::Player
);
define_stats_event!(
    WhiffEvent,
    WHIFF_EVENT_DEFINITION,
    "whiff",
    "Whiff",
    EventCategory::Other,
    summary = "A committed attempt near the ball that resolves as a clear miss.",
    approach = [
        "Start candidates when a player gets within hitbox distance of the ball while moving or dodging toward it with sufficient alignment and closing speed.",
        "Track the full attempt span and closest-approach evidence while the candidate remains near the ball.",
        "Resolve as a whiff only when the player separates beyond the candidate window without any touch; touches, expiry, missing players, and play boundaries discard the candidate.",
    ],
    scope = EventScope::Player
);
define_stats_event!(
    BeatenToBallEvent,
    BEATEN_TO_BALL_EVENT_DEFINITION,
    "beaten_to_ball",
    "Beaten to ball",
    EventCategory::Other,
    summary = "A player who was actively challenging for the ball when an opponent beat them to the touch.",
    approach = [
        "Keep a short rolling motion history for every player relative to the ball during live play.",
        "At each confirmed touch, evaluate every non-touching opponent's lookback window for sustained convergence toward the ball and commitment (approach speed or a dodge toward the ball).",
        "Emit when the loss margin at the touch is narrow: small estimated time-to-ball or close hitbox distance, hard-capped on distance.",
    ],
    scope = EventScope::Player
);
define_stats_event!(
    PowerslideEvent,
    POWERSLIDE_EVENT_DEFINITION,
    "powerslide",
    "Powerslide",
    EventCategory::Basic,
    summary = "A state-change event for effective grounded powerslide use.",
    approach = [
        "Read each player's powerslide-active input/state on every frame.",
        "Treat powerslide as effective only while the player is close enough to the ground.",
        "Emit when a player's effective powerslide state changes between active and inactive.",
    ],
    scope = EventScope::Player
);
define_stats_event!(
    TouchClassificationEvent,
    TOUCH_CLASSIFICATION_EVENT_DEFINITION,
    "touch",
    "Touch",
    EventCategory::Basic,
    summary = "A classified ball touch carrying a set of independent tags: strength, surface/height context, action, and an outcome-based possession tag.",
    approach = [
        "Carry classification as a set of (group, value) tags rather than rivalrous fields, so independent reads coexist: a boom that the hitter recovers is tagged both action=boom and possession=advance.",
        "Tag strength kind (control, medium hit, hard hit) from the ball speed change, plus surface, height band, and dodge context for the touching player at contact time.",
        "Resolve a single mutually-exclusive action by precedence: replay-confirmed saves and shots first, then geometric save/shot trajectory projections, then clears out of the defensive third, then passes led toward a teammate, then booms hit hard downfield into space. A touch matching none of these has no action tag at all, rather than a catch-all value.",
        "Retroactively raise the action to shot/save by outcome when stronger evidence arrives after the touch: a scored goal (the scorer's touch), a replay shot/save stat event that lands after the touch, or a settled post-touch trajectory that crosses the goal mouth. Upgrades only ever raise the action, never downgrade it.",
        "Tag a touch contested independently of its action (a contested shot stays a shot and is also flagged contested), rather than collapsing contests into the action.",
        "Retroactively add a possession tag by outcome on pass/clear/boom and action-less touches: control when the toucher stays close to the ball while matching its velocity for most of a short follow window or wins the follow-up touch with the ball kept near, advance when they win the follow-up touch after playing the ball clear into space.",
        "Tag a touch's reception as a first touch when it starts a new reception: the previous global touch was by a different player or far enough in the past.",
    ],
    scope = EventScope::Player
);
define_stats_event!(
    BoostPickupEvent,
    BOOST_PICKUP_EVENT_DEFINITION,
    "boost_pickups",
    "Boost Pickup",
    EventCategory::Other,
    hidden = true,
    variants = BOOST_PICKUP_VARIANTS,
    scope = EventScope::Player
);
define_stats_event!(
    RespawnEvent,
    BOOST_RESPAWN_EVENT_DEFINITION,
    "boost_respawn",
    "Respawn",
    EventCategory::Other,
    scope = EventScope::Player
);
define_stats_event!(
    BumpEvent,
    BUMP_EVENT_DEFINITION,
    "bump",
    "Bump",
    EventCategory::Other,
    scope = EventScope::Player
);
define_stats_event!(
    DemolitionEvent,
    DEMOLITION_EVENT_DEFINITION,
    "demolition",
    "Demolition",
    EventCategory::Basic,
    scope = EventScope::Player
);
define_stats_event!(
    PossessionEvent,
    POSSESSION_EVENT_DEFINITION,
    "possession",
    "Possession",
    EventCategory::Other,
    scope = EventScope::Team
);
define_stats_event!(
    LoosePossessionEvent,
    LOOSE_POSSESSION_EVENT_DEFINITION,
    "loose_possession",
    "Loose Possession",
    EventCategory::Other,
    summary = "A team-possession span under the loose definition: the last team to touch owns the ball until the opponent takes it away.",
    approach = [
        "Track the last team to touch the ball, keeping possession through loose balls, teammate passes, and repelled 50-50 challenges.",
        "Transfer possession only when the opponent demonstrably wins the ball, backdating the boundary to the opponent's takeover touch so there is no neutral gap.",
        "Credit neutral only before the first touch of a live stretch or during a contested scramble off a neutral ball.",
    ],
    scope = EventScope::Team
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
    ],
    scope = EventScope::Player
);
define_stats_event!(
    BallHalfEvent,
    PRESSURE_EVENT_DEFINITION,
    "ball_half",
    "Ball Half",
    EventCategory::Other,
    scope = EventScope::Team
);
define_stats_event!(
    TerritorialPressureEvent,
    TERRITORIAL_PRESSURE_EVENT_DEFINITION,
    "territorial_pressure",
    "Territorial Pressure",
    EventCategory::Other,
    scope = EventScope::Team
);
define_stats_event!(
    MovementEvent,
    MOVEMENT_EVENT_DEFINITION,
    "movement",
    "Movement",
    EventCategory::Other,
    scope = EventScope::Player
);
define_stats_event!(
    PlayerActivityEvent,
    PLAYER_ACTIVITY_EVENT_DEFINITION,
    "player_activity",
    "Player Activity",
    EventCategory::Positioning,
    scope = EventScope::Player
);
define_stats_event!(
    FieldThirdEvent,
    FIELD_THIRD_EVENT_DEFINITION,
    "field_third",
    "Field Third",
    EventCategory::Positioning,
    scope = EventScope::Player
);
define_stats_event!(
    FieldHalfEvent,
    FIELD_HALF_EVENT_DEFINITION,
    "field_half",
    "Field Half",
    EventCategory::Positioning,
    scope = EventScope::Player
);
define_stats_event!(
    BallDepthEvent,
    BALL_DEPTH_EVENT_DEFINITION,
    "ball_depth",
    "Ball Depth",
    EventCategory::Positioning,
    scope = EventScope::Player
);
define_stats_event!(
    BallThirdEvent,
    BALL_THIRD_EVENT_DEFINITION,
    "ball_third",
    "Ball Third",
    EventCategory::Positioning,
    scope = EventScope::Player
);
define_stats_event!(
    DepthRoleEvent,
    DEPTH_ROLE_EVENT_DEFINITION,
    "depth_role",
    "Depth Role",
    EventCategory::Positioning,
    scope = EventScope::Player
);
define_stats_event!(
    BallProximityEvent,
    BALL_PROXIMITY_EVENT_DEFINITION,
    "ball_proximity",
    "Ball Proximity",
    EventCategory::Positioning,
    scope = EventScope::Player
);
define_stats_event!(
    ShadowDefenseEvent,
    SHADOW_DEFENSE_EVENT_DEFINITION,
    "shadow_defense",
    "Shadow Defense",
    EventCategory::Positioning,
    scope = EventScope::Player
);
define_stats_event!(
    RotationRoleEvent,
    ROTATION_ROLE_EVENT_DEFINITION,
    "rotation_role",
    "Rotation Role",
    EventCategory::Positioning,
    scope = EventScope::Player
);
define_stats_event!(
    FirstManChangeEvent,
    FIRST_MAN_CHANGE_EVENT_DEFINITION,
    "first_man_change",
    "First-Man Change",
    EventCategory::Positioning,
    scope = EventScope::Player
);
define_stats_event!(
    FlipResetEvent,
    FLIP_RESET_EVENT_DEFINITION,
    "flip_reset",
    "Flip Reset",
    EventCategory::Mechanic,
    summary = "An on-ball dodge refresh that is confirmed when the player uses the gained dodge and touches the ball again before landing.",
    approach = [
        "Consume on-ball dodge refreshes detected from replay state as pending flip-reset candidates.",
        "Require a later dodge start by the same player while the reset is still pending.",
        "Confirm only when that player touches the ball while dodge-active before landing and within the reset-to-touch window.",
    ],
    scope = EventScope::Player
);
define_stats_event!(
    Event,
    TIMELINE_ENVELOPE_EVENT_DEFINITION,
    "event",
    "Event",
    EventCategory::Basic,
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
        // Scoreboard deltas commit at their frame; goal rows appear once the
        // scoreboard attributes the goal, which happens during the post-goal
        // stoppage.
        &TIMELINE_EVENT_DEFINITION,
        "timeline",
        FinalizationHorizon::NextStoppage,
        "match_stats",
        "MatchStatsNode",
        "MatchStatsCalculator",
    ),
    produced_event(
        // Same attribution path as `timeline` goal rows.
        &CORE_PLAYER_SCOREBOARD_EVENT_DEFINITION,
        "core_player",
        FinalizationHorizon::NextStoppage,
        "match_stats",
        "MatchStatsNode",
        "MatchStatsCalculator",
    ),
];

pub(crate) const DEMO_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &DEMOLITION_EVENT_DEFINITION,
    "demolition",
    FinalizationHorizon::EndPlus(0.0),
    "demo",
    "DemoNode",
    "DemoCalculator",
)];

pub(crate) const BACKBOARD_BOUNCE_STATE_EMITTED_EVENTS: &[EmittedEvent] = &[contributed_event(
    &BACKBOARD_BOUNCE_EVENT_DEFINITION,
    "backboard_bounce_state",
    "BackboardBounceStateNode",
    "BackboardBounceCalculator",
)];

// The `backboard` stream is projected by the backboard node (which attributes
// bounces to players), not by the raw bounce-state node above.
pub(crate) const BACKBOARD_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &BACKBOARD_BOUNCE_EVENT_DEFINITION,
    "backboard",
    FinalizationHorizon::EndPlus(0.0),
    "backboard",
    "BackboardNode",
    "BackboardCalculator",
)];

pub(crate) const KICKOFF_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    // A kickoff's presented end freezes when the kickoff concludes, but the
    // event only commits once its in-flight goal-attribution window closes
    // (`KICKOFF_GOAL_MAX_SECONDS` = 12s from first touch) or the next
    // kickoff begins.
    &KICKOFF_EVENT_DEFINITION,
    "kickoff",
    FinalizationHorizon::EndPlus(12.0),
    "kickoff",
    "KickoffNode",
    "KickoffCalculator",
)];

pub(crate) const CEILING_SHOT_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &CEILING_SHOT_EVENT_DEFINITION,
    "ceiling_shot",
    FinalizationHorizon::EndPlus(0.0),
    "ceiling_shot",
    "CeilingShotNode",
    "CeilingShotCalculator",
)];

pub(crate) const WALL_AERIAL_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &WALL_AERIAL_EVENT_DEFINITION,
    "wall_aerial",
    FinalizationHorizon::EndPlus(0.0),
    "wall_aerial",
    "WallAerialNode",
    "WallAerialCalculator",
)];

pub(crate) const WALL_AERIAL_SHOT_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &WALL_AERIAL_SHOT_EVENT_DEFINITION,
    "wall_aerial_shot",
    FinalizationHorizon::EndPlus(0.0),
    "wall_aerial_shot",
    "WallAerialShotNode",
    "WallAerialShotCalculator",
)];

pub(crate) const CENTER_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &CENTER_EVENT_DEFINITION,
    "center",
    FinalizationHorizon::EndPlus(0.0),
    "center",
    "CenterNode",
    "CenterCalculator",
)];

pub(crate) const FLICK_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &FLICK_EVENT_DEFINITION,
    "flick",
    FinalizationHorizon::EndPlus(0.0),
    "flick",
    "FlickNode",
    "FlickCalculator",
)];

pub(crate) const DODGE_RESET_EMITTED_EVENTS: &[EmittedEvent] = &[
    produced_event(
        // A pending reset's outcome usually resolves within the 2s
        // reset-to-touch window or on landing, but landing has no time bound;
        // every pending outcome force-resolves at the next live-play boundary
        // (goal / play end).
        &DODGE_RESET_EVENT_DEFINITION,
        "dodge_reset",
        FinalizationHorizon::NextStoppage,
        "dodge_reset",
        "DodgeResetNode",
        "DodgeResetCalculator",
    ),
    produced_event(
        &FLIP_RESET_EVENT_DEFINITION,
        "flip_reset",
        FinalizationHorizon::EndPlus(0.0),
        "dodge_reset",
        "DodgeResetNode",
        "DodgeResetCalculator",
    ),
];

pub(crate) const DOUBLE_TAP_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &DOUBLE_TAP_EVENT_DEFINITION,
    "double_tap",
    FinalizationHorizon::EndPlus(0.0),
    "double_tap",
    "DoubleTapNode",
    "DoubleTapCalculator",
)];

pub(crate) const ONE_TIMER_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &ONE_TIMER_EVENT_DEFINITION,
    "one_timer",
    FinalizationHorizon::EndPlus(0.0),
    "one_timer",
    "OneTimerNode",
    "OneTimerCalculator",
)];

pub(crate) const PASS_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &PASS_EVENT_DEFINITION,
    "pass",
    FinalizationHorizon::EndPlus(0.0),
    "pass",
    "PassNode",
    "PassCalculator",
)];

pub(crate) const BALL_CARRY_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &BALL_CARRY_EVENT_DEFINITION,
    "ball_carry",
    FinalizationHorizon::EndPlus(0.0),
    "ball_carry",
    "BallCarryNode",
    "BallCarryCalculator",
)];

pub(crate) const AIR_DRIBBLE_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    // A sequence commits when the ball drops out of the air-dribble
    // envelope: the 3s touch-gap cap (`AIR_DRIBBLE_TOUCH_MAX_GAP_SECONDS`)
    // plus the ball's fall time to the 300uu floor.
    &BALL_CARRY_EVENT_DEFINITION,
    "air_dribble",
    FinalizationHorizon::EndPlus(4.0),
    "air_dribble",
    "AirDribbleNode",
    "AirDribbleCalculator",
)];

pub(crate) const CONTROLLED_PLAY_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &CONTROLLED_PLAY_EVENT_DEFINITION,
    "controlled_play",
    FinalizationHorizon::EndPlus(0.0),
    "controlled_play",
    "ControlledPlayNode",
    "ControlledPlayCalculator",
)];

pub(crate) const FIFTY_FIFTY_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &FIFTY_FIFTY_EVENT_DEFINITION,
    "fifty_fifty",
    FinalizationHorizon::EndPlus(0.0),
    "fifty_fifty",
    "FiftyFiftyNode",
    "FiftyFiftyCalculator",
)];

pub(crate) const RUSH_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &RUSH_EVENT_DEFINITION,
    "rush",
    FinalizationHorizon::EndPlus(0.0),
    "rush",
    "RushNode",
    "RushCalculator",
)];

pub(crate) const DODGE_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    // The resolved end freezes inside the 0.18s impulse window while the
    // push waits out the 0.45s rotation window (~0.27s residual lag).
    &DODGE_EVENT_DEFINITION,
    "dodge",
    FinalizationHorizon::EndPlus(0.5),
    "dodge",
    "FlipImpulseNode",
    "FlipImpulseCalculator",
)];

pub(crate) const SPEED_FLIP_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &SPEED_FLIP_EVENT_DEFINITION,
    "speed_flip",
    FinalizationHorizon::EndPlus(0.0),
    "speed_flip",
    "SpeedFlipNode",
    "SpeedFlipCalculator",
)];

pub(crate) const HALF_FLIP_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &HALF_FLIP_EVENT_DEFINITION,
    "half_flip",
    FinalizationHorizon::EndPlus(0.0),
    "half_flip",
    "HalfFlipNode",
    "HalfFlipCalculator",
)];

pub(crate) const HALF_VOLLEY_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &HALF_VOLLEY_EVENT_DEFINITION,
    "half_volley",
    FinalizationHorizon::EndPlus(0.0),
    "half_volley",
    "HalfVolleyNode",
    "HalfVolleyCalculator",
)];

pub(crate) const WAVEDASH_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &WAVEDASH_EVENT_DEFINITION,
    "wavedash",
    FinalizationHorizon::EndPlus(0.0),
    "wavedash",
    "WavedashNode",
    "WavedashCalculator",
)];

pub(crate) const WHIFF_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &WHIFF_EVENT_DEFINITION,
    "whiff",
    FinalizationHorizon::EndPlus(0.0),
    "whiff",
    "WhiffNode",
    "WhiffCalculator",
)];

pub(crate) const BEATEN_TO_BALL_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &BEATEN_TO_BALL_EVENT_DEFINITION,
    "beaten_to_ball",
    FinalizationHorizon::EndPlus(0.0),
    "beaten_to_ball",
    "BeatenToBallNode",
    "BeatenToBallCalculator",
)];

pub(crate) const POWERSLIDE_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &POWERSLIDE_EVENT_DEFINITION,
    "powerslide",
    FinalizationHorizon::EndPlus(0.0),
    "powerslide",
    "PowerslideNode",
    "PowerslideCalculator",
)];

pub(crate) const TOUCH_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    // MatchEnd justification: touch enrichment cannot be soundly bounded
    // by the next stoppage — the goal-shot attribution window
    // (`GOAL_SHOT_ATTRIBUTION_WINDOW_SECONDS` = 10s) deliberately crosses
    // the goal boundary to upgrade the scorer's pre-goal touch, and
    // ball-movement credit finalizes on supersession (event-driven, no
    // timer), so promotion runs until finish.
    &TOUCH_CLASSIFICATION_EVENT_DEFINITION,
    "touch",
    FinalizationHorizon::MatchEnd,
    "touch",
    "TouchNode",
    "TouchCalculator",
)];

pub(crate) const BOOST_EMITTED_EVENTS: &[EmittedEvent] = &[
    produced_event(
        // Pickup detection reconciles reported pad events with inferred
        // boost-amount jumps across chained `PICKUP_MATCH_FRAME_WINDOW`
        // (3-frame) deferrals, and the committed event is backdated to the
        // observed jump — so a pickup can commit a handful of frames after
        // its stamped time (0.5s covers the chain with low-frame-rate
        // headroom).
        &BOOST_PICKUP_EVENT_DEFINITION,
        "boost_pickups",
        FinalizationHorizon::EndPlus(0.5),
        "boost",
        "BoostNode",
        "BoostCalculator",
    ),
    produced_event(
        &BOOST_RESPAWN_EVENT_DEFINITION,
        "boost_respawn",
        FinalizationHorizon::EndPlus(0.0),
        "boost",
        "BoostNode",
        "BoostCalculator",
    ),
];

pub(crate) const BUMP_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &BUMP_EVENT_DEFINITION,
    "bump",
    FinalizationHorizon::EndPlus(0.0),
    "bump",
    "BumpNode",
    "BumpCalculator",
)];

pub(crate) const POSSESSION_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    // A segment's terminating boundary can chain two 3s resolution windows
    // (`POSSESSION_RESOLUTION_WINDOW_SECONDS`) before it commits.
    &POSSESSION_EVENT_DEFINITION,
    "possession",
    FinalizationHorizon::EndPlus(6.0),
    "possession",
    "PossessionNode",
    "PossessionCalculator",
)];

pub(crate) const PLAYER_POSSESSION_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    // A suspended span commits once the 2s merge gap
    // (`PLAYER_POSSESSION_MERGE_GAP_SECONDS`) elapses without the same
    // player re-establishing control.
    &PLAYER_POSSESSION_EVENT_DEFINITION,
    "player_possession",
    FinalizationHorizon::EndPlus(2.0),
    "player_possession",
    "PlayerPossessionNode",
    "PlayerPossessionCalculator",
)];

pub(crate) const LOOSE_POSSESSION_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    // Bounded by the single 1.5s challenge-resolution window
    // (`LOOSE_RESOLUTION_WINDOW_SECONDS`).
    &LOOSE_POSSESSION_EVENT_DEFINITION,
    "loose_possession",
    FinalizationHorizon::EndPlus(1.5),
    "loose_possession",
    "LoosePossessionNode",
    "LoosePossessionCalculator",
)];

pub(crate) const BALL_HALF_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    // The inactive span covering a stoppage keeps its recorded end at the
    // stoppage's first frame (nothing extends it while play is dead) and only
    // commits when the state next changes — i.e. when live play resumes.
    &PRESSURE_EVENT_DEFINITION,
    "ball_half",
    FinalizationHorizon::NextStoppage,
    "ball_half",
    "BallHalfNode",
    "BallHalfCalculator",
)];

pub(crate) const BALL_THIRD_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    // Same shape as `ball_half`: the stoppage-covering inactive span's end
    // freezes at the stoppage start and commits at play resumption.
    &BALL_THIRD_EVENT_DEFINITION,
    "ball_third",
    FinalizationHorizon::NextStoppage,
    "ball_third",
    "BallThirdNode",
    "BallThirdCalculator",
)];

pub(crate) const TERRITORIAL_BALL_HALF_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &TERRITORIAL_PRESSURE_EVENT_DEFINITION,
    "territorial_pressure",
    FinalizationHorizon::EndPlus(0.0),
    "territorial_pressure",
    "TerritorialPressureNode",
    "TerritorialPressureCalculator",
)];

pub(crate) const MOVEMENT_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    &MOVEMENT_EVENT_DEFINITION,
    "movement",
    FinalizationHorizon::EndPlus(0.0),
    "movement",
    "MovementNode",
    "MovementCalculator",
)];

pub(crate) const POSITIONING_EMITTED_EVENTS: &[EmittedEvent] = &[
    produced_event(
        &PLAYER_ACTIVITY_EVENT_DEFINITION,
        "player_activity",
        FinalizationHorizon::EndPlus(0.0),
        "positioning",
        "PositioningNode",
        "PositioningCalculator",
    ),
    produced_event(
        &FIELD_THIRD_EVENT_DEFINITION,
        "field_third",
        FinalizationHorizon::EndPlus(0.0),
        "positioning",
        "PositioningNode",
        "PositioningCalculator",
    ),
    produced_event(
        &FIELD_HALF_EVENT_DEFINITION,
        "field_half",
        FinalizationHorizon::EndPlus(0.0),
        "positioning",
        "PositioningNode",
        "PositioningCalculator",
    ),
    produced_event(
        &BALL_DEPTH_EVENT_DEFINITION,
        "ball_depth",
        FinalizationHorizon::EndPlus(0.0),
        "positioning",
        "PositioningNode",
        "PositioningCalculator",
    ),
    produced_event(
        &DEPTH_ROLE_EVENT_DEFINITION,
        "depth_role",
        FinalizationHorizon::EndPlus(0.0),
        "positioning",
        "PositioningNode",
        "PositioningCalculator",
    ),
    produced_event(
        &BALL_PROXIMITY_EVENT_DEFINITION,
        "ball_proximity",
        FinalizationHorizon::EndPlus(0.0),
        "positioning",
        "PositioningNode",
        "PositioningCalculator",
    ),
    produced_event(
        &SHADOW_DEFENSE_EVENT_DEFINITION,
        "shadow_defense",
        FinalizationHorizon::EndPlus(0.0),
        "positioning",
        "PositioningNode",
        "PositioningCalculator",
    ),
];

pub(crate) const ROTATION_EMITTED_EVENTS: &[EmittedEvent] = &[
    produced_event(
        &ROTATION_ROLE_EVENT_DEFINITION,
        "rotation_role",
        FinalizationHorizon::EndPlus(0.0),
        "rotation",
        "RotationNode",
        "RotationCalculator",
    ),
    produced_event(
        &FIRST_MAN_CHANGE_EVENT_DEFINITION,
        "first_man_change",
        FinalizationHorizon::EndPlus(0.0),
        "rotation",
        "RotationNode",
        "RotationCalculator",
    ),
];

pub(crate) const GOAL_CONTEXT_EMITTED_EVENTS: &[EmittedEvent] = &[produced_event(
    // MatchEnd justification: `pressure_duration_before_goal` attaches
    // only at finish (it reads the finalized territorial-pressure
    // sessions), and scorer reconciliation has no in-code time bound.
    &GOAL_CONTEXT_EVENT_DEFINITION,
    "goal_context",
    FinalizationHorizon::MatchEnd,
    "goal_context",
    "GoalContextNode",
    "MatchStatsCalculator",
)];

#[cfg(test)]
#[path = "event_definition_tests.rs"]
mod tests;
