import "./styles.css";
import {
  ReplayPlaylistPlayer,
  createBallchasingOverlayPlugin,
  createBoostPadOverlayPlugin,
  createTimelineOverlayPlugin,
  createFullReplayPlaylistItem,
  createReplaySource,
  loadReplayFromBytes,
  timeBound,
} from "../../player/src/lib.ts";

function mustElement(selector) {
  const element = document.querySelector(selector);
  if (!element) {
    throw new Error(`Missing element for selector: ${selector}`);
  }

  return element;
}

const app = mustElement("#app");

app.innerHTML = `
  <main class="shell">
    <section class="hero">
      <div>
        <p class="eyebrow">subtr-actor / player demo</p>
        <h1>Replay player library example</h1>
        <p class="lede">
          Load a Rocket League replay, parse it through the local wasm bindings,
          and drive the reusable player API with demo controls.
        </p>
      </div>
      <label class="file-picker">
          <span>Choose replay(s)</span>
          <input id="replay-file" type="file" accept=".replay" multiple />
      </label>
    </section>
    <section class="workspace">
      <div class="viewport-panel">
        <div id="viewport" class="viewport"></div>
        <div id="scoreboard" class="scoreboard" hidden>
          <section class="scoreboard-team scoreboard-team-blue">
            <span class="scoreboard-team-accent" aria-hidden="true"></span>
            <span class="scoreboard-team-label">Blue</span>
            <strong id="blue-score" class="scoreboard-score">0</strong>
          </section>
          <section class="scoreboard-center">
            <span id="scoreboard-phase" class="scoreboard-phase">Replay</span>
            <strong id="scoreboard-clock" class="scoreboard-clock">5:00</strong>
          </section>
          <section class="scoreboard-team scoreboard-team-orange">
            <strong id="orange-score" class="scoreboard-score">0</strong>
            <span class="scoreboard-team-label scoreboard-team-label-right">Orange</span>
            <span class="scoreboard-team-accent" aria-hidden="true"></span>
          </section>
        </div>
        <div id="kickoff-overlay" class="kickoff-overlay" hidden>
          <span class="kickoff-label">Kickoff</span>
          <strong id="kickoff-countdown" class="kickoff-countdown">3</strong>
        </div>
        <button id="play-overlay" class="play-overlay" hidden type="button">
          <span class="play-overlay-icon" aria-hidden="true"></span>
          <span class="play-overlay-label">Resume replay</span>
        </button>
        <div id="empty-state" class="empty-state">
          Choose a replay to populate the scene.
        </div>
      </div>
      <aside class="sidebar">
        <div class="panel">
          <h2>Camera</h2>
          <div class="transport-row">
            <select id="attached-player" disabled>
              <option value="">Free camera</option>
            </select>
          </div>
          <div class="camera-presets" role="group" aria-label="Camera views">
            <button id="camera-view-free" type="button" disabled>Free</button>
            <button id="camera-view-follow" type="button" disabled>Follow</button>
            <button id="camera-view-overhead" type="button" disabled>Overhead</button>
            <button id="camera-view-side" type="button" disabled>Diagonal</button>
          </div>
          <label>
            <span class="label">Follow Distance</span>
            <input
              id="camera-distance"
              type="range"
              min="0.75"
              max="4"
              step="0.05"
              value="2.25"
              disabled
            />
          </label>
          <strong id="camera-distance-readout">2.25x</strong>
          <label class="toggle">
            <input id="ball-cam" type="checkbox" disabled />
            <span>Ball cam</span>
          </label>
        </div>
        <div class="panel">
          <h2>Transport</h2>
          <div class="transport-row">
            <button id="toggle-playback" disabled>Play</button>
            <select id="playback-rate" disabled>
              <option value="0.25">0.25x</option>
              <option value="0.5">0.5x</option>
              <option value="1" selected>1.0x</option>
              <option value="1.5">1.5x</option>
              <option value="2">2.0x</option>
            </select>
          </div>
          <label class="toggle">
            <input id="skip-post-goal-transitions" type="checkbox" checked />
            <span>Skip post-goal resets</span>
          </label>
          <label class="toggle">
            <input id="skip-kickoffs" type="checkbox" />
            <span>Skip kickoff countdowns</span>
          </label>
          <div class="stat-grid">
            <div>
              <span class="label">Time</span>
              <strong id="time-readout">0.00s</strong>
            </div>
            <div>
              <span class="label">Remaining</span>
              <strong id="remaining-readout">--</strong>
            </div>
            <div>
              <span class="label">Frame</span>
              <strong id="frame-readout">0</strong>
            </div>
            <div>
              <span class="label">Duration</span>
              <strong id="duration-readout">0.00s</strong>
            </div>
          </div>
        </div>
        <div class="panel">
          <h2>Playlist</h2>
          <div class="transport-row">
            <button id="previous-item" disabled>Previous</button>
            <button id="next-item" disabled>Next</button>
          </div>
          <label>
            <span class="label">Advance</span>
            <select id="playlist-advance" disabled>
              <option value="auto" selected>Auto</option>
              <option value="manual">Manual</option>
            </select>
          </label>
          <label>
            <span class="label">End</span>
            <select id="playlist-end" disabled>
              <option value="stop" selected>Stop</option>
              <option value="loop">Loop</option>
            </select>
          </label>
          <dl class="info-list">
            <div>
              <dt>Item</dt>
              <dd id="playlist-item-readout">--</dd>
            </div>
            <div>
              <dt>Clip</dt>
              <dd id="playlist-clip-readout">--</dd>
            </div>
          </dl>
        </div>
        <div class="panel">
          <h2>Replay</h2>
          <dl class="info-list">
            <div>
              <dt>Status</dt>
              <dd id="status-readout">Waiting for file</dd>
            </div>
            <div>
              <dt>Teams</dt>
              <dd id="teams-readout">--</dd>
            </div>
            <div>
              <dt>Players</dt>
              <dd id="players-readout">--</dd>
            </div>
          </dl>
        </div>
      </aside>
    </section>
  </main>
`;

const fileInput = mustElement("#replay-file");
const viewport = mustElement("#viewport");
const emptyState = mustElement("#empty-state");
const togglePlayback = mustElement("#toggle-playback");
const playbackRate = mustElement("#playback-rate");
const cameraDistance = mustElement("#camera-distance");
const cameraDistanceReadout = mustElement("#camera-distance-readout");
const attachedPlayer = mustElement("#attached-player");
const cameraViewFreeButton = mustElement("#camera-view-free");
const cameraViewFollowButton = mustElement("#camera-view-follow");
const cameraViewOverheadButton = mustElement("#camera-view-overhead");
const cameraViewSideButton = mustElement("#camera-view-side");
const ballCam = mustElement("#ball-cam");
const statusReadout = mustElement("#status-readout");
const teamsReadout = mustElement("#teams-readout");
const playersReadout = mustElement("#players-readout");
const timeReadout = mustElement("#time-readout");
const remainingReadout = mustElement("#remaining-readout");
const frameReadout = mustElement("#frame-readout");
const durationReadout = mustElement("#duration-readout");
const skipPostGoalTransitions = mustElement("#skip-post-goal-transitions");
const skipKickoffs = mustElement("#skip-kickoffs");
const scoreboard = mustElement("#scoreboard");
const blueScore = mustElement("#blue-score");
const orangeScore = mustElement("#orange-score");
const scoreboardPhase = mustElement("#scoreboard-phase");
const scoreboardClock = mustElement("#scoreboard-clock");
const kickoffOverlay = mustElement("#kickoff-overlay");
const kickoffCountdown = mustElement("#kickoff-countdown");
const playOverlay = mustElement("#play-overlay");
const previousItem = mustElement("#previous-item");
const nextItem = mustElement("#next-item");
const playlistAdvance = mustElement("#playlist-advance");
const playlistEnd = mustElement("#playlist-end");
const playlistItemReadout = mustElement("#playlist-item-readout");
const playlistClipReadout = mustElement("#playlist-clip-readout");

let replayPlayer = null;
let unsubscribe = null;
let currentReplayRaw = null;
let activePlaylistItems = [];
let renderedReplaySourceId = null;

function setControlsEnabled(enabled) {
  togglePlayback.disabled = !enabled;
  playbackRate.disabled = !enabled;
  attachedPlayer.disabled = !enabled;
  previousItem.disabled = !enabled;
  nextItem.disabled = !enabled;
  playlistAdvance.disabled = !enabled;
  playlistEnd.disabled = !enabled;
  syncCameraModeButtons(enabled ? replayPlayer?.getSnapshot() : undefined);
}

function getCameraViewButton(mode) {
  switch (mode) {
    case "free":
      return cameraViewFreeButton;
    case "follow":
      return cameraViewFollowButton;
    case "overhead":
      return cameraViewOverheadButton;
    case "side":
      return cameraViewSideButton;
    default:
      throw new Error(`Unknown camera mode: ${mode}`);
  }
}

function syncCameraModeButtons(snapshot) {
  const activeMode = snapshot?.cameraViewMode ?? "free";
  const hasReplay = replayPlayer !== null && snapshot !== undefined;
  const canFollow = (snapshot?.attachedPlayerId ?? null) !== null;

  for (const mode of ["free", "follow"]) {
    const button = getCameraViewButton(mode);
    button.disabled = !hasReplay || (mode === "follow" && !canFollow);
    const active = mode === activeMode;
    button.dataset.active = active ? "true" : "false";
    button.setAttribute("aria-pressed", active ? "true" : "false");
  }

  cameraViewOverheadButton.disabled = !hasReplay;
  cameraViewSideButton.disabled = !hasReplay;
  cameraViewOverheadButton.dataset.active = "false";
  cameraViewSideButton.dataset.active = "false";
  cameraViewOverheadButton.setAttribute("aria-pressed", "false");
  cameraViewSideButton.setAttribute("aria-pressed", "false");
}

function formatClock(secondsRemaining) {
  if (!Number.isFinite(secondsRemaining)) {
    return "--:--";
  }

  const safeSeconds = Math.max(0, Math.round(secondsRemaining));
  const minutes = Math.floor(safeSeconds / 60);
  const seconds = safeSeconds % 60;
  return `${minutes}:${String(seconds).padStart(2, "0")}`;
}

function getScoreAtFrame(frameIndex) {
  const goalEvents = currentReplayRaw?.goal_events;
  if (!Array.isArray(goalEvents) || goalEvents.length === 0) {
    return {
      teamZeroScore: 0,
      teamOneScore: 0,
    };
  }

  let teamZeroScore = 0;
  let teamOneScore = 0;
  for (const event of goalEvents) {
    if ((event?.frame ?? Number.POSITIVE_INFINITY) > frameIndex) {
      break;
    }

    if (typeof event?.team_zero_score === "number") {
      teamZeroScore = event.team_zero_score;
    }
    if (typeof event?.team_one_score === "number") {
      teamOneScore = event.team_one_score;
    }
  }

  return {
    teamZeroScore,
    teamOneScore,
  };
}

function renderSnapshot(snapshot) {
  const currentReplay = replayPlayer?.getCurrentReplay?.() ?? null;
  currentReplayRaw = currentReplay?.raw ?? null;
  const replaySourceId =
    replayPlayer?.getCurrentResolvedItem?.()?.source.replay.id ?? null;
  if (currentReplay && replaySourceId !== renderedReplaySourceId) {
    renderedReplaySourceId = replaySourceId;
    populateAttachedPlayerOptions(currentReplay.replay.players);
    teamsReadout.textContent = `${currentReplay.replay.teamZeroNames.length} blue / ${currentReplay.replay.teamOneNames.length} orange`;
    playersReadout.textContent = currentReplay.replay.players
      .map((player) => player.name)
      .join(", ");
  }
  const metadata = currentReplay?.replay.frames[snapshot.frameIndex];
  const kickoffMetadata =
    snapshot.activeMetadata?.kind === "kickoff-countdown"
      ? snapshot.activeMetadata
      : null;
  const { teamZeroScore, teamOneScore } = getScoreAtFrame(snapshot.frameIndex);

  timeReadout.textContent = `${snapshot.currentTime.toFixed(2)}s`;
  frameReadout.textContent = `${snapshot.frameIndex}`;
  durationReadout.textContent = `${snapshot.duration.toFixed(2)}s`;
  playlistAdvance.value = snapshot.advanceMode;
  playlistEnd.value = snapshot.endMode;
  playlistItemReadout.textContent =
    snapshot.itemCount === 0
      ? "--"
      : `${snapshot.itemIndex + 1} / ${snapshot.itemCount}`;
  playlistClipReadout.textContent = snapshot.item?.label ?? "--";
  togglePlayback.textContent = snapshot.playing ? "Pause" : "Play";
  cameraDistance.value = `${snapshot.cameraDistanceScale}`;
  cameraDistanceReadout.textContent = `${snapshot.cameraDistanceScale.toFixed(2)}x`;
  ballCam.checked = snapshot.ballCamEnabled;
  attachedPlayer.value = snapshot.attachedPlayerId ?? "";
  syncCameraModeButtons(snapshot);
  const hasAttachedCamera = replayPlayer === null
    ? false
    : snapshot.cameraViewMode === "follow" && snapshot.attachedPlayerId !== null;
  cameraDistance.disabled = !hasAttachedCamera;
  ballCam.disabled = !hasAttachedCamera;
  skipPostGoalTransitions.checked = snapshot.skipPostGoalTransitionsEnabled;
  skipKickoffs.checked = snapshot.skipKickoffsEnabled;

  remainingReadout.textContent =
    metadata === undefined ? "--" : `${metadata.secondsRemaining}s`;
  scoreboard.hidden = replayPlayer === null;
  blueScore.textContent = `${teamZeroScore}`;
  orangeScore.textContent = `${teamOneScore}`;
  scoreboardClock.textContent = formatClock(metadata?.secondsRemaining);
  scoreboardPhase.textContent = kickoffMetadata ? "Kickoff" : "Live";
  playOverlay.hidden = replayPlayer === null || snapshot.playing;

  const inKickoff = kickoffMetadata !== null;
  kickoffOverlay.hidden = !inKickoff;
  if (inKickoff) {
    kickoffCountdown.textContent = `${kickoffMetadata.countdown}`;
  }
}

function populateAttachedPlayerOptions(players) {
  attachedPlayer.replaceChildren();
  attachedPlayer.append(new Option("Free camera", ""));

  for (const player of players) {
    attachedPlayer.append(
      new Option(
        `${player.name} (${player.isTeamZero ? "Blue" : "Orange"})`,
        player.id
      )
    );
  }
}

function createDemoPlaylistSource(file, index) {
  return createReplaySource(`file:${index}:${file.name}:${file.size}`, async () => {
    const bytes = new Uint8Array(await file.arrayBuffer());
    return loadReplayFromBytes(bytes, {
      useWorker: true,
      reportEveryNFrames: 500,
    });
  });
}

function createDemoPlaylistItem(file, index, fileCount) {
  const source = createDemoPlaylistSource(file, index);
  if (fileCount === 1) {
    return createFullReplayPlaylistItem(source, {
      label: file.name,
    });
  }

  return {
    replay: source,
    start: timeBound(10),
    end: timeBound(25),
    label: `${file.name} · 10s-25s`,
    meta: {
      fileName: file.name,
    },
  };
}

async function loadReplayFiles(files) {
  statusReadout.textContent = "Preparing playlist...";
  setControlsEnabled(false);
  scoreboard.hidden = true;
  kickoffOverlay.hidden = true;
  playOverlay.hidden = true;

  if (unsubscribe) {
    unsubscribe();
    unsubscribe = null;
  }

  replayPlayer?.destroy();
  replayPlayer = null;
  currentReplayRaw = null;
  renderedReplaySourceId = null;
  activePlaylistItems = files.map((file, index) =>
    createDemoPlaylistItem(file, index, files.length)
  );

  try {
    replayPlayer = new ReplayPlaylistPlayer(viewport, activePlaylistItems, {
      autoplay: true,
      advanceMode: playlistAdvance.value,
      endMode: playlistEnd.value,
      initialCameraDistanceScale: 2.25,
      initialAttachedPlayerId: null,
      initialBallCamEnabled: false,
      initialBoostMeterEnabled: false,
      initialSkipPostGoalTransitionsEnabled: skipPostGoalTransitions.checked,
      initialSkipKickoffsEnabled: skipKickoffs.checked,
      preloadPolicy: { kind: "adjacent", ahead: 1, behind: 1 },
      plugins: [
        createBallchasingOverlayPlugin(),
        createBoostPadOverlayPlugin(),
        createTimelineOverlayPlugin({
          replayEventKinds: ["goal", "save", "demo"],
        }),
      ],
    });
    statusReadout.textContent = "Loading first playlist item...";
    await replayPlayer.waitForCurrentItem();
    unsubscribe = replayPlayer.subscribe(renderSnapshot);

    const { replay } = replayPlayer.getCurrentReplay();
    populateAttachedPlayerOptions(replay.players);

    emptyState.hidden = true;
    teamsReadout.textContent = `${replay.teamZeroNames.length} blue / ${replay.teamOneNames.length} orange`;
    playersReadout.textContent = replay.players.map((player) => player.name).join(", ");
    statusReadout.textContent =
      files.length === 1
        ? `Loaded ${files[0].name}`
        : `Loaded ${files.length} replay playlist`;
    setControlsEnabled(true);
    renderSnapshot(replayPlayer.getSnapshot());
  } catch (error) {
    throw error;
  }
}

fileInput.addEventListener("change", async () => {
  const files = Array.from(fileInput.files ?? []);
  if (files.length === 0) {
    return;
  }

  try {
    await loadReplayFiles(files);
  } catch (error) {
    statusReadout.textContent =
      error instanceof Error ? error.message : "Failed to load replay";
  }
});

togglePlayback.addEventListener("click", () => {
  replayPlayer?.togglePlayback();
});

playbackRate.addEventListener("change", () => {
  replayPlayer?.setPlaybackRate(Number(playbackRate.value));
});

cameraDistance.addEventListener("input", () => {
  replayPlayer?.setCameraDistanceScale(Number(cameraDistance.value));
});

attachedPlayer.addEventListener("change", () => {
  replayPlayer?.setAttachedPlayer(attachedPlayer.value || null);
});

cameraViewFreeButton.addEventListener("click", () => {
  replayPlayer?.setCameraViewMode("free");
});

cameraViewFollowButton.addEventListener("click", () => {
  replayPlayer?.setCameraViewMode("follow");
});

cameraViewOverheadButton.addEventListener("click", () => {
  replayPlayer?.setFreeCameraPreset("overhead");
});

cameraViewSideButton.addEventListener("click", () => {
  replayPlayer?.setFreeCameraPreset("side");
});

ballCam.addEventListener("change", () => {
  replayPlayer?.setBallCamEnabled(ballCam.checked);
});

playOverlay.addEventListener("click", () => {
  replayPlayer?.play();
});

previousItem.addEventListener("click", async () => {
  if (!replayPlayer) {
    return;
  }
  await replayPlayer.previous();
  const replay = replayPlayer.getCurrentReplay()?.replay;
  if (replay) {
    populateAttachedPlayerOptions(replay.players);
  }
});

nextItem.addEventListener("click", async () => {
  if (!replayPlayer) {
    return;
  }
  await replayPlayer.next();
  const replay = replayPlayer.getCurrentReplay()?.replay;
  if (replay) {
    populateAttachedPlayerOptions(replay.players);
  }
});

playlistAdvance.addEventListener("change", () => {
  replayPlayer?.setAdvanceMode(playlistAdvance.value);
});

playlistEnd.addEventListener("change", () => {
  replayPlayer?.setEndMode(playlistEnd.value);
});

skipPostGoalTransitions.addEventListener("change", () => {
  replayPlayer?.setSkipPostGoalTransitionsEnabled(skipPostGoalTransitions.checked);
});

skipKickoffs.addEventListener("change", () => {
  replayPlayer?.setSkipKickoffsEnabled(skipKickoffs.checked);
});

syncCameraModeButtons();
