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
    clearPendingFrameEvents();
    return;
  }
  if (!wasInGame) {
    resetLiveState();
    pushMatchContext(gameWrapper->GetGameEventAsServer());
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

  ServerWrapper server = gameWrapper->GetGameEventAsServer();
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
