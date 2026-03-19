use std::any::Any;
use std::collections::{HashMap, HashSet};

use crate::*;

pub type DerivedSignalId = &'static str;

pub const TOUCH_STATE_SIGNAL_ID: DerivedSignalId = "touch_state";
pub const POSSESSION_STATE_SIGNAL_ID: DerivedSignalId = "possession_state";

#[derive(Debug, Clone, Default)]
pub struct TouchState {
    pub touch_events: Vec<TouchEvent>,
    pub last_touch: Option<TouchEvent>,
    pub last_touch_player: Option<PlayerId>,
    pub last_touch_team_is_team_0: Option<bool>,
}

#[derive(Debug, Clone, Default)]
pub struct PossessionState {
    pub active_team_before_sample: Option<bool>,
    pub current_team_is_team_0: Option<bool>,
    pub active_player_before_sample: Option<PlayerId>,
    pub current_player: Option<PlayerId>,
}

#[derive(Default)]
pub struct AnalysisContext {
    values: HashMap<DerivedSignalId, Box<dyn Any>>,
}

impl AnalysisContext {
    pub fn get<T: 'static>(&self, id: DerivedSignalId) -> Option<&T> {
        self.values.get(id)?.downcast_ref::<T>()
    }

    fn insert_box(&mut self, id: DerivedSignalId, value: Box<dyn Any>) {
        self.values.insert(id, value);
    }

    fn clear(&mut self) {
        self.values.clear();
    }
}

pub trait DerivedSignal {
    fn id(&self) -> DerivedSignalId;

    fn dependencies(&self) -> &'static [DerivedSignalId] {
        &[]
    }

    fn on_replay_meta(&mut self, _meta: &ReplayMeta) -> SubtrActorResult<()> {
        Ok(())
    }

    fn evaluate(
        &mut self,
        sample: &StatsSample,
        _ctx: &AnalysisContext,
    ) -> SubtrActorResult<Option<Box<dyn Any>>>;

    fn finish(&mut self) -> SubtrActorResult<()> {
        Ok(())
    }
}

#[derive(Default)]
pub struct DerivedSignalGraph {
    nodes: Vec<Box<dyn DerivedSignal>>,
    evaluation_order: Vec<usize>,
    context: AnalysisContext,
    order_dirty: bool,
}

impl DerivedSignalGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_signal<S: DerivedSignal + 'static>(mut self, signal: S) -> Self {
        self.push(signal);
        self
    }

    pub fn push<S: DerivedSignal + 'static>(&mut self, signal: S) {
        self.nodes.push(Box::new(signal));
        self.order_dirty = true;
    }

    pub fn on_replay_meta(&mut self, meta: &ReplayMeta) -> SubtrActorResult<()> {
        self.rebuild_order_if_needed()?;
        for node in &mut self.nodes {
            node.on_replay_meta(meta)?;
        }
        Ok(())
    }

    pub fn evaluate(&mut self, sample: &StatsSample) -> SubtrActorResult<&AnalysisContext> {
        self.rebuild_order_if_needed()?;
        self.context.clear();

        for node_index in &self.evaluation_order {
            let node = &mut self.nodes[*node_index];
            if let Some(value) = node.evaluate(sample, &self.context)? {
                self.context.insert_box(node.id(), value);
            }
        }

        Ok(&self.context)
    }

    pub fn finish(&mut self) -> SubtrActorResult<()> {
        for node in &mut self.nodes {
            node.finish()?;
        }
        Ok(())
    }

    fn rebuild_order_if_needed(&mut self) -> SubtrActorResult<()> {
        if !self.order_dirty {
            return Ok(());
        }

        let id_to_index: HashMap<_, _> = self
            .nodes
            .iter()
            .enumerate()
            .map(|(index, node)| (node.id(), index))
            .collect();
        let mut visiting = HashSet::new();
        let mut visited = HashSet::new();
        let mut order = Vec::with_capacity(self.nodes.len());

        for node in &self.nodes {
            Self::visit_node(
                node.id(),
                &id_to_index,
                &self.nodes,
                &mut visiting,
                &mut visited,
                &mut order,
            )?;
        }

        self.evaluation_order = order.into_iter().map(|id| id_to_index[&id]).collect();
        self.order_dirty = false;
        Ok(())
    }

    fn visit_node(
        node_id: DerivedSignalId,
        id_to_index: &HashMap<DerivedSignalId, usize>,
        nodes: &[Box<dyn DerivedSignal>],
        visiting: &mut HashSet<DerivedSignalId>,
        visited: &mut HashSet<DerivedSignalId>,
        order: &mut Vec<DerivedSignalId>,
    ) -> SubtrActorResult<()> {
        if visited.contains(&node_id) {
            return Ok(());
        }
        if !visiting.insert(node_id) {
            return SubtrActorError::new_result(SubtrActorErrorVariant::DerivedSignalGraphError(
                format!("Cycle detected in derived signal graph at {node_id}"),
            ));
        }

        let node = &nodes[id_to_index[&node_id]];
        for dependency in node.dependencies() {
            if !id_to_index.contains_key(dependency) {
                return SubtrActorError::new_result(
                    SubtrActorErrorVariant::DerivedSignalGraphError(format!(
                        "Missing derived signal dependency {dependency} for {node_id}"
                    )),
                );
            }
            Self::visit_node(dependency, id_to_index, nodes, visiting, visited, order)?;
        }

        visiting.remove(&node_id);
        visited.insert(node_id);
        order.push(node_id);
        Ok(())
    }
}

#[derive(Default)]
pub struct TouchStateSignal {
    previous_ball_linear_velocity: Option<glam::Vec3>,
    previous_ball_angular_velocity: Option<glam::Vec3>,
    current_last_touch: Option<TouchEvent>,
    recent_touch_candidates: HashMap<PlayerId, TouchEvent>,
    live_play_tracker: LivePlayTracker,
}

impl TouchStateSignal {
    pub fn new() -> Self {
        Self::default()
    }

    fn should_emit_candidate(&self, candidate: &TouchEvent) -> bool {
        const SAME_PLAYER_TOUCH_COOLDOWN_FRAMES: usize = 7;

        let Some(previous_touch) = self.current_last_touch.as_ref() else {
            return true;
        };

        let same_player =
            previous_touch.player.is_some() && previous_touch.player == candidate.player;
        if !same_player {
            return true;
        }

        candidate.frame.saturating_sub(previous_touch.frame) >= SAME_PLAYER_TOUCH_COOLDOWN_FRAMES
    }

    fn prune_recent_touch_candidates(&mut self, current_frame: usize) {
        const TOUCH_CANDIDATE_WINDOW_FRAMES: usize = 4;

        self.recent_touch_candidates.retain(|_, candidate| {
            current_frame.saturating_sub(candidate.frame) <= TOUCH_CANDIDATE_WINDOW_FRAMES
        });
    }

    fn current_ball_angular_velocity(sample: &StatsSample) -> Option<glam::Vec3> {
        sample
            .ball
            .as_ref()
            .map(|ball| {
                ball.rigid_body
                    .angular_velocity
                    .unwrap_or(boxcars::Vector3f {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    })
            })
            .map(|velocity| vec_to_glam(&velocity))
    }

    fn current_ball_linear_velocity(sample: &StatsSample) -> Option<glam::Vec3> {
        sample.ball.as_ref().map(BallSample::velocity)
    }

    fn is_touch_candidate(&self, sample: &StatsSample) -> bool {
        const BALL_GRAVITY_Z: f32 = -650.0;
        const TOUCH_LINEAR_IMPULSE_THRESHOLD: f32 = 120.0;
        const TOUCH_ANGULAR_VELOCITY_DELTA_THRESHOLD: f32 = 0.5;

        let Some(current_linear_velocity) = Self::current_ball_linear_velocity(sample) else {
            return false;
        };
        let Some(previous_linear_velocity) = self.previous_ball_linear_velocity else {
            return false;
        };
        let Some(current_angular_velocity) = Self::current_ball_angular_velocity(sample) else {
            return false;
        };
        let Some(previous_angular_velocity) = self.previous_ball_angular_velocity else {
            return false;
        };

        let expected_linear_delta = glam::Vec3::new(0.0, 0.0, BALL_GRAVITY_Z * sample.dt.max(0.0));
        let residual_linear_impulse =
            current_linear_velocity - previous_linear_velocity - expected_linear_delta;
        let angular_velocity_delta = current_angular_velocity - previous_angular_velocity;

        residual_linear_impulse.length() > TOUCH_LINEAR_IMPULSE_THRESHOLD
            || angular_velocity_delta.length() > TOUCH_ANGULAR_VELOCITY_DELTA_THRESHOLD
    }

    fn proximity_touch_candidates(
        &self,
        sample: &StatsSample,
        max_collision_distance: f32,
    ) -> Vec<TouchEvent> {
        const OCTANE_HITBOX_LENGTH: f32 = 118.01;
        const OCTANE_HITBOX_WIDTH: f32 = 84.2;
        const OCTANE_HITBOX_HEIGHT: f32 = 36.16;
        const OCTANE_HITBOX_OFFSET: f32 = 13.88;
        const OCTANE_HITBOX_ELEVATION: f32 = 17.05;

        let Some(ball) = sample.ball.as_ref() else {
            return Vec::new();
        };
        let ball_position = vec_to_glam(&ball.rigid_body.location);

        let mut candidates = sample
            .players
            .iter()
            .filter_map(|player| {
                let rigid_body = player.rigid_body.as_ref()?;
                let player_position = vec_to_glam(&rigid_body.location);
                let local_ball_position = quat_to_glam(&rigid_body.rotation).inverse()
                    * (ball_position - player_position);

                let x_distance = if local_ball_position.x
                    < -OCTANE_HITBOX_LENGTH / 2.0 + OCTANE_HITBOX_OFFSET
                {
                    (-OCTANE_HITBOX_LENGTH / 2.0 + OCTANE_HITBOX_OFFSET) - local_ball_position.x
                } else if local_ball_position.x > OCTANE_HITBOX_LENGTH / 2.0 + OCTANE_HITBOX_OFFSET
                {
                    local_ball_position.x - (OCTANE_HITBOX_LENGTH / 2.0 + OCTANE_HITBOX_OFFSET)
                } else {
                    0.0
                };
                let y_distance = if local_ball_position.y < -OCTANE_HITBOX_WIDTH / 2.0 {
                    (-OCTANE_HITBOX_WIDTH / 2.0) - local_ball_position.y
                } else if local_ball_position.y > OCTANE_HITBOX_WIDTH / 2.0 {
                    local_ball_position.y - OCTANE_HITBOX_WIDTH / 2.0
                } else {
                    0.0
                };
                let z_distance = if local_ball_position.z
                    < -OCTANE_HITBOX_HEIGHT / 2.0 + OCTANE_HITBOX_ELEVATION
                {
                    (-OCTANE_HITBOX_HEIGHT / 2.0 + OCTANE_HITBOX_ELEVATION) - local_ball_position.z
                } else if local_ball_position.z
                    > OCTANE_HITBOX_HEIGHT / 2.0 + OCTANE_HITBOX_ELEVATION
                {
                    local_ball_position.z - (OCTANE_HITBOX_HEIGHT / 2.0 + OCTANE_HITBOX_ELEVATION)
                } else {
                    0.0
                };

                let collision_distance =
                    glam::Vec3::new(x_distance, y_distance, z_distance).length();
                if collision_distance > max_collision_distance {
                    return None;
                }

                Some(TouchEvent {
                    time: sample.time,
                    frame: sample.frame_number,
                    team_is_team_0: player.is_team_0,
                    player: Some(player.player_id.clone()),
                    closest_approach_distance: Some(collision_distance),
                })
            })
            .collect::<Vec<_>>();

        candidates.sort_by(|left, right| {
            let left_distance = left.closest_approach_distance.unwrap_or(f32::INFINITY);
            let right_distance = right.closest_approach_distance.unwrap_or(f32::INFINITY);
            left_distance.total_cmp(&right_distance)
        });
        candidates
    }

    fn candidate_touch_event(&self, sample: &StatsSample) -> Option<TouchEvent> {
        const TOUCH_COLLISION_DISTANCE_THRESHOLD: f32 = 300.0;

        self.proximity_touch_candidates(sample, TOUCH_COLLISION_DISTANCE_THRESHOLD)
            .into_iter()
            .next()
    }

    fn update_recent_touch_candidates(&mut self, sample: &StatsSample) {
        const PROXIMITY_CANDIDATE_DISTANCE_THRESHOLD: f32 = 220.0;

        for candidate in
            self.proximity_touch_candidates(sample, PROXIMITY_CANDIDATE_DISTANCE_THRESHOLD)
        {
            let Some(player_id) = candidate.player.clone() else {
                continue;
            };

            self.recent_touch_candidates.insert(player_id, candidate);
        }
    }

    fn candidate_for_player(&self, player_id: &PlayerId) -> Option<TouchEvent> {
        self.recent_touch_candidates.get(player_id).cloned()
    }

    fn contested_touch_candidates(&self, primary: &TouchEvent) -> Vec<TouchEvent> {
        const CONTESTED_TOUCH_DISTANCE_MARGIN: f32 = 80.0;

        let primary_distance = primary.closest_approach_distance.unwrap_or(f32::INFINITY);

        let best_opposing_candidate = self
            .recent_touch_candidates
            .values()
            .filter(|candidate| candidate.team_is_team_0 != primary.team_is_team_0)
            .filter(|candidate| {
                candidate.closest_approach_distance.unwrap_or(f32::INFINITY)
                    <= primary_distance + CONTESTED_TOUCH_DISTANCE_MARGIN
            })
            .min_by(|left, right| {
                let left_distance = left.closest_approach_distance.unwrap_or(f32::INFINITY);
                let right_distance = right.closest_approach_distance.unwrap_or(f32::INFINITY);
                left_distance.total_cmp(&right_distance)
            })
            .cloned();

        best_opposing_candidate.into_iter().collect()
    }

    fn confirmed_touch_events(&self, sample: &StatsSample) -> Vec<TouchEvent> {
        let mut touch_events = Vec::new();
        let mut confirmed_players = HashSet::new();

        if self.is_touch_candidate(sample) {
            if let Some(candidate) = self.candidate_touch_event(sample) {
                for contested_candidate in self.contested_touch_candidates(&candidate) {
                    if let Some(player_id) = contested_candidate.player.clone() {
                        confirmed_players.insert(player_id);
                    }
                    touch_events.push(contested_candidate);
                }
                if let Some(player_id) = candidate.player.clone() {
                    confirmed_players.insert(player_id);
                }
                touch_events.push(candidate);
            }
        }

        for dodge_refresh in &sample.dodge_refreshed_events {
            if !confirmed_players.insert(dodge_refresh.player.clone()) {
                continue;
            }
            let Some(candidate) = self.candidate_for_player(&dodge_refresh.player) else {
                continue;
            };
            touch_events.push(candidate);
        }

        touch_events
    }
}

impl DerivedSignal for TouchStateSignal {
    fn id(&self) -> DerivedSignalId {
        TOUCH_STATE_SIGNAL_ID
    }

    fn evaluate(
        &mut self,
        sample: &StatsSample,
        _ctx: &AnalysisContext,
    ) -> SubtrActorResult<Option<Box<dyn Any>>> {
        let live_play = self.live_play_tracker.is_live_play(sample);
        let touch_events = if live_play {
            self.prune_recent_touch_candidates(sample.frame_number);
            self.update_recent_touch_candidates(sample);
            self.confirmed_touch_events(sample)
                .into_iter()
                .filter(|candidate| self.should_emit_candidate(candidate))
                .collect()
        } else {
            self.current_last_touch = None;
            self.recent_touch_candidates.clear();
            Vec::new()
        };

        if let Some(last_touch) = touch_events.last() {
            self.current_last_touch = Some(last_touch.clone());
        }
        self.previous_ball_linear_velocity = Self::current_ball_linear_velocity(sample);
        self.previous_ball_angular_velocity = Self::current_ball_angular_velocity(sample);

        let output = TouchState {
            touch_events,
            last_touch: self.current_last_touch.clone(),
            last_touch_player: self
                .current_last_touch
                .as_ref()
                .and_then(|touch| touch.player.clone()),
            last_touch_team_is_team_0: self
                .current_last_touch
                .as_ref()
                .map(|touch| touch.team_is_team_0),
        };
        Ok(Some(Box::new(output)))
    }
}

#[derive(Default)]
pub struct PossessionStateSignal {
    tracker: PossessionTracker,
    live_play_tracker: LivePlayTracker,
}

impl PossessionStateSignal {
    pub fn new() -> Self {
        Self::default()
    }
}

impl DerivedSignal for PossessionStateSignal {
    fn id(&self) -> DerivedSignalId {
        POSSESSION_STATE_SIGNAL_ID
    }

    fn dependencies(&self) -> &'static [DerivedSignalId] {
        &[TOUCH_STATE_SIGNAL_ID]
    }

    fn evaluate(
        &mut self,
        sample: &StatsSample,
        ctx: &AnalysisContext,
    ) -> SubtrActorResult<Option<Box<dyn Any>>> {
        let live_play = self.live_play_tracker.is_live_play(sample);
        if !live_play {
            self.tracker.reset();
            return Ok(Some(Box::new(PossessionState {
                active_team_before_sample: None,
                current_team_is_team_0: None,
                active_player_before_sample: None,
                current_player: None,
            })));
        }

        let touch_state = ctx
            .get::<TouchState>(TOUCH_STATE_SIGNAL_ID)
            .cloned()
            .unwrap_or_default();
        Ok(Some(Box::new(
            self.tracker.update(sample, &touch_state.touch_events),
        )))
    }
}

#[derive(Default)]
pub struct FiftyFiftyStateSignal {
    active_event: Option<ActiveFiftyFifty>,
    last_resolved_event: Option<FiftyFiftyEvent>,
    kickoff_touch_window_open: bool,
    live_play_tracker: LivePlayTracker,
}

impl FiftyFiftyStateSignal {
    pub fn new() -> Self {
        Self::default()
    }

    fn reset(&mut self) {
        self.active_event = None;
    }

    fn maybe_resolve_active_event(
        &mut self,
        sample: &StatsSample,
        possession_state: &PossessionState,
    ) -> Option<FiftyFiftyEvent> {
        let active = self.active_event.as_ref()?;
        let age = (sample.time - active.last_touch_time).max(0.0);
        if age < FIFTY_FIFTY_RESOLUTION_DELAY_SECONDS {
            return None;
        }

        let winning_team_is_team_0 = FiftyFiftyReducer::winning_team_from_ball(active, sample);
        let possession_team_is_team_0 = possession_state.current_team_is_team_0;
        let should_resolve = winning_team_is_team_0.is_some()
            || possession_team_is_team_0.is_some()
            || age >= FIFTY_FIFTY_MAX_DURATION_SECONDS;
        if !should_resolve {
            return None;
        }

        let active = self.active_event.take()?;
        let event = FiftyFiftyEvent {
            start_time: active.start_time,
            start_frame: active.start_frame,
            resolve_time: sample.time,
            resolve_frame: sample.frame_number,
            is_kickoff: active.is_kickoff,
            team_zero_player: active.team_zero_player,
            team_one_player: active.team_one_player,
            team_zero_position: active.team_zero_position,
            team_one_position: active.team_one_position,
            midpoint: active.midpoint,
            plane_normal: active.plane_normal,
            winning_team_is_team_0,
            possession_team_is_team_0,
        };
        self.last_resolved_event = Some(event.clone());
        Some(event)
    }
}

impl DerivedSignal for FiftyFiftyStateSignal {
    fn id(&self) -> DerivedSignalId {
        FIFTY_FIFTY_STATE_SIGNAL_ID
    }

    fn dependencies(&self) -> &'static [DerivedSignalId] {
        &[TOUCH_STATE_SIGNAL_ID, POSSESSION_STATE_SIGNAL_ID]
    }

    fn evaluate(
        &mut self,
        sample: &StatsSample,
        ctx: &AnalysisContext,
    ) -> SubtrActorResult<Option<Box<dyn Any>>> {
        let live_play = self.live_play_tracker.is_live_play(sample);
        let touch_state = ctx
            .get::<TouchState>(TOUCH_STATE_SIGNAL_ID)
            .cloned()
            .unwrap_or_default();
        let possession_state = ctx
            .get::<PossessionState>(POSSESSION_STATE_SIGNAL_ID)
            .cloned()
            .unwrap_or_default();

        if FiftyFiftyReducer::kickoff_phase_active(sample) {
            self.kickoff_touch_window_open = true;
        }

        if !live_play {
            self.reset();
            return Ok(Some(Box::new(FiftyFiftyState {
                active_event: None,
                resolved_events: Vec::new(),
                last_resolved_event: self.last_resolved_event.clone(),
            })));
        }

        let has_touch = !touch_state.touch_events.is_empty();
        let has_contested_touch = touch_state
            .touch_events
            .iter()
            .any(|touch| touch.team_is_team_0)
            && touch_state
                .touch_events
                .iter()
                .any(|touch| !touch.team_is_team_0);

        if let Some(active_event) = self.active_event.as_mut() {
            let age = (sample.time - active_event.last_touch_time).max(0.0);
            if age <= FIFTY_FIFTY_CONTINUATION_TOUCH_WINDOW_SECONDS
                && active_event.contains_team_touch(&touch_state.touch_events)
            {
                active_event.last_touch_time = sample.time;
                active_event.last_touch_frame = sample.frame_number;
            }
        }

        let mut resolved_events = Vec::new();
        if let Some(event) = self.maybe_resolve_active_event(sample, &possession_state) {
            resolved_events.push(event);
        }

        if has_contested_touch {
            if self.active_event.is_none() {
                self.active_event = FiftyFiftyReducer::contested_touch(
                    sample,
                    &touch_state.touch_events,
                    self.kickoff_touch_window_open,
                );
            }
        } else if has_touch {
            if let Some(active_event) = self.active_event.as_mut() {
                let age = (sample.time - active_event.last_touch_time).max(0.0);
                if age <= FIFTY_FIFTY_CONTINUATION_TOUCH_WINDOW_SECONDS
                    && active_event.contains_team_touch(&touch_state.touch_events)
                {
                    active_event.last_touch_time = sample.time;
                    active_event.last_touch_frame = sample.frame_number;
                }
            }
        }

        if has_touch {
            self.kickoff_touch_window_open = false;
        }

        Ok(Some(Box::new(FiftyFiftyState {
            active_event: self.active_event.clone(),
            resolved_events,
            last_resolved_event: self.last_resolved_event.clone(),
        })))
    }
}

pub fn default_derived_signal_graph() -> DerivedSignalGraph {
    DerivedSignalGraph::new()
        .with_signal(TouchStateSignal::new())
        .with_signal(PossessionStateSignal::new())
        .with_signal(FiftyFiftyStateSignal::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::reducers::TouchReducer;
    use boxcars::{Quaternion, RemoteId, RigidBody, Vector3f};

    #[derive(Default)]
    struct TestSignal {
        id: DerivedSignalId,
        deps: &'static [DerivedSignalId],
    }

    impl DerivedSignal for TestSignal {
        fn id(&self) -> DerivedSignalId {
            self.id
        }

        fn dependencies(&self) -> &'static [DerivedSignalId] {
            self.deps
        }

        fn evaluate(
            &mut self,
            _sample: &StatsSample,
            _ctx: &AnalysisContext,
        ) -> SubtrActorResult<Option<Box<dyn Any>>> {
            Ok(None)
        }
    }

    #[test]
    fn topo_sorts_dependencies_before_dependents() {
        let mut graph = DerivedSignalGraph::new()
            .with_signal(TestSignal {
                id: "c",
                deps: &["b"],
            })
            .with_signal(TestSignal { id: "a", deps: &[] })
            .with_signal(TestSignal {
                id: "b",
                deps: &["a"],
            });

        graph.rebuild_order_if_needed().unwrap();
        let ordered_ids: Vec<_> = graph
            .evaluation_order
            .iter()
            .map(|index| graph.nodes[*index].id())
            .collect();
        assert_eq!(ordered_ids, vec!["a", "b", "c"]);
    }

    fn rigid_body(x: f32, y: f32, z: f32, ang_vel_z: f32) -> RigidBody {
        RigidBody {
            sleeping: false,
            location: Vector3f { x, y, z },
            rotation: Quaternion {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                w: 1.0,
            },
            linear_velocity: Some(Vector3f {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            }),
            angular_velocity: Some(Vector3f {
                x: 0.0,
                y: 0.0,
                z: ang_vel_z,
            }),
        }
    }

    fn sample(frame_number: usize, time: f32, ball_ang_vel_z: f32) -> StatsSample {
        StatsSample {
            frame_number,
            time,
            dt: 1.0 / 120.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: Some(true),
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: Some([1, 1]),
            ball: Some(BallSample {
                rigid_body: rigid_body(70.0, 0.0, 20.0, ball_ang_vel_z),
            }),
            players: vec![
                PlayerSample {
                    player_id: RemoteId::Steam(1),
                    is_team_0: true,
                    rigid_body: Some(rigid_body(0.0, 0.0, 0.0, 0.0)),
                    boost_amount: None,
                    last_boost_amount: None,
                    boost_active: false,
                    dodge_active: false,
                    powerslide_active: false,
                    match_goals: None,
                    match_assists: None,
                    match_saves: None,
                    match_shots: None,
                    match_score: None,
                },
                PlayerSample {
                    player_id: RemoteId::Steam(2),
                    is_team_0: false,
                    rigid_body: Some(rigid_body(3000.0, 0.0, 0.0, 0.0)),
                    boost_amount: None,
                    last_boost_amount: None,
                    boost_active: false,
                    dodge_active: false,
                    powerslide_active: false,
                    match_goals: None,
                    match_assists: None,
                    match_saves: None,
                    match_shots: None,
                    match_score: None,
                },
            ],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        }
    }

    #[test]
    fn touch_signal_dedupes_same_player_consecutive_touches() {
        let mut graph = default_derived_signal_graph();
        let mut reducer = TouchReducer::new();

        let first = sample(0, 0.0, 0.0);
        let second = sample(1, 1.0 / 120.0, 5.0);
        let third = sample(2, 2.0 / 120.0, 10.0);

        let first_ctx = graph.evaluate(&first).unwrap();
        reducer.on_sample_with_context(&first, first_ctx).unwrap();

        let second_ctx = graph.evaluate(&second).unwrap();
        let second_touch_state = second_ctx.get::<TouchState>(TOUCH_STATE_SIGNAL_ID).unwrap();
        assert_eq!(second_touch_state.touch_events.len(), 1);
        assert_eq!(
            second_touch_state.last_touch_player,
            Some(RemoteId::Steam(1))
        );
        reducer.on_sample_with_context(&second, second_ctx).unwrap();

        let third_ctx = graph.evaluate(&third).unwrap();
        let third_touch_state = third_ctx.get::<TouchState>(TOUCH_STATE_SIGNAL_ID).unwrap();
        assert_eq!(third_touch_state.touch_events.len(), 0);
        assert_eq!(
            third_touch_state.last_touch_player,
            Some(RemoteId::Steam(1))
        );
        reducer.on_sample_with_context(&third, third_ctx).unwrap();

        let stats = reducer.player_stats().get(&RemoteId::Steam(1)).unwrap();
        assert_eq!(stats.touch_count, 1);
        assert!(stats.is_last_touch);
    }

    #[test]
    fn touch_signal_confirms_nearby_candidate_from_dodge_refresh() {
        let mut graph = default_derived_signal_graph();
        let mut reducer = TouchReducer::new();

        let first = sample(0, 0.0, 0.0);
        let mut second = sample(1, 1.0 / 120.0, 0.0);
        second.dodge_refreshed_events.push(DodgeRefreshedEvent {
            time: second.time,
            frame: second.frame_number,
            player: RemoteId::Steam(1),
            is_team_0: true,
            counter_value: 1,
        });

        let first_ctx = graph.evaluate(&first).unwrap();
        reducer.on_sample_with_context(&first, first_ctx).unwrap();

        let second_ctx = graph.evaluate(&second).unwrap();
        let second_touch_state = second_ctx.get::<TouchState>(TOUCH_STATE_SIGNAL_ID).unwrap();
        assert_eq!(second_touch_state.touch_events.len(), 1);
        assert_eq!(
            second_touch_state.last_touch_player,
            Some(RemoteId::Steam(1))
        );
        reducer.on_sample_with_context(&second, second_ctx).unwrap();

        let stats = reducer.player_stats().get(&RemoteId::Steam(1)).unwrap();
        assert_eq!(stats.touch_count, 1);
        assert!(stats.is_last_touch);
    }

    #[test]
    fn touch_signal_clears_last_touch_across_kickoff() {
        let mut graph = default_derived_signal_graph();

        let first = sample(0, 0.0, 0.0);
        let second = sample(1, 1.0 / 120.0, 5.0);
        let mut kickoff = sample(2, 2.0 / 120.0, 10.0);
        kickoff.game_state = Some(55);
        kickoff.kickoff_countdown_time = Some(3);
        kickoff.ball_has_been_hit = Some(false);

        graph.evaluate(&first).unwrap();
        let second_ctx = graph.evaluate(&second).unwrap();
        let second_touch_state = second_ctx.get::<TouchState>(TOUCH_STATE_SIGNAL_ID).unwrap();
        assert_eq!(
            second_touch_state.last_touch_player,
            Some(RemoteId::Steam(1))
        );

        let kickoff_ctx = graph.evaluate(&kickoff).unwrap();
        let kickoff_touch_state = kickoff_ctx
            .get::<TouchState>(TOUCH_STATE_SIGNAL_ID)
            .unwrap();
        assert!(kickoff_touch_state.touch_events.is_empty());
        assert_eq!(kickoff_touch_state.last_touch, None);
        assert_eq!(kickoff_touch_state.last_touch_player, None);
        assert_eq!(kickoff_touch_state.last_touch_team_is_team_0, None);
    }
}
