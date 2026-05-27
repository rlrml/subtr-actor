use super::*;
use std::collections::{BTreeMap, BTreeSet};

fn checked_in_header_text() -> String {
    let header_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("include")
        .join("subtr_actor_bakkesmod.h");
    std::fs::read_to_string(&header_path)
        .unwrap_or_else(|_| panic!("failed to read {}", header_path.display()))
}

fn header_enum_values(enum_name: &str) -> BTreeMap<String, i32> {
    let header = checked_in_header_text();
    let start = format!("typedef enum {enum_name} {{");
    let end = format!("}} {enum_name};");
    let mut in_enum = false;
    let mut values = BTreeMap::new();
    for line in header.lines() {
        let line = line.trim();
        if line == start {
            in_enum = true;
            continue;
        }
        if in_enum && line == end {
            return values;
        }
        if !in_enum || line.is_empty() {
            continue;
        }

        let line = line.trim_end_matches(',');
        let Some((name, value)) = line.split_once(" = ") else {
            continue;
        };
        values.insert(
            name.to_owned(),
            value
                .parse::<i32>()
                .unwrap_or_else(|_| panic!("invalid enum value in {enum_name}: {line}")),
        );
    }
    panic!("did not find enum {enum_name} in checked-in header");
}

fn header_struct_fields(struct_name: &str) -> Vec<String> {
    header_struct_field_declarations(struct_name)
        .into_iter()
        .map(|(_, field)| field)
        .collect()
}

fn header_struct_field_declarations(struct_name: &str) -> Vec<(String, String)> {
    let header = checked_in_header_text();
    let start = format!("typedef struct {struct_name} {{");
    let end = format!("}} {struct_name};");
    let mut in_struct = false;
    let mut fields = Vec::new();
    for line in header.lines() {
        let line = line.trim();
        if line == start {
            in_struct = true;
            continue;
        }
        if in_struct && line == end {
            return fields;
        }
        if !in_struct || line.is_empty() {
            continue;
        }

        let line = line.trim_end_matches(';');
        let Some((field_type, field)) = line.rsplit_once(' ') else {
            continue;
        };
        let pointer_prefix = field
            .chars()
            .take_while(|character| *character == '*')
            .collect::<String>();
        let field_type = if pointer_prefix.is_empty() {
            field_type.to_owned()
        } else {
            format!("{field_type} {pointer_prefix}")
        };
        fields.push((field_type, field.trim_start_matches('*').to_owned()));
    }
    panic!("did not find struct {struct_name} in checked-in header");
}

fn rust_struct_fields(struct_name: &str) -> Vec<String> {
    let start = format!("pub struct {struct_name} {{");
    let mut in_struct = false;
    let mut fields = Vec::new();
    for line in rust_source_files().into_iter().flat_map(str::lines) {
        let line = line.trim();
        if line == start {
            in_struct = true;
            continue;
        }
        if in_struct && line == "}" {
            return fields;
        }
        if !in_struct || line.is_empty() {
            continue;
        }

        let Some(field) = line.strip_prefix("pub ") else {
            continue;
        };
        let Some((name, _)) = field.split_once(':') else {
            continue;
        };
        fields.push(name.to_owned());
    }
    panic!("did not find struct {struct_name} in Rust source");
}

fn header_exported_function_names() -> BTreeSet<String> {
    checked_in_header_text()
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            let start = line.find("subtr_actor_bakkesmod_")?;
            let rest = &line[start..];
            let end = rest.find('(')?;
            Some(rest[..end].to_owned())
        })
        .collect()
}

fn rust_exported_function_names() -> BTreeSet<String> {
    rust_source_files()
        .into_iter()
        .flat_map(str::lines)
        .filter_map(|line| {
            let line = line.trim();
            if !line.starts_with("pub ") || !line.contains(" extern \"C\" fn ") {
                return None;
            }
            let (_, rest) = line.split_once("fn ")?;
            let end = rest.find('(')?;
            let name = &rest[..end];
            name.starts_with("subtr_actor_bakkesmod_")
                .then(|| name.to_owned())
        })
        .collect()
}

fn rust_source_files() -> [&'static str; 32] {
    [
        include_str!("abi.rs"),
        include_str!("abi_events.rs"),
        include_str!("abi_goal_context.rs"),
        include_str!("abi_live_frame.rs"),
        include_str!("abi_math.rs"),
        include_str!("abi_mechanics.rs"),
        include_str!("engine_constants.rs"),
        include_str!("lib.rs"),
        include_str!("ffi_drain.rs"),
        include_str!("ffi_graph_json.rs"),
        include_str!("ffi_graph_json_analysis_node.rs"),
        include_str!("ffi_graph_json_analysis_node_names.rs"),
        include_str!("ffi_graph_json_info.rs"),
        include_str!("ffi_graph_json_output.rs"),
        include_str!("ffi_lifecycle.rs"),
        include_str!("ffi_live_json.rs"),
        include_str!("ffi_live_json_events.rs"),
        include_str!("ffi_live_json_frame.rs"),
        include_str!("ffi_live_json_stats.rs"),
        include_str!("ffi_live_json_timeline.rs"),
        include_str!("ffi_pending.rs"),
        include_str!("ffi_process_frame.rs"),
        include_str!("ffi_stats_json.rs"),
        include_str!("ffi_stats_json_config.rs"),
        include_str!("ffi_stats_json_frame.rs"),
        include_str!("ffi_stats_json_module.rs"),
        include_str!("graph_output.rs"),
        include_str!("live.rs"),
        include_str!("replay_annotations.rs"),
        include_str!("replay_annotations_lifecycle.rs"),
        include_str!("replay_annotations_poll.rs"),
        include_str!("timeline.rs"),
    ]
}

#[test]
fn checked_in_header_matches_event_abi_enums() {
    assert_eq!(
        header_enum_values("SaBoostPadEventKind"),
        BTreeMap::from([
            (
                "SaBoostPadEventKindPickedUp".to_owned(),
                SaBoostPadEventKind::PickedUp as i32,
            ),
            (
                "SaBoostPadEventKindAvailable".to_owned(),
                SaBoostPadEventKind::Available as i32,
            ),
        ])
    );
    assert_eq!(
        header_enum_values("SaPlayerStatEventKind"),
        BTreeMap::from([
            (
                "SaPlayerStatEventKindShot".to_owned(),
                SaPlayerStatEventKind::Shot as i32,
            ),
            (
                "SaPlayerStatEventKindSave".to_owned(),
                SaPlayerStatEventKind::Save as i32,
            ),
            (
                "SaPlayerStatEventKindAssist".to_owned(),
                SaPlayerStatEventKind::Assist as i32,
            ),
        ])
    );
    assert_eq!(
        header_enum_values("SaMechanicKind"),
        BTreeMap::from([
            (
                "SaMechanicKindSpeedFlip".to_owned(),
                SaMechanicKind::SpeedFlip as i32,
            ),
            (
                "SaMechanicKindHalfFlip".to_owned(),
                SaMechanicKind::HalfFlip as i32,
            ),
            (
                "SaMechanicKindWavedash".to_owned(),
                SaMechanicKind::Wavedash as i32,
            ),
            (
                "SaMechanicKindBallCarry".to_owned(),
                SaMechanicKind::BallCarry as i32,
            ),
            (
                "SaMechanicKindAirDribble".to_owned(),
                SaMechanicKind::AirDribble as i32,
            ),
            (
                "SaMechanicKindCeilingShot".to_owned(),
                SaMechanicKind::CeilingShot as i32,
            ),
            (
                "SaMechanicKindWallAerial".to_owned(),
                SaMechanicKind::WallAerial as i32,
            ),
            (
                "SaMechanicKindWallAerialShot".to_owned(),
                SaMechanicKind::WallAerialShot as i32,
            ),
            (
                "SaMechanicKindCenter".to_owned(),
                SaMechanicKind::Center as i32,
            ),
            (
                "SaMechanicKindFlipReset".to_owned(),
                SaMechanicKind::FlipReset as i32,
            ),
            (
                "SaMechanicKindDoubleTap".to_owned(),
                SaMechanicKind::DoubleTap as i32,
            ),
            (
                "SaMechanicKindFlick".to_owned(),
                SaMechanicKind::Flick as i32,
            ),
            (
                "SaMechanicKindMustyFlick".to_owned(),
                SaMechanicKind::MustyFlick as i32,
            ),
            (
                "SaMechanicKindOneTimer".to_owned(),
                SaMechanicKind::OneTimer as i32,
            ),
            ("SaMechanicKindPass".to_owned(), SaMechanicKind::Pass as i32),
            (
                "SaMechanicKindHalfVolley".to_owned(),
                SaMechanicKind::HalfVolley as i32,
            ),
            (
                "SaMechanicKindWhiff".to_owned(),
                SaMechanicKind::Whiff as i32,
            ),
            ("SaMechanicKindBump".to_owned(), SaMechanicKind::Bump as i32),
            (
                "SaMechanicKindBackboard".to_owned(),
                SaMechanicKind::Backboard as i32,
            ),
            (
                "SaMechanicKindBoostPickup".to_owned(),
                SaMechanicKind::BoostPickup as i32,
            ),
            ("SaMechanicKindDemo".to_owned(), SaMechanicKind::Demo as i32),
            (
                "SaMechanicKindFiftyFifty".to_owned(),
                SaMechanicKind::FiftyFifty as i32,
            ),
            (
                "SaMechanicKindAerialGoal".to_owned(),
                SaMechanicKind::AerialGoal as i32,
            ),
            (
                "SaMechanicKindHighAerialGoal".to_owned(),
                SaMechanicKind::HighAerialGoal as i32,
            ),
            (
                "SaMechanicKindLongDistanceGoal".to_owned(),
                SaMechanicKind::LongDistanceGoal as i32,
            ),
            (
                "SaMechanicKindOwnHalfGoal".to_owned(),
                SaMechanicKind::OwnHalfGoal as i32,
            ),
            (
                "SaMechanicKindEmptyNetGoal".to_owned(),
                SaMechanicKind::EmptyNetGoal as i32,
            ),
            (
                "SaMechanicKindCounterAttackGoal".to_owned(),
                SaMechanicKind::CounterAttackGoal as i32,
            ),
            (
                "SaMechanicKindFlickGoal".to_owned(),
                SaMechanicKind::FlickGoal as i32,
            ),
            (
                "SaMechanicKindDoubleTapGoal".to_owned(),
                SaMechanicKind::DoubleTapGoal as i32,
            ),
            (
                "SaMechanicKindOneTimerGoal".to_owned(),
                SaMechanicKind::OneTimerGoal as i32,
            ),
            (
                "SaMechanicKindPassingGoal".to_owned(),
                SaMechanicKind::PassingGoal as i32,
            ),
            (
                "SaMechanicKindAirDribbleGoal".to_owned(),
                SaMechanicKind::AirDribbleGoal as i32,
            ),
            (
                "SaMechanicKindFlipResetGoal".to_owned(),
                SaMechanicKind::FlipResetGoal as i32,
            ),
            (
                "SaMechanicKindHalfVolleyGoal".to_owned(),
                SaMechanicKind::HalfVolleyGoal as i32,
            ),
            ("SaMechanicKindGoal".to_owned(), SaMechanicKind::Goal as i32),
            ("SaMechanicKindShot".to_owned(), SaMechanicKind::Shot as i32),
            ("SaMechanicKindSave".to_owned(), SaMechanicKind::Save as i32),
            (
                "SaMechanicKindAssist".to_owned(),
                SaMechanicKind::Assist as i32,
            ),
            (
                "SaMechanicKindDeath".to_owned(),
                SaMechanicKind::Death as i32,
            ),
        ])
    );
    assert_eq!(
        header_enum_values("SaTeamEventKind"),
        BTreeMap::from([(
            "SaTeamEventKindRush".to_owned(),
            SaTeamEventKind::Rush as i32,
        )])
    );
    assert_eq!(
        header_enum_values("SaGoalBuildupKind"),
        BTreeMap::from([
            (
                "SaGoalBuildupKindCounterAttack".to_owned(),
                SaGoalBuildupKind::CounterAttack as i32,
            ),
            (
                "SaGoalBuildupKindSustainedPressure".to_owned(),
                SaGoalBuildupKind::SustainedPressure as i32,
            ),
            (
                "SaGoalBuildupKindOther".to_owned(),
                SaGoalBuildupKind::Other as i32,
            ),
        ])
    );
}

#[test]
fn checked_in_header_declares_every_exported_function() {
    assert_eq!(
        header_exported_function_names(),
        rust_exported_function_names()
    );
}

#[test]
fn checked_in_header_matches_event_abi_struct_fields() {
    for struct_name in [
        "SaVec3",
        "SaQuat",
        "SaRigidBody",
        "SaPlayerFrame",
        "SaEventTiming",
        "SaTouchEvent",
        "SaDodgeRefreshedEvent",
        "SaBoostPadEvent",
        "SaGoalEvent",
        "SaPlayerStatEvent",
        "SaDemolishEvent",
        "SaLiveFrame",
        "SaMechanicEvent",
        "SaTeamEvent",
        "SaGoalContextEvent",
    ] {
        assert_eq!(
            header_struct_fields(struct_name),
            rust_struct_fields(struct_name),
            "checked-in header field order should match Rust repr(C) struct {struct_name}"
        );
    }
}

#[test]
fn checked_in_header_matches_event_abi_struct_field_types() {
    let expected = BTreeMap::from([
        (
            "SaVec3",
            vec![("float", "x"), ("float", "y"), ("float", "z")],
        ),
        (
            "SaQuat",
            vec![
                ("float", "x"),
                ("float", "y"),
                ("float", "z"),
                ("float", "w"),
            ],
        ),
        (
            "SaRigidBody",
            vec![
                ("SaVec3", "location"),
                ("SaQuat", "rotation"),
                ("SaVec3", "linear_velocity"),
                ("SaVec3", "angular_velocity"),
                ("uint8_t", "has_linear_velocity"),
                ("uint8_t", "has_angular_velocity"),
                ("uint8_t", "sleeping"),
            ],
        ),
        (
            "SaPlayerFrame",
            vec![
                ("uint32_t", "player_index"),
                ("const char *", "player_name"),
                ("uint8_t", "is_team_0"),
                ("uint8_t", "has_rigid_body"),
                ("SaRigidBody", "rigid_body"),
                ("float", "boost_amount"),
                ("float", "last_boost_amount"),
                ("uint8_t", "boost_active"),
                ("uint8_t", "jump_active"),
                ("uint8_t", "double_jump_active"),
                ("uint8_t", "dodge_active"),
                ("uint8_t", "powerslide_active"),
                ("uint8_t", "has_match_stats"),
                ("int32_t", "match_goals"),
                ("int32_t", "match_assists"),
                ("int32_t", "match_saves"),
                ("int32_t", "match_shots"),
                ("int32_t", "match_score"),
            ],
        ),
        (
            "SaEventTiming",
            vec![
                ("uint64_t", "frame_number"),
                ("float", "time"),
                ("int32_t", "seconds_remaining"),
                ("uint8_t", "has_timing"),
                ("uint8_t", "has_seconds_remaining"),
            ],
        ),
        (
            "SaTouchEvent",
            vec![
                ("SaEventTiming", "timing"),
                ("uint32_t", "player_index"),
                ("uint8_t", "has_player"),
                ("uint8_t", "is_team_0"),
                ("float", "closest_approach_distance"),
                ("uint8_t", "has_closest_approach_distance"),
            ],
        ),
        (
            "SaDodgeRefreshedEvent",
            vec![
                ("SaEventTiming", "timing"),
                ("uint32_t", "player_index"),
                ("uint8_t", "is_team_0"),
                ("int32_t", "counter_value"),
            ],
        ),
        (
            "SaBoostPadEvent",
            vec![
                ("SaEventTiming", "timing"),
                ("uint32_t", "pad_id"),
                ("SaBoostPadEventKind", "kind"),
                ("uint8_t", "sequence"),
                ("uint32_t", "player_index"),
                ("uint8_t", "has_player"),
            ],
        ),
        (
            "SaGoalEvent",
            vec![
                ("SaEventTiming", "timing"),
                ("uint8_t", "scoring_team_is_team_0"),
                ("uint32_t", "player_index"),
                ("uint8_t", "has_player"),
                ("int32_t", "team_zero_score"),
                ("uint8_t", "has_team_zero_score"),
                ("int32_t", "team_one_score"),
                ("uint8_t", "has_team_one_score"),
            ],
        ),
        (
            "SaPlayerStatEvent",
            vec![
                ("SaEventTiming", "timing"),
                ("uint32_t", "player_index"),
                ("uint8_t", "is_team_0"),
                ("SaPlayerStatEventKind", "kind"),
                ("uint8_t", "has_shot_ball"),
                ("SaRigidBody", "shot_ball"),
                ("uint8_t", "has_shot_player"),
                ("SaRigidBody", "shot_player"),
            ],
        ),
        (
            "SaDemolishEvent",
            vec![
                ("SaEventTiming", "timing"),
                ("uint32_t", "attacker_index"),
                ("uint32_t", "victim_index"),
                ("SaVec3", "attacker_velocity"),
                ("SaVec3", "victim_velocity"),
                ("SaVec3", "victim_location"),
                ("float", "active_duration_seconds"),
            ],
        ),
        (
            "SaLiveFrame",
            vec![
                ("uint64_t", "frame_number"),
                ("float", "time"),
                ("float", "dt"),
                ("int32_t", "seconds_remaining"),
                ("uint8_t", "has_seconds_remaining"),
                ("int32_t", "game_state"),
                ("uint8_t", "has_game_state"),
                ("int32_t", "kickoff_countdown_time"),
                ("uint8_t", "has_kickoff_countdown_time"),
                ("uint8_t", "ball_has_been_hit"),
                ("uint8_t", "has_ball_has_been_hit"),
                ("int32_t", "team_zero_score"),
                ("uint8_t", "has_team_zero_score"),
                ("int32_t", "team_one_score"),
                ("uint8_t", "has_team_one_score"),
                ("uint8_t", "possession_team_is_team_0"),
                ("uint8_t", "has_possession_team"),
                ("uint8_t", "scored_on_team_is_team_0"),
                ("uint8_t", "has_scored_on_team"),
                ("uint8_t", "live_play"),
                ("uint8_t", "has_live_play"),
                ("uint8_t", "has_ball"),
                ("SaRigidBody", "ball"),
                ("const SaPlayerFrame *", "players"),
                ("size_t", "player_count"),
                ("const SaTouchEvent *", "touches"),
                ("size_t", "touch_count"),
                ("const SaDodgeRefreshedEvent *", "dodge_refreshes"),
                ("size_t", "dodge_refresh_count"),
                ("const SaBoostPadEvent *", "boost_pad_events"),
                ("size_t", "boost_pad_event_count"),
                ("const SaGoalEvent *", "goals"),
                ("size_t", "goal_count"),
                ("const SaPlayerStatEvent *", "player_stat_events"),
                ("size_t", "player_stat_event_count"),
                ("const SaDemolishEvent *", "demolishes"),
                ("size_t", "demolish_count"),
            ],
        ),
        (
            "SaMechanicEvent",
            vec![
                ("SaMechanicKind", "kind"),
                ("uint32_t", "player_index"),
                ("uint8_t", "is_team_0"),
                ("uint64_t", "frame_number"),
                ("float", "time"),
                ("float", "confidence"),
            ],
        ),
        (
            "SaTeamEvent",
            vec![
                ("SaTeamEventKind", "kind"),
                ("uint8_t", "is_team_0"),
                ("uint64_t", "start_frame"),
                ("uint64_t", "end_frame"),
                ("float", "start_time"),
                ("float", "end_time"),
                ("uint32_t", "attackers"),
                ("uint32_t", "defenders"),
                ("float", "confidence"),
            ],
        ),
        (
            "SaGoalContextEvent",
            vec![
                ("uint64_t", "frame_number"),
                ("float", "time"),
                ("uint8_t", "scoring_team_is_team_0"),
                ("uint8_t", "has_scorer"),
                ("uint32_t", "scorer_index"),
                ("uint8_t", "has_scoring_team_most_back_player"),
                ("uint32_t", "scoring_team_most_back_player_index"),
                ("uint8_t", "has_defending_team_most_back_player"),
                ("uint32_t", "defending_team_most_back_player_index"),
                ("uint8_t", "has_ball_position"),
                ("SaVec3", "ball_position"),
                ("uint8_t", "has_ball_air_time_before_goal"),
                ("float", "ball_air_time_before_goal"),
                ("SaGoalBuildupKind", "goal_buildup"),
            ],
        ),
    ]);

    for (struct_name, expected_fields) in expected {
        let expected_fields = expected_fields
            .into_iter()
            .map(|(field_type, field)| (field_type.to_owned(), field.to_owned()))
            .collect::<Vec<_>>();
        assert_eq!(
            header_struct_field_declarations(struct_name),
            expected_fields,
            "checked-in header field types should match the intended C ABI for {struct_name}"
        );
    }
}
