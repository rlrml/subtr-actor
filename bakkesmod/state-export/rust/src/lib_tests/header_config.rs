use std::collections::{BTreeMap, BTreeSet};

use super::*;

fn checked_in_header_text() -> String {
    let header_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("include")
        .join("state_export.h");
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

fn header_define_values(prefix: &str) -> BTreeMap<String, i64> {
    checked_in_header_text()
        .lines()
        .filter_map(|line| {
            let rest = line.trim().strip_prefix("#define ")?;
            let (name, value) = rest.split_once(' ')?;
            if !name.starts_with(prefix) {
                return None;
            }
            Some((name.to_owned(), value.trim().parse::<i64>().ok()?))
        })
        .collect()
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
    let source = include_str!("../abi.rs");
    let start = format!("pub struct {struct_name} {{");
    let mut in_struct = false;
    let mut fields = Vec::new();
    for line in source.lines() {
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

fn exported_symbol_at(line: &str) -> Option<String> {
    let start = line.find("state_export_")?;
    let rest = &line[start..];
    let end = rest
        .find(|character: char| {
            !(character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_')
        })
        .unwrap_or(rest.len());
    rest[end..]
        .starts_with('(')
        .then(|| rest[..end].to_owned())
}

fn header_exported_function_names() -> BTreeSet<String> {
    checked_in_header_text()
        .lines()
        .filter_map(exported_symbol_at)
        .collect()
}

fn rust_exported_function_names() -> BTreeSet<String> {
    include_str!("../ffi.rs")
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if !line.starts_with("pub ") || !line.contains(" extern \"C\" fn ") {
                return None;
            }
            let (_, rest) = line.split_once("fn ")?;
            let end = rest.find('(')?;
            let name = &rest[..end];
            name.starts_with("state_export_").then(|| name.to_owned())
        })
        .collect()
}

#[test]
fn checked_in_header_declares_every_exported_function() {
    assert_eq!(
        header_exported_function_names(),
        rust_exported_function_names()
    );
}

#[test]
fn checked_in_header_matches_abi_enums_and_constants() {
    assert_eq!(
        header_enum_values("SeBoostPadEventKind"),
        BTreeMap::from([
            (
                "SeBoostPadEventKindPickedUp".to_owned(),
                SeBoostPadEventKind::PickedUp as i32,
            ),
            (
                "SeBoostPadEventKindAvailable".to_owned(),
                SeBoostPadEventKind::Available as i32,
            ),
        ])
    );
    assert_eq!(
        header_enum_values("SePlayerStatEventKind"),
        BTreeMap::from([
            (
                "SePlayerStatEventKindShot".to_owned(),
                SePlayerStatEventKind::Shot as i32,
            ),
            (
                "SePlayerStatEventKindSave".to_owned(),
                SePlayerStatEventKind::Save as i32,
            ),
            (
                "SePlayerStatEventKindAssist".to_owned(),
                SePlayerStatEventKind::Assist as i32,
            ),
        ])
    );
    assert_eq!(
        header_define_values("SE_REMOTE_ID_PLATFORM_"),
        BTreeMap::from([
            (
                "SE_REMOTE_ID_PLATFORM_NONE".to_owned(),
                SE_REMOTE_ID_PLATFORM_NONE as i64,
            ),
            (
                "SE_REMOTE_ID_PLATFORM_STEAM".to_owned(),
                SE_REMOTE_ID_PLATFORM_STEAM as i64,
            ),
            (
                "SE_REMOTE_ID_PLATFORM_EPIC".to_owned(),
                SE_REMOTE_ID_PLATFORM_EPIC as i64,
            ),
            (
                "SE_REMOTE_ID_PLATFORM_XBOX".to_owned(),
                SE_REMOTE_ID_PLATFORM_XBOX as i64,
            ),
            (
                "SE_REMOTE_ID_PLATFORM_PSYNET".to_owned(),
                SE_REMOTE_ID_PLATFORM_PSYNET as i64,
            ),
            (
                "SE_REMOTE_ID_PLATFORM_SWITCH".to_owned(),
                SE_REMOTE_ID_PLATFORM_SWITCH as i64,
            ),
            (
                "SE_REMOTE_ID_PLATFORM_SPLITSCREEN".to_owned(),
                SE_REMOTE_ID_PLATFORM_SPLITSCREEN as i64,
            ),
            (
                "SE_REMOTE_ID_PLATFORM_PLAYSTATION".to_owned(),
                SE_REMOTE_ID_PLATFORM_PLAYSTATION as i64,
            ),
            (
                "SE_REMOTE_ID_PLATFORM_QQ".to_owned(),
                SE_REMOTE_ID_PLATFORM_QQ as i64,
            ),
        ])
    );
    assert_eq!(
        header_define_values("SE_STATE_"),
        BTreeMap::from([
            ("SE_STATE_STOPPED".to_owned(), SE_STATE_STOPPED as i64),
            ("SE_STATE_LISTENING".to_owned(), SE_STATE_LISTENING as i64),
            ("SE_STATE_ERROR".to_owned(), SE_STATE_ERROR as i64),
        ])
    );
    assert_eq!(
        header_define_values("SE_DEFAULT_STATE_EXPORT_PORT"),
        BTreeMap::from([(
            "SE_DEFAULT_STATE_EXPORT_PORT".to_owned(),
            DEFAULT_STATE_EXPORT_PORT as i64,
        )])
    );
}

#[test]
fn checked_in_header_matches_abi_struct_fields() {
    for struct_name in [
        "SeVec3",
        "SeQuat",
        "SeRigidBody",
        "SeControllerInput",
        "SeCameraState",
        "SeRemoteId",
        "SePlayerFrame",
        "SeEventTiming",
        "SeTouchEvent",
        "SeDodgeRefreshedEvent",
        "SeBoostPadEvent",
        "SeGoalEvent",
        "SePlayerStatEvent",
        "SeDemolishEvent",
        "SeFrame",
        "SeConfig",
        "SeStatus",
        "SeMatchContext",
    ] {
        assert_eq!(
            header_struct_fields(struct_name),
            rust_struct_fields(struct_name),
            "checked-in header field order should match Rust repr(C) struct {struct_name}"
        );
    }
}

#[test]
fn checked_in_header_matches_abi_struct_field_types() {
    let expected = BTreeMap::from([
        (
            "SeVec3",
            vec![("float", "x"), ("float", "y"), ("float", "z")],
        ),
        (
            "SeQuat",
            vec![
                ("float", "x"),
                ("float", "y"),
                ("float", "z"),
                ("float", "w"),
            ],
        ),
        (
            "SeRigidBody",
            vec![
                ("SeVec3", "location"),
                ("SeQuat", "rotation"),
                ("SeVec3", "linear_velocity"),
                ("SeVec3", "angular_velocity"),
                ("uint8_t", "has_linear_velocity"),
                ("uint8_t", "has_angular_velocity"),
                ("uint8_t", "sleeping"),
            ],
        ),
        (
            "SeControllerInput",
            vec![
                ("float", "throttle"),
                ("float", "steer"),
                ("float", "pitch"),
                ("float", "yaw"),
                ("float", "roll"),
                ("float", "dodge_forward"),
                ("float", "dodge_strafe"),
                ("uint8_t", "handbrake"),
                ("uint8_t", "jump"),
                ("uint8_t", "activate_boost"),
                ("uint8_t", "holding_boost"),
            ],
        ),
        (
            "SeCameraState",
            vec![
                ("uint8_t", "pitch"),
                ("uint8_t", "yaw"),
                ("uint8_t", "has_pitch"),
                ("uint8_t", "has_yaw"),
                ("uint8_t", "ball_cam_active"),
                ("uint8_t", "has_ball_cam"),
            ],
        ),
        (
            "SeRemoteId",
            vec![
                ("uint64_t", "online_id"),
                ("const char *", "epic_id"),
                ("uint32_t", "splitscreen_index"),
                ("uint8_t", "platform"),
            ],
        ),
        (
            "SePlayerFrame",
            vec![
                ("uint32_t", "player_index"),
                ("const char *", "player_name"),
                ("uint8_t", "is_team_0"),
                ("uint8_t", "has_rigid_body"),
                ("SeRigidBody", "rigid_body"),
                ("float", "boost_amount"),
                ("float", "last_boost_amount"),
                ("uint8_t", "boost_active"),
                ("uint8_t", "jump_active"),
                ("uint8_t", "double_jump_active"),
                ("uint8_t", "dodge_active"),
                ("uint8_t", "powerslide_active"),
                ("int32_t", "car_body_id"),
                ("uint8_t", "has_car_body_id"),
                ("uint8_t", "has_match_stats"),
                ("int32_t", "match_goals"),
                ("int32_t", "match_assists"),
                ("int32_t", "match_saves"),
                ("int32_t", "match_shots"),
                ("int32_t", "match_score"),
                ("uint8_t", "has_input"),
                ("SeControllerInput", "input"),
                ("SeCameraState", "camera"),
                ("uint8_t", "has_dodge_impulse"),
                ("SeVec3", "dodge_impulse"),
                ("uint8_t", "has_dodge_torque"),
                ("SeVec3", "dodge_torque"),
                ("SeRemoteId", "remote_id"),
            ],
        ),
        (
            "SeEventTiming",
            vec![
                ("uint64_t", "frame_number"),
                ("float", "time"),
                ("int32_t", "seconds_remaining"),
                ("uint8_t", "has_timing"),
                ("uint8_t", "has_seconds_remaining"),
            ],
        ),
        (
            "SeTouchEvent",
            vec![
                ("SeEventTiming", "timing"),
                ("uint32_t", "player_index"),
                ("uint8_t", "has_player"),
                ("uint8_t", "is_team_0"),
                ("float", "closest_approach_distance"),
                ("uint8_t", "has_closest_approach_distance"),
            ],
        ),
        (
            "SeDodgeRefreshedEvent",
            vec![
                ("SeEventTiming", "timing"),
                ("uint32_t", "player_index"),
                ("uint8_t", "is_team_0"),
                ("int32_t", "counter_value"),
            ],
        ),
        (
            "SeBoostPadEvent",
            vec![
                ("SeEventTiming", "timing"),
                ("uint32_t", "pad_id"),
                ("SeBoostPadEventKind", "kind"),
                ("uint8_t", "sequence"),
                ("uint32_t", "player_index"),
                ("uint8_t", "has_player"),
            ],
        ),
        (
            "SeGoalEvent",
            vec![
                ("SeEventTiming", "timing"),
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
            "SePlayerStatEvent",
            vec![
                ("SeEventTiming", "timing"),
                ("uint32_t", "player_index"),
                ("uint8_t", "is_team_0"),
                ("SePlayerStatEventKind", "kind"),
                ("uint8_t", "has_shot_ball"),
                ("SeRigidBody", "shot_ball"),
                ("uint8_t", "has_shot_player"),
                ("SeRigidBody", "shot_player"),
            ],
        ),
        (
            "SeDemolishEvent",
            vec![
                ("SeEventTiming", "timing"),
                ("uint32_t", "attacker_index"),
                ("uint32_t", "victim_index"),
                ("SeVec3", "attacker_velocity"),
                ("SeVec3", "victim_velocity"),
                ("SeVec3", "victim_location"),
                ("float", "active_duration_seconds"),
            ],
        ),
        (
            "SeFrame",
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
                ("SeRigidBody", "ball"),
                ("const SePlayerFrame *", "players"),
                ("size_t", "player_count"),
                ("const SeTouchEvent *", "touches"),
                ("size_t", "touch_count"),
                ("const SeDodgeRefreshedEvent *", "dodge_refreshes"),
                ("size_t", "dodge_refresh_count"),
                ("const SeBoostPadEvent *", "boost_pad_events"),
                ("size_t", "boost_pad_event_count"),
                ("const SeGoalEvent *", "goals"),
                ("size_t", "goal_count"),
                ("const SePlayerStatEvent *", "player_stat_events"),
                ("size_t", "player_stat_event_count"),
                ("const SeDemolishEvent *", "demolishes"),
                ("size_t", "demolish_count"),
            ],
        ),
        (
            "SeConfig",
            vec![
                ("const char *", "server_name"),
                ("uint32_t", "max_queued_frames"),
                ("uint32_t", "max_client_queue"),
                ("uint16_t", "port"),
                ("uint8_t", "bind_any_interface"),
            ],
        ),
        (
            "SeStatus",
            vec![
                ("int32_t", "state"),
                ("uint32_t", "client_count"),
                ("uint16_t", "port"),
                ("uint64_t", "frames_sent"),
                ("uint64_t", "frames_dropped"),
            ],
        ),
        (
            "SeMatchContext",
            vec![
                ("const char *", "match_guid"),
                ("const char *", "map_name"),
                ("int32_t", "playlist_id"),
                ("uint8_t", "has_playlist_id"),
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
