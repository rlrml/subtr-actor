// Included by StateExportPlugin.cpp; shares the plugin translation unit.
//
// Self-rescheduling export tick, ported from
// bakkesmod/subtr-actor/plugin/live_tick.cpp: 250ms idle cadence when
// disabled or out of game, the game-time rate limit from
// state_export_sample_interval_ms when active, and wasInGame edge handling
// for the match-context lifecycle.
void StateExportPlugin::scheduleExportTick(float delaySeconds) {
  auto cancelled = exportTickCancelled;
  gameWrapper->SetTimeout(
      [this, cancelled](GameWrapper *) {
        if (!cancelled || *cancelled) {
          return;
        }

        tick();
        const bool active = exportEnabled() && gameWrapper->IsInGame();
        const float nextDelay = active ? sampleIntervalSeconds() : 0.25f;
        scheduleExportTick(nextDelay);
      },
      delaySeconds);
}

void StateExportPlugin::refreshStatus() {
  if (engine && engineStatus) {
    engineStatus(engine, &lastStatus);
    lastErrorText = engineErrorMessage();
  } else {
    lastStatus = SeStatus{};
    lastErrorText.clear();
  }
}

void StateExportPlugin::tick() {
  if (!rustLoaded || !engine) {
    return;
  }

  // Atomics-only on the Rust side; polling every tick is effectively free
  // and keeps the client-count gate and the settings window current.
  refreshStatus();

  if (!exportEnabled() || !gameWrapper->IsInGame()) {
    if (wasInGame) {
      notifyMatchEndAndReset(exportEnabled() ? "left game" : "export disabled");
      resetLiveState();
    }
    wasInGame = false;
    awaitingNextMatch = false;
    clearPendingFrameEvents();
    return;
  }

  ServerWrapper server = gameWrapper->GetGameEventAsServer();
  if (!server.IsNull() && server.GetbMatchEnded() != 0) {
    if (!awaitingNextMatch) {
      notifyMatchEndAndReset("match-ended state");
      resetLiveState();
    }
    awaitingNextMatch = true;
    wasInGame = true;
    clearPendingFrameEvents();
    return;
  }

  // After an Exhibition ends, Rocket League clears bMatchEnded and creates an
  // empty replacement server while it is still showing "Choose Team". Do not
  // mistake that interstitial for the next match; bRoundActive becomes true
  // once a player has joined and the next kickoff is actually playable.
  if (awaitingNextMatch &&
      (server.IsNull() || server.GetbRoundActive() == 0)) {
    clearPendingFrameEvents();
    return;
  }

  if (!wasInGame || awaitingNextMatch) {
    resetLiveState();
    pushMatchContext(server);
    awaitingNextMatch = false;
  }
  wasInGame = true;
  maybeRefreshMatchContext();

  // With no clients connected there is nobody to broadcast to; skip the
  // sampling work entirely (and drop events recorded meanwhile) unless the
  // user opted into always-on sampling.
  if (lastStatus.client_count == 0 && !sampleWhenNoClientsEnabled()) {
    clearPendingFrameEvents();
    return;
  }

  if (!server.IsNull()) {
    const float now = server.GetSecondsElapsed();
    if (lastProcessedGameTime && now >= *lastProcessedGameTime &&
        now - *lastProcessedGameTime < sampleIntervalSeconds()) {
      return;
    }
    lastProcessedGameTime = now;
  }

  SeFrame frame = sampleFrame();
  const int32_t pushResult = pushFrame(engine, &frame);
  if (pushResult != 0) {
    cvarManager->log(std::format(
        "state-export: push_frame failed: {} ({})",
        pushResult,
        engineErrorMessage()));
    return;
  }

  commitPendingFrameEvents();
  clearPendingFrameEvents();
}

void StateExportPlugin::resetLiveState() {
  frameNumber = 0;
  lastTime = 0.0f;
  lastProcessedGameTime.reset();
  sampledPlayers.clear();
  sampledPlayerNames.clear();
  sampledPlayerEpicIds.clear();
  clearPendingFrameEvents();
  lastBoostAmounts.clear();
  carPlayerIndices.clear();
  priPlayerIndices.clear();
  uniqueIdPlayerIndices.clear();
  stableBotPlayerIndices.clear();
  stablePriPlayerIndices.clear();
  lastPlayerStats.clear();
  suppressedPlayerStatDeltas.clear();
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
}
