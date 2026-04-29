mod common;

use serde_json::Value;
use subtr_actor::{
    builtin_stats_module_names, evaluate_replay_plausibility, Collector, FrameRateDecorator,
    NDArrayCollector, PlayerFrame, ReplayDataCollector, StatsCollector, StatsFrameResolution,
    StatsTimelineCollector,
};

struct PostEacFixture {
    path: &'static str,
    player_count: usize,
    match_type: &'static str,
}

const POST_EAC_FIXTURES: &[PostEacFixture] = &[
    PostEacFixture {
        path: "assets/post-eac-ranked-duel-2026-04-28-a.replay",
        player_count: 2,
        match_type: "Online",
    },
    PostEacFixture {
        path: "assets/post-eac-ranked-duel-2026-04-28-b.replay",
        player_count: 2,
        match_type: "Online",
    },
    PostEacFixture {
        path: "assets/post-eac-ranked-doubles-2026-04-28.replay",
        player_count: 4,
        match_type: "Online",
    },
    PostEacFixture {
        path: "assets/post-eac-ranked-standard-2026-04-28.replay",
        player_count: 6,
        match_type: "Online",
    },
    PostEacFixture {
        path: "assets/post-eac-private-2026-04-28.replay",
        player_count: 2,
        match_type: "Private",
    },
];

const POST_EAC_BUILD_VERSION: &str = "260316.80791.512269";

fn header_string<'a>(replay: &'a boxcars::Replay, key: &str) -> Option<&'a str> {
    replay
        .properties
        .iter()
        .find(|(property_key, _)| property_key == key)
        .and_then(|(_, value)| value.as_string())
}

fn assert_finite_json(value: &Value, path: &str) {
    match value {
        Value::Null | Value::Bool(_) | Value::String(_) => {}
        Value::Number(number) => {
            assert!(
                number.as_f64().is_some_and(f64::is_finite),
                "expected finite number at {path}, got {number}"
            );
        }
        Value::Array(values) => {
            for (index, value) in values.iter().enumerate() {
                assert_finite_json(value, &format!("{path}[{index}]"));
            }
        }
        Value::Object(entries) => {
            for (key, value) in entries {
                assert_finite_json(value, &format!("{path}.{key}"));
            }
        }
    }
}

fn assert_collected_stats_modules(value: &Value, path: &str) {
    let modules = value
        .get("modules")
        .and_then(Value::as_object)
        .unwrap_or_else(|| panic!("expected {path} collected stats to serialize module map"));

    assert_eq!(
        modules.len(),
        builtin_stats_module_names().len(),
        "expected {path} to serialize every builtin stats module"
    );
    for module_name in builtin_stats_module_names() {
        assert!(
            modules.contains_key(*module_name),
            "expected {path} collected stats to contain module {module_name}"
        );
    }
}

#[test]
fn post_eac_replays_parse_and_match_expected_headers() {
    for fixture in POST_EAC_FIXTURES {
        let replay = common::parse_replay(fixture.path);

        assert_eq!(replay.major_version, 868, "{} major_version", fixture.path);
        assert_eq!(replay.minor_version, 32, "{} minor_version", fixture.path);
        assert_eq!(replay.net_version, Some(11), "{} net_version", fixture.path);
        assert_eq!(
            header_string(&replay, "BuildVersion"),
            Some(POST_EAC_BUILD_VERSION),
            "{} BuildVersion",
            fixture.path
        );
        assert_eq!(
            header_string(&replay, "MatchType"),
            Some(fixture.match_type),
            "{} MatchType",
            fixture.path
        );
        assert!(
            replay
                .network_frames
                .as_ref()
                .is_some_and(|frames| !frames.frames.is_empty()),
            "{} should expose parsed network frames",
            fixture.path
        );
    }
}

#[test]
fn post_eac_replays_emit_structured_replay_data() {
    for fixture in POST_EAC_FIXTURES {
        let replay = common::parse_replay(fixture.path);
        let replay_data = ReplayDataCollector::new()
            .get_replay_data(&replay)
            .unwrap_or_else(|error| {
                panic!(
                    "failed to collect structured replay data for {}: {error:?}",
                    fixture.path
                )
            });

        assert_eq!(
            replay_data.meta.player_count(),
            fixture.player_count,
            "{} player count",
            fixture.path
        );
        assert!(
            replay_data.frame_data.frame_count() > 0,
            "{} should emit frame data",
            fixture.path
        );
        assert!(
            replay_data.frame_data.duration() > 0.0,
            "{} should have positive duration",
            fixture.path
        );
        assert!(
            replay_data
                .frame_data
                .players
                .iter()
                .all(|(_, player_data)| player_data.frames().iter().any(|frame| {
                    matches!(frame, PlayerFrame::Data { rigid_body, .. } if rigid_body.location.x.is_finite())
                })),
            "{} should emit finite player rigid-body samples for every player",
            fixture.path
        );
        assert!(
            !replay_data.touch_events.is_empty(),
            "{} should expose exact touch events",
            fixture.path
        );
        assert!(
            !replay_data.player_stat_events.is_empty(),
            "{} should expose exact player-stat events",
            fixture.path
        );

        let replay_json =
            serde_json::to_value(&replay_data).expect("ReplayData should serialize to JSON");
        assert_finite_json(&replay_json, fixture.path);
    }
}

#[test]
fn post_eac_replays_emit_ndarray_features() {
    for fixture in POST_EAC_FIXTURES {
        let replay = common::parse_replay(fixture.path);
        let mut collector = NDArrayCollector::<f32>::from_strings(
            &[
                "BallRigidBody",
                "BallRigidBodyQuaternions",
                "BallRigidBodyBasis",
                "VelocityAddedBallRigidBodyNoVelocities",
                "InterpolatedBallRigidBodyNoVelocities",
                "SecondsRemaining",
                "CurrentTime",
                "FrameTime",
                "ReplicatedStateName",
                "ReplicatedGameStateTimeRemaining",
                "BallHasBeenHit",
            ],
            &[
                "PlayerRigidBody",
                "PlayerRigidBodyQuaternions",
                "PlayerRigidBodyBasis",
                "PlayerRelativeBallPosition",
                "PlayerRelativeBallVelocity",
                "PlayerLocalRelativeBallPosition",
                "PlayerLocalRelativeBallVelocity",
                "VelocityAddedPlayerRigidBodyNoVelocities",
                "InterpolatedPlayerRigidBodyNoVelocities",
                "PlayerBallDistance",
                "PlayerBoost",
                "PlayerJump",
                "PlayerAnyJump",
                "PlayerDodgeRefreshed",
                "PlayerDemolishedBy",
            ],
        )
        .expect("post-EAC ndarray feature selection should be valid");

        FrameRateDecorator::new_from_fps(30.0, &mut collector)
            .process_replay(&replay)
            .unwrap_or_else(|error| {
                panic!(
                    "failed to collect ndarray features for {}: {error:?}",
                    fixture.path
                )
            });
        let (meta, array) = collector.get_meta_and_ndarray().unwrap_or_else(|error| {
            panic!(
                "failed to build ndarray output for {}: {error:?}",
                fixture.path
            )
        });

        assert_eq!(
            meta.replay_meta.player_count(),
            fixture.player_count,
            "{} ndarray player count",
            fixture.path
        );
        assert!(
            array.nrows() > 0,
            "{} should emit ndarray rows",
            fixture.path
        );
        assert!(
            array.ncols() > 0,
            "{} should emit ndarray columns",
            fixture.path
        );
        assert!(
            array.iter().all(|value| value.is_finite()),
            "{} ndarray should contain only finite values",
            fixture.path
        );
    }
}

#[test]
fn post_eac_replays_emit_stats_outputs() {
    for fixture in POST_EAC_FIXTURES {
        let replay = common::parse_replay(fixture.path);
        let collected = StatsCollector::new()
            .get_stats(&replay)
            .unwrap_or_else(|error| {
                panic!(
                    "failed to collect aggregate stats for {}: {error:?}",
                    fixture.path
                )
            });
        assert_eq!(
            collected.replay_meta.player_count(),
            fixture.player_count,
            "{} aggregate stats player count",
            fixture.path
        );
        assert_eq!(
            collected.module_names().count(),
            builtin_stats_module_names().len(),
            "{} should collect every builtin stats module",
            fixture.path
        );

        let collected_json =
            serde_json::to_value(&collected).expect("CollectedStats should serialize to JSON");
        assert_collected_stats_modules(&collected_json, fixture.path);
        assert_finite_json(&collected_json, fixture.path);

        let timeline = StatsTimelineCollector::new()
            .with_frame_resolution(StatsFrameResolution::TimeStep { seconds: 1.0 })
            .get_replay_data(&replay)
            .unwrap_or_else(|error| {
                panic!(
                    "failed to collect typed stats timeline for {}: {error:?}",
                    fixture.path
                )
            });
        assert_eq!(
            timeline.replay_meta.player_count(),
            fixture.player_count,
            "{} typed timeline player count",
            fixture.path
        );
        assert!(
            timeline.frames.len() > 1,
            "{} typed timeline should include sampled frames",
            fixture.path
        );
        assert!(
            timeline
                .frames
                .windows(2)
                .all(|frames| frames[1].time >= frames[0].time),
            "{} typed timeline frame times should be sorted",
            fixture.path
        );
        assert!(
            timeline.frames.iter().all(|frame| {
                frame.players.len() == fixture.player_count
                    && frame.time.is_finite()
                    && frame.dt.is_finite()
            }),
            "{} typed timeline frames should carry finite player snapshots",
            fixture.path
        );

        let timeline_json =
            serde_json::to_value(&timeline).expect("ReplayStatsTimeline should serialize to JSON");
        assert_finite_json(&timeline_json, fixture.path);

        let playback_value = StatsCollector::new()
            .with_frame_resolution(StatsFrameResolution::TimeStep { seconds: 1.0 })
            .capture_frames()
            .get_captured_data(&replay)
            .unwrap_or_else(|error| {
                panic!(
                    "failed to capture dynamic stats frames for {}: {error:?}",
                    fixture.path
                )
            })
            .into_stats_timeline_value()
            .unwrap_or_else(|error| {
                panic!(
                    "failed to convert dynamic stats frames for {}: {error:?}",
                    fixture.path
                )
            });
        assert_finite_json(&playback_value, fixture.path);
    }
}

#[test]
fn post_eac_replay_motion_plausibility_passes() {
    for fixture in POST_EAC_FIXTURES {
        let replay = common::parse_replay(fixture.path);
        let replay_data = ReplayDataCollector::new()
            .get_replay_data(&replay)
            .unwrap_or_else(|error| {
                panic!(
                    "failed to collect replay data for {}: {error:?}",
                    fixture.path
                )
            });
        let report = evaluate_replay_plausibility(&replay_data);

        assert!(
            report.all_motion_consistent(),
            "{} should have plausible velocity/displacement consistency",
            fixture.path
        );
        assert!(
            report.all_field_bounds_plausible(),
            "{} should stay within plausible field and speed bounds",
            fixture.path
        );
        assert!(
            report.all_quaternion_norms_plausible(),
            "{} should expose unit-length rotations",
            fixture.path
        );
    }
}
