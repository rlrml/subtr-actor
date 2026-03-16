import "./styles.css";
import { ReplayPlayer, loadReplayFromBytes } from "../../player/src/lib.ts";

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
        <span>Choose replay</span>
        <input id="replay-file" type="file" accept=".replay" />
      </label>
    </section>
    <section class="workspace">
      <div class="viewport-panel">
        <div id="viewport" class="viewport"></div>
        <div id="empty-state" class="empty-state">
          Choose a replay to populate the scene.
        </div>
      </div>
      <aside class="sidebar">
        <div class="panel">
          <h2>Camera</h2>
          <div class="transport-row">
            <select id="camera-mode" disabled>
              <option value="overview" selected>Overview</option>
              <option value="tracked">Tracked</option>
            </select>
            <select id="tracked-player" disabled>
              <option value="">No player</option>
            </select>
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
          <input id="timeline" type="range" min="0" max="0" step="0.01" value="0" disabled />
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
const cameraMode = mustElement("#camera-mode");
const cameraDistance = mustElement("#camera-distance");
const cameraDistanceReadout = mustElement("#camera-distance-readout");
const trackedPlayer = mustElement("#tracked-player");
const ballCam = mustElement("#ball-cam");
const timeline = mustElement("#timeline");
const statusReadout = mustElement("#status-readout");
const teamsReadout = mustElement("#teams-readout");
const playersReadout = mustElement("#players-readout");
const timeReadout = mustElement("#time-readout");
const remainingReadout = mustElement("#remaining-readout");
const frameReadout = mustElement("#frame-readout");
const durationReadout = mustElement("#duration-readout");

let replayPlayer = null;
let unsubscribe = null;

function setControlsEnabled(enabled) {
  togglePlayback.disabled = !enabled;
  playbackRate.disabled = !enabled;
  cameraMode.disabled = !enabled;
  cameraDistance.disabled = !enabled;
  trackedPlayer.disabled = !enabled;
  ballCam.disabled = !enabled;
  timeline.disabled = !enabled;
}

function renderSnapshot(snapshot) {
  timeReadout.textContent = `${snapshot.currentTime.toFixed(2)}s`;
  frameReadout.textContent = `${snapshot.frameIndex}`;
  durationReadout.textContent = `${snapshot.duration.toFixed(2)}s`;
  timeline.value = `${snapshot.currentTime}`;
  togglePlayback.textContent = snapshot.playing ? "Pause" : "Play";
  cameraMode.value = snapshot.cameraMode;
  cameraDistance.value = `${snapshot.cameraDistanceScale}`;
  cameraDistanceReadout.textContent = `${snapshot.cameraDistanceScale.toFixed(2)}x`;
  ballCam.checked = snapshot.ballCamEnabled;
  trackedPlayer.value = snapshot.trackedPlayerId ?? "";

  if (replayPlayer) {
    const metadata = replayPlayer.replay.frames[snapshot.frameIndex];
    remainingReadout.textContent =
      metadata === undefined ? "--" : `${metadata.secondsRemaining}s`;
  }
}

async function loadReplayFile(file) {
  statusReadout.textContent = "Parsing replay...";
  setControlsEnabled(false);

  if (unsubscribe) {
    unsubscribe();
    unsubscribe = null;
  }

  replayPlayer?.destroy();
  replayPlayer = null;

  const bytes = new Uint8Array(await file.arrayBuffer());
  const { replay } = await loadReplayFromBytes(bytes);

  replayPlayer = new ReplayPlayer(viewport, replay, {
    initialCameraMode: "overview",
    initialCameraDistanceScale: 2.25,
    initialTrackedPlayerId: replay.players[0]?.id ?? null,
    initialBallCamEnabled: false,
  });
  unsubscribe = replayPlayer.subscribe(renderSnapshot);

  trackedPlayer.innerHTML = [
    '<option value="">No player</option>',
    ...replay.players.map(
      (player) =>
        `<option value="${player.id}">${player.name} (${player.isTeamZero ? "Blue" : "Orange"})</option>`
    ),
  ].join("");

  emptyState.hidden = true;
  timeline.min = "0";
  timeline.max = `${replay.duration}`;
  timeline.step = "0.01";
  teamsReadout.textContent = `${replay.teamZeroNames.length} blue / ${replay.teamOneNames.length} orange`;
  playersReadout.textContent = replay.players.map((player) => player.name).join(", ");
  statusReadout.textContent = `Loaded ${file.name}`;
  setControlsEnabled(true);
}

fileInput.addEventListener("change", async () => {
  const file = fileInput.files?.[0];
  if (!file) {
    return;
  }

  try {
    await loadReplayFile(file);
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

cameraMode.addEventListener("change", () => {
  replayPlayer?.setCameraMode(cameraMode.value);
});

cameraDistance.addEventListener("input", () => {
  replayPlayer?.setCameraDistanceScale(Number(cameraDistance.value));
});

trackedPlayer.addEventListener("change", () => {
  replayPlayer?.setTrackedPlayer(trackedPlayer.value || null);
});

ballCam.addEventListener("change", () => {
  replayPlayer?.setBallCamEnabled(ballCam.checked);
});

timeline.addEventListener("input", () => {
  replayPlayer?.seek(Number(timeline.value));
});
