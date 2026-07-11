macro_rules! assert_layout {
    ($ty:ty, size = $size:expr_2021, align = $align:expr_2021) => {
        assert_eq!(
            std::mem::size_of::<$ty>(),
            $size,
            "size of {}",
            stringify!($ty)
        );
        assert_eq!(
            std::mem::align_of::<$ty>(),
            $align,
            "alignment of {}",
            stringify!($ty)
        );
    };
}

macro_rules! assert_offset {
    ($ty:ty, $field:tt, $offset:expr_2021) => {
        assert_eq!(
            std::mem::offset_of!($ty, $field),
            $offset,
            "offset of {}.{}",
            stringify!($ty),
            stringify!($field)
        );
    };
}

#[test]
fn rust_abi_layout_matches_plugin_header_expectations() {
    assert_layout!(SeBoostPadEventKind, size = 4, align = 4);
    assert_layout!(SePlayerStatEventKind, size = 4, align = 4);

    assert_layout!(SeVec3, size = 12, align = 4);
    assert_offset!(SeVec3, x, 0);
    assert_offset!(SeVec3, y, 4);
    assert_offset!(SeVec3, z, 8);

    assert_layout!(SeQuat, size = 16, align = 4);
    assert_offset!(SeQuat, x, 0);
    assert_offset!(SeQuat, y, 4);
    assert_offset!(SeQuat, z, 8);
    assert_offset!(SeQuat, w, 12);

    assert_layout!(SeRigidBody, size = 56, align = 4);
    assert_offset!(SeRigidBody, location, 0);
    assert_offset!(SeRigidBody, rotation, 12);
    assert_offset!(SeRigidBody, linear_velocity, 28);
    assert_offset!(SeRigidBody, angular_velocity, 40);
    assert_offset!(SeRigidBody, has_linear_velocity, 52);
    assert_offset!(SeRigidBody, has_angular_velocity, 53);
    assert_offset!(SeRigidBody, sleeping, 54);

    assert_layout!(SeControllerInput, size = 32, align = 4);
    assert_offset!(SeControllerInput, throttle, 0);
    assert_offset!(SeControllerInput, steer, 4);
    assert_offset!(SeControllerInput, pitch, 8);
    assert_offset!(SeControllerInput, yaw, 12);
    assert_offset!(SeControllerInput, roll, 16);
    assert_offset!(SeControllerInput, dodge_forward, 20);
    assert_offset!(SeControllerInput, dodge_strafe, 24);
    assert_offset!(SeControllerInput, handbrake, 28);
    assert_offset!(SeControllerInput, jump, 29);
    assert_offset!(SeControllerInput, activate_boost, 30);
    assert_offset!(SeControllerInput, holding_boost, 31);

    assert_layout!(SeCameraState, size = 6, align = 1);
    assert_offset!(SeCameraState, pitch, 0);
    assert_offset!(SeCameraState, yaw, 1);
    assert_offset!(SeCameraState, has_pitch, 2);
    assert_offset!(SeCameraState, has_yaw, 3);
    assert_offset!(SeCameraState, ball_cam_active, 4);
    assert_offset!(SeCameraState, has_ball_cam, 5);

    assert_layout!(SeRemoteId, size = 24, align = 8);
    assert_offset!(SeRemoteId, online_id, 0);
    assert_offset!(SeRemoteId, epic_id, 8);
    assert_offset!(SeRemoteId, splitscreen_index, 16);
    assert_offset!(SeRemoteId, platform, 20);

    assert_layout!(SePlayerFrame, size = 216, align = 8);
    assert_offset!(SePlayerFrame, player_index, 0);
    assert_offset!(SePlayerFrame, player_name, 8);
    assert_offset!(SePlayerFrame, is_team_0, 16);
    assert_offset!(SePlayerFrame, has_rigid_body, 17);
    assert_offset!(SePlayerFrame, rigid_body, 20);
    assert_offset!(SePlayerFrame, boost_amount, 76);
    assert_offset!(SePlayerFrame, last_boost_amount, 80);
    assert_offset!(SePlayerFrame, boost_active, 84);
    assert_offset!(SePlayerFrame, jump_active, 85);
    assert_offset!(SePlayerFrame, double_jump_active, 86);
    assert_offset!(SePlayerFrame, dodge_active, 87);
    assert_offset!(SePlayerFrame, powerslide_active, 88);
    assert_offset!(SePlayerFrame, car_body_id, 92);
    assert_offset!(SePlayerFrame, has_car_body_id, 96);
    assert_offset!(SePlayerFrame, has_match_stats, 97);
    assert_offset!(SePlayerFrame, match_goals, 100);
    assert_offset!(SePlayerFrame, match_assists, 104);
    assert_offset!(SePlayerFrame, match_saves, 108);
    assert_offset!(SePlayerFrame, match_shots, 112);
    assert_offset!(SePlayerFrame, match_score, 116);
    assert_offset!(SePlayerFrame, has_input, 120);
    assert_offset!(SePlayerFrame, input, 124);
    assert_offset!(SePlayerFrame, camera, 156);
    assert_offset!(SePlayerFrame, has_dodge_impulse, 162);
    assert_offset!(SePlayerFrame, dodge_impulse, 164);
    assert_offset!(SePlayerFrame, has_dodge_torque, 176);
    assert_offset!(SePlayerFrame, dodge_torque, 180);
    assert_offset!(SePlayerFrame, remote_id, 192);

    assert_layout!(SeEventTiming, size = 24, align = 8);
    assert_offset!(SeEventTiming, frame_number, 0);
    assert_offset!(SeEventTiming, time, 8);
    assert_offset!(SeEventTiming, seconds_remaining, 12);
    assert_offset!(SeEventTiming, has_timing, 16);
    assert_offset!(SeEventTiming, has_seconds_remaining, 17);

    assert_layout!(SeTouchEvent, size = 40, align = 8);
    assert_offset!(SeTouchEvent, timing, 0);
    assert_offset!(SeTouchEvent, player_index, 24);
    assert_offset!(SeTouchEvent, has_player, 28);
    assert_offset!(SeTouchEvent, is_team_0, 29);
    assert_offset!(SeTouchEvent, closest_approach_distance, 32);
    assert_offset!(SeTouchEvent, has_closest_approach_distance, 36);

    assert_layout!(SeDodgeRefreshedEvent, size = 40, align = 8);
    assert_offset!(SeDodgeRefreshedEvent, timing, 0);
    assert_offset!(SeDodgeRefreshedEvent, player_index, 24);
    assert_offset!(SeDodgeRefreshedEvent, is_team_0, 28);
    assert_offset!(SeDodgeRefreshedEvent, counter_value, 32);

    assert_layout!(SeBoostPadEvent, size = 48, align = 8);
    assert_offset!(SeBoostPadEvent, timing, 0);
    assert_offset!(SeBoostPadEvent, pad_id, 24);
    assert_offset!(SeBoostPadEvent, kind, 28);
    assert_offset!(SeBoostPadEvent, sequence, 32);
    assert_offset!(SeBoostPadEvent, player_index, 36);
    assert_offset!(SeBoostPadEvent, has_player, 40);

    assert_layout!(SeGoalEvent, size = 56, align = 8);
    assert_offset!(SeGoalEvent, timing, 0);
    assert_offset!(SeGoalEvent, scoring_team_is_team_0, 24);
    assert_offset!(SeGoalEvent, player_index, 28);
    assert_offset!(SeGoalEvent, has_player, 32);
    assert_offset!(SeGoalEvent, team_zero_score, 36);
    assert_offset!(SeGoalEvent, has_team_zero_score, 40);
    assert_offset!(SeGoalEvent, team_one_score, 44);
    assert_offset!(SeGoalEvent, has_team_one_score, 48);

    assert_layout!(SePlayerStatEvent, size = 160, align = 8);
    assert_offset!(SePlayerStatEvent, timing, 0);
    assert_offset!(SePlayerStatEvent, player_index, 24);
    assert_offset!(SePlayerStatEvent, is_team_0, 28);
    assert_offset!(SePlayerStatEvent, kind, 32);
    assert_offset!(SePlayerStatEvent, has_shot_ball, 36);
    assert_offset!(SePlayerStatEvent, shot_ball, 40);
    assert_offset!(SePlayerStatEvent, has_shot_player, 96);
    assert_offset!(SePlayerStatEvent, shot_player, 100);

    assert_layout!(SeDemolishEvent, size = 72, align = 8);
    assert_offset!(SeDemolishEvent, timing, 0);
    assert_offset!(SeDemolishEvent, attacker_index, 24);
    assert_offset!(SeDemolishEvent, victim_index, 28);
    assert_offset!(SeDemolishEvent, attacker_velocity, 32);
    assert_offset!(SeDemolishEvent, victim_velocity, 44);
    assert_offset!(SeDemolishEvent, victim_location, 56);
    assert_offset!(SeDemolishEvent, active_duration_seconds, 68);

    assert_layout!(SeFrame, size = 232, align = 8);
    assert_offset!(SeFrame, frame_number, 0);
    assert_offset!(SeFrame, time, 8);
    assert_offset!(SeFrame, dt, 12);
    assert_offset!(SeFrame, seconds_remaining, 16);
    assert_offset!(SeFrame, has_seconds_remaining, 20);
    assert_offset!(SeFrame, game_state, 24);
    assert_offset!(SeFrame, has_game_state, 28);
    assert_offset!(SeFrame, kickoff_countdown_time, 32);
    assert_offset!(SeFrame, has_kickoff_countdown_time, 36);
    assert_offset!(SeFrame, ball_has_been_hit, 37);
    assert_offset!(SeFrame, has_ball_has_been_hit, 38);
    assert_offset!(SeFrame, team_zero_score, 40);
    assert_offset!(SeFrame, has_team_zero_score, 44);
    assert_offset!(SeFrame, team_one_score, 48);
    assert_offset!(SeFrame, has_team_one_score, 52);
    assert_offset!(SeFrame, possession_team_is_team_0, 53);
    assert_offset!(SeFrame, has_possession_team, 54);
    assert_offset!(SeFrame, scored_on_team_is_team_0, 55);
    assert_offset!(SeFrame, has_scored_on_team, 56);
    assert_offset!(SeFrame, live_play, 57);
    assert_offset!(SeFrame, has_live_play, 58);
    assert_offset!(SeFrame, has_ball, 59);
    assert_offset!(SeFrame, ball, 60);
    assert_offset!(SeFrame, players, 120);
    assert_offset!(SeFrame, player_count, 128);
    assert_offset!(SeFrame, touches, 136);
    assert_offset!(SeFrame, touch_count, 144);
    assert_offset!(SeFrame, dodge_refreshes, 152);
    assert_offset!(SeFrame, dodge_refresh_count, 160);
    assert_offset!(SeFrame, boost_pad_events, 168);
    assert_offset!(SeFrame, boost_pad_event_count, 176);
    assert_offset!(SeFrame, goals, 184);
    assert_offset!(SeFrame, goal_count, 192);
    assert_offset!(SeFrame, player_stat_events, 200);
    assert_offset!(SeFrame, player_stat_event_count, 208);
    assert_offset!(SeFrame, demolishes, 216);
    assert_offset!(SeFrame, demolish_count, 224);

    assert_layout!(SeConfig, size = 24, align = 8);
    assert_offset!(SeConfig, server_name, 0);
    assert_offset!(SeConfig, max_queued_frames, 8);
    assert_offset!(SeConfig, max_client_queue, 12);
    assert_offset!(SeConfig, port, 16);
    assert_offset!(SeConfig, bind_any_interface, 18);

    assert_layout!(SeStatus, size = 32, align = 8);
    assert_offset!(SeStatus, state, 0);
    assert_offset!(SeStatus, client_count, 4);
    assert_offset!(SeStatus, port, 8);
    assert_offset!(SeStatus, frames_sent, 16);
    assert_offset!(SeStatus, frames_dropped, 24);

    assert_layout!(SeMatchContext, size = 24, align = 8);
    assert_offset!(SeMatchContext, match_guid, 0);
    assert_offset!(SeMatchContext, map_name, 8);
    assert_offset!(SeMatchContext, playlist_id, 16);
    assert_offset!(SeMatchContext, has_playlist_id, 20);
}
