#include "SubtrActorPlugin.h"

#include <algorithm>
#include <array>
#include <cmath>
#include <cstdint>

namespace {

constexpr float PI = 3.14159265358979323846f;
constexpr float UNREAL_ROTATOR_TO_RADIANS = (2.0f * PI) / 65536.0f;
constexpr float BOOST_PICKUP_ATTRIBUTION_RADIUS = 450.0f;
constexpr float STANDARD_BOOST_PAD_MATCH_RADIUS = 900.0f;
constexpr float DEMO_ACTIVE_DURATION_SECONDS = 3.0f;
constexpr float GOAL_EVENT_DEDUPE_WINDOW_SECONDS = 3.0f;
constexpr uint32_t NON_STANDARD_BOOST_PAD_ID_START = 1000;
constexpr uint64_t DODGE_REFRESH_TOUCH_FRAME_WINDOW = 2;
constexpr int GAME_STATE_KICKOFF_COUNTDOWN = 55;
constexpr int GAME_STATE_GOAL_SCORED_REPLAY = 86;

struct StandardBoostPad {
  uint32_t id;
  Vector location;
};

const std::array<StandardBoostPad, 34> STANDARD_BOOST_PADS{{
    {1, {0.0f, -4240.0f, 70.0f}},
    {2, {0.0f, 4240.0f, 70.0f}},
    {3, {-1792.0f, -4184.0f, 70.0f}},
    {4, {1792.0f, -4184.0f, 70.0f}},
    {5, {-1792.0f, 4184.0f, 70.0f}},
    {6, {1792.0f, 4184.0f, 70.0f}},
    {7, {-3072.0f, -4096.0f, 73.0f}},
    {8, {3072.0f, -4096.0f, 73.0f}},
    {9, {-3072.0f, 4096.0f, 73.0f}},
    {10, {3072.0f, 4096.0f, 73.0f}},
    {11, {-940.0f, -3308.0f, 70.0f}},
    {12, {940.0f, -3308.0f, 70.0f}},
    {13, {-940.0f, 3308.0f, 70.0f}},
    {14, {940.0f, 3308.0f, 70.0f}},
    {15, {0.0f, -2816.0f, 70.0f}},
    {16, {0.0f, 2816.0f, 70.0f}},
    {17, {-3584.0f, -2484.0f, 70.0f}},
    {18, {3584.0f, -2484.0f, 70.0f}},
    {19, {-3584.0f, 2484.0f, 70.0f}},
    {20, {3584.0f, 2484.0f, 70.0f}},
    {21, {-1788.0f, -2300.0f, 70.0f}},
    {22, {1788.0f, -2300.0f, 70.0f}},
    {23, {-1788.0f, 2300.0f, 70.0f}},
    {24, {1788.0f, 2300.0f, 70.0f}},
    {25, {-2048.0f, -1036.0f, 70.0f}},
    {26, {2048.0f, -1036.0f, 70.0f}},
    {27, {-2048.0f, 1036.0f, 70.0f}},
    {28, {2048.0f, 1036.0f, 70.0f}},
    {29, {0.0f, -1024.0f, 70.0f}},
    {30, {0.0f, 1024.0f, 70.0f}},
    {31, {-3584.0f, 0.0f, 73.0f}},
    {32, {3584.0f, 0.0f, 73.0f}},
    {33, {-1024.0f, 0.0f, 70.0f}},
    {34, {1024.0f, 0.0f, 70.0f}},
}};

std::optional<uint32_t> nearestStandardBoostPadId(Vector location) {
  std::optional<uint32_t> bestId;
  float bestDistance = STANDARD_BOOST_PAD_MATCH_RADIUS;
  for (const auto &pad : STANDARD_BOOST_PADS) {
    const float distance = (location - pad.location).magnitude();
    if (distance <= bestDistance) {
      bestDistance = distance;
      bestId = pad.id;
    }
  }
  return bestId;
}

SaVec3 toSaVec3(Vector value) {
  return SaVec3{value.X, value.Y, value.Z};
}

SaQuat rotatorToQuat(Rotator rotation) {
  const float pitch = rotation.Pitch * UNREAL_ROTATOR_TO_RADIANS;
  const float yaw = rotation.Yaw * UNREAL_ROTATOR_TO_RADIANS;
  const float roll = rotation.Roll * UNREAL_ROTATOR_TO_RADIANS;

  const float cy = std::cos(yaw * 0.5f);
  const float sy = std::sin(yaw * 0.5f);
  const float cp = std::cos(pitch * 0.5f);
  const float sp = std::sin(pitch * 0.5f);
  const float cr = std::cos(roll * 0.5f);
  const float sr = std::sin(roll * 0.5f);

  return SaQuat{
      sr * cp * cy - cr * sp * sy,
      cr * sp * cy + sr * cp * sy,
      cr * cp * sy - sr * sp * cy,
      cr * cp * cy + sr * sp * sy,
  };
}

} // namespace

SaLiveFrame SubtrActorPlugin::sampleFrame() {
  ServerWrapper server = gameWrapper->GetGameEventAsServer();
  CarWrapper car = gameWrapper->GetLocalCar();

  const float now = server.IsNull() ? lastTime : server.GetSecondsElapsed();
  const float dt = frameNumber == 0 ? 0.0f : std::max(0.0f, now - lastTime);
  lastTime = now;

  samplePlayers(server, car);

  SaLiveFrame frame{};
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

void SubtrActorPlugin::samplePlayers(ServerWrapper server, CarWrapper localCar) {
  sampledPlayers.clear();
  sampledPlayerNames.clear();
  carPlayerIndices.clear();
  priPlayerIndices.clear();

  if (!server.IsNull()) {
    ArrayWrapper<CarWrapper> cars = server.GetCars();
    ArrayWrapper<PriWrapper> pris = server.GetPRIs();
    const int carCount = cars.IsNull() ? 0 : cars.Count();
    const int priCount = pris.IsNull() ? 0 : pris.Count();
    const auto reserveCount = static_cast<size_t>(std::max(0, carCount + priCount));
    sampledPlayers.reserve(reserveCount);
    sampledPlayerNames.reserve(reserveCount);

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
    sampledPlayers.push_back(samplePlayer(localCar, 0));
  }
}

SaRigidBody SubtrActorPlugin::sampleRigidBody(ActorWrapper actor) {
  SaRigidBody body{};
  if (actor.IsNull()) {
    body.sleeping = 1;
    return body;
  }

  body.location = toSaVec3(actor.GetLocation());
  body.rotation = rotatorToQuat(actor.GetRotation());
  body.linear_velocity = toSaVec3(actor.GetVelocity());
  body.angular_velocity = toSaVec3(actor.GetAngularVelocity());
  body.has_linear_velocity = 1;
  body.has_angular_velocity = 1;
  body.sleeping = 0;
  return body;
}

SaPlayerFrame SubtrActorPlugin::samplePlayer(PriWrapper pri, uint32_t playerIndex) {
  SaPlayerFrame player{};
  player.player_index = playerIndex;
  player.is_team_0 = 1;
  populatePlayerFromPri(player, pri, playerIndex);
  return player;
}

void SubtrActorPlugin::populatePlayerFromPri(
    SaPlayerFrame &player,
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
  priPlayerIndices[pri.memory_address] = playerIndex;
  recordPlayerStatDeltas(pri, playerIndex, player.is_team_0);
}

SaPlayerFrame SubtrActorPlugin::samplePlayer(CarWrapper car, uint32_t playerIndex) {
  SaPlayerFrame player{};
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

  BoostWrapper boost = car.GetBoostComponent();
  if (!boost.IsNull()) {
    const auto previousBoost = lastBoostAmounts.find(playerIndex);
    player.boost_amount = static_cast<float>(boost.GetReplicatedBoostAmount());
    player.last_boost_amount =
        previousBoost == lastBoostAmounts.end() ? player.boost_amount : previousBoost->second;
    player.boost_active = player.boost_amount < player.last_boost_amount ? 1 : 0;
    lastBoostAmounts[playerIndex] = player.boost_amount;
  }

  return player;
}

void SubtrActorPlugin::resetLiveState() {
  frameNumber = 0;
  inputTickNumber = 0;
  lastTime = 0.0f;
  lastProcessedGameTime.reset();
  resetProfileTiming();
  sampledPlayers.clear();
  sampledPlayerNames.clear();
  clearPendingFrameEvents();
  lastBoostAmounts.clear();
  carPlayerIndices.clear();
  priPlayerIndices.clear();
  uniqueIdPlayerIndices.clear();
  stablePriPlayerIndices.clear();
  lastPlayerStats.clear();
  suppressedPlayerStatDeltas.clear();
  lastDoubleJumped.clear();
  lastCanJump.clear();
  lastBallTouchFrames.clear();
  dodgeRefreshCounters.clear();
  boostPadIds.clear();
  boostPadSequences.clear();
  lastTeamScores.reset();
  lastGoalEvent.reset();
  lastTouch.reset();
  nextPlayerIndex = 0;
  nextBoostPadId = 1;
  messages.clear();
}

void SubtrActorPlugin::clearPendingFrameEvents() {
  pendingTouches.clear();
  pendingDodgeRefreshes.clear();
  pendingBoostPadEvents.clear();
  pendingGoals.clear();
  pendingPlayerStatEvents.clear();
  pendingDemolishes.clear();
}

void SubtrActorPlugin::commitPendingFrameEvents() {
  if (!pendingGoals.empty()) {
    lastGoalEvent = pendingGoals.back();
  }
}

void SubtrActorPlugin::attachPendingFrameEvents(SaLiveFrame &frame) {
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

SaEventTiming SubtrActorPlugin::currentEventTiming() {
  SaEventTiming timing{};
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

void SubtrActorPlugin::recordTouch(CarWrapper car) {
  if (car.IsNull()) {
    return;
  }

  SaTouchEvent event{};
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
      event.closest_approach_distance = (ball.GetLocation() - car.GetLocation()).magnitude();
      event.has_closest_approach_distance = 1;
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

void SubtrActorPlugin::recordDodgeRefreshFromJumpState(
    CarWrapper car,
    uint32_t playerIndex,
    uint8_t isTeam0) {
  if (car.IsNull()) {
    return;
  }

  const bool canJump = car.GetbCanJump() != 0;
  const bool onGround = car.GetbOnGround() != 0 || car.IsOnGround();
  lastDoubleJumped[playerIndex] = car.GetbDoubleJumped() != 0;
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

  SaDodgeRefreshedEvent event{};
  event.timing = currentEventTiming();
  event.player_index = playerIndex;
  event.is_team_0 = isTeam0;
  event.counter_value = ++dodgeRefreshCounters[playerIndex];
  pendingDodgeRefreshes.push_back(event);
}

void SubtrActorPlugin::recordBoostPadEvent(ActorWrapper pickup, SaBoostPadEventKind kind) {
  if (pickup.IsNull()) {
    return;
  }

  SaBoostPadEvent event{};
  event.timing = currentEventTiming();
  event.pad_id = boostPadId(pickup);
  event.kind = kind;
  if (kind == SaBoostPadEventKindPickedUp) {
    event.sequence = ++boostPadSequences[pickup.memory_address];
    if (auto playerIndex = playerIndexForNearestCar(pickup, BOOST_PICKUP_ATTRIBUTION_RADIUS)) {
      event.player_index = *playerIndex;
      event.has_player = 1;
    }
  }
  pendingBoostPadEvents.push_back(event);
}

void SubtrActorPlugin::recordGoal(
    ServerWrapper server,
    GoalWrapper goal,
    int scoreIndex,
    int assistIndex) {
  SaGoalEvent event{};
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

  recordExplicitPlayerStat(priForScoreIndex(server, assistIndex), SaPlayerStatEventKindAssist);
}

void SubtrActorPlugin::recordDemolish(CarWrapper victim, ActorWrapper demolisher) {
  if (victim.IsNull() || demolisher.IsNull()) {
    return;
  }

  CarWrapper attacker(demolisher.memory_address);
  const auto victimIndex = playerIndexForCar(victim);
  const auto attackerIndex = playerIndexForCar(attacker);
  if (!victimIndex || !attackerIndex) {
    return;
  }

  SaDemolishEvent event{};
  event.timing = currentEventTiming();
  event.attacker_index = *attackerIndex;
  event.victim_index = *victimIndex;
  event.attacker_velocity = toSaVec3(attacker.GetVelocity());
  event.victim_velocity = toSaVec3(victim.GetVelocity());
  event.victim_location = toSaVec3(victim.GetLocation());
  event.active_duration_seconds = DEMO_ACTIVE_DURATION_SECONDS;
  pendingDemolishes.push_back(event);
}

void SubtrActorPlugin::recordPlayerStatDeltas(
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
  auto pushStats = [&](int previous, int next, SaPlayerStatEventKind kind, int PlayerStatSnapshot::*field) {
    const int count = consumeSuppressed(next - previous, field);
    for (int i = 0; i < count; i += 1) {
      SaPlayerStatEvent event{};
      event.timing = currentEventTiming();
      event.player_index = playerIndex;
      event.is_team_0 = isTeam0;
      event.kind = kind;
      if (kind == SaPlayerStatEventKindShot) {
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
  pushStats(it->second.shots, current.shots, SaPlayerStatEventKindShot, &PlayerStatSnapshot::shots);
  pushStats(it->second.saves, current.saves, SaPlayerStatEventKindSave, &PlayerStatSnapshot::saves);
  pushStats(
      it->second.assists,
      current.assists,
      SaPlayerStatEventKindAssist,
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

void SubtrActorPlugin::recordExplicitPlayerStat(PriWrapper pri, SaPlayerStatEventKind kind) {
  if (pri.IsNull()) {
    return;
  }

  const auto playerIndex = playerIndexForPri(pri);
  if (!playerIndex) {
    return;
  }

  SaPlayerStatEvent event{};
  event.timing = currentEventTiming();
  event.player_index = *playerIndex;
  event.is_team_0 = pri.GetTeamNum() == 0 ? 1 : 0;
  event.kind = kind;
  pendingPlayerStatEvents.push_back(event);

  if (lastPlayerStats.find(pri.memory_address) == lastPlayerStats.end()) {
    return;
  }

  PlayerStatSnapshot &suppressed = suppressedPlayerStatDeltas[pri.memory_address];
  if (kind == SaPlayerStatEventKindShot) {
    suppressed.shots += 1;
  } else if (kind == SaPlayerStatEventKindSave) {
    suppressed.saves += 1;
  } else if (kind == SaPlayerStatEventKindAssist) {
    suppressed.assists += 1;
  }
}

std::optional<uint32_t> SubtrActorPlugin::playerIndexForCar(CarWrapper car) {
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

std::optional<uint32_t> SubtrActorPlugin::playerIndexForPri(PriWrapper pri) {
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

PriWrapper SubtrActorPlugin::priForScoreIndex(ServerWrapper server, int scoreIndex) {
  if (server.IsNull() || scoreIndex < 0) {
    return PriWrapper(0);
  }

  ArrayWrapper<PriWrapper> pris = server.GetPRIs();
  if (pris.IsNull() || scoreIndex >= pris.Count()) {
    return PriWrapper(0);
  }

  return pris.Get(scoreIndex);
}

std::optional<uint32_t> SubtrActorPlugin::playerIndexForScoreIndex(
    ServerWrapper server,
    int scoreIndex) {
  return playerIndexForPri(priForScoreIndex(server, scoreIndex));
}

std::optional<uint32_t> SubtrActorPlugin::playerIndexForNearestCar(
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

uint32_t SubtrActorPlugin::stablePlayerIndexForPri(PriWrapper pri, uint32_t fallbackIndex) {
  if (pri.IsNull()) {
    return fallbackIndex;
  }

  const std::string uniqueId = pri.GetbBot() != 0 ? "" : pri.GetUniqueIdWrapper().GetIdString();
  if (!uniqueId.empty()) {
    const auto existing = uniqueIdPlayerIndices.find(uniqueId);
    if (existing != uniqueIdPlayerIndices.end()) {
      return existing->second;
    }

    const uint32_t playerIndex = nextPlayerIndex++;
    uniqueIdPlayerIndices[uniqueId] = playerIndex;
    return playerIndex;
  }

  const auto existing = stablePriPlayerIndices.find(pri.memory_address);
  if (existing != stablePriPlayerIndices.end()) {
    return existing->second;
  }

  const uint32_t playerIndex = nextPlayerIndex++;
  stablePriPlayerIndices[pri.memory_address] = playerIndex;
  return playerIndex;
}

uint32_t SubtrActorPlugin::boostPadId(ActorWrapper pickup) {
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

void SubtrActorPlugin::sampleTeamScores(ServerWrapper server, SaLiveFrame &frame) {
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

void SubtrActorPlugin::sampleTeamScores(ServerWrapper server, SaGoalEvent &goal) {
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

std::optional<bool> SubtrActorPlugin::scoringTeamFromScoreDelta(
    const SaGoalEvent &goal) const {
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

void SubtrActorPlugin::rememberTeamScores(const SaLiveFrame &frame) {
  if (frame.has_team_zero_score != 0 && frame.has_team_one_score != 0) {
    lastTeamScores = std::make_pair(frame.team_zero_score, frame.team_one_score);
  }
}

void SubtrActorPlugin::rememberTeamScores(const SaGoalEvent &goal) {
  if (goal.has_team_zero_score != 0 && goal.has_team_one_score != 0) {
    lastTeamScores = std::make_pair(goal.team_zero_score, goal.team_one_score);
  }
}

bool SubtrActorPlugin::goalEventIsDuplicate(const SaGoalEvent &goal) const {
  const SaGoalEvent *previous = nullptr;
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
