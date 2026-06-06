use super::*;

fn vec3_json(value: &Vector3f) -> Value {
    json!({
        "x": value.x,
        "y": value.y,
        "z": value.z,
    })
}

fn quat_json(value: &Quaternion) -> Value {
    json!({
        "x": value.x,
        "y": value.y,
        "z": value.z,
        "w": value.w,
    })
}

fn rigid_body_json(value: &RigidBody) -> Value {
    json!({
        "location": vec3_json(&value.location),
        "rotation": quat_json(&value.rotation),
        "sleeping": value.sleeping,
        "linear_velocity": value.linear_velocity.as_ref().map(vec3_json),
        "angular_velocity": value.angular_velocity.as_ref().map(vec3_json),
    })
}

fn ball_frame_state_json(state: &BallFrameState) -> Value {
    match state {
        BallFrameState::Missing => json!({
            "kind": "Missing",
            "ball": Value::Null,
        }),
        BallFrameState::Present(ball) => json!({
            "kind": "Present",
            "ball": ball_sample_json(ball),
        }),
    }
}

fn ball_sample_json(sample: &BallSample) -> Value {
    json!({
        "rigid_body": rigid_body_json(&sample.rigid_body),
    })
}

fn player_sample_json(sample: &PlayerSample) -> Value {
    json!({
        "player_id": sample.player_id,
        "is_team_0": sample.is_team_0,
        "rigid_body": sample.rigid_body.as_ref().map(rigid_body_json),
        "boost_amount": sample.boost_amount,
        "last_boost_amount": sample.last_boost_amount,
        "boost_active": sample.boost_active,
        "dodge_active": sample.dodge_active,
        "powerslide_active": sample.powerslide_active,
        "match_goals": sample.match_goals,
        "match_assists": sample.match_assists,
        "match_saves": sample.match_saves,
        "match_shots": sample.match_shots,
        "match_score": sample.match_score,
    })
}

fn demo_event_sample_json(sample: &DemoEventSample) -> Value {
    json!({
        "attacker": sample.attacker,
        "victim": sample.victim,
    })
}

fn vertical_band_label(band: PlayerVerticalBand) -> &'static str {
    match band {
        PlayerVerticalBand::Ground => "ground",
        PlayerVerticalBand::LowAir => "low_air",
        PlayerVerticalBand::HighAir => "high_air",
    }
}

fn player_vertical_state_json(state: &PlayerVerticalState) -> Value {
    let mut players = state
        .players
        .iter()
        .map(|(player_id, sample)| {
            json!({
                "player_id": player_id,
                "height": sample.height,
                "band": vertical_band_label(sample.band),
            })
        })
        .collect::<Vec<_>>();
    players.sort_by_key(|value| value["player_id"].to_string());
    json!({ "players": players })
}

fn settings_json(calculator: &SettingsCalculator) -> Value {
    let mut player_settings = calculator
        .player_settings()
        .iter()
        .map(|(player_id, settings)| {
            json!({
                "player_id": player_id,
                "settings": {
                    "steering_sensitivity": settings.steering_sensitivity,
                    "camera_fov": settings.camera_fov,
                    "camera_height": settings.camera_height,
                    "camera_pitch": settings.camera_pitch,
                    "camera_distance": settings.camera_distance,
                    "camera_stiffness": settings.camera_stiffness,
                    "camera_swivel_speed": settings.camera_swivel_speed,
                    "camera_transition_speed": settings.camera_transition_speed,
                },
            })
        })
        .collect::<Vec<_>>();
    player_settings.sort_by_key(|value| value["player_id"].to_string());
    json!({ "player_settings": player_settings })
}

pub fn builtin_analysis_node_json(
    node_name: &str,
    graph: &AnalysisGraph,
) -> SubtrActorResult<Value> {
    let value = match node_name {
        "core" | "match_stats" => builtin_module_json("core", graph)?,
        "stats_timeline_events" => serialize_to_json_value(
            &graph_state::<StatsTimelineEventsState>(graph, node_name)?.events,
        )?,
        "stats_timeline_frame" => graph_state::<StatsTimelineFrameState>(graph, node_name)?
            .frame
            .as_ref()
            .map(serialize_to_json_value)
            .transpose()?
            .unwrap_or(Value::Null),
        "frame_info" => {
            let state = graph_state::<FrameInfo>(graph, node_name)?;
            json!({
                "frame_number": state.frame_number,
                "time": state.time,
                "dt": state.dt,
                "seconds_remaining": state.seconds_remaining,
            })
        }
        "gameplay_state" => {
            let state = graph_state::<GameplayState>(graph, node_name)?;
            json!({
                "game_state": state.game_state,
                "ball_has_been_hit": state.ball_has_been_hit,
                "kickoff_countdown_time": state.kickoff_countdown_time,
                "team_zero_score": state.team_zero_score,
                "team_one_score": state.team_one_score,
                "possession_team_is_team_0": state.possession_team_is_team_0,
                "scored_on_team_is_team_0": state.scored_on_team_is_team_0,
                "current_in_game_team_player_counts": state.current_in_game_team_player_counts,
                "is_live_play": state.is_live_play(),
                "kickoff_phase_active": state.kickoff_phase_active(),
            })
        }
        "ball_frame_state" => {
            ball_frame_state_json(graph_state::<BallFrameState>(graph, node_name)?)
        }
        "player_frame_state" => {
            let state = graph_state::<PlayerFrameState>(graph, node_name)?;
            json!({
                "players": state.players.iter().map(player_sample_json).collect::<Vec<_>>(),
            })
        }
        "frame_events_state" => {
            let state = graph_state::<FrameEventsState>(graph, node_name)?;
            json!({
                "active_demos": state.active_demos.iter().map(demo_event_sample_json).collect::<Vec<_>>(),
                "demo_events": state.demo_events,
                "boost_pad_events": state.boost_pad_events,
                "touch_events": state.touch_events,
                "dodge_refreshed_events": state.dodge_refreshed_events,
                "player_stat_events": state.player_stat_events,
                "goal_events": state.goal_events,
            })
        }
        "live_play" => serialize_to_json_value(graph_state::<LivePlayState>(graph, node_name)?)?,
        "touch_state" => {
            let state = graph_state::<TouchState>(graph, node_name)?;
            json!({
                "touch_events": state.touch_events,
                "last_touch": state.last_touch,
                "last_touch_player": state.last_touch_player,
                "last_touch_team_is_team_0": state.last_touch_team_is_team_0,
            })
        }
        "possession_state" => {
            let state = graph_state::<PossessionState>(graph, node_name)?;
            json!({
                "active_team_before_sample": state.active_team_before_sample,
                "current_team_is_team_0": state.current_team_is_team_0,
                "active_player_before_sample": state.active_player_before_sample,
                "current_player": state.current_player,
            })
        }
        "backboard_bounce_state" => {
            let state = graph_state::<BackboardBounceState>(graph, node_name)?;
            json!({
                "bounce_events": state.bounce_events,
                "last_bounce_event": state.last_bounce_event,
            })
        }
        "continuous_ball_control" => {
            let state = graph_state::<ContinuousBallControlState>(graph, node_name)?;
            json!({
                "completed_sequences": state.completed_sequences.iter().map(|sequence| {
                    json!({
                        "player_id": sequence.player_id,
                        "is_team_0": sequence.is_team_0,
                        "kind": sequence.kind,
                        "start_frame": sequence.start_frame,
                        "end_frame": sequence.end_frame,
                        "start_time": sequence.start_time,
                        "end_time": sequence.end_time,
                        "duration": sequence.duration,
                        "straight_line_distance": sequence.straight_line_distance,
                        "path_distance": sequence.path_distance,
                        "average_horizontal_gap": sequence.average_horizontal_gap,
                        "average_vertical_gap": sequence.average_vertical_gap,
                        "average_speed": sequence.average_speed,
                        "start_position": {
                            "x": sequence.start_position.x,
                            "y": sequence.start_position.y,
                            "z": sequence.start_position.z,
                        },
                        "end_position": {
                            "x": sequence.end_position.x,
                            "y": sequence.end_position.y,
                            "z": sequence.end_position.z,
                        },
                        "touch_count": sequence.touch_count,
                        "air_touch_count": sequence.air_touch_count,
                    })
                }).collect::<Vec<_>>(),
            })
        }
        "fifty_fifty_state" => {
            let state = graph_state::<FiftyFiftyState>(graph, node_name)?;
            json!({
                "active_event": state.active_event.as_ref().map(|event| {
                    json!({
                        "start_time": event.start_time,
                        "start_frame": event.start_frame,
                        "last_touch_time": event.last_touch_time,
                        "last_touch_frame": event.last_touch_frame,
                        "is_kickoff": event.is_kickoff,
                        "team_zero_player": event.team_zero_player,
                        "team_one_player": event.team_one_player,
                        "team_zero_position": event.team_zero_position,
                        "team_one_position": event.team_one_position,
                        "midpoint": event.midpoint,
                        "plane_normal": event.plane_normal,
                    })
                }),
                "resolved_events": state.resolved_events,
                "last_resolved_event": state.last_resolved_event,
            })
        }
        "player_vertical_state" => {
            player_vertical_state_json(graph_state::<PlayerVerticalState>(graph, node_name)?)
        }
        "settings" => settings_json(graph_state::<SettingsCalculator>(graph, node_name)?),
        module_name if builtin_stats_module_names().contains(&module_name) => {
            builtin_module_json(module_name, graph)?
        }
        _ => {
            return SubtrActorError::new_result(SubtrActorErrorVariant::UnknownStatsModuleName(
                node_name.to_owned(),
            ));
        }
    };
    Ok(value)
}

pub fn builtin_analysis_nodes_json(graph: &AnalysisGraph) -> SubtrActorResult<Value> {
    let mut values = Map::new();
    for node_name in builtin_analysis_node_names() {
        values.insert(
            (*node_name).to_owned(),
            builtin_analysis_node_json(node_name, graph)?,
        );
    }
    Ok(Value::Object(values))
}

pub(crate) fn builtin_snapshot_frame_json(
    module_name: &str,
    graph: &AnalysisGraph,
    _replay_meta: &ReplayMeta,
) -> SubtrActorResult<Option<Value>> {
    let value = match module_name {
        "core" => {
            let calculator = graph_state::<MatchStatsCalculator>(graph, module_name)?;
            let mut player_stats: Vec<_> = calculator
                .player_stats()
                .iter()
                .map(|(player_id, stats)| OwnedPlayerStatsEntry {
                    player_id: player_id.clone(),
                    stats: CorePlayerStatsSnapshot::from(stats),
                })
                .collect();
            player_stats.sort_by(|left, right| {
                format!("{:?}", left.player_id).cmp(&format!("{:?}", right.player_id))
            });
            serialize_to_json_value(&CoreStatsSnapshotExport {
                team_zero: calculator.team_zero_stats().into(),
                team_one: calculator.team_one_stats().into(),
                player_stats,
            })?
        }
        "backboard" => {
            let calculator = graph_state::<BackboardCalculator>(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsExport {
                team_zero: calculator.team_zero_stats(),
                team_one: calculator.team_one_stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
            })?
        }
        "ceiling_shot" => {
            let calculator = graph_state::<CeilingShotCalculator>(graph, module_name)?;
            serialize_to_json_value(&PlayerStatsExport {
                player_stats: player_stats_entries(calculator.player_stats()),
            })?
        }
        "wall_aerial" => {
            let calculator = graph_state::<WallAerialCalculator>(graph, module_name)?;
            serialize_to_json_value(&PlayerStatsExport {
                player_stats: player_stats_entries(calculator.player_stats()),
            })?
        }
        "wall_aerial_shot" => {
            let calculator = graph_state::<WallAerialShotCalculator>(graph, module_name)?;
            serialize_to_json_value(&PlayerStatsExport {
                player_stats: player_stats_entries(calculator.player_stats()),
            })?
        }
        "center" => {
            let calculator = graph_state::<CenterCalculator>(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsExport {
                team_zero: calculator.team_zero_stats(),
                team_one: calculator.team_one_stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
            })?
        }
        "double_tap" => {
            let calculator = graph_state::<DoubleTapCalculator>(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsExport {
                team_zero: calculator.team_zero_stats(),
                team_one: calculator.team_one_stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
            })?
        }
        "one_timer" => {
            let calculator = graph_state::<OneTimerCalculator>(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsExport {
                team_zero: calculator.team_zero_stats(),
                team_one: calculator.team_one_stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
            })?
        }
        "half_volley" => {
            let calculator = graph_state::<HalfVolleyCalculator>(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsExport {
                team_zero: calculator.team_zero_stats(),
                team_one: calculator.team_one_stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
            })?
        }
        "pass" => {
            let calculator = graph_state::<PassCalculator>(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsExport {
                team_zero: calculator.team_zero_stats(),
                team_one: calculator.team_one_stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
            })?
        }
        "aerial_goal"
        | "high_aerial_goal"
        | "long_distance_goal"
        | "own_half_goal"
        | "empty_net_goal"
        | "counter_attack_goal"
        | "flick_goal"
        | "double_tap_goal"
        | "one_timer_goal"
        | "passing_goal"
        | "air_dribble_goal"
        | "flip_reset_goal"
        | "bump_goal"
        | "demo_goal"
        | "half_volley_goal" => serialize_to_json_value(&serde_json::json!({}))?,
        "fifty_fifty" => {
            let calculator = graph_state::<FiftyFiftyCalculator>(graph, module_name)?;
            serialize_to_json_value(&StatsWithPlayerStatsExport {
                stats: calculator.stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
            })?
        }
        "possession" => {
            let calculator = graph_state::<PossessionCalculator>(graph, module_name)?;
            serialize_to_json_value(&StatsExport {
                stats: calculator.stats(),
            })?
        }
        "pressure" => {
            let calculator = graph_state::<PressureCalculator>(graph, module_name)?;
            serialize_to_json_value(&StatsExport {
                stats: calculator.stats(),
            })?
        }
        "territorial_pressure" => {
            let calculator = graph_state::<TerritorialPressureCalculator>(graph, module_name)?;
            serialize_to_json_value(&StatsExport {
                stats: calculator.stats(),
            })?
        }
        "rotation" => {
            let calculator = graph_state::<RotationCalculator>(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsExport {
                team_zero: calculator.team_zero_stats(),
                team_one: calculator.team_one_stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
            })?
        }
        "rush" => {
            let calculator = graph_state::<RushCalculator>(graph, module_name)?;
            serialize_to_json_value(&StatsExport {
                stats: calculator.stats(),
            })?
        }
        "touch" => {
            let calculator = graph_state::<TouchCalculator>(graph, module_name)?;
            let player_stats = calculator
                .player_stats()
                .iter()
                .map(|(player_id, stats)| OwnedPlayerStatsEntry {
                    player_id: player_id.clone(),
                    stats: stats.clone().with_complete_labeled_touch_counts(),
                })
                .collect();
            serialize_to_json_value(&OwnedPlayerStatsExport { player_stats })?
        }
        "whiff" => {
            let calculator = graph_state::<WhiffCalculator>(graph, module_name)?;
            serialize_to_json_value(&PlayerStatsExport {
                player_stats: player_stats_entries(calculator.player_stats()),
            })?
        }
        "wavedash" => {
            let calculator = graph_state::<WavedashCalculator>(graph, module_name)?;
            serialize_to_json_value(&PlayerStatsExport {
                player_stats: player_stats_entries(calculator.player_stats()),
            })?
        }
        "speed_flip" => {
            let calculator = graph_state::<SpeedFlipCalculator>(graph, module_name)?;
            serialize_to_json_value(&PlayerStatsExport {
                player_stats: player_stats_entries(calculator.player_stats()),
            })?
        }
        "half_flip" => {
            let calculator = graph_state::<HalfFlipCalculator>(graph, module_name)?;
            serialize_to_json_value(&PlayerStatsExport {
                player_stats: player_stats_entries(calculator.player_stats()),
            })?
        }
        "flick" => {
            let calculator = graph_state::<FlickCalculator>(graph, module_name)?;
            serialize_to_json_value(&PlayerStatsExport {
                player_stats: player_stats_entries(calculator.player_stats()),
            })?
        }
        "musty_flick" => {
            let calculator = graph_state::<MustyFlickCalculator>(graph, module_name)?;
            serialize_to_json_value(&PlayerStatsExport {
                player_stats: player_stats_entries(calculator.player_stats()),
            })?
        }
        "dodge_reset" => {
            let calculator = graph_state::<DodgeResetCalculator>(graph, module_name)?;
            serialize_to_json_value(&PlayerStatsExport {
                player_stats: player_stats_entries(calculator.player_stats()),
            })?
        }
        "ball_carry" => {
            let calculator = graph_state::<BallCarryCalculator>(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsExport {
                team_zero: calculator.team_zero_stats(),
                team_one: calculator.team_one_stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
            })?
        }
        "air_dribble" => {
            let calculator = graph_state::<BallCarryCalculator>(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsExport {
                team_zero: calculator.team_zero_air_dribble_stats(),
                team_one: calculator.team_one_air_dribble_stats(),
                player_stats: player_stats_entries(calculator.player_air_dribble_stats()),
            })?
        }
        "boost" => {
            let calculator = graph_state::<BoostCalculator>(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsExport {
                team_zero: calculator.team_zero_stats(),
                team_one: calculator.team_one_stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
            })?
        }
        "bump" => {
            let calculator = graph_state::<BumpCalculator>(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsExport {
                team_zero: calculator.team_zero_stats(),
                team_one: calculator.team_one_stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
            })?
        }
        "movement" => {
            let calculator = graph_state::<MovementCalculator>(graph, module_name)?;
            let player_stats = calculator
                .player_stats()
                .iter()
                .map(|(player_id, stats)| OwnedPlayerStatsEntry {
                    player_id: player_id.clone(),
                    stats: stats.clone().with_complete_labeled_tracked_time(),
                })
                .collect();
            serialize_to_json_value(&TeamOwnedPlayerStatsExport {
                team_zero: calculator.team_zero_stats(),
                team_one: calculator.team_one_stats(),
                player_stats,
            })?
        }
        "positioning" => {
            let calculator = graph_state::<PositioningCalculator>(graph, module_name)?;
            serialize_to_json_value(&PlayerStatsExport {
                player_stats: player_stats_entries(calculator.player_stats()),
            })?
        }
        "powerslide" => {
            let calculator = graph_state::<PowerslideCalculator>(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsExport {
                team_zero: calculator.team_zero_stats(),
                team_one: calculator.team_one_stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
            })?
        }
        "demo" => {
            let calculator = graph_state::<DemoCalculator>(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsExport {
                team_zero: calculator.team_zero_stats(),
                team_one: calculator.team_one_stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
            })?
        }
        _ => {
            return SubtrActorError::new_result(SubtrActorErrorVariant::UnknownStatsModuleName(
                module_name.to_owned(),
            ))
        }
    };
    Ok(Some(value))
}

pub(crate) fn builtin_snapshot_config_json(
    module_name: &str,
    graph: &AnalysisGraph,
) -> SubtrActorResult<Option<Value>> {
    let value = match module_name {
        "positioning" => {
            let calculator = graph_state::<PositioningCalculator>(graph, module_name)?;
            Some(serialize_to_json_value(&serde_json::json!({
                "most_back_forward_threshold_y": calculator.config().most_back_forward_threshold_y,
                "level_ball_depth_margin": calculator.config().level_ball_depth_margin,
            }))?)
        }
        "pressure" => {
            let calculator = graph_state::<PressureCalculator>(graph, module_name)?;
            Some(serialize_to_json_value(&serde_json::json!({
                "pressure_neutral_zone_half_width_y": calculator.config().neutral_zone_half_width_y,
            }))?)
        }
        "territorial_pressure" => {
            let calculator = graph_state::<TerritorialPressureCalculator>(graph, module_name)?;
            Some(serialize_to_json_value(&serde_json::json!({
                "territorial_pressure_neutral_zone_half_width_y": calculator.config().neutral_zone_half_width_y,
                "territorial_pressure_min_establish_seconds": calculator.config().min_establish_seconds,
                "territorial_pressure_min_establish_third_seconds": calculator.config().min_establish_third_seconds,
                "territorial_pressure_relief_grace_seconds": calculator.config().relief_grace_seconds,
                "territorial_pressure_confirmed_relief_grace_seconds": calculator.config().confirmed_relief_grace_seconds,
            }))?)
        }
        "rotation" => {
            let calculator = graph_state::<RotationCalculator>(graph, module_name)?;
            Some(serialize_to_json_value(&serde_json::json!({
                "role_depth_margin": calculator.config().role_depth_margin,
                "first_man_ambiguity_margin": calculator.config().first_man_ambiguity_margin,
                "first_man_debounce_seconds": calculator.config().first_man_debounce_seconds,
            }))?)
        }
        "rush" => {
            let calculator = graph_state::<RushCalculator>(graph, module_name)?;
            Some(serialize_to_json_value(&serde_json::json!({
                "rush_max_start_y": calculator.config().max_start_y,
                "rush_attack_support_distance_y": calculator.config().attack_support_distance_y,
                "rush_defender_distance_y": calculator.config().defender_distance_y,
                "rush_min_possession_retained_seconds": calculator.config().min_possession_retained_seconds,
            }))?)
        }
        "aerial_goal" => {
            let calculator = graph_state::<AerialGoalCalculator>(graph, module_name)?;
            Some(serialize_to_json_value(&serde_json::json!({
                "aerial_goal_min_ball_z": calculator.config().min_ball_z,
            }))?)
        }
        "high_aerial_goal" => {
            let calculator = graph_state::<HighAerialGoalCalculator>(graph, module_name)?;
            Some(serialize_to_json_value(&serde_json::json!({
                "high_aerial_goal_min_ball_z": calculator.config().min_ball_z,
            }))?)
        }
        "long_distance_goal" => {
            let calculator = graph_state::<LongDistanceGoalCalculator>(graph, module_name)?;
            Some(serialize_to_json_value(&serde_json::json!({
                "long_distance_goal_max_attacking_y": calculator.config().max_attacking_y,
            }))?)
        }
        "own_half_goal" => {
            let calculator = graph_state::<OwnHalfGoalCalculator>(graph, module_name)?;
            Some(serialize_to_json_value(&serde_json::json!({
                "own_half_goal_max_attacking_y": calculator.config().max_attacking_y,
            }))?)
        }
        "empty_net_goal" => {
            let calculator = graph_state::<EmptyNetGoalCalculator>(graph, module_name)?;
            Some(serialize_to_json_value(&serde_json::json!({
                "empty_net_min_defender_y_margin": calculator.config().min_defender_y_margin,
                "empty_net_min_defender_distance": calculator.config().min_defender_distance,
                "empty_net_max_touch_attacking_y": calculator.config().max_touch_attacking_y,
            }))?)
        }
        "flick_goal" => {
            let calculator = graph_state::<FlickGoalCalculator>(graph, module_name)?;
            Some(serialize_to_json_value(&serde_json::json!({
                "flick_goal_max_event_to_goal_seconds": calculator.config().max_event_to_goal_seconds,
            }))?)
        }
        "double_tap_goal" => {
            let calculator = graph_state::<DoubleTapGoalCalculator>(graph, module_name)?;
            Some(serialize_to_json_value(&serde_json::json!({
                "double_tap_goal_max_event_to_goal_seconds": calculator.config().max_event_to_goal_seconds,
            }))?)
        }
        "one_timer_goal" => {
            let calculator = graph_state::<OneTimerGoalCalculator>(graph, module_name)?;
            Some(serialize_to_json_value(&serde_json::json!({
                "one_timer_goal_max_event_to_goal_seconds": calculator.config().max_event_to_goal_seconds,
            }))?)
        }
        "passing_goal" => {
            let calculator = graph_state::<PassingGoalCalculator>(graph, module_name)?;
            Some(serialize_to_json_value(&serde_json::json!({
                "passing_goal_max_pass_to_goal_seconds": calculator.config().max_pass_to_goal_seconds,
            }))?)
        }
        "air_dribble_goal" => {
            let calculator = graph_state::<AirDribbleGoalCalculator>(graph, module_name)?;
            Some(serialize_to_json_value(&serde_json::json!({
                "air_dribble_goal_max_end_to_goal_seconds": calculator.config().max_end_to_goal_seconds,
            }))?)
        }
        "flip_reset_goal" => {
            let calculator = graph_state::<FlipResetGoalCalculator>(graph, module_name)?;
            Some(serialize_to_json_value(&serde_json::json!({
                "flip_reset_goal_max_event_to_goal_seconds": calculator.config().max_event_to_goal_seconds,
            }))?)
        }
        "bump_goal" => {
            let calculator = graph_state::<BumpGoalCalculator>(graph, module_name)?;
            Some(serialize_to_json_value(&serde_json::json!({
                "bump_goal_max_event_to_goal_seconds": calculator.config().max_event_to_goal_seconds,
            }))?)
        }
        "demo_goal" => {
            let calculator = graph_state::<DemoGoalCalculator>(graph, module_name)?;
            Some(serialize_to_json_value(&serde_json::json!({
                "demo_goal_max_event_to_goal_seconds": calculator.config().max_event_to_goal_seconds,
            }))?)
        }
        "half_volley_goal" => {
            let calculator = graph_state::<HalfVolleyGoalCalculator>(graph, module_name)?;
            Some(serialize_to_json_value(&serde_json::json!({
                "half_volley_goal_max_touch_to_goal_seconds": calculator.config().max_touch_to_goal_seconds,
                "half_volley_goal_min_goal_alignment": calculator.config().min_goal_alignment,
            }))?)
        }
        "half_volley" => {
            let calculator = graph_state::<HalfVolleyCalculator>(graph, module_name)?;
            Some(serialize_to_json_value(&serde_json::json!({
                "half_volley_max_bounce_to_touch_seconds": calculator.config().max_bounce_to_touch_seconds,
                "half_volley_min_ball_speed": calculator.config().min_ball_speed,
            }))?)
        }
        "core"
        | "backboard"
        | "ceiling_shot"
        | "wall_aerial"
        | "wall_aerial_shot"
        | "center"
        | "double_tap"
        | "one_timer"
        | "pass"
        | "fifty_fifty"
        | "possession"
        | "touch"
        | "whiff"
        | "wavedash"
        | "speed_flip"
        | "half_flip"
        | "flick"
        | "musty_flick"
        | "dodge_reset"
        | "ball_carry"
        | "air_dribble"
        | "counter_attack_goal"
        | "boost"
        | "bump"
        | "movement"
        | "powerslide"
        | "demo" => None,
        _ => {
            return SubtrActorError::new_result(SubtrActorErrorVariant::UnknownStatsModuleName(
                module_name.to_owned(),
            ))
        }
    };
    Ok(value)
}
