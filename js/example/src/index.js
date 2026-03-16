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
        <div id="kickoff-overlay" class="kickoff-overlay" hidden>
          <span class="kickoff-label">Kickoff</span>
          <strong id="kickoff-countdown" class="kickoff-countdown">3</strong>
        </div>
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
            <input id="skip-kickoffs" type="checkbox" />
            <span>Skip kickoffs</span>
          </label>
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
const cameraDistance = mustElement("#camera-distance");
const cameraDistanceReadout = mustElement("#camera-distance-readout");
const attachedPlayer = mustElement("#attached-player");
const ballCam = mustElement("#ball-cam");
const timeline = mustElement("#timeline");
const statusReadout = mustElement("#status-readout");
const teamsReadout = mustElement("#teams-readout");
const playersReadout = mustElement("#players-readout");
const timeReadout = mustElement("#time-readout");
const remainingReadout = mustElement("#remaining-readout");
const frameReadout = mustElement("#frame-readout");
const durationReadout = mustElement("#duration-readout");
const skipKickoffs = mustElement("#skip-kickoffs");
const kickoffOverlay = mustElement("#kickoff-overlay");
const kickoffCountdown = mustElement("#kickoff-countdown");

let replayPlayer = null;
let unsubscribe = null;

function setControlsEnabled(enabled) {
  togglePlayback.disabled = !enabled;
  playbackRate.disabled = !enabled;
  attachedPlayer.disabled = !enabled;
  cameraDistance.disabled = !enabled;
  ballCam.disabled = !enabled;
  timeline.disabled = !enabled;
}

function getKickoffDisplayCountdown(replay, snapshot) {
  const currentFrame = replay.frames[snapshot.frameIndex];
  if (!currentFrame || currentFrame.kickoffCountdown <= 0) {
    return null;
  }

  let startIndex = snapshot.frameIndex;
  while (
    startIndex > 0 &&
    (replay.frames[startIndex - 1]?.kickoffCountdown ?? 0) > 0
  ) {
    startIndex -= 1;
  }

  let endIndex = snapshot.frameIndex + 1;
  while (
    endIndex < replay.frames.length &&
    replay.frames[endIndex].kickoffCountdown > 0
  ) {
    endIndex += 1;
  }

  let maxCountdown = 0;
  for (let index = startIndex; index < endIndex; index += 1) {
    maxCountdown = Math.max(maxCountdown, replay.frames[index].kickoffCountdown);
  }

  const kickoffEndTime = replay.frames[endIndex]?.time ?? replay.duration;
  const secondsRemaining = Math.max(0, kickoffEndTime - snapshot.currentTime);
  return Math.max(1, Math.min(maxCountdown, Math.ceil(secondsRemaining)));
}

function renderSnapshot(snapshot) {
  const metadata = replayPlayer?.replay.frames[snapshot.frameIndex];
  const kickoffDisplayCountdown = replayPlayer
    ? getKickoffDisplayCountdown(replayPlayer.replay, snapshot)
    : null;

  timeReadout.textContent = `${snapshot.currentTime.toFixed(2)}s`;
  frameReadout.textContent = `${snapshot.frameIndex}`;
  durationReadout.textContent = `${snapshot.duration.toFixed(2)}s`;
  timeline.value = `${snapshot.currentTime}`;
  togglePlayback.textContent = snapshot.playing ? "Pause" : "Play";
  cameraDistance.value = `${snapshot.cameraDistanceScale}`;
  cameraDistanceReadout.textContent = `${snapshot.cameraDistanceScale.toFixed(2)}x`;
  ballCam.checked = snapshot.ballCamEnabled;
  attachedPlayer.value = snapshot.attachedPlayerId ?? "";
  cameraDistance.disabled = replayPlayer === null || snapshot.attachedPlayerId === null;
  ballCam.disabled = replayPlayer === null || snapshot.attachedPlayerId === null;
  skipKickoffs.checked = snapshot.skipKickoffsEnabled;

  remainingReadout.textContent =
    metadata === undefined ? "--" : `${metadata.secondsRemaining}s`;

  const inKickoff = kickoffDisplayCountdown !== null;
  kickoffOverlay.hidden = !inKickoff;
  if (inKickoff) {
    kickoffCountdown.textContent = `${kickoffDisplayCountdown}`;
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
    initialCameraDistanceScale: 2.25,
    initialAttachedPlayerId: null,
    initialBallCamEnabled: false,
    initialSkipKickoffsEnabled: skipKickoffs.checked,
  });
  unsubscribe = replayPlayer.subscribe(renderSnapshot);

  populateAttachedPlayerOptions(replay.players);

  emptyState.hidden = true;
  timeline.min = "0";
  timeline.max = `${replay.duration}`;
  timeline.step = "0.01";
  teamsReadout.textContent = `${replay.teamZeroNames.length} blue / ${replay.teamOneNames.length} orange`;
  playersReadout.textContent = replay.players.map((player) => player.name).join(", ");
  statusReadout.textContent = `Loaded ${file.name}`;
  setControlsEnabled(true);
  renderSnapshot(replayPlayer.getSnapshot());
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

cameraDistance.addEventListener("input", () => {
  replayPlayer?.setCameraDistanceScale(Number(cameraDistance.value));
});

attachedPlayer.addEventListener("change", () => {
  replayPlayer?.setAttachedPlayer(attachedPlayer.value || null);
});

ballCam.addEventListener("change", () => {
  replayPlayer?.setBallCamEnabled(ballCam.checked);
});

skipKickoffs.addEventListener("change", () => {
  replayPlayer?.setSkipKickoffsEnabled(skipKickoffs.checked);
});

timeline.addEventListener("input", () => {
  replayPlayer?.seek(Number(timeline.value));
});
