use crate::*;
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct StatsTimelineConfig {
    pub most_back_forward_threshold_y: f32,
    pub level_ball_depth_margin: f32,
    pub closest_to_ball_switch_margin: f32,
    pub closest_to_ball_switch_min_seconds: f32,
    pub pressure_neutral_zone_half_width_y: f32,
    pub territorial_pressure_neutral_zone_half_width_y: f32,
    pub territorial_pressure_min_establish_seconds: f32,
    pub territorial_pressure_min_establish_third_seconds: f32,
    pub territorial_pressure_relief_grace_seconds: f32,
    pub territorial_pressure_confirmed_relief_grace_seconds: f32,
    pub rotation_role_depth_margin: f32,
    pub rotation_first_man_ambiguity_margin: f32,
    pub rotation_first_man_debounce_seconds: f32,
    pub rush_max_start_y: f32,
    pub rush_attack_support_distance_y: f32,
    pub rush_defender_distance_y: f32,
    pub rush_min_possession_retained_seconds: f32,
    pub aerial_goal_min_ball_z: f32,
    pub high_aerial_goal_min_ball_z: f32,
    pub long_distance_goal_max_attacking_y: f32,
    pub own_half_goal_max_attacking_y: f32,
    pub empty_net_min_defender_y_margin: f32,
    pub empty_net_min_defender_distance: f32,
    pub empty_net_max_touch_attacking_y: f32,
    pub flick_goal_max_event_to_goal_seconds: f32,
    pub ceiling_shot_goal_max_event_to_goal_seconds: f32,
    pub double_tap_goal_max_event_to_goal_seconds: f32,
    pub one_timer_goal_max_event_to_goal_seconds: f32,
    pub air_dribble_goal_max_end_to_goal_seconds: f32,
    pub flip_reset_goal_max_event_to_goal_seconds: f32,
    pub bump_goal_max_event_to_goal_seconds: f32,
    pub demo_goal_max_event_to_goal_seconds: f32,
    pub half_volley_max_bounce_to_touch_seconds: f32,
    pub half_volley_min_ball_speed: f32,
    pub half_volley_goal_max_touch_to_goal_seconds: f32,
    pub half_volley_goal_min_goal_alignment: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct ReplayStatsTimeline {
    pub config: StatsTimelineConfig,
    pub replay_meta: ReplayMeta,
    pub events: ReplayStatsTimelineEvents,
    pub frames: Vec<ReplayStatsFrame>,
}

impl ReplayStatsTimeline {
    pub fn frame_by_number(&self, frame_number: usize) -> Option<&ReplayStatsFrame> {
        self.frames
            .iter()
            .find(|frame| frame.frame_number == frame_number)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct ReplayStatsTimelineScaffold {
    pub config: StatsTimelineConfig,
    pub replay_meta: ReplayMeta,
    pub events: ReplayStatsTimelineEvents,
    pub frames: Vec<ReplayStatsFrameScaffold>,
    /// Whole-match distance totals per player. Distance is a continuous magnitude that cannot
    /// be reconstructed from events, so it is computed over the entire match and shipped once
    /// here rather than per frame, keeping the scaffold frames a pure event-only product.
    pub positioning_summary: Vec<ReplayStatsPositioningSummary>,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct ReplayStatsPositioningSummary {
    #[serde(rename = "player_id")]
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub player_id: PlayerId,
    pub is_team_0: bool,
    pub distance: PositioningSignalSnapshot,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct ReplayStatsFrameScaffold {
    pub frame_number: usize,
    pub time: f32,
    pub dt: f32,
    pub seconds_remaining: Option<i32>,
    pub game_state: Option<i32>,
    pub ball_has_been_hit: Option<bool>,
    pub kickoff_countdown_time: Option<i32>,
    pub gameplay_phase: GameplayPhase,
    pub is_live_play: bool,
    #[ts(type = "Record<string, unknown>")]
    pub team_zero: BTreeMap<String, serde_json::Value>,
    #[ts(type = "Record<string, unknown>")]
    pub team_one: BTreeMap<String, serde_json::Value>,
    pub players: Vec<ReplayStatsPlayerIdentity>,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct ReplayStatsPlayerIdentity {
    #[serde(rename = "player_id")]
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub player_id: PlayerId,
    pub name: String,
    pub is_team_0: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct ReplayStatsTimelineEvents {
    pub events: Vec<Event>,
}

pub fn stats_timeline_event_label(stream: &str) -> String {
    let label = match stream {
        "timeline" => "Timeline",
        "core_player" => "Core player",
        "core_player_goal_context" => "Core player goal context",
        "possession" => "Possession",
        "pressure" => "Pressure",
        "territorial_pressure" => "Territorial pressure",
        "movement" => "Movement",
        "positioning_activity" => "Positioning activity",
        "positioning_possession" => "Positioning possession",
        "positioning_field_zone" => "Positioning field zone",
        "positioning_ball_depth" => "Positioning ball depth",
        "positioning_teammate_role" => "Positioning teammate role",
        "positioning_ball_proximity" => "Positioning ball proximity",
        "positioning_goal_context" => "Positioning goal context",
        "rotation_player" => "Rotation player",
        "rotation_role_span" => "Rotation role span",
        "rotation_depth_span" => "Rotation depth span",
        "rotation_first_man_stint" => "Rotation first-man stint",
        "rotation_team" => "Rotation team",
        "goal_context" => "Goal context",
        "backboard" => "Backboard",
        "air_dribble" => "Air dribble",
        "ball_carry" => "Ball carry",
        "controlled_play" => "Controlled play",
        "ceiling_shot" => "Ceiling shot",
        "wall_aerial" => "Wall aerial",
        "wall_aerial_shot" => "Wall aerial shot",
        "center" => "Center",
        "dodge_reset" => "Flip reset",
        "flip_reset" => "Flip reset",
        "double_tap" => "Double tap",
        "one_timer" => "One-timer",
        "pass" => "Pass",
        "fifty_fifty" => "50/50",
        "kickoff" => "Kickoff",
        "rush" => "Rush",
        "dodge" => "Dodge",
        "speed_flip" => "Speed flip",
        "half_flip" => "Half flip",
        "half_volley" => "Half-volley",
        "wavedash" => "Wavedash",
        "whiff" => "Whiff",
        "powerslide" => "Powerslide",
        "touch" => "Touch",
        "boost_pickups" => "Boost pickup",
        "boost_ledger" => "Boost ledger",
        "boost_bucket" => "Boost bucket",
        "boost_state" => "Boost state",
        "bump" => "Bump",
        "flick" => "Flick",
        "musty_flick" => "Musty flick",
        _ => return title_case_event_stream(stream),
    };
    label.to_owned()
}

fn title_case_event_stream(stream: &str) -> String {
    stream
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect::<String>(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[serde(tag = "type", rename_all = "snake_case")]
#[ts(export)]
pub enum EventTiming {
    Moment {
        frame: usize,
        time: f32,
    },
    Span {
        start_frame: usize,
        end_frame: usize,
        start_time: f32,
        end_time: f32,
    },
}

impl EventTiming {
    pub fn start(&self) -> (usize, f32) {
        match self {
            Self::Moment { frame, time } => (*frame, *time),
            Self::Span {
                start_frame,
                start_time,
                ..
            } => (*start_frame, *start_time),
        }
    }

    pub fn end(&self) -> (usize, f32) {
        match self {
            Self::Moment { frame, time } => (*frame, *time),
            Self::Span {
                end_frame,
                end_time,
                ..
            } => (*end_frame, *end_time),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
#[ts(export)]
pub enum EventPropertyValue {
    Text(String),
    Unsigned(u32),
    Float(f32),
    Boolean(bool),
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct EventProperty {
    pub key: String,
    pub value: EventPropertyValue,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct EventMeta {
    pub id: String,
    pub stream: String,
    pub label: String,
    pub timing: EventTiming,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(as = "Option<crate::interop::ts_bindings::RemoteIdTs>")]
    pub primary_player: Option<PlayerId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(as = "Option<crate::interop::ts_bindings::RemoteIdTs>")]
    pub secondary_player: Option<PlayerId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_position: Option<[f32; 3]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ball_position: Option<[f32; 3]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub team_is_team_0: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub properties: Vec<EventProperty>,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[serde(tag = "kind", content = "payload", rename_all = "snake_case")]
#[ts(export)]
pub enum EventPayload {
    Timeline(TimelineEvent),
    CorePlayer(CorePlayerScoreboardEvent),
    CorePlayerGoalContext(CorePlayerGoalContextEvent),
    Possession(PossessionEvent),
    Pressure(PressureEvent),
    TerritorialPressure(TerritorialPressureEvent),
    Movement(MovementEvent),
    PositioningActivity(PositioningActivityEvent),
    PositioningFieldZone(PositioningFieldZoneEvent),
    PositioningBallRelativeDepth(PositioningBallRelativeDepthEvent),
    PositioningTeammateRole(PositioningTeammateRoleEvent),
    PositioningBallProximity(PositioningBallProximityEvent),
    RotationPlayer(RotationPlayerEvent),
    RotationRoleSpan(RotationRoleSpanEvent),
    RotationDepthSpan(RotationDepthSpanEvent),
    RotationFirstManStint(RotationFirstManStintEvent),
    RotationTeam(RotationTeamEvent),
    GoalContext(GoalContextEvent),
    Backboard(BackboardBounceEvent),
    CeilingShot(CeilingShotEvent),
    WallAerial(WallAerialEvent),
    WallAerialShot(WallAerialShotEvent),
    Center(CenterEvent),
    Flick(FlickEvent),
    MustyFlick(MustyFlickEvent),
    DodgeReset(DodgeResetEvent),
    DoubleTap(DoubleTapEvent),
    FiftyFifty(FiftyFiftyEvent),
    Kickoff(Box<KickoffEvent>),
    OneTimer(OneTimerEvent),
    Pass(PassEvent),
    BallCarry(BallCarryEvent),
    ControlledPlay(ControlledPlayEvent),
    Rush(RushEvent),
    Dodge(DodgeEvent),
    SpeedFlip(SpeedFlipEvent),
    HalfFlip(HalfFlipEvent),
    HalfVolley(HalfVolleyEvent),
    Wavedash(WavedashEvent),
    Whiff(WhiffEvent),
    Powerslide(PowerslideEvent),
    Touch(TouchClassificationEvent),
    BoostPickup(BoostPickupComparisonEvent),
    BoostLedger(BoostLedgerEvent),
    BoostBucket(BoostBucketEvent),
    BoostState(BoostStateEvent),
    Bump(BumpEvent),
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct Event {
    pub meta: EventMeta,
    pub payload: EventPayload,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct ReplayStatsFrame {
    pub frame_number: usize,
    pub time: f32,
    pub dt: f32,
    pub seconds_remaining: Option<i32>,
    pub game_state: Option<i32>,
    pub ball_has_been_hit: Option<bool>,
    pub kickoff_countdown_time: Option<i32>,
    pub gameplay_phase: GameplayPhase,
    pub is_live_play: bool,
    pub team_zero: TeamStatsSnapshot,
    pub team_one: TeamStatsSnapshot,
    pub players: Vec<PlayerStatsSnapshot>,
}

/// Team-owned fields in the materialized stats timeline export.
///
/// This is a serialization/client DTO, not an analysis-graph dependency
/// surface. Analysis nodes that need another calculator's data should depend
/// on that calculator's concrete node state through `AnalysisNode::dependencies`
/// and read it from `AnalysisStateContext`.
///
/// The field list is a curated compatibility schema for full snapshot
/// timelines. It is not the authoritative registry of team analysis outputs;
/// use the module-keyed stats/graph surfaces when callers need discoverability.
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct TeamStatsSnapshot {
    pub fifty_fifty: FiftyFiftyTeamStats,
    pub possession: PossessionTeamStats,
    pub pressure: PressureTeamStats,
    pub territorial_pressure: TerritorialPressureTeamStats,
    pub rotation: RotationTeamStats,
    pub rush: RushTeamStats,
    pub core: CoreTeamStats,
    pub backboard: BackboardTeamStats,
    pub double_tap: DoubleTapTeamStats,
    pub one_timer: OneTimerTeamStats,
    pub pass: PassTeamStats,
    pub kickoff: KickoffTeamStats,
    pub ball_carry: BallCarryStats,
    pub controlled_play: ControlledPlayStats,
    pub air_dribble: AirDribbleStats,
    pub boost: BoostStats,
    pub bump: BumpTeamStats,
    pub half_volley: HalfVolleyTeamStats,
    pub movement: MovementStats,
    pub positioning: PositioningTeamStats,
    pub powerslide: PowerslideStats,
    pub demo: DemoTeamStats,
}

/// Player-owned fields in the materialized stats timeline export.
///
/// Like `TeamStatsSnapshot`, this is a serialization/client DTO. It should not
/// be used as an upstream data dependency between analysis nodes.
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct PlayerStatsSnapshot {
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub player_id: PlayerId,
    pub name: String,
    pub is_team_0: bool,
    pub core: CorePlayerStats,
    pub backboard: BackboardPlayerStats,
    pub ceiling_shot: CeilingShotStats,
    pub wall_aerial: WallAerialStats,
    pub wall_aerial_shot: WallAerialShotStats,
    pub double_tap: DoubleTapPlayerStats,
    pub one_timer: OneTimerPlayerStats,
    pub pass: PassPlayerStats,
    pub fifty_fifty: FiftyFiftyPlayerStats,
    pub kickoff: KickoffPlayerStats,
    pub speed_flip: SpeedFlipStats,
    pub half_flip: HalfFlipStats,
    pub half_volley: HalfVolleyPlayerStats,
    pub wavedash: WavedashStats,
    pub touch: TouchStats,
    pub whiff: WhiffStats,
    pub flick: FlickStats,
    pub musty_flick: MustyFlickStats,
    pub dodge_reset: DodgeResetStats,
    pub ball_carry: BallCarryStats,
    pub controlled_play: ControlledPlayStats,
    pub air_dribble: AirDribbleStats,
    pub boost: BoostStats,
    pub bump: BumpPlayerStats,
    pub movement: MovementStats,
    pub positioning: PositioningStats,
    pub rotation: RotationPlayerStats,
    pub powerslide: PowerslideStats,
    pub demo: DemoPlayerStats,
}
