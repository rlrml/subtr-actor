// Included by StateExportPlugin.cpp; shares the plugin translation unit.
//
// Frame sampling, ported from bakkesmod/subtr-actor/plugin/live_sampling.cpp
// onto the Se* structs (whose prefix is byte-identical to the Sa* shapes),
// plus the state-export superset: controller input, camera state, dodge
// torque, and platform identity. All string pointers handed to the Rust side
// (player names, epic ids) stay alive in per-tick std::string caches for the
// duration of the push_frame call, which copies synchronously.
SeFrame StateExportPlugin::sampleFrame() {
  ServerWrapper server = gameWrapper->GetGameEventAsServer();
  CarWrapper car = gameWrapper->GetLocalCar();

  const float now = server.IsNull() ? lastTime : server.GetSecondsElapsed();
  const float dt = frameNumber == 0 ? 0.0f : std::max(0.0f, now - lastTime);
  lastTime = now;

  samplePlayers(server, car);

  SeFrame frame{};
  frame.frame_number = frameNumber++;
  frame.time = now;
  frame.dt = dt;
  frame.live_play = 0;
  frame.has_live_play = 0;
  frame.ball_has_been_hit =
      server.IsNull() ? 1 : static_cast<uint8_t>(server.GetbBallHasBeenHit() != 0);
  frame.has_ball_has_been_hit = 1;
  frame.players = sampledPlayers.empty() ? nullptr : sampledPlayers.data();
  frame.player_count = sampledPlayers.size();
  if (!server.IsNull()) {
    frame.seconds_remaining = server.GetSecondsRemaining();
    frame.has_seconds_remaining = 1;
    frame.kickoff_countdown_time = server.GetReplicatedGameStateTimeRemaining();
    frame.has_kickoff_countdown_time = 1;
    if (server.GetbPlayReplays() != 0) {
      frame.game_state = GAME_STATE_GOAL_SCORED_REPLAY;
      frame.has_game_state = 1;
    } else if (frame.kickoff_countdown_time > 0) {
      frame.game_state = GAME_STATE_KICKOFF_COUNTDOWN;
      frame.has_game_state = 1;
    }
    sampleTeamScores(server, frame);
    rememberTeamScores(frame);
    const unsigned char scoredOnTeam = server.GetReplicatedScoredOnTeam();
    if (scoredOnTeam == 0 || scoredOnTeam == 1) {
      frame.scored_on_team_is_team_0 = scoredOnTeam == 0 ? 1 : 0;
      frame.has_scored_on_team = 1;
    }
  }

  if (!server.IsNull()) {
    BallWrapper ball = server.GetBall();
    if (!ball.IsNull()) {
      frame.has_ball = 1;
      frame.ball = sampleRigidBody(ball);
      const unsigned char hitTeam = ball.GetHitTeamNum();
      if (hitTeam == 0 || hitTeam == 1) {
        frame.possession_team_is_team_0 = hitTeam == 0 ? 1 : 0;
        frame.has_possession_team = 1;
      }
    }
  }

  attachPendingFrameEvents(frame);
  return frame;
}

void StateExportPlugin::samplePlayers(ServerWrapper server, CarWrapper localCar) {
  sampledPlayers.clear();
  sampledPlayerNames.clear();
  sampledPlayerEpicIds.clear();
  carPlayerIndices.clear();
  priPlayerIndices.clear();

  if (!server.IsNull()) {
    ArrayWrapper<CarWrapper> cars = server.GetCars();
    ArrayWrapper<PriWrapper> pris = server.GetPRIs();
    const int carCount = cars.IsNull() ? 0 : cars.Count();
    const int priCount = pris.IsNull() ? 0 : pris.Count();
    const auto reserveCount = static_cast<size_t>(std::max(0, carCount + priCount));
    sampledPlayers.reserve(reserveCount);
    // Reserving up front keeps the cached name/epic-id strings from moving
    // (short strings live inline in the SSO buffer, so a vector reallocation
    // would invalidate the c_str pointers already stored into frames).
    sampledPlayerNames.reserve(reserveCount);
    sampledPlayerEpicIds.reserve(reserveCount);

    if (!cars.IsNull()) {
      for (int i = 0; i < carCount; i += 1) {
        CarWrapper car = cars.Get(i);
        if (!car.IsNull()) {
          sampledPlayers.push_back(samplePlayer(car, static_cast<uint32_t>(i)));
        }
      }
    }

    if (!pris.IsNull()) {
      for (int i = 0; i < priCount; i += 1) {
        PriWrapper pri = pris.Get(i);
        if (pri.IsNull() || priPlayerIndices.find(pri.memory_address) != priPlayerIndices.end()) {
          continue;
        }
        sampledPlayers.push_back(
            samplePlayer(pri, static_cast<uint32_t>(sampledPlayers.size())));
      }
    }
  }

  if (sampledPlayers.empty() && !localCar.IsNull()) {
    sampledPlayers.reserve(1);
    sampledPlayerNames.reserve(1);
    sampledPlayerEpicIds.reserve(1);
    sampledPlayers.push_back(samplePlayer(localCar, 0));
  }
}

SeRigidBody StateExportPlugin::sampleRigidBody(ActorWrapper actor) {
  SeRigidBody body{};
  if (actor.IsNull()) {
    body.sleeping = 1;
    return body;
  }

  body.location = toSeVec3(actor.GetLocation());
  body.rotation = rotatorToQuat(actor.GetRotation());
  body.linear_velocity = toSeVec3(actor.GetVelocity());
  body.angular_velocity = toSeVec3(actor.GetAngularVelocity());
  body.has_linear_velocity = 1;
  body.has_angular_velocity = 1;
  body.sleeping = 0;
  return body;
}

SePlayerFrame StateExportPlugin::samplePlayer(PriWrapper pri, uint32_t playerIndex) {
  SePlayerFrame player{};
  player.player_index = playerIndex;
  player.is_team_0 = 1;
  populatePlayerFromPri(player, pri, playerIndex);
  return player;
}

void StateExportPlugin::populatePlayerFromPri(
    SePlayerFrame &player,
    PriWrapper pri,
    uint32_t fallbackIndex) {
  if (pri.IsNull()) {
    return;
  }

  const uint32_t playerIndex = stablePlayerIndexForPri(pri, fallbackIndex);
  player.player_index = playerIndex;
  sampledPlayerNames.push_back(pri.GetPlayerName().ToString());
  player.player_name = sampledPlayerNames.back().c_str();
  player.is_team_0 = pri.GetTeamNum() == 0 ? 1 : 0;
  player.has_match_stats = 1;
  player.match_goals = pri.GetMatchGoals();
  player.match_assists = pri.GetMatchAssists();
  player.match_saves = pri.GetMatchSaves();
  player.match_shots = pri.GetMatchShots();
  player.match_score = pri.GetMatchScore();
  sampleCameraState(player, pri);
  sampleRemoteId(player, pri);
  priPlayerIndices[pri.memory_address] = playerIndex;
  recordPlayerStatDeltas(pri, playerIndex, player.is_team_0);
}

SePlayerFrame StateExportPlugin::samplePlayer(CarWrapper car, uint32_t playerIndex) {
  SePlayerFrame player{};
  player.player_index = playerIndex;
  player.is_team_0 = 1;
  if (car.IsNull()) {
    player.has_rigid_body = 0;
    return player;
  }

  PriWrapper pri = car.GetPRI();
  if (!pri.IsNull()) {
    populatePlayerFromPri(player, pri, playerIndex);
    playerIndex = player.player_index;
  } else {
    nextPlayerIndex = std::max(nextPlayerIndex, playerIndex + 1);
  }
  carPlayerIndices[car.memory_address] = playerIndex;
  recordDodgeRefreshFromJumpState(car, playerIndex, player.is_team_0);

  player.has_rigid_body = 1;
  player.rigid_body = sampleRigidBody(car);
  player.jump_active = car.GetbJumped() != 0;
  player.double_jump_active = car.GetbDoubleJumped() != 0;
  player.dodge_active =
      car.GetDodgeComponent().IsNull() ? 0 : car.GetDodgeComponent().GetbActive();
  player.powerslide_active = car.GetbReplicatedHandbrake() != 0;
  const int loadoutBody = car.GetLoadoutBody();
  if (loadoutBody > 0) {
    player.car_body_id = loadoutBody;
    player.has_car_body_id = 1;
  }

  BoostWrapper boost = car.GetBoostComponent();
  if (!boost.IsNull()) {
    const auto previousBoost = lastBoostAmounts.find(playerIndex);
    player.boost_amount = static_cast<float>(boost.GetReplicatedBoostAmount());
    player.last_boost_amount =
        previousBoost == lastBoostAmounts.end() ? player.boost_amount : previousBoost->second;
    player.boost_active = player.boost_amount < player.last_boost_amount ? 1 : 0;
    lastBoostAmounts[playerIndex] = player.boost_amount;
  }

  sampleControllerInput(player, car);
  sampleDodgeState(player, car);
  return player;
}

void StateExportPlugin::sampleControllerInput(SePlayerFrame &player, CarWrapper car) {
  if (car.IsNull()) {
    return;
  }

  const ControllerInput input = car.GetInput();
  player.has_input = 1;
  player.input.throttle = input.Throttle;
  player.input.steer = input.Steer;
  player.input.pitch = input.Pitch;
  player.input.yaw = input.Yaw;
  player.input.roll = input.Roll;
  player.input.dodge_forward = input.DodgeForward;
  player.input.dodge_strafe = input.DodgeStrafe;
  player.input.handbrake = input.Handbrake != 0;
  player.input.jump = input.Jump != 0;
  player.input.activate_boost = input.ActivateBoost != 0;
  player.input.holding_boost = input.HoldingBoost != 0;
}

void StateExportPlugin::sampleCameraState(SePlayerFrame &player, PriWrapper pri) {
  if (pri.IsNull()) {
    return;
  }

  // Replay-style byte camera angles plus the ball-cam flag, all replicated
  // on the PRI (PriWrapper::GetCameraPitch/GetCameraYaw return the raw
  // unsigned char the replay attributes store).
  player.camera.pitch = pri.GetCameraPitch();
  player.camera.has_pitch = 1;
  player.camera.yaw = pri.GetCameraYaw();
  player.camera.has_yaw = 1;
  player.camera.ball_cam_active = pri.GetbUsingSecondaryCamera() != 0 ? 1 : 0;
  player.camera.has_ball_cam = 1;
}

void StateExportPlugin::sampleDodgeState(SePlayerFrame &player, CarWrapper car) {
  if (car.IsNull()) {
    return;
  }

  DodgeComponentWrapper dodge = car.GetDodgeComponent();
  if (dodge.IsNull()) {
    return;
  }

  // GetDodgeTorque mirrors the replay DodgeTorque attribute (the
  // car-relative flip axis of the most recent dodge). The SDK exposes no
  // applied-impulse vector getter (GetDodgeImpulse2 computes a hypothetical
  // from a direction argument), so dodge_impulse stays has_ = 0.
  player.dodge_torque = toSeVec3(dodge.GetDodgeTorque());
  player.has_dodge_torque = 1;
}

void StateExportPlugin::sampleRemoteId(SePlayerFrame &player, PriWrapper pri) {
  if (pri.IsNull() || pri.GetbBot() != 0) {
    return;
  }

  UniqueIDWrapper uniqueId = pri.GetUniqueIdWrapper();
  player.remote_id.platform = seRemoteIdPlatform(uniqueId.GetPlatform());
  player.remote_id.online_id = uniqueId.GetUID();
  player.remote_id.splitscreen_index = uniqueId.GetSplitscreenID();
  if (player.remote_id.platform == SE_REMOTE_ID_PLATFORM_EPIC) {
    // The epic-id string cache outlives the push_frame call this frame is
    // handed to (the Rust side copies synchronously), like player_name.
    sampledPlayerEpicIds.push_back(uniqueId.GetEpicAccountID());
    player.remote_id.epic_id = sampledPlayerEpicIds.back().c_str();
  }
}

uint32_t StateExportPlugin::stablePlayerIndexForPri(PriWrapper pri, uint32_t fallbackIndex) {
  if (pri.IsNull()) {
    return fallbackIndex;
  }

  const bool isBot = pri.GetbBot() != 0;
  const std::string uniqueId = isBot ? "" : pri.GetUniqueIdWrapper().GetIdString();
  if (!uniqueId.empty()) {
    const auto existing = uniqueIdPlayerIndices.find(uniqueId);
    if (existing != uniqueIdPlayerIndices.end()) {
      return existing->second;
    }

    const uint32_t playerIndex = nextPlayerIndex++;
    uniqueIdPlayerIndices[uniqueId] = playerIndex;
    return playerIndex;
  }

  if (isBot) {
    const std::string name = pri.GetPlayerName().ToString();
    if (!name.empty()) {
      const std::string botKey = std::format("{}:{}", pri.GetTeamNum(), name);
      const auto existingBot = stableBotPlayerIndices.find(botKey);
      if (existingBot != stableBotPlayerIndices.end()) {
        stablePriPlayerIndices[pri.memory_address] = existingBot->second;
        return existingBot->second;
      }

      // Preserve an index already assigned while the bot's replicated name
      // was temporarily empty, then bind the stable team/name key to it.
      const auto existingPri = stablePriPlayerIndices.find(pri.memory_address);
      const uint32_t playerIndex = existingPri == stablePriPlayerIndices.end()
                                       ? nextPlayerIndex++
                                       : existingPri->second;
      stableBotPlayerIndices[botKey] = playerIndex;
      stablePriPlayerIndices[pri.memory_address] = playerIndex;
      return playerIndex;
    }
  }

  const auto existing = stablePriPlayerIndices.find(pri.memory_address);
  if (existing != stablePriPlayerIndices.end()) {
    return existing->second;
  }

  const uint32_t playerIndex = nextPlayerIndex++;
  stablePriPlayerIndices[pri.memory_address] = playerIndex;
  return playerIndex;
}

void StateExportPlugin::clearPendingFrameEvents() {
  pendingTouches.clear();
  pendingDodgeRefreshes.clear();
  pendingBoostPadEvents.clear();
  pendingGoals.clear();
  pendingPlayerStatEvents.clear();
  pendingDemolishes.clear();
}

void StateExportPlugin::commitPendingFrameEvents() {
  if (!pendingGoals.empty()) {
    lastGoalEvent = pendingGoals.back();
  }
}

void StateExportPlugin::attachPendingFrameEvents(SeFrame &frame) {
  frame.touches = pendingTouches.empty() ? nullptr : pendingTouches.data();
  frame.touch_count = pendingTouches.size();
  frame.dodge_refreshes = pendingDodgeRefreshes.empty() ? nullptr : pendingDodgeRefreshes.data();
  frame.dodge_refresh_count = pendingDodgeRefreshes.size();
  frame.boost_pad_events =
      pendingBoostPadEvents.empty() ? nullptr : pendingBoostPadEvents.data();
  frame.boost_pad_event_count = pendingBoostPadEvents.size();
  frame.goals = pendingGoals.empty() ? nullptr : pendingGoals.data();
  frame.goal_count = pendingGoals.size();
  frame.player_stat_events =
      pendingPlayerStatEvents.empty() ? nullptr : pendingPlayerStatEvents.data();
  frame.player_stat_event_count = pendingPlayerStatEvents.size();
  frame.demolishes = pendingDemolishes.empty() ? nullptr : pendingDemolishes.data();
  frame.demolish_count = pendingDemolishes.size();
}

SeEventTiming StateExportPlugin::currentEventTiming() {
  SeEventTiming timing{};
  timing.frame_number = frameNumber;
  ServerWrapper server = gameWrapper->GetGameEventAsServer();
  timing.time = server.IsNull() ? lastTime : server.GetSecondsElapsed();
  if (!server.IsNull()) {
    timing.seconds_remaining = server.GetSecondsRemaining();
    timing.has_seconds_remaining = 1;
  }
  timing.has_timing = 1;
  return timing;
}

void StateExportPlugin::sampleTeamScores(ServerWrapper server, SeFrame &frame) {
  if (server.IsNull()) {
    return;
  }

  ArrayWrapper<TeamWrapper> teams = server.GetTeams();
  if (teams.IsNull()) {
    return;
  }

  const int teamCount = teams.Count();
  for (int i = 0; i < teamCount; i += 1) {
    TeamWrapper team = teams.Get(i);
    if (team.IsNull()) {
      continue;
    }
    const int teamIndex = team.GetTeamIndex();
    if (teamIndex == 0) {
      frame.team_zero_score = team.GetScore();
      frame.has_team_zero_score = 1;
    } else if (teamIndex == 1) {
      frame.team_one_score = team.GetScore();
      frame.has_team_one_score = 1;
    }
  }
}

void StateExportPlugin::sampleTeamScores(ServerWrapper server, SeGoalEvent &goal) {
  if (server.IsNull()) {
    return;
  }

  ArrayWrapper<TeamWrapper> teams = server.GetTeams();
  if (teams.IsNull()) {
    return;
  }

  const int teamCount = teams.Count();
  for (int i = 0; i < teamCount; i += 1) {
    TeamWrapper team = teams.Get(i);
    if (team.IsNull()) {
      continue;
    }
    const int teamIndex = team.GetTeamIndex();
    if (teamIndex == 0) {
      goal.team_zero_score = team.GetScore();
      goal.has_team_zero_score = 1;
    } else if (teamIndex == 1) {
      goal.team_one_score = team.GetScore();
      goal.has_team_one_score = 1;
    }
  }
}

std::optional<bool> StateExportPlugin::scoringTeamFromScoreDelta(
    const SeGoalEvent &goal) const {
  if (!lastTeamScores || goal.has_team_zero_score == 0 || goal.has_team_one_score == 0) {
    return std::nullopt;
  }

  const bool teamZeroScored = goal.team_zero_score > lastTeamScores->first;
  const bool teamOneScored = goal.team_one_score > lastTeamScores->second;
  if (teamZeroScored == teamOneScored) {
    return std::nullopt;
  }
  return teamZeroScored;
}

void StateExportPlugin::rememberTeamScores(const SeFrame &frame) {
  if (frame.has_team_zero_score != 0 && frame.has_team_one_score != 0) {
    lastTeamScores = std::make_pair(frame.team_zero_score, frame.team_one_score);
  }
}

void StateExportPlugin::rememberTeamScores(const SeGoalEvent &goal) {
  if (goal.has_team_zero_score != 0 && goal.has_team_one_score != 0) {
    lastTeamScores = std::make_pair(goal.team_zero_score, goal.team_one_score);
  }
}

bool StateExportPlugin::goalEventIsDuplicate(const SeGoalEvent &goal) const {
  const SeGoalEvent *previous = nullptr;
  if (!pendingGoals.empty()) {
    previous = &pendingGoals.back();
  } else if (lastGoalEvent) {
    previous = &*lastGoalEvent;
  }
  if (!previous) {
    return false;
  }

  if (goal.has_team_zero_score != 0 && goal.has_team_one_score != 0 &&
      previous->has_team_zero_score != 0 && previous->has_team_one_score != 0) {
    return goal.team_zero_score == previous->team_zero_score &&
           goal.team_one_score == previous->team_one_score;
  }

  return goal.scoring_team_is_team_0 == previous->scoring_team_is_team_0 &&
         std::abs(goal.timing.time - previous->timing.time) <=
             GOAL_EVENT_DEDUPE_WINDOW_SECONDS;
}
