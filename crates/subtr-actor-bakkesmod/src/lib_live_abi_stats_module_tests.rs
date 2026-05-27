use super::*;

#[test]
fn live_abi_exposes_every_builtin_stats_module_by_name() {
    let engine = subtr_actor_bakkesmod_engine_create();
    let touches = [SaTouchEvent {
        timing: SaEventTiming::default(),
        player_index: 0,
        has_player: 1,
        is_team_0: 1,
        closest_approach_distance: 0.0,
        has_closest_approach_distance: 1,
    }];
    let players_by_frame = (1..=12)
        .map(|frame_number| {
            [player_at(SaVec3 {
                x: frame_number as f32 * 20.0,
                y: 0.0,
                z: 20.0,
            })]
        })
        .collect::<Vec<_>>();
    let mut frames = Vec::new();
    for (offset, players) in players_by_frame.iter().enumerate() {
        let frame_number = offset as u64 + 1;
        let mut frame = live_frame(
            frame_number,
            test_rigid_body(
                SaVec3 {
                    x: frame_number as f32 * 20.0,
                    y: 0.0,
                    z: 120.0,
                },
                SaVec3::default(),
            ),
            players,
        );
        frame.has_live_play = 1;
        if frame_number == 1 {
            frame.touches = touches.as_ptr();
            frame.touch_count = touches.len();
        }
        frames.push(frame);
    }

    for frame in &frames {
        assert_eq!(
            unsafe { subtr_actor_bakkesmod_process_frame(engine, frame) },
            0
        );
    }
    assert_eq!(unsafe { subtr_actor_bakkesmod_finish(engine) }, 0);

    let stats = live_stats_json_value(engine);
    let modules = stats["modules"]
        .as_object()
        .expect("stats json should expose a modules object");
    for module_name in builtin_stats_module_names() {
        assert_eq!(
                live_stats_module_json_value(engine, module_name),
                modules
                    .get(*module_name)
                    .cloned()
                    .unwrap_or_else(|| panic!("stats snapshot should include {module_name}")),
                "named BakkesMod stats module ABI should match full stats snapshot module {module_name}"
            );
    }

    let unknown = std::ffi::CString::new("not_a_module").unwrap();
    assert_eq!(
        unsafe { subtr_actor_bakkesmod_stats_module_json_len(engine, unknown.as_ptr()) },
        0
    );
    assert_eq!(
        unsafe {
            subtr_actor_bakkesmod_write_stats_module_json(
                engine,
                unknown.as_ptr(),
                ptr::null_mut(),
                10,
            )
        },
        0
    );
    assert_eq!(
        unsafe { subtr_actor_bakkesmod_stats_module_json_len(engine, ptr::null()) },
        0
    );
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}

#[test]
fn live_abi_exposes_every_builtin_stats_module_frame_and_config_by_name() {
    let engine = subtr_actor_bakkesmod_engine_create();
    let touches = [SaTouchEvent {
        timing: SaEventTiming::default(),
        player_index: 0,
        has_player: 1,
        is_team_0: 1,
        closest_approach_distance: 0.0,
        has_closest_approach_distance: 1,
    }];
    let players_by_frame = (1..=12)
        .map(|frame_number| {
            [player_at(SaVec3 {
                x: frame_number as f32 * 20.0,
                y: 0.0,
                z: 20.0,
            })]
        })
        .collect::<Vec<_>>();
    let mut frames = Vec::new();
    for (offset, players) in players_by_frame.iter().enumerate() {
        let frame_number = offset as u64 + 1;
        let mut frame = live_frame(
            frame_number,
            test_rigid_body(
                SaVec3 {
                    x: frame_number as f32 * 20.0,
                    y: 0.0,
                    z: 120.0,
                },
                SaVec3::default(),
            ),
            players,
        );
        frame.has_live_play = 1;
        if frame_number == 1 {
            frame.touches = touches.as_ptr();
            frame.touch_count = touches.len();
        }
        frames.push(frame);
    }

    for frame in &frames {
        assert_eq!(
            unsafe { subtr_actor_bakkesmod_process_frame(engine, frame) },
            0
        );
    }
    assert_eq!(unsafe { subtr_actor_bakkesmod_finish(engine) }, 0);

    let stats = live_stats_json_value(engine);
    let config = stats["config"]
        .as_object()
        .expect("stats json should expose a config object");
    let frame_modules = stats["frame"]["modules"]
        .as_object()
        .expect("stats json should expose frame modules");
    for module_name in builtin_stats_module_names() {
        assert_eq!(
                live_stats_module_frame_json_value(engine, module_name),
                frame_modules
                    .get(*module_name)
                    .cloned()
                    .unwrap_or(serde_json::Value::Null),
                "named BakkesMod stats module frame ABI should match full stats frame module {module_name}"
            );
        assert_eq!(
                live_stats_module_config_json_value(engine, module_name),
                config
                    .get(*module_name)
                    .cloned()
                    .unwrap_or(serde_json::Value::Null),
                "named BakkesMod stats module config ABI should match full stats config module {module_name}"
            );
    }

    let unknown = std::ffi::CString::new("not_a_module").unwrap();
    assert_eq!(
        unsafe { subtr_actor_bakkesmod_stats_module_frame_json_len(engine, unknown.as_ptr()) },
        0
    );
    assert_eq!(
        unsafe { subtr_actor_bakkesmod_stats_module_config_json_len(engine, unknown.as_ptr()) },
        0
    );
    assert_eq!(
        unsafe {
            subtr_actor_bakkesmod_write_stats_module_frame_json(
                engine,
                unknown.as_ptr(),
                ptr::null_mut(),
                10,
            )
        },
        0
    );
    assert_eq!(
        unsafe {
            subtr_actor_bakkesmod_write_stats_module_config_json(
                engine,
                unknown.as_ptr(),
                ptr::null_mut(),
                10,
            )
        },
        0
    );
    unsafe { subtr_actor_bakkesmod_engine_destroy(engine) };
}
