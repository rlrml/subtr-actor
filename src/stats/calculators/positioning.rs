use super::*;

const GOAL_CAUGHT_AHEAD_MAX_BALL_Y: f32 = -1200.0;
const GOAL_CAUGHT_AHEAD_MIN_PLAYER_Y: f32 = -250.0;
const GOAL_CAUGHT_AHEAD_MIN_BALL_DELTA_Y: f32 = 2200.0;
const DEFAULT_LEVEL_BALL_DEPTH_MARGIN: f32 = 150.0;
const DEFAULT_CLOSEST_TO_BALL_SWITCH_MARGIN: f32 = 100.0;
const DEFAULT_CLOSEST_TO_BALL_SWITCH_MIN_SECONDS: f32 = 0.2;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PositioningEvent {
    pub time: f32,
    pub frame: usize,
    pub end_time: f32,
    pub end_frame: usize,
    pub duration: f32,
    pub player: PlayerId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_position: Option<[f32; 3]>,
    pub is_team_0: bool,
    pub active: bool,
    pub tracked: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub distance_to_teammates: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub distance_to_ball: Option<f32>,
    pub possession_state: PositioningPossessionState,
    pub demolished: bool,
    pub no_teammates: bool,
    pub teammate_role: PositioningTeammateRoleState,
    pub defensive_zone_fraction: f32,
    pub neutral_zone_fraction: f32,
    pub offensive_zone_fraction: f32,
    pub defensive_half_fraction: f32,
    pub offensive_half_fraction: f32,
    pub closest_to_ball: bool,
    pub closest_to_ball_team: bool,
    pub closest_to_ball_absolute: bool,
    pub farthest_from_ball: bool,
    pub behind_ball_fraction: f32,
    pub level_with_ball_fraction: f32,
    pub in_front_of_ball_fraction: f32,
    pub caught_ahead_of_play_on_conceded_goal: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct PositioningActivityEvent {
    pub time: f32,
    pub frame: usize,
    pub end_time: f32,
    pub end_frame: usize,
    pub duration: f32,
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_position: Option<[f32; 3]>,
    pub is_team_0: bool,
    pub active: bool,
    pub tracked: bool,
    pub demolished: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct PositioningPossessionEvent {
    pub time: f32,
    pub frame: usize,
    pub end_time: f32,
    pub end_frame: usize,
    pub duration: f32,
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_position: Option<[f32; 3]>,
    pub is_team_0: bool,
    pub possession_state: PositioningPossessionState,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct PositioningFieldZoneEvent {
    pub time: f32,
    pub frame: usize,
    pub end_time: f32,
    pub end_frame: usize,
    pub duration: f32,
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_position: Option<[f32; 3]>,
    pub is_team_0: bool,
    pub defensive_zone_fraction: f32,
    pub neutral_zone_fraction: f32,
    pub offensive_zone_fraction: f32,
    pub defensive_half_fraction: f32,
    pub offensive_half_fraction: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct PositioningBallDepthEvent {
    pub time: f32,
    pub frame: usize,
    pub end_time: f32,
    pub end_frame: usize,
    pub duration: f32,
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_position: Option<[f32; 3]>,
    pub is_team_0: bool,
    pub behind_ball_fraction: f32,
    pub level_with_ball_fraction: f32,
    pub in_front_of_ball_fraction: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct PositioningTeammateRoleEvent {
    pub time: f32,
    pub frame: usize,
    pub end_time: f32,
    pub end_frame: usize,
    pub duration: f32,
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_position: Option<[f32; 3]>,
    pub is_team_0: bool,
    pub teammate_role: PositioningTeammateRoleState,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct PositioningBallProximityEvent {
    pub time: f32,
    pub frame: usize,
    pub end_time: f32,
    pub end_frame: usize,
    pub duration: f32,
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_position: Option<[f32; 3]>,
    pub is_team_0: bool,
    pub closest_to_ball_team: bool,
    pub closest_to_ball_absolute: bool,
    pub farthest_from_ball: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct PositioningGoalContextEvent {
    pub time: f32,
    pub frame: usize,
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_position: Option<[f32; 3]>,
    pub is_team_0: bool,
    pub caught_ahead_of_play_on_conceded_goal: bool,
}

impl PositioningEvent {
    fn new(
        frame: &FrameInfo,
        player: PlayerId,
        player_position: Option<[f32; 3]>,
        is_team_0: bool,
    ) -> Self {
        Self {
            time: frame.time,
            frame: frame.frame_number,
            end_time: frame.time,
            end_frame: frame.frame_number,
            duration: 0.0,
            player,
            player_position,
            is_team_0,
            active: false,
            tracked: false,
            distance_to_teammates: None,
            distance_to_ball: None,
            possession_state: PositioningPossessionState::Neutral,
            demolished: false,
            no_teammates: false,
            teammate_role: PositioningTeammateRoleState::Unknown,
            defensive_zone_fraction: 0.0,
            neutral_zone_fraction: 0.0,
            offensive_zone_fraction: 0.0,
            defensive_half_fraction: 0.0,
            offensive_half_fraction: 0.0,
            closest_to_ball: false,
            closest_to_ball_team: false,
            closest_to_ball_absolute: false,
            farthest_from_ball: false,
            behind_ball_fraction: 0.0,
            level_with_ball_fraction: 0.0,
            in_front_of_ball_fraction: 0.0,
            caught_ahead_of_play_on_conceded_goal: false,
        }
    }

    fn has_delta(&self) -> bool {
        self.duration != 0.0 || self.caught_ahead_of_play_on_conceded_goal
    }

    pub fn activity_event(&self) -> Option<PositioningActivityEvent> {
        (self.duration != 0.0 && (self.active || self.tracked || self.demolished)).then(|| {
            PositioningActivityEvent {
                time: self.time,
                frame: self.frame,
                end_time: self.end_time,
                end_frame: self.end_frame,
                duration: self.duration,
                player: self.player.clone(),
                player_position: self.player_position,
                is_team_0: self.is_team_0,
                active: self.active,
                tracked: self.tracked,
                demolished: self.demolished,
            }
        })
    }

    pub fn possession_event(&self) -> Option<PositioningPossessionEvent> {
        (self.tracked && self.possession_state != PositioningPossessionState::Neutral).then(|| {
            PositioningPossessionEvent {
                time: self.time,
                frame: self.frame,
                end_time: self.end_time,
                end_frame: self.end_frame,
                duration: self.duration,
                player: self.player.clone(),
                player_position: self.player_position,
                is_team_0: self.is_team_0,
                possession_state: self.possession_state,
            }
        })
    }

    pub fn field_zone_event(&self) -> Option<PositioningFieldZoneEvent> {
        self.tracked.then(|| PositioningFieldZoneEvent {
            time: self.time,
            frame: self.frame,
            end_time: self.end_time,
            end_frame: self.end_frame,
            duration: self.duration,
            player: self.player.clone(),
            player_position: self.player_position,
            is_team_0: self.is_team_0,
            defensive_zone_fraction: self.defensive_zone_fraction,
            neutral_zone_fraction: self.neutral_zone_fraction,
            offensive_zone_fraction: self.offensive_zone_fraction,
            defensive_half_fraction: self.defensive_half_fraction,
            offensive_half_fraction: self.offensive_half_fraction,
        })
    }

    pub fn ball_depth_event(&self) -> Option<PositioningBallDepthEvent> {
        self.tracked.then(|| PositioningBallDepthEvent {
            time: self.time,
            frame: self.frame,
            end_time: self.end_time,
            end_frame: self.end_frame,
            duration: self.duration,
            player: self.player.clone(),
            player_position: self.player_position,
            is_team_0: self.is_team_0,
            behind_ball_fraction: self.behind_ball_fraction,
            level_with_ball_fraction: self.level_with_ball_fraction,
            in_front_of_ball_fraction: self.in_front_of_ball_fraction,
        })
    }

    pub fn teammate_role_event(&self) -> Option<PositioningTeammateRoleEvent> {
        (self.tracked && self.teammate_role != PositioningTeammateRoleState::Unknown).then(|| {
            PositioningTeammateRoleEvent {
                time: self.time,
                frame: self.frame,
                end_time: self.end_time,
                end_frame: self.end_frame,
                duration: self.duration,
                player: self.player.clone(),
                player_position: self.player_position,
                is_team_0: self.is_team_0,
                teammate_role: self.teammate_role,
            }
        })
    }

    pub fn ball_proximity_event(&self) -> Option<PositioningBallProximityEvent> {
        (self.tracked
            && (self.closest_to_ball
                || self.closest_to_ball_team
                || self.closest_to_ball_absolute
                || self.farthest_from_ball))
            .then(|| PositioningBallProximityEvent {
                time: self.time,
                frame: self.frame,
                end_time: self.end_time,
                end_frame: self.end_frame,
                duration: self.duration,
                player: self.player.clone(),
                player_position: self.player_position,
                is_team_0: self.is_team_0,
                closest_to_ball_team: self.closest_to_ball || self.closest_to_ball_team,
                closest_to_ball_absolute: self.closest_to_ball_absolute,
                farthest_from_ball: self.farthest_from_ball,
            })
    }

    pub fn goal_context_event(&self) -> Option<PositioningGoalContextEvent> {
        self.caught_ahead_of_play_on_conceded_goal
            .then(|| PositioningGoalContextEvent {
                time: self.time,
                frame: self.frame,
                player: self.player.clone(),
                player_position: self.player_position,
                is_team_0: self.is_team_0,
                caught_ahead_of_play_on_conceded_goal: true,
            })
    }
}

/// A projected positioning facet event that represents a span of game time for a
/// single player and can therefore be coalesced with adjacent same-state spans.
///
/// The internal [`PositioningEvent`] stream is still computed once per frame (it is
/// transient calculator state and never serialized), but the per-facet projections
/// that are emitted to the timeline collapse consecutive frames with an identical
/// categorical state into a single span. Because every per-frame contribution to the
/// exported stats is `duration * fraction` (or `duration` for booleans), summing the
/// merged span's `duration` reproduces the per-frame totals exactly: a run of frames
/// that sits purely in one state has `fraction == 1.0` each frame, and the only frames
/// that stay standalone are sub-frame boundary crossings whose fractions differ.
trait CoalescibleSpan {
    fn span_player(&self) -> &PlayerId;
    /// Whether `self` and `next` share the same categorical state and can be merged.
    fn same_state(&self, next: &Self) -> bool;
    /// Extend `self` to absorb the immediately-following `next` span for this player.
    fn merge_span(&mut self, next: &Self);
}

macro_rules! impl_coalescible_span {
    ($t:ty, |$a:ident, $b:ident| $eq:expr) => {
        impl CoalescibleSpan for $t {
            fn span_player(&self) -> &PlayerId {
                &self.player
            }
            fn same_state(&self, next: &Self) -> bool {
                let $a = self;
                let $b = next;
                $eq
            }
            fn merge_span(&mut self, next: &Self) {
                self.end_time = next.end_time;
                self.end_frame = next.end_frame;
                self.duration += next.duration;
                self.player_position = next.player_position;
            }
        }
    };
}

/// Compare fraction fields by bit pattern so that pure-state runs (exactly `1.0`/`0.0`)
/// merge while sub-frame crossing frames stay distinct, without tripping float-cmp lints.
fn same_fraction(left: f32, right: f32) -> bool {
    left.to_bits() == right.to_bits()
}

impl_coalescible_span!(
    PositioningActivityEvent,
    |a, b| a.active == b.active && a.tracked == b.tracked && a.demolished == b.demolished
);
impl_coalescible_span!(PositioningPossessionEvent, |a, b| a.possession_state
    == b.possession_state);
impl_coalescible_span!(PositioningFieldZoneEvent, |a, b| same_fraction(
    a.defensive_zone_fraction,
    b.defensive_zone_fraction
) && same_fraction(a.neutral_zone_fraction, b.neutral_zone_fraction)
    && same_fraction(a.offensive_zone_fraction, b.offensive_zone_fraction)
    && same_fraction(a.defensive_half_fraction, b.defensive_half_fraction)
    && same_fraction(a.offensive_half_fraction, b.offensive_half_fraction));
impl_coalescible_span!(PositioningBallDepthEvent, |a, b| same_fraction(
    a.behind_ball_fraction,
    b.behind_ball_fraction
) && same_fraction(a.level_with_ball_fraction, b.level_with_ball_fraction)
    && same_fraction(a.in_front_of_ball_fraction, b.in_front_of_ball_fraction));
impl_coalescible_span!(PositioningTeammateRoleEvent, |a, b| a.teammate_role
    == b.teammate_role);
impl_coalescible_span!(
    PositioningBallProximityEvent,
    |a, b| a.closest_to_ball_team == b.closest_to_ball_team
        && a.closest_to_ball_absolute == b.closest_to_ball_absolute
        && a.farthest_from_ball == b.farthest_from_ball
);

/// Project each internal per-frame [`PositioningEvent`] through `project` and coalesce
/// consecutive same-state spans per player into a single span. A frame that projects to
/// `None` (the facet is inactive for that player) closes the player's open span so a
/// later resumption starts a fresh span rather than bridging the gap.
fn coalesce_facet<E: CoalescibleSpan>(
    events: &[PositioningEvent],
    project: impl Fn(&PositioningEvent) -> Option<E>,
) -> Vec<E> {
    let mut open: HashMap<PlayerId, usize> = HashMap::new();
    let mut out: Vec<E> = Vec::new();
    for event in events {
        match project(event) {
            Some(span) => {
                let player = span.span_player().clone();
                if let Some(&index) = open.get(&player) {
                    if out[index].same_state(&span) {
                        out[index].merge_span(&span);
                        continue;
                    }
                }
                open.insert(player, out.len());
                out.push(span);
            }
            None => {
                open.remove(&event.player);
            }
        }
    }
    out
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum PositioningPossessionState {
    HasPossession,
    NoPossession,
    #[default]
    Neutral,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum PositioningTeammateRoleState {
    NoTeammates,
    MostBack,
    MostForward,
    Mid,
    Other,
    #[default]
    Unknown,
}

/// Cumulative per-player distance totals.
///
/// Distance to the ball/teammates is a continuous magnitude, not a discrete occurrence, so
/// it cannot be reconstructed from an event stream; the calculator accumulates these running
/// totals as it processes frames and the timeline ships them directly. The possession-split
/// distance sums are bucketed by the per-frame possession state, but the possession *times*
/// themselves are categorical and are emitted as `PositioningPossessionEvent` spans instead.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct PositioningSignalSnapshot {
    pub sum_distance_to_teammates: f32,
    pub sum_distance_to_ball: f32,
    pub sum_distance_to_ball_has_possession: f32,
    pub sum_distance_to_ball_no_possession: f32,
}

impl PositioningSignalSnapshot {
    fn accumulate(&mut self, event: &PositioningEvent) {
        if !event.tracked {
            return;
        }
        if let Some(distance) = event.distance_to_teammates {
            self.sum_distance_to_teammates += distance * event.duration;
        }
        if let Some(distance) = event.distance_to_ball {
            self.sum_distance_to_ball += distance * event.duration;
            match event.possession_state {
                PositioningPossessionState::HasPossession => {
                    self.sum_distance_to_ball_has_possession += distance * event.duration;
                }
                PositioningPossessionState::NoPossession => {
                    self.sum_distance_to_ball_no_possession += distance * event.duration;
                }
                PositioningPossessionState::Neutral => {}
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct PositioningCalculatorConfig {
    pub most_back_forward_threshold_y: f32,
    pub level_ball_depth_margin: f32,
    pub closest_to_ball_switch_margin: f32,
    pub closest_to_ball_switch_min_seconds: f32,
}

impl Default for PositioningCalculatorConfig {
    fn default() -> Self {
        Self {
            most_back_forward_threshold_y: DEFAULT_MOST_BACK_FORWARD_THRESHOLD_Y,
            level_ball_depth_margin: DEFAULT_LEVEL_BALL_DEPTH_MARGIN,
            closest_to_ball_switch_margin: DEFAULT_CLOSEST_TO_BALL_SWITCH_MARGIN,
            closest_to_ball_switch_min_seconds: DEFAULT_CLOSEST_TO_BALL_SWITCH_MIN_SECONDS,
        }
    }
}

#[derive(Debug, Clone)]
struct ClosestToBallCandidate {
    player_id: PlayerId,
    player_position: Option<[f32; 3]>,
    is_team_0: bool,
    distance: f32,
}

impl ClosestToBallCandidate {
    fn from_player(player: &PlayerSample, position: glam::Vec3, ball_position: glam::Vec3) -> Self {
        Self {
            player_id: player.player_id.clone(),
            player_position: Some(position.to_array()),
            is_team_0: player.is_team_0,
            distance: position.distance(ball_position),
        }
    }
}

#[derive(Debug, Clone, Default)]
struct ClosestToBallDebouncer {
    current_player: Option<PlayerId>,
    pending_player: Option<PlayerId>,
    pending_seconds: f32,
}

impl ClosestToBallDebouncer {
    fn select(
        &mut self,
        candidates: &[ClosestToBallCandidate],
        dt: f32,
        switch_margin: f32,
        switch_min_seconds: f32,
    ) -> Option<ClosestToBallCandidate> {
        let raw_closest = candidates
            .iter()
            .min_by(|left, right| left.distance.partial_cmp(&right.distance).unwrap())?;
        let Some(current_player) = self.current_player.as_ref() else {
            self.current_player = Some(raw_closest.player_id.clone());
            self.pending_player = None;
            self.pending_seconds = 0.0;
            return Some(raw_closest.clone());
        };
        let Some(current) = candidates
            .iter()
            .find(|candidate| candidate.player_id == *current_player)
        else {
            self.current_player = Some(raw_closest.player_id.clone());
            self.pending_player = None;
            self.pending_seconds = 0.0;
            return Some(raw_closest.clone());
        };
        if raw_closest.player_id == current.player_id {
            self.pending_player = None;
            self.pending_seconds = 0.0;
            return Some(current.clone());
        }
        if raw_closest.distance + switch_margin >= current.distance {
            self.pending_player = None;
            self.pending_seconds = 0.0;
            return Some(current.clone());
        }
        if self.pending_player.as_ref() == Some(&raw_closest.player_id) {
            self.pending_seconds += dt;
        } else {
            self.pending_player = Some(raw_closest.player_id.clone());
            self.pending_seconds = dt;
        }
        if self.pending_seconds >= switch_min_seconds.max(0.0) {
            self.current_player = Some(raw_closest.player_id.clone());
            self.pending_player = None;
            self.pending_seconds = 0.0;
            Some(raw_closest.clone())
        } else {
            Some(current.clone())
        }
    }

    fn clear(&mut self) {
        self.current_player = None;
        self.pending_player = None;
        self.pending_seconds = 0.0;
    }
}

#[derive(Debug, Clone, Default)]
pub struct PositioningCalculator {
    config: PositioningCalculatorConfig,
    previous_ball_position: Option<glam::Vec3>,
    previous_player_positions: HashMap<PlayerId, glam::Vec3>,
    absolute_closest_to_ball: ClosestToBallDebouncer,
    team_zero_closest_to_ball: ClosestToBallDebouncer,
    team_one_closest_to_ball: ClosestToBallDebouncer,
    events: EventStream<PositioningEvent>,
    signal: HashMap<PlayerId, PositioningSignalSnapshot>,
}

impl PositioningCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(config: PositioningCalculatorConfig) -> Self {
        Self {
            config,
            ..Self::default()
        }
    }

    pub fn config(&self) -> &PositioningCalculatorConfig {
        &self.config
    }

    pub fn events(&self) -> &[PositioningEvent] {
        self.events.all()
    }

    pub fn new_events(&self) -> &[PositioningEvent] {
        self.events.new_events()
    }

    pub fn projected_events(&self) -> Vec<PositioningEvent> {
        self.events.all().to_vec()
    }

    pub fn activity_events(&self) -> Vec<PositioningActivityEvent> {
        coalesce_facet(self.events(), PositioningEvent::activity_event)
    }

    pub fn possession_events(&self) -> Vec<PositioningPossessionEvent> {
        coalesce_facet(self.events(), PositioningEvent::possession_event)
    }

    pub fn field_zone_events(&self) -> Vec<PositioningFieldZoneEvent> {
        coalesce_facet(self.events(), PositioningEvent::field_zone_event)
    }

    pub fn ball_depth_events(&self) -> Vec<PositioningBallDepthEvent> {
        coalesce_facet(self.events(), PositioningEvent::ball_depth_event)
    }

    pub fn teammate_role_events(&self) -> Vec<PositioningTeammateRoleEvent> {
        coalesce_facet(self.events(), PositioningEvent::teammate_role_event)
    }

    pub fn ball_proximity_events(&self) -> Vec<PositioningBallProximityEvent> {
        coalesce_facet(self.events(), PositioningEvent::ball_proximity_event)
    }

    pub fn goal_context_events(&self) -> Vec<PositioningGoalContextEvent> {
        self.events()
            .iter()
            .filter_map(PositioningEvent::goal_context_event)
            .collect()
    }

    pub fn flush_pending_events(&mut self) {}

    fn event_delta<'a>(
        deltas: &'a mut HashMap<PlayerId, PositioningEvent>,
        frame: &FrameInfo,
        player_id: &PlayerId,
        player_position: Option<[f32; 3]>,
        is_team_0: bool,
    ) -> &'a mut PositioningEvent {
        deltas.entry(player_id.clone()).or_insert_with(|| {
            PositioningEvent::new(frame, player_id.clone(), player_position, is_team_0)
        })
    }

    fn record_goal_positioning_events(
        &mut self,
        frame: &FrameInfo,
        players: &PlayerFrameState,
        events: &FrameEventsState,
        ball_position: glam::Vec3,
        event_deltas: &mut HashMap<PlayerId, PositioningEvent>,
    ) {
        for goal_event in &events.goal_events {
            let defending_team_is_team_0 = !goal_event.scoring_team_is_team_0;
            let normalized_ball_y = normalized_y(defending_team_is_team_0, ball_position);
            if normalized_ball_y > GOAL_CAUGHT_AHEAD_MAX_BALL_Y {
                continue;
            }

            for player in players
                .players
                .iter()
                .filter(|player| player.is_team_0 == defending_team_is_team_0)
            {
                let Some(position) = player.position() else {
                    continue;
                };
                let normalized_player_y = normalized_y(defending_team_is_team_0, position);
                if normalized_player_y < GOAL_CAUGHT_AHEAD_MIN_PLAYER_Y {
                    continue;
                }
                if normalized_player_y - normalized_ball_y < GOAL_CAUGHT_AHEAD_MIN_BALL_DELTA_Y {
                    continue;
                }

                Self::event_delta(
                    event_deltas,
                    frame,
                    &player.player_id,
                    Some(position.to_array()),
                    player.is_team_0,
                )
                .caught_ahead_of_play_on_conceded_goal = true;
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn process_sample(
        &mut self,
        frame: &FrameInfo,
        gameplay: &GameplayState,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        events: &FrameEventsState,
        live_play_state: &LivePlayState,
        possession_player_before_sample: Option<&PlayerId>,
    ) -> SubtrActorResult<()> {
        let live_play = live_play_state.is_live_play;
        if frame.dt == 0.0 {
            if let Some(ball) = ball.sample() {
                self.previous_ball_position = Some(ball.position());
            }
            for player in &players.players {
                if let Some(position) = player.position() {
                    self.previous_player_positions
                        .insert(player.player_id.clone(), position);
                }
            }
            return Ok(());
        }

        let Some(ball) = ball.sample() else {
            return Ok(());
        };
        let ball_position = ball.position();
        let mut event_deltas = HashMap::new();
        if !events.goal_events.is_empty() {
            self.record_goal_positioning_events(
                frame,
                players,
                events,
                ball_position,
                &mut event_deltas,
            );
        }
        let demoed_players: HashSet<_> = events
            .active_demos
            .iter()
            .map(|demo| demo.victim.clone())
            .collect();

        for player in &players.players {
            let is_demoed = demoed_players.contains(&player.player_id);
            if live_play && is_demoed {
                let delta = Self::event_delta(
                    &mut event_deltas,
                    frame,
                    &player.player_id,
                    player.position().map(|position| position.to_array()),
                    player.is_team_0,
                );
                delta.duration = frame.dt;
                delta.active = true;
                delta.demolished = true;
                continue;
            }

            let Some(position) = player.position() else {
                continue;
            };
            let previous_position = self
                .previous_player_positions
                .get(&player.player_id)
                .copied()
                .unwrap_or(position);
            let previous_ball_position = self.previous_ball_position.unwrap_or(ball_position);
            let normalized_position_y = normalized_y(player.is_team_0, position);
            let normalized_previous_position_y = normalized_y(player.is_team_0, previous_position);
            let normalized_ball_y = normalized_y(player.is_team_0, ball_position);
            let normalized_previous_ball_y = normalized_y(player.is_team_0, previous_ball_position);

            if live_play {
                let distance_to_ball = position.distance(ball_position);
                let delta = Self::event_delta(
                    &mut event_deltas,
                    frame,
                    &player.player_id,
                    Some(position.to_array()),
                    player.is_team_0,
                );
                delta.duration = frame.dt;
                delta.active = true;
                delta.tracked = true;
                delta.distance_to_ball = Some(distance_to_ball);

                if possession_player_before_sample == Some(&player.player_id) {
                    delta.possession_state = PositioningPossessionState::HasPossession;
                } else if possession_player_before_sample.is_some() {
                    delta.possession_state = PositioningPossessionState::NoPossession;
                }

                let defensive_zone_fraction = interval_fraction_below_threshold(
                    normalized_previous_position_y,
                    normalized_position_y,
                    -FIELD_ZONE_BOUNDARY_Y,
                );
                let offensive_zone_fraction = interval_fraction_above_threshold(
                    normalized_previous_position_y,
                    normalized_position_y,
                    FIELD_ZONE_BOUNDARY_Y,
                );
                let neutral_zone_fraction = interval_fraction_in_scalar_range(
                    normalized_previous_position_y,
                    normalized_position_y,
                    -FIELD_ZONE_BOUNDARY_Y,
                    FIELD_ZONE_BOUNDARY_Y,
                );
                delta.defensive_zone_fraction = defensive_zone_fraction;
                delta.neutral_zone_fraction = neutral_zone_fraction;
                delta.offensive_zone_fraction = offensive_zone_fraction;

                let defensive_half_fraction = interval_fraction_below_threshold(
                    normalized_previous_position_y,
                    normalized_position_y,
                    0.0,
                );
                delta.defensive_half_fraction = defensive_half_fraction;
                delta.offensive_half_fraction = 1.0 - defensive_half_fraction;

                let previous_ball_delta =
                    normalized_previous_position_y - normalized_previous_ball_y;
                let current_ball_delta = normalized_position_y - normalized_ball_y;
                let (behind_ball_fraction, level_ball_fraction, in_front_ball_fraction) =
                    ball_depth_fractions(
                        self.config.level_ball_depth_margin,
                        previous_ball_delta,
                        current_ball_delta,
                    );
                delta.behind_ball_fraction = behind_ball_fraction;
                delta.level_with_ball_fraction = level_ball_fraction;
                delta.in_front_of_ball_fraction = in_front_ball_fraction;
            }
        }

        if live_play {
            let positioned_players: Vec<_> = players
                .players
                .iter()
                .filter(|player| !demoed_players.contains(&player.player_id))
                .filter_map(|player| {
                    player.position().map(|position| {
                        ClosestToBallCandidate::from_player(player, position, ball_position)
                    })
                })
                .collect();
            if let Some(closest_player) = self.absolute_closest_to_ball.select(
                &positioned_players,
                frame.dt,
                self.config.closest_to_ball_switch_margin,
                self.config.closest_to_ball_switch_min_seconds,
            ) {
                Self::event_delta(
                    &mut event_deltas,
                    frame,
                    &closest_player.player_id,
                    closest_player.player_position,
                    closest_player.is_team_0,
                )
                .closest_to_ball_absolute = true;
            } else {
                self.absolute_closest_to_ball.clear();
            }

            for is_team_0 in [true, false] {
                let team_present_player_count = players
                    .players
                    .iter()
                    .filter(|player| player.is_team_0 == is_team_0)
                    .count();
                let team_roster_count = gameplay.current_in_game_team_player_count(is_team_0).max(
                    players
                        .players
                        .iter()
                        .filter(|player| player.is_team_0 == is_team_0)
                        .count(),
                );
                let team_players: Vec<_> = players
                    .players
                    .iter()
                    .filter(|player| player.is_team_0 == is_team_0)
                    .filter(|player| !demoed_players.contains(&player.player_id))
                    .filter_map(|player| player.position().map(|position| (player, position)))
                    .collect();

                if team_players.is_empty() {
                    if is_team_0 {
                        self.team_zero_closest_to_ball.clear();
                    } else {
                        self.team_one_closest_to_ball.clear();
                    }
                    continue;
                }

                for (player, position) in &team_players {
                    let teammate_distance_sum: f32 = team_players
                        .iter()
                        .filter(|(other_player, _)| other_player.player_id != player.player_id)
                        .map(|(_, other_position)| position.distance(*other_position))
                        .sum();
                    let teammate_count = team_players.len().saturating_sub(1);
                    if teammate_count > 0 {
                        Self::event_delta(
                            &mut event_deltas,
                            frame,
                            &player.player_id,
                            Some(position.to_array()),
                            player.is_team_0,
                        )
                        .distance_to_teammates =
                            Some(teammate_distance_sum / teammate_count as f32);
                    }
                }

                if team_roster_count < 2
                    || team_present_player_count < team_roster_count
                    || team_players.len() < 2
                {
                    for (player, position) in &team_players {
                        Self::event_delta(
                            &mut event_deltas,
                            frame,
                            &player.player_id,
                            Some(position.to_array()),
                            player.is_team_0,
                        )
                        .teammate_role = PositioningTeammateRoleState::NoTeammates;
                    }
                } else {
                    let mut sorted_team: Vec<_> = team_players
                        .iter()
                        .map(|(info, pos)| (info.player_id.clone(), normalized_y(is_team_0, *pos)))
                        .collect();
                    sorted_team.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap());

                    let team_spread = sorted_team.last().map(|(_, y)| *y).unwrap_or(0.0)
                        - sorted_team.first().map(|(_, y)| *y).unwrap_or(0.0);

                    if team_spread <= self.config.most_back_forward_threshold_y {
                        for (player_id, _) in &sorted_team {
                            let player_position = players.player_position(player_id);
                            Self::event_delta(
                                &mut event_deltas,
                                frame,
                                player_id,
                                player_position,
                                is_team_0,
                            )
                            .teammate_role = PositioningTeammateRoleState::Other;
                        }
                    } else {
                        let min_y = sorted_team.first().map(|(_, y)| *y).unwrap_or(0.0);
                        let max_y = sorted_team.last().map(|(_, y)| *y).unwrap_or(0.0);
                        let can_assign_mid_role = sorted_team.len() == 3;

                        for (player_id, y) in &sorted_team {
                            let near_back =
                                (*y - min_y) <= self.config.most_back_forward_threshold_y;
                            let near_front =
                                (max_y - *y) <= self.config.most_back_forward_threshold_y;

                            if near_back && !near_front {
                                Self::event_delta(
                                    &mut event_deltas,
                                    frame,
                                    player_id,
                                    players.player_position(player_id),
                                    is_team_0,
                                )
                                .teammate_role = PositioningTeammateRoleState::MostBack;
                            } else if near_front && !near_back {
                                Self::event_delta(
                                    &mut event_deltas,
                                    frame,
                                    player_id,
                                    players.player_position(player_id),
                                    is_team_0,
                                )
                                .teammate_role = PositioningTeammateRoleState::MostForward;
                            } else if can_assign_mid_role {
                                Self::event_delta(
                                    &mut event_deltas,
                                    frame,
                                    player_id,
                                    players.player_position(player_id),
                                    is_team_0,
                                )
                                .teammate_role = PositioningTeammateRoleState::Mid;
                            } else {
                                Self::event_delta(
                                    &mut event_deltas,
                                    frame,
                                    player_id,
                                    players.player_position(player_id),
                                    is_team_0,
                                )
                                .teammate_role = PositioningTeammateRoleState::Other;
                            }
                        }
                    }
                }

                let team_candidates: Vec<_> = team_players
                    .iter()
                    .map(|(player, position)| {
                        ClosestToBallCandidate::from_player(player, *position, ball_position)
                    })
                    .collect();
                let closest_player = if is_team_0 {
                    self.team_zero_closest_to_ball.select(
                        &team_candidates,
                        frame.dt,
                        self.config.closest_to_ball_switch_margin,
                        self.config.closest_to_ball_switch_min_seconds,
                    )
                } else {
                    self.team_one_closest_to_ball.select(
                        &team_candidates,
                        frame.dt,
                        self.config.closest_to_ball_switch_margin,
                        self.config.closest_to_ball_switch_min_seconds,
                    )
                };
                if let Some(closest_player) = closest_player {
                    let delta = Self::event_delta(
                        &mut event_deltas,
                        frame,
                        &closest_player.player_id,
                        closest_player.player_position,
                        closest_player.is_team_0,
                    );
                    delta.closest_to_ball = true;
                    delta.closest_to_ball_team = true;
                }

                if let Some((farthest_player, _)) = team_players.iter().max_by(|(_, a), (_, b)| {
                    a.distance(ball_position)
                        .partial_cmp(&b.distance(ball_position))
                        .unwrap()
                }) {
                    Self::event_delta(
                        &mut event_deltas,
                        frame,
                        &farthest_player.player_id,
                        farthest_player
                            .position()
                            .map(|position| position.to_array()),
                        farthest_player.is_team_0,
                    )
                    .farthest_from_ball = true;
                }
            }
        } else {
            self.absolute_closest_to_ball.clear();
            self.team_zero_closest_to_ball.clear();
            self.team_one_closest_to_ball.clear();
        }

        let frame_events: Vec<_> = event_deltas
            .into_values()
            .filter(PositioningEvent::has_delta)
            .collect();
        self.record_positioning_delta_events(frame_events);

        self.previous_ball_position = Some(ball_position);
        for player in &players.players {
            if let Some(position) = player.position() {
                self.previous_player_positions
                    .insert(player.player_id.clone(), position);
            }
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        frame: &FrameInfo,
        gameplay: &GameplayState,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        events: &FrameEventsState,
        live_play_state: &LivePlayState,
        possession_player_before_sample: Option<&PlayerId>,
    ) -> SubtrActorResult<()> {
        self.events.begin_update();
        self.process_sample(
            frame,
            gameplay,
            ball,
            players,
            events,
            live_play_state,
            possession_player_before_sample,
        )
    }

    fn record_positioning_delta_events(&mut self, mut frame_events: Vec<PositioningEvent>) {
        frame_events.sort_by(|left, right| {
            format!("{:?}", left.player).cmp(&format!("{:?}", right.player))
        });
        for event in &frame_events {
            self.signal
                .entry(event.player.clone())
                .or_default()
                .accumulate(event);
        }
        self.events.extend(frame_events);
    }

    /// Cumulative distance/possession signal for `player` through the frames processed so
    /// far. Continuous distance is shipped as this per-frame snapshot rather than as events.
    pub fn player_signal(&self, player: &PlayerId) -> PositioningSignalSnapshot {
        self.signal.get(player).copied().unwrap_or_default()
    }
}

fn ball_depth_fractions(level_margin: f32, start_delta: f32, end_delta: f32) -> (f32, f32, f32) {
    let behind_fraction = interval_fraction_below_threshold(start_delta, end_delta, -level_margin);
    let level_fraction =
        interval_fraction_in_scalar_range(start_delta, end_delta, -level_margin, level_margin);
    let in_front_fraction = (1.0 - behind_fraction - level_fraction).clamp(0.0, 1.0);
    (behind_fraction, level_fraction, in_front_fraction)
}

#[cfg(test)]
#[path = "positioning_tests.rs"]
mod tests;
