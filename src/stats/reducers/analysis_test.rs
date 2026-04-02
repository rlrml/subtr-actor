use super::*;
use crate::stats::reducers::StatsReducer;
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
        _sample: &CoreSample,
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

fn sample(frame_number: usize, time: f32, ball_ang_vel_z: f32) -> CoreSample {
    CoreSample {
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
