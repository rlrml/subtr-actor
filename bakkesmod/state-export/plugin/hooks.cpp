// Included by StateExportPlugin.cpp; shares the plugin translation unit.
//
// Game-event hooks recording into the pending Se event vectors that the next
// sampled frame carries, ported from the subtr-actor live plugin, plus the
// match-lifecycle context calls (set_match_context / notify_match_end).
void StateExportPlugin::hookGameEvents() {
  gameWrapper->HookEventWithCallerPost<BallWrapper>(
      BALL_TOUCH_EVENT,
      [this](BallWrapper, void *params, std::string) {
        if (!exportEnabled() || !engine) {
          return;
        }
        if (!params) {
          return;
        }
        const auto *touchParams = static_cast<const BallTouchParams *>(params);
        recordTouch(CarWrapper(touchParams->hitCar));
      });

  gameWrapper->HookEventWithCallerPost<ActorWrapper>(
      BOOST_PICKED_UP_EVENT,
      [this](ActorWrapper pickup, void *, std::string) {
        if (!exportEnabled() || !engine) {
          return;
        }
        recordBoostPadEvent(pickup, SeBoostPadEventKindPickedUp);
      });
  gameWrapper->HookEventWithCallerPost<ActorWrapper>(
      BOOST_SPAWNED_EVENT,
      [this](ActorWrapper pickup, void *, std::string) {
        if (!exportEnabled() || !engine) {
          return;
        }
        recordBoostPadEvent(pickup, SeBoostPadEventKindAvailable);
      });

  gameWrapper->HookEventWithCallerPost<ServerWrapper>(
      GOAL_SCORED_EVENT,
      [this](ServerWrapper server, void *params, std::string) {
        if (!exportEnabled() || !engine) {
          return;
        }
        auto goal = GoalWrapper(0);
        int scoreIndex = -1;
        int assistIndex = -1;
        if (params) {
          const auto *goalParams = static_cast<const GoalScoredParams *>(params);
          goal = GoalWrapper(goalParams->goal);
          scoreIndex = goalParams->scoreIndex;
          assistIndex = goalParams->assistIndex;
        }
        recordGoal(server, goal, scoreIndex, assistIndex);
      });

  gameWrapper->HookEventWithCallerPost<CarWrapper>(
      CAR_DEMOLISHED_EVENT,
      [this](CarWrapper victim, void *params, std::string) {
        if (!exportEnabled() || !engine) {
          return;
        }
        if (!params) {
          return;
        }
        const auto *demolishParams = static_cast<const CarDemolishedParams *>(params);
        recordDemolish(victim, ActorWrapper(demolishParams->demolisher));
      });

  gameWrapper->HookEventPost(
      MATCH_ENDED_EVENT,
      [this](std::string) {
        if (!exportEnabled() || !engine) {
          return;
        }
        // The tick's leave-game edge also fires notify_match_end, but the
        // hook lands at the true match end (before the podium/exit flow);
        // the Rust side treats a repeated call as a no-op.
        notifyMatchEndAndReset("match ended");
        resetLiveState();
        awaitingNextMatch = true;
      });
}

void StateExportPlugin::unhookGameEvents() {
  gameWrapper->UnhookEventPost(BALL_TOUCH_EVENT);
  gameWrapper->UnhookEventPost(BOOST_PICKED_UP_EVENT);
  gameWrapper->UnhookEventPost(BOOST_SPAWNED_EVENT);
  gameWrapper->UnhookEventPost(GOAL_SCORED_EVENT);
  gameWrapper->UnhookEventPost(CAR_DEMOLISHED_EVENT);
  gameWrapper->UnhookEventPost(MATCH_ENDED_EVENT);
}

void StateExportPlugin::recordTouch(CarWrapper car) {
  if (car.IsNull()) {
    return;
  }

  SeTouchEvent event{};
  event.timing = currentEventTiming();
  if (auto playerIndex = playerIndexForCar(car)) {
    event.player_index = *playerIndex;
    event.has_player = 1;
  }

  bool hasHitTeam = false;
  ServerWrapper server = gameWrapper->GetGameEventAsServer();
  if (!server.IsNull()) {
    BallWrapper ball = server.GetBall();
    if (!ball.IsNull()) {
      const unsigned char hitTeam = ball.GetHitTeamNum();
      if (hitTeam == 0 || hitTeam == 1) {
        event.is_team_0 = hitTeam == 0 ? 1 : 0;
        hasHitTeam = true;
      }
    }
  }
  if (!hasHitTeam) {
    PriWrapper pri = car.GetPRI();
    event.is_team_0 = pri.IsNull() || pri.GetTeamNum() == 0 ? 1 : 0;
  }
  if (event.has_player != 0) {
    lastTouch = TouchAttribution{
        event.player_index,
        event.is_team_0,
    };
    lastBallTouchFrames[event.player_index] = frameNumber;
  }
  pendingTouches.push_back(event);
}

void StateExportPlugin::recordDodgeRefreshFromJumpState(
    CarWrapper car,
    uint32_t playerIndex,
    uint8_t isTeam0) {
  if (car.IsNull()) {
    return;
  }

  const bool canJump = car.GetbCanJump() != 0;
  const bool onGround = car.GetbOnGround() != 0 || car.IsOnGround();
  const auto previousCanJump = lastCanJump.find(playerIndex);
  const bool canJumpWasKnown = previousCanJump != lastCanJump.end();
  const bool regainedJump = canJumpWasKnown && !previousCanJump->second && canJump;
  lastCanJump[playerIndex] = canJump;

  const auto touchFrame = lastBallTouchFrames.find(playerIndex);
  const bool recentlyTouchedBall =
      touchFrame != lastBallTouchFrames.end() &&
      frameNumber >= touchFrame->second &&
      frameNumber - touchFrame->second <= DODGE_REFRESH_TOUCH_FRAME_WINDOW;
  if (!regainedJump || onGround || !recentlyTouchedBall) {
    return;
  }

  SeDodgeRefreshedEvent event{};
  event.timing = currentEventTiming();
  event.player_index = playerIndex;
  event.is_team_0 = isTeam0;
  event.counter_value = ++dodgeRefreshCounters[playerIndex];
  pendingDodgeRefreshes.push_back(event);
}

void StateExportPlugin::recordBoostPadEvent(ActorWrapper pickup, SeBoostPadEventKind kind) {
  if (pickup.IsNull()) {
    return;
  }

  SeBoostPadEvent event{};
  event.timing = currentEventTiming();
  event.pad_id = boostPadId(pickup);
  event.kind = kind;
  if (kind == SeBoostPadEventKindPickedUp) {
    event.sequence = ++boostPadSequences[pickup.memory_address];
    if (auto playerIndex = playerIndexForNearestCar(pickup, BOOST_PICKUP_ATTRIBUTION_RADIUS)) {
      event.player_index = *playerIndex;
      event.has_player = 1;
    }
  }
  pendingBoostPadEvents.push_back(event);
}

void StateExportPlugin::recordGoal(
    ServerWrapper server,
    GoalWrapper goal,
    int scoreIndex,
    int assistIndex) {
  SeGoalEvent event{};
  event.timing = currentEventTiming();
  sampleTeamScores(server, event);
  if (auto scoringTeam = scoringTeamFromScoreDelta(event)) {
    event.scoring_team_is_team_0 = *scoringTeam ? 1 : 0;
  } else if (!goal.IsNull()) {
    event.scoring_team_is_team_0 = goal.GetTeamNum() == 0 ? 0 : 1;
  } else if (!server.IsNull()) {
    const unsigned char scoredOnTeam = server.GetReplicatedScoredOnTeam();
    if (scoredOnTeam == 0 || scoredOnTeam == 1) {
      event.scoring_team_is_team_0 = scoredOnTeam == 0 ? 0 : 1;
    }
  }
  if (auto scorerIndex = playerIndexForScoreIndex(server, scoreIndex)) {
    event.player_index = *scorerIndex;
    event.has_player = 1;
  } else if (lastTouch && lastTouch->is_team_0 == event.scoring_team_is_team_0) {
    event.player_index = lastTouch->player_index;
    event.has_player = 1;
  }
  if (goalEventIsDuplicate(event)) {
    return;
  }
  rememberTeamScores(event);
  pendingGoals.push_back(event);

  recordExplicitPlayerStat(priForScoreIndex(server, assistIndex), SePlayerStatEventKindAssist);
}

void StateExportPlugin::recordDemolish(CarWrapper victim, ActorWrapper demolisher) {
  if (victim.IsNull() || demolisher.IsNull()) {
    return;
  }

  CarWrapper attacker(demolisher.memory_address);
  const auto victimIndex = playerIndexForCar(victim);
  const auto attackerIndex = playerIndexForCar(attacker);
  if (!victimIndex || !attackerIndex) {
    return;
  }

  SeDemolishEvent event{};
  event.timing = currentEventTiming();
  event.attacker_index = *attackerIndex;
  event.victim_index = *victimIndex;
  event.attacker_velocity = toSeVec3(attacker.GetVelocity());
  event.victim_velocity = toSeVec3(victim.GetVelocity());
  event.victim_location = toSeVec3(victim.GetLocation());
  event.active_duration_seconds = DEMO_ACTIVE_DURATION_SECONDS;
  pendingDemolishes.push_back(event);
}

void StateExportPlugin::recordPlayerStatDeltas(
    PriWrapper pri,
    uint32_t playerIndex,
    uint8_t isTeam0) {
  if (pri.IsNull()) {
    return;
  }

  const PlayerStatSnapshot current{
      pri.GetMatchShots(),
      pri.GetMatchSaves(),
      pri.GetMatchAssists(),
      pri.GetMatchDemolishes(),
  };
  auto [it, inserted] = lastPlayerStats.emplace(pri.memory_address, current);
  if (inserted) {
    return;
  }

  auto suppressions = suppressedPlayerStatDeltas.find(pri.memory_address);
  auto consumeSuppressed = [&](int count, int PlayerStatSnapshot::*field) {
    if (count <= 0 || suppressions == suppressedPlayerStatDeltas.end()) {
      return count;
    }

    int &suppressed = suppressions->second.*field;
    const int consumed = std::min(count, suppressed);
    suppressed -= consumed;
    return count - consumed;
  };
  auto pushStats = [&](int previous, int next, SePlayerStatEventKind kind, int PlayerStatSnapshot::*field) {
    const int count = consumeSuppressed(next - previous, field);
    for (int i = 0; i < count; i += 1) {
      SePlayerStatEvent event{};
      event.timing = currentEventTiming();
      event.player_index = playerIndex;
      event.is_team_0 = isTeam0;
      event.kind = kind;
      if (kind == SePlayerStatEventKindShot) {
        ServerWrapper server = gameWrapper->GetGameEventAsServer();
        if (!server.IsNull()) {
          BallWrapper ball = server.GetBall();
          if (!ball.IsNull()) {
            event.has_shot_ball = 1;
            event.shot_ball = sampleRigidBody(ball);
          }
        }

        CarWrapper car = pri.GetCar();
        if (!car.IsNull()) {
          event.has_shot_player = 1;
          event.shot_player = sampleRigidBody(car);
        }
      }
      pendingPlayerStatEvents.push_back(event);
    }
  };
  pushStats(it->second.shots, current.shots, SePlayerStatEventKindShot, &PlayerStatSnapshot::shots);
  pushStats(it->second.saves, current.saves, SePlayerStatEventKindSave, &PlayerStatSnapshot::saves);
  pushStats(
      it->second.assists,
      current.assists,
      SePlayerStatEventKindAssist,
      &PlayerStatSnapshot::assists);
  it->second = current;
  if (suppressions != suppressedPlayerStatDeltas.end() &&
      suppressions->second.shots == 0 &&
      suppressions->second.saves == 0 &&
      suppressions->second.assists == 0 &&
      suppressions->second.demolishes == 0) {
    suppressedPlayerStatDeltas.erase(suppressions);
  }
}

void StateExportPlugin::recordExplicitPlayerStat(PriWrapper pri, SePlayerStatEventKind kind) {
  if (pri.IsNull()) {
    return;
  }

  const auto playerIndex = playerIndexForPri(pri);
  if (!playerIndex) {
    return;
  }

  SePlayerStatEvent event{};
  event.timing = currentEventTiming();
  event.player_index = *playerIndex;
  event.is_team_0 = pri.GetTeamNum() == 0 ? 1 : 0;
  event.kind = kind;
  pendingPlayerStatEvents.push_back(event);

  if (lastPlayerStats.find(pri.memory_address) == lastPlayerStats.end()) {
    return;
  }

  PlayerStatSnapshot &suppressed = suppressedPlayerStatDeltas[pri.memory_address];
  if (kind == SePlayerStatEventKindShot) {
    suppressed.shots += 1;
  } else if (kind == SePlayerStatEventKindSave) {
    suppressed.saves += 1;
  } else if (kind == SePlayerStatEventKindAssist) {
    suppressed.assists += 1;
  }
}

std::optional<uint32_t> StateExportPlugin::playerIndexForCar(CarWrapper car) {
  if (car.IsNull()) {
    return std::nullopt;
  }

  const auto carMatch = carPlayerIndices.find(car.memory_address);
  if (carMatch != carPlayerIndices.end()) {
    return carMatch->second;
  }

  PriWrapper pri = car.GetPRI();
  if (!pri.IsNull()) {
    return playerIndexForPri(pri);
  }
  return std::nullopt;
}

std::optional<uint32_t> StateExportPlugin::playerIndexForPri(PriWrapper pri) {
  if (pri.IsNull()) {
    return std::nullopt;
  }

  const auto priMatch = priPlayerIndices.find(pri.memory_address);
  if (priMatch != priPlayerIndices.end()) {
    return priMatch->second;
  }

  const uint32_t playerIndex = stablePlayerIndexForPri(pri, nextPlayerIndex);
  priPlayerIndices[pri.memory_address] = playerIndex;
  return playerIndex;
}

PriWrapper StateExportPlugin::priForScoreIndex(ServerWrapper server, int scoreIndex) {
  if (server.IsNull() || scoreIndex < 0) {
    return PriWrapper(0);
  }

  ArrayWrapper<PriWrapper> pris = server.GetPRIs();
  if (pris.IsNull() || scoreIndex >= pris.Count()) {
    return PriWrapper(0);
  }

  return pris.Get(scoreIndex);
}

std::optional<uint32_t> StateExportPlugin::playerIndexForScoreIndex(
    ServerWrapper server,
    int scoreIndex) {
  return playerIndexForPri(priForScoreIndex(server, scoreIndex));
}

std::optional<uint32_t> StateExportPlugin::playerIndexForNearestCar(
    ActorWrapper actor,
    float maxDistance) {
  if (actor.IsNull()) {
    return std::nullopt;
  }

  ServerWrapper server = gameWrapper->GetGameEventAsServer();
  if (server.IsNull()) {
    return std::nullopt;
  }

  ArrayWrapper<CarWrapper> cars = server.GetCars();
  if (cars.IsNull()) {
    return std::nullopt;
  }

  const Vector actorLocation = actor.GetLocation();
  const int carCount = cars.Count();
  std::optional<uint32_t> bestIndex;
  float bestDistance = maxDistance;
  for (int i = 0; i < carCount; i += 1) {
    CarWrapper car = cars.Get(i);
    if (car.IsNull()) {
      continue;
    }

    const float distance = (car.GetLocation() - actorLocation).magnitude();
    if (distance <= bestDistance) {
      if (auto playerIndex = playerIndexForCar(car)) {
        bestDistance = distance;
        bestIndex = *playerIndex;
      }
    }
  }

  return bestIndex;
}

uint32_t StateExportPlugin::boostPadId(ActorWrapper pickup) {
  if (!pickup.IsNull()) {
    if (auto standardPadId = nearestStandardBoostPadId(pickup.GetLocation())) {
      return *standardPadId;
    }
  }

  const uintptr_t pickupAddress = pickup.memory_address;
  const auto existing = boostPadIds.find(pickupAddress);
  if (existing != boostPadIds.end()) {
    return existing->second;
  }

  const uint32_t id = NON_STANDARD_BOOST_PAD_ID_START + nextBoostPadId++;
  boostPadIds[pickupAddress] = id;
  return id;
}

void StateExportPlugin::pushMatchContext(ServerWrapper server) {
  if (!engine || !setMatchContext) {
    return;
  }

  pushedMatchGuid = server.IsNull() ? std::string{} : server.GetMatchGUID();
  pushedMapName = gameWrapper ? gameWrapper->GetCurrentMap() : std::string{};
  pushedHasPlaylistId = false;
  pushedPlaylistId = 0;
  if (!server.IsNull()) {
    GameSettingPlaylistWrapper playlist = server.GetPlaylist();
    if (!playlist.IsNull()) {
      pushedPlaylistId = playlist.GetPlaylistId();
      pushedHasPlaylistId = true;
    }
  }

  SeMatchContext context{};
  context.match_guid = pushedMatchGuid.empty() ? nullptr : pushedMatchGuid.c_str();
  context.map_name = pushedMapName.empty() ? nullptr : pushedMapName.c_str();
  context.playlist_id = pushedPlaylistId;
  context.has_playlist_id = pushedHasPlaylistId ? 1 : 0;
  setMatchContext(engine, &context);
  matchContextPushed = true;
}

void StateExportPlugin::maybeRefreshMatchContext() {
  if (!matchContextPushed || !pushedMatchGuid.empty()) {
    return;
  }

  // The match GUID often replicates a moment after the map loads; keep
  // retrying (a string fetch per tick, only while it is still empty) until
  // it appears, then leave the context alone for the rest of the match.
  ServerWrapper server = gameWrapper->GetGameEventAsServer();
  if (server.IsNull() || server.GetMatchGUID().empty()) {
    return;
  }
  pushMatchContext(server);
}

void StateExportPlugin::notifyMatchEndAndReset(const char *reason) {
  if (engine && notifyMatchEnd && matchContextPushed) {
    cvarManager->log(std::format("state-export: match end ({})", reason));
    notifyMatchEnd(engine);
  }
  matchContextPushed = false;
  pushedMatchGuid.clear();
  pushedMapName.clear();
  pushedPlaylistId = 0;
  pushedHasPlaylistId = false;
}
