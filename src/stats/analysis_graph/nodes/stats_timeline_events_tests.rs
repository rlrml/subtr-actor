use std::collections::HashMap;

use boxcars::{Quaternion, RemoteId, RigidBody, Vector3f};

use crate::stats::analysis_graph::{AnalysisGraph, graph_with_all_analysis_nodes};
use crate::stats::calculators::{
    BallFrameState, BallSample, FrameEventsState, FrameInfo, FrameInput, GameplayState,
    PlayerFrameState, PlayerSample,
};
use crate::{
    BoostPadEvent, BoostPadEventKind, CarHitbox, EventLifecycle, PlayerId, PlayerInfo, ReplayMeta,
};

const FPS: f32 = 30.0;
const PICKUP_FRAME: usize = 15;
const LAST_FRAME: usize = 89;

fn player_id(index: u32) -> PlayerId {
    RemoteId::SplitScreen(index)
}

fn rigid_body(location: Vector3f) -> RigidBody {
    RigidBody {
        sleeping: false,
        location,
        rotation: Quaternion {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        },
        linear_velocity: Some(Vector3f {
            x: 0.0,
            y: 100.0,
            z: 0.0,
        }),
        angular_velocity: Some(Vector3f {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }),
    }
}

fn player_info(index: u32) -> PlayerInfo {
    PlayerInfo {
        remote_id: player_id(index),
        stats: None,
        name: format!("Player {index}"),
        car_body_id: None,
        car_body_name: None,
        car_hitbox_family: Some("Octane".to_owned()),
        camera_settings: None,
    }
}

fn replay_meta() -> ReplayMeta {
    ReplayMeta {
        team_zero: vec![player_info(0)],
        team_one: vec![player_info(1)],
        game_type: Default::default(),
        season: None,
        all_headers: Vec::new(),
    }
}

fn player_sample(index: u32, is_team_0: bool, location: Vector3f) -> PlayerSample {
    PlayerSample {
        player_id: player_id(index),
        is_team_0,
        hitbox: CarHitbox::octane(),
        rigid_body: Some(rigid_body(location)),
        boost_amount: Some(85.0),
        last_boost_amount: Some(85.0),
        boost_active: false,
        dodge_active: false,
        dodge_torque: None,
        powerslide_active: false,
        match_goals: None,
        match_assists: None,
        match_saves: None,
        match_shots: None,
        match_score: None,
    }
}

/// A minimal scripted frame: two players far from the ball, plus an explicit
/// boost pad pickup at [`PICKUP_FRAME`] so at least one discrete timeline
/// event exists mid-match.
fn frame_input(frame_number: usize, time: f32) -> FrameInput {
    let progress = frame_number as f32 * 10.0;
    let mut frame_events_state = FrameEventsState::default();
    if frame_number == PICKUP_FRAME {
        frame_events_state.boost_pad_events.push(BoostPadEvent {
            time,
            frame: frame_number,
            pad_id: "34".to_owned(),
            player: Some(player_id(0)),
            player_position: Some(Vector3f {
                x: -1000.0,
                y: -4000.0 + progress,
                z: 17.0,
            }),
            kind: BoostPadEventKind::PickedUp { sequence: 1 },
        });
    }
    FrameInput::from_parts(
        FrameInfo {
            frame_number,
            time,
            dt: if frame_number == 0 { 0.0 } else { 1.0 / FPS },
            seconds_remaining: Some(280),
        },
        GameplayState {
            game_state: None,
            ball_has_been_hit: Some(true),
            kickoff_countdown_time: None,
            team_zero_score: Some(0),
            team_one_score: Some(0),
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: [1, 1],
        },
        BallFrameState::Present(BallSample {
            rigid_body: rigid_body(Vector3f {
                x: 0.0,
                y: 4000.0,
                z: 92.75,
            }),
        }),
        PlayerFrameState {
            players: vec![
                player_sample(
                    0,
                    true,
                    Vector3f {
                        x: -1000.0,
                        y: -4000.0 + progress,
                        z: 17.0,
                    },
                ),
                player_sample(
                    1,
                    false,
                    Vector3f {
                        x: 1000.0,
                        y: 4000.0 - progress,
                        z: 17.0,
                    },
                ),
            ],
        },
        frame_events_state,
    )
}

fn evaluate_frame(graph: &mut AnalysisGraph, frame_number: usize, time: f32) {
    graph
        .evaluate_with_state(&frame_input(frame_number, time))
        .expect("graph should evaluate a synthetic frame");
}

/// Evaluates the scripted match, projecting the graph's events at most once
/// per `projection_interval` of game time (the driver-owned cadence;
/// `Some(0.0)` projects on every frame, `None` never projects).
fn evaluate_scripted_match(graph: &mut AnalysisGraph, projection_interval: Option<f32>) {
    graph
        .on_replay_meta(&replay_meta())
        .expect("graph should accept replay meta");
    let mut last_projection_time: Option<f32> = None;
    for frame_number in 0..=LAST_FRAME {
        let time = frame_number as f32 / FPS;
        evaluate_frame(graph, frame_number, time);
        if let Some(interval) = projection_interval
            && last_projection_time.is_none_or(|last| time - last >= interval)
        {
            graph
                .project_events_now()
                .expect("interim projection should not violate lifecycle invariants");
            last_projection_time = Some(time);
        }
    }
}

/// The graph store's reduced current view (what a live consumer reconstructs
/// from the transaction log, and what batch consumers read after finish),
/// cloned for comparison.
fn store_events(graph: &AnalysisGraph) -> Vec<crate::Event> {
    graph
        .event_transaction_log()
        .current_events()
        .into_iter()
        .cloned()
        .collect()
}

#[test]
fn interim_projection_publishes_events_before_finish() {
    let mut graph = graph_with_all_analysis_nodes();
    evaluate_scripted_match(&mut graph, Some(1.0));

    // The pickup lands at ~0.5s and the last interim projection happens at
    // ~2.0s, so the store's view is populated without any finish call.
    let events = store_events(&graph);
    let streams: Vec<&str> = events
        .iter()
        .map(|event| event.meta.stream.as_str())
        .collect();
    assert!(
        streams.contains(&"boost_pickups"),
        "interim projection should surface the scripted pickup, got streams {streams:?}"
    );
}

#[test]
fn finish_only_graph_stays_empty_until_finish() {
    let mut graph = graph_with_all_analysis_nodes();
    evaluate_scripted_match(&mut graph, None);

    assert_eq!(
        graph.event_transaction_log().transaction_count(),
        0,
        "no projection should run during evaluation"
    );

    graph.finish().expect("graph should finish");
    assert!(
        !store_events(&graph).is_empty(),
        "finish's single projection should surface the scripted match's events"
    );
}

/// Ids and content are cadence-invariant: projecting every frame, once a
/// second, or only at finish yields bit-identical final event lists (ids and
/// lifecycles included — finish finalizes everything).
#[test]
fn interim_projection_finish_matches_finish_only_bit_for_bit() {
    let mut finish_only = graph_with_all_analysis_nodes();
    evaluate_scripted_match(&mut finish_only, None);
    finish_only.finish().expect("graph should finish");

    // Interval 0.0 projects on every evaluated frame.
    for interval in [1.0, 0.0] {
        let mut interim = graph_with_all_analysis_nodes();
        evaluate_scripted_match(&mut interim, Some(interval));
        interim.finish().expect("graph should finish");
        assert_eq!(
            store_events(&finish_only),
            store_events(&interim),
            "projection interval {interval} must not change the store's reduced view"
        );
    }
}

/// An id observed mid-match is the id the event still has at finish, with the
/// same stream and payload kind — nothing is re-identified by later evidence
/// or by the finish projection.
#[test]
fn mid_match_ids_survive_to_finish_unchanged() {
    let mut graph = graph_with_all_analysis_nodes();
    evaluate_scripted_match(&mut graph, Some(1.0));

    let mid_match: HashMap<String, String> = store_events(&graph)
        .iter()
        .map(|event| (event.meta.id.clone(), event.meta.stream.clone()))
        .collect();
    assert!(
        mid_match.values().any(|stream| stream == "boost_pickups"),
        "the scripted pickup should be observable mid-match"
    );

    graph.finish().expect("graph should finish");
    let final_events = store_events(&graph);
    let final_by_id: HashMap<&str, &str> = final_events
        .iter()
        .map(|event| (event.meta.id.as_str(), event.meta.stream.as_str()))
        .collect();
    for (id, stream) in &mid_match {
        assert_eq!(
            final_by_id.get(id.as_str()).copied(),
            Some(stream.as_str()),
            "mid-match id {id} must survive to finish on the same stream"
        );
    }
}

/// Interim lifecycles: a committed moment (the scripted pickup) is already
/// finalized mid-match, while the open positioning spans are only confirmed;
/// finish upgrades everything.
#[test]
fn interim_projection_marks_open_spans_confirmed_and_committed_moments_finalized() {
    let mut graph = graph_with_all_analysis_nodes();
    evaluate_scripted_match(&mut graph, Some(1.0));

    let events = store_events(&graph);
    let pickup = events
        .iter()
        .find(|event| event.meta.stream == "boost_pickups")
        .expect("the scripted pickup should be projected mid-match");
    assert_eq!(pickup.meta.lifecycle, EventLifecycle::Finalized);
    assert!(
        events
            .iter()
            .any(|event| event.meta.lifecycle == EventLifecycle::Confirmed),
        "open spans should be projected as Confirmed mid-match"
    );

    graph.finish().expect("graph should finish");
    assert!(
        store_events(&graph)
            .iter()
            .all(|event| event.meta.lifecycle == EventLifecycle::Finalized),
        "finish must finalize every event"
    );
}
