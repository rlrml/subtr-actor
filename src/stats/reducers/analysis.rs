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
        ctx: &AnalysisContext,
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

        self.evaluation_order = order
            .into_iter()
            .map(|id| id_to_index[&id])
            .collect();
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
            return SubtrActorError::new_result(
                SubtrActorErrorVariant::DerivedSignalGraphError(format!(
                "Cycle detected in derived signal graph at {node_id}"
            )),
            );
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
            Self::visit_node(
                dependency,
                id_to_index,
                nodes,
                visiting,
                visited,
                order,
            )?;
        }

        visiting.remove(&node_id);
        visited.insert(node_id);
        order.push(node_id);
        Ok(())
    }
}

#[derive(Default)]
pub struct TouchStateSignal {
    previous_ball_angular_velocity: Option<glam::Vec3>,
    current_last_touch: Option<TouchEvent>,
    live_play_tracker: LivePlayTracker,
}

impl TouchStateSignal {
    pub fn new() -> Self {
        Self::default()
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

    fn is_touch_candidate(&self, sample: &StatsSample) -> bool {
        const TOUCH_ANGULAR_VELOCITY_DELTA_EPSILON: f32 = 1e-3;

        let Some(current_velocity) = Self::current_ball_angular_velocity(sample) else {
            return false;
        };
        let Some(previous_velocity) = self.previous_ball_angular_velocity else {
            return false;
        };

        (current_velocity - previous_velocity).abs().max_element() > TOUCH_ANGULAR_VELOCITY_DELTA_EPSILON
    }

    fn candidate_touch_event(&self, sample: &StatsSample) -> Option<TouchEvent> {
        const OCTANE_HITBOX_LENGTH: f32 = 118.01;
        const OCTANE_HITBOX_WIDTH: f32 = 84.2;
        const OCTANE_HITBOX_HEIGHT: f32 = 36.16;
        const OCTANE_HITBOX_OFFSET: f32 = 13.88;
        const OCTANE_HITBOX_ELEVATION: f32 = 17.05;
        const TOUCH_COLLISION_DISTANCE_THRESHOLD: f32 = 300.0;

        let ball = sample.ball.as_ref()?;
        let ball_position = vec_to_glam(&ball.rigid_body.location);

        let best_candidate = sample
            .players
            .iter()
            .filter(|player| {
                sample
                    .possession_team_is_team_0
                    .map(|team_is_team_0| player.is_team_0 == team_is_team_0)
                    .unwrap_or(true)
            })
            .filter_map(|player| {
                let rigid_body = player.rigid_body.as_ref()?;
                let player_position = vec_to_glam(&rigid_body.location);
                let local_ball_position =
                    quat_to_glam(&rigid_body.rotation).inverse() * (ball_position - player_position);

                let x_distance = if local_ball_position.x < -OCTANE_HITBOX_LENGTH / 2.0 + OCTANE_HITBOX_OFFSET {
                    (-OCTANE_HITBOX_LENGTH / 2.0 + OCTANE_HITBOX_OFFSET) - local_ball_position.x
                } else if local_ball_position.x > OCTANE_HITBOX_LENGTH / 2.0 + OCTANE_HITBOX_OFFSET {
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
                let z_distance =
                    if local_ball_position.z < -OCTANE_HITBOX_HEIGHT / 2.0 + OCTANE_HITBOX_ELEVATION {
                        (-OCTANE_HITBOX_HEIGHT / 2.0 + OCTANE_HITBOX_ELEVATION) - local_ball_position.z
                    } else if local_ball_position.z
                        > OCTANE_HITBOX_HEIGHT / 2.0 + OCTANE_HITBOX_ELEVATION
                    {
                        local_ball_position.z
                            - (OCTANE_HITBOX_HEIGHT / 2.0 + OCTANE_HITBOX_ELEVATION)
                    } else {
                        0.0
                    };

                let collision_distance =
                    glam::Vec3::new(x_distance, y_distance, z_distance).length();
                Some((player, collision_distance))
            })
            .min_by(|(_, left), (_, right)| left.total_cmp(right));

        let team_is_team_0 = sample.possession_team_is_team_0.or_else(|| {
            best_candidate.map(|(player, _)| player.is_team_0)
        })?;
        let (player, collision_distance) = best_candidate
            .filter(|(_, distance)| *distance <= TOUCH_COLLISION_DISTANCE_THRESHOLD)
            .map(|(player, distance)| (Some(player.player_id.clone()), Some(distance)))
            .unwrap_or((None, None));

        Some(TouchEvent {
            time: sample.time,
            frame: sample.frame_number,
            team_is_team_0,
            player,
            closest_approach_distance: collision_distance,
        })
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
        let touch_events = if live_play && self.is_touch_candidate(sample) {
            self.candidate_touch_event(sample).into_iter().collect()
        } else {
            Vec::new()
        };

        if let Some(last_touch) = touch_events.last() {
            self.current_last_touch = Some(last_touch.clone());
        }
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
    current_team_is_team_0: Option<bool>,
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
        let touch_state = ctx
            .get::<TouchState>(TOUCH_STATE_SIGNAL_ID)
            .cloned()
            .unwrap_or_default();

        let active_team_before_sample = if touch_state.touch_events.is_empty() {
            self.current_team_is_team_0
                .or(sample.possession_team_is_team_0)
        } else {
            self.current_team_is_team_0
        };

        if let Some(team_is_team_0) = touch_state.last_touch_team_is_team_0 {
            self.current_team_is_team_0 = Some(team_is_team_0);
        } else {
            self.current_team_is_team_0 = sample
                .possession_team_is_team_0
                .or(self.current_team_is_team_0);
        }

        Ok(Some(Box::new(PossessionState {
            active_team_before_sample,
            current_team_is_team_0: self.current_team_is_team_0,
        })))
    }
}

pub fn default_derived_signal_graph() -> DerivedSignalGraph {
    DerivedSignalGraph::new()
        .with_signal(TouchStateSignal::new())
        .with_signal(PossessionStateSignal::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use boxcars::{Quaternion, RemoteId, RigidBody, Vector3f};
    use crate::stats::reducers::TouchReducer;

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
            .with_signal(TestSignal {
                id: "a",
                deps: &[],
            })
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
    fn touch_signal_detects_same_player_consecutive_touches() {
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
        assert_eq!(third_touch_state.touch_events.len(), 1);
        assert_eq!(
            third_touch_state.last_touch_player,
            Some(RemoteId::Steam(1))
        );
        reducer.on_sample_with_context(&third, third_ctx).unwrap();

        let stats = reducer.player_stats().get(&RemoteId::Steam(1)).unwrap();
        assert_eq!(stats.touch_count, 2);
        assert!(stats.is_last_touch);
    }
}
