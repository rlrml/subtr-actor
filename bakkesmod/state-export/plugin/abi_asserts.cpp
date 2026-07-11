// Included by StateExportPlugin.cpp; shares the plugin translation unit.
//
// Compile-time mirror of rust/src/lib_tests/abi_layout.rs: every struct
// size/alignment plus the field offsets locked there. A drifted header fails
// this build instead of corrupting frames at runtime.
namespace {

#if defined(__clang__) || defined(__GNUC__)
#define SE_OFFSETOF(type, member) __builtin_offsetof(type, member)
#else
#define SE_OFFSETOF(type, member) offsetof(type, member)
#endif

static_assert(sizeof(SeBoostPadEventKind) == 4);
static_assert(sizeof(SePlayerStatEventKind) == 4);

static_assert(std::is_standard_layout_v<SeVec3>);
static_assert(sizeof(SeVec3) == 12);
static_assert(alignof(SeVec3) == 4);
static_assert(SE_OFFSETOF(SeVec3, x) == 0);
static_assert(SE_OFFSETOF(SeVec3, y) == 4);
static_assert(SE_OFFSETOF(SeVec3, z) == 8);

static_assert(std::is_standard_layout_v<SeQuat>);
static_assert(sizeof(SeQuat) == 16);
static_assert(alignof(SeQuat) == 4);
static_assert(SE_OFFSETOF(SeQuat, x) == 0);
static_assert(SE_OFFSETOF(SeQuat, y) == 4);
static_assert(SE_OFFSETOF(SeQuat, z) == 8);
static_assert(SE_OFFSETOF(SeQuat, w) == 12);

static_assert(std::is_standard_layout_v<SeRigidBody>);
static_assert(sizeof(SeRigidBody) == 56);
static_assert(alignof(SeRigidBody) == 4);
static_assert(SE_OFFSETOF(SeRigidBody, location) == 0);
static_assert(SE_OFFSETOF(SeRigidBody, rotation) == 12);
static_assert(SE_OFFSETOF(SeRigidBody, linear_velocity) == 28);
static_assert(SE_OFFSETOF(SeRigidBody, angular_velocity) == 40);
static_assert(SE_OFFSETOF(SeRigidBody, has_linear_velocity) == 52);
static_assert(SE_OFFSETOF(SeRigidBody, has_angular_velocity) == 53);
static_assert(SE_OFFSETOF(SeRigidBody, sleeping) == 54);

static_assert(std::is_standard_layout_v<SeControllerInput>);
static_assert(sizeof(SeControllerInput) == 32);
static_assert(alignof(SeControllerInput) == 4);
static_assert(SE_OFFSETOF(SeControllerInput, throttle) == 0);
static_assert(SE_OFFSETOF(SeControllerInput, steer) == 4);
static_assert(SE_OFFSETOF(SeControllerInput, pitch) == 8);
static_assert(SE_OFFSETOF(SeControllerInput, yaw) == 12);
static_assert(SE_OFFSETOF(SeControllerInput, roll) == 16);
static_assert(SE_OFFSETOF(SeControllerInput, dodge_forward) == 20);
static_assert(SE_OFFSETOF(SeControllerInput, dodge_strafe) == 24);
static_assert(SE_OFFSETOF(SeControllerInput, handbrake) == 28);
static_assert(SE_OFFSETOF(SeControllerInput, jump) == 29);
static_assert(SE_OFFSETOF(SeControllerInput, activate_boost) == 30);
static_assert(SE_OFFSETOF(SeControllerInput, holding_boost) == 31);

static_assert(std::is_standard_layout_v<SeCameraState>);
static_assert(sizeof(SeCameraState) == 6);
static_assert(alignof(SeCameraState) == 1);
static_assert(SE_OFFSETOF(SeCameraState, pitch) == 0);
static_assert(SE_OFFSETOF(SeCameraState, yaw) == 1);
static_assert(SE_OFFSETOF(SeCameraState, has_pitch) == 2);
static_assert(SE_OFFSETOF(SeCameraState, has_yaw) == 3);
static_assert(SE_OFFSETOF(SeCameraState, ball_cam_active) == 4);
static_assert(SE_OFFSETOF(SeCameraState, has_ball_cam) == 5);

static_assert(std::is_standard_layout_v<SeRemoteId>);
static_assert(sizeof(SeRemoteId) == 24);
static_assert(alignof(SeRemoteId) == 8);
static_assert(SE_OFFSETOF(SeRemoteId, online_id) == 0);
static_assert(SE_OFFSETOF(SeRemoteId, epic_id) == 8);
static_assert(SE_OFFSETOF(SeRemoteId, splitscreen_index) == 16);
static_assert(SE_OFFSETOF(SeRemoteId, platform) == 20);

// SePlayerFrame begins with a byte-identical copy of SaPlayerFrame (offsets
// 0-119); the superset fields start at has_input (120).
static_assert(std::is_standard_layout_v<SePlayerFrame>);
static_assert(sizeof(SePlayerFrame) == 216);
static_assert(alignof(SePlayerFrame) == 8);
static_assert(SE_OFFSETOF(SePlayerFrame, player_index) == 0);
static_assert(SE_OFFSETOF(SePlayerFrame, player_name) == 8);
static_assert(SE_OFFSETOF(SePlayerFrame, is_team_0) == 16);
static_assert(SE_OFFSETOF(SePlayerFrame, has_rigid_body) == 17);
static_assert(SE_OFFSETOF(SePlayerFrame, rigid_body) == 20);
static_assert(SE_OFFSETOF(SePlayerFrame, boost_amount) == 76);
static_assert(SE_OFFSETOF(SePlayerFrame, last_boost_amount) == 80);
static_assert(SE_OFFSETOF(SePlayerFrame, boost_active) == 84);
static_assert(SE_OFFSETOF(SePlayerFrame, jump_active) == 85);
static_assert(SE_OFFSETOF(SePlayerFrame, double_jump_active) == 86);
static_assert(SE_OFFSETOF(SePlayerFrame, dodge_active) == 87);
static_assert(SE_OFFSETOF(SePlayerFrame, powerslide_active) == 88);
static_assert(SE_OFFSETOF(SePlayerFrame, car_body_id) == 92);
static_assert(SE_OFFSETOF(SePlayerFrame, has_car_body_id) == 96);
static_assert(SE_OFFSETOF(SePlayerFrame, has_match_stats) == 97);
static_assert(SE_OFFSETOF(SePlayerFrame, match_goals) == 100);
static_assert(SE_OFFSETOF(SePlayerFrame, match_assists) == 104);
static_assert(SE_OFFSETOF(SePlayerFrame, match_saves) == 108);
static_assert(SE_OFFSETOF(SePlayerFrame, match_shots) == 112);
static_assert(SE_OFFSETOF(SePlayerFrame, match_score) == 116);
static_assert(SE_OFFSETOF(SePlayerFrame, has_input) == 120);
static_assert(SE_OFFSETOF(SePlayerFrame, input) == 124);
static_assert(SE_OFFSETOF(SePlayerFrame, camera) == 156);
static_assert(SE_OFFSETOF(SePlayerFrame, has_dodge_impulse) == 162);
static_assert(SE_OFFSETOF(SePlayerFrame, dodge_impulse) == 164);
static_assert(SE_OFFSETOF(SePlayerFrame, has_dodge_torque) == 176);
static_assert(SE_OFFSETOF(SePlayerFrame, dodge_torque) == 180);
static_assert(SE_OFFSETOF(SePlayerFrame, remote_id) == 192);

static_assert(std::is_standard_layout_v<SeEventTiming>);
static_assert(sizeof(SeEventTiming) == 24);
static_assert(alignof(SeEventTiming) == 8);
static_assert(SE_OFFSETOF(SeEventTiming, frame_number) == 0);
static_assert(SE_OFFSETOF(SeEventTiming, time) == 8);
static_assert(SE_OFFSETOF(SeEventTiming, seconds_remaining) == 12);
static_assert(SE_OFFSETOF(SeEventTiming, has_timing) == 16);
static_assert(SE_OFFSETOF(SeEventTiming, has_seconds_remaining) == 17);

static_assert(std::is_standard_layout_v<SeTouchEvent>);
static_assert(sizeof(SeTouchEvent) == 40);
static_assert(alignof(SeTouchEvent) == 8);
static_assert(SE_OFFSETOF(SeTouchEvent, timing) == 0);
static_assert(SE_OFFSETOF(SeTouchEvent, player_index) == 24);
static_assert(SE_OFFSETOF(SeTouchEvent, has_player) == 28);
static_assert(SE_OFFSETOF(SeTouchEvent, is_team_0) == 29);
static_assert(SE_OFFSETOF(SeTouchEvent, closest_approach_distance) == 32);
static_assert(SE_OFFSETOF(SeTouchEvent, has_closest_approach_distance) == 36);

static_assert(std::is_standard_layout_v<SeDodgeRefreshedEvent>);
static_assert(sizeof(SeDodgeRefreshedEvent) == 40);
static_assert(alignof(SeDodgeRefreshedEvent) == 8);
static_assert(SE_OFFSETOF(SeDodgeRefreshedEvent, timing) == 0);
static_assert(SE_OFFSETOF(SeDodgeRefreshedEvent, player_index) == 24);
static_assert(SE_OFFSETOF(SeDodgeRefreshedEvent, is_team_0) == 28);
static_assert(SE_OFFSETOF(SeDodgeRefreshedEvent, counter_value) == 32);

static_assert(std::is_standard_layout_v<SeBoostPadEvent>);
static_assert(sizeof(SeBoostPadEvent) == 48);
static_assert(alignof(SeBoostPadEvent) == 8);
static_assert(SE_OFFSETOF(SeBoostPadEvent, timing) == 0);
static_assert(SE_OFFSETOF(SeBoostPadEvent, pad_id) == 24);
static_assert(SE_OFFSETOF(SeBoostPadEvent, kind) == 28);
static_assert(SE_OFFSETOF(SeBoostPadEvent, sequence) == 32);
static_assert(SE_OFFSETOF(SeBoostPadEvent, player_index) == 36);
static_assert(SE_OFFSETOF(SeBoostPadEvent, has_player) == 40);

static_assert(std::is_standard_layout_v<SeGoalEvent>);
static_assert(sizeof(SeGoalEvent) == 56);
static_assert(alignof(SeGoalEvent) == 8);
static_assert(SE_OFFSETOF(SeGoalEvent, timing) == 0);
static_assert(SE_OFFSETOF(SeGoalEvent, scoring_team_is_team_0) == 24);
static_assert(SE_OFFSETOF(SeGoalEvent, player_index) == 28);
static_assert(SE_OFFSETOF(SeGoalEvent, has_player) == 32);
static_assert(SE_OFFSETOF(SeGoalEvent, team_zero_score) == 36);
static_assert(SE_OFFSETOF(SeGoalEvent, has_team_zero_score) == 40);
static_assert(SE_OFFSETOF(SeGoalEvent, team_one_score) == 44);
static_assert(SE_OFFSETOF(SeGoalEvent, has_team_one_score) == 48);

static_assert(std::is_standard_layout_v<SePlayerStatEvent>);
static_assert(sizeof(SePlayerStatEvent) == 160);
static_assert(alignof(SePlayerStatEvent) == 8);
static_assert(SE_OFFSETOF(SePlayerStatEvent, timing) == 0);
static_assert(SE_OFFSETOF(SePlayerStatEvent, player_index) == 24);
static_assert(SE_OFFSETOF(SePlayerStatEvent, is_team_0) == 28);
static_assert(SE_OFFSETOF(SePlayerStatEvent, kind) == 32);
static_assert(SE_OFFSETOF(SePlayerStatEvent, has_shot_ball) == 36);
static_assert(SE_OFFSETOF(SePlayerStatEvent, shot_ball) == 40);
static_assert(SE_OFFSETOF(SePlayerStatEvent, has_shot_player) == 96);
static_assert(SE_OFFSETOF(SePlayerStatEvent, shot_player) == 100);

static_assert(std::is_standard_layout_v<SeDemolishEvent>);
static_assert(sizeof(SeDemolishEvent) == 72);
static_assert(alignof(SeDemolishEvent) == 8);
static_assert(SE_OFFSETOF(SeDemolishEvent, timing) == 0);
static_assert(SE_OFFSETOF(SeDemolishEvent, attacker_index) == 24);
static_assert(SE_OFFSETOF(SeDemolishEvent, victim_index) == 28);
static_assert(SE_OFFSETOF(SeDemolishEvent, attacker_velocity) == 32);
static_assert(SE_OFFSETOF(SeDemolishEvent, victim_velocity) == 44);
static_assert(SE_OFFSETOF(SeDemolishEvent, victim_location) == 56);
static_assert(SE_OFFSETOF(SeDemolishEvent, active_duration_seconds) == 68);

// SeFrame is byte-identical in layout to SaLiveFrame.
static_assert(std::is_standard_layout_v<SeFrame>);
static_assert(sizeof(SeFrame) == 232);
static_assert(alignof(SeFrame) == 8);
static_assert(SE_OFFSETOF(SeFrame, frame_number) == 0);
static_assert(SE_OFFSETOF(SeFrame, time) == 8);
static_assert(SE_OFFSETOF(SeFrame, dt) == 12);
static_assert(SE_OFFSETOF(SeFrame, seconds_remaining) == 16);
static_assert(SE_OFFSETOF(SeFrame, has_seconds_remaining) == 20);
static_assert(SE_OFFSETOF(SeFrame, game_state) == 24);
static_assert(SE_OFFSETOF(SeFrame, has_game_state) == 28);
static_assert(SE_OFFSETOF(SeFrame, kickoff_countdown_time) == 32);
static_assert(SE_OFFSETOF(SeFrame, has_kickoff_countdown_time) == 36);
static_assert(SE_OFFSETOF(SeFrame, ball_has_been_hit) == 37);
static_assert(SE_OFFSETOF(SeFrame, has_ball_has_been_hit) == 38);
static_assert(SE_OFFSETOF(SeFrame, team_zero_score) == 40);
static_assert(SE_OFFSETOF(SeFrame, has_team_zero_score) == 44);
static_assert(SE_OFFSETOF(SeFrame, team_one_score) == 48);
static_assert(SE_OFFSETOF(SeFrame, has_team_one_score) == 52);
static_assert(SE_OFFSETOF(SeFrame, possession_team_is_team_0) == 53);
static_assert(SE_OFFSETOF(SeFrame, has_possession_team) == 54);
static_assert(SE_OFFSETOF(SeFrame, scored_on_team_is_team_0) == 55);
static_assert(SE_OFFSETOF(SeFrame, has_scored_on_team) == 56);
static_assert(SE_OFFSETOF(SeFrame, live_play) == 57);
static_assert(SE_OFFSETOF(SeFrame, has_live_play) == 58);
static_assert(SE_OFFSETOF(SeFrame, has_ball) == 59);
static_assert(SE_OFFSETOF(SeFrame, ball) == 60);
static_assert(SE_OFFSETOF(SeFrame, players) == 120);
static_assert(SE_OFFSETOF(SeFrame, player_count) == 128);
static_assert(SE_OFFSETOF(SeFrame, touches) == 136);
static_assert(SE_OFFSETOF(SeFrame, touch_count) == 144);
static_assert(SE_OFFSETOF(SeFrame, dodge_refreshes) == 152);
static_assert(SE_OFFSETOF(SeFrame, dodge_refresh_count) == 160);
static_assert(SE_OFFSETOF(SeFrame, boost_pad_events) == 168);
static_assert(SE_OFFSETOF(SeFrame, boost_pad_event_count) == 176);
static_assert(SE_OFFSETOF(SeFrame, goals) == 184);
static_assert(SE_OFFSETOF(SeFrame, goal_count) == 192);
static_assert(SE_OFFSETOF(SeFrame, player_stat_events) == 200);
static_assert(SE_OFFSETOF(SeFrame, player_stat_event_count) == 208);
static_assert(SE_OFFSETOF(SeFrame, demolishes) == 216);
static_assert(SE_OFFSETOF(SeFrame, demolish_count) == 224);

static_assert(std::is_standard_layout_v<SeConfig>);
static_assert(sizeof(SeConfig) == 24);
static_assert(alignof(SeConfig) == 8);
static_assert(SE_OFFSETOF(SeConfig, server_name) == 0);
static_assert(SE_OFFSETOF(SeConfig, max_queued_frames) == 8);
static_assert(SE_OFFSETOF(SeConfig, max_client_queue) == 12);
static_assert(SE_OFFSETOF(SeConfig, port) == 16);
static_assert(SE_OFFSETOF(SeConfig, bind_any_interface) == 18);

static_assert(std::is_standard_layout_v<SeStatus>);
static_assert(sizeof(SeStatus) == 32);
static_assert(alignof(SeStatus) == 8);
static_assert(SE_OFFSETOF(SeStatus, state) == 0);
static_assert(SE_OFFSETOF(SeStatus, client_count) == 4);
static_assert(SE_OFFSETOF(SeStatus, port) == 8);
static_assert(SE_OFFSETOF(SeStatus, frames_sent) == 16);
static_assert(SE_OFFSETOF(SeStatus, frames_dropped) == 24);

static_assert(std::is_standard_layout_v<SeMatchContext>);
static_assert(sizeof(SeMatchContext) == 24);
static_assert(alignof(SeMatchContext) == 8);
static_assert(SE_OFFSETOF(SeMatchContext, match_guid) == 0);
static_assert(SE_OFFSETOF(SeMatchContext, map_name) == 8);
static_assert(SE_OFFSETOF(SeMatchContext, playlist_id) == 16);
static_assert(SE_OFFSETOF(SeMatchContext, has_playlist_id) == 20);

} // namespace

#undef SE_OFFSETOF
