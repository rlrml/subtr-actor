import "./styles.css";
import { ReplayPlayer, loadReplayFromBytes } from "../lib";
import type { CameraMode, ReplayPlayerSnapshot } from "../types";

function mustElement<T extends Element>(selector: string): T {
  const element = document.querySelector<T>(selector);
  if (!element) {
    throw new Error(`Missing element for selector: ${selector}`);
  }

  return element;
}

const app = mustElement<HTMLDivElement>("#app");

app.innerHTML = `
  <main class="shell">
    <section class="hero">
      <div>
        <p class="eyebrow">subtr-actor / clean rewrite</p>
        <h1>three.js replay player</h1>
        <p class="lede">
          Load a Rocket League replay from disk, parse it through the local wasm bindings,
          and inspect a live 3D playback scene.
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
          Drop in a replay to populate the scene.
        </div>
      </div>
      <aside class="sidebar">
        <div class="panel">
          <h2>Camera</h2>
          <div class="transport-row">
            <select id="camera-mode" disabled>
              <option value="overview" selected>Overview</option>
              <option value="attached">Attached</option>
              <option value="third-person">Third Person</option>
            </select>
            <select id="tracked-player" disabled>
              <option value="">No player</option>
            </select>
          </div>
        </div>
        <div class="panel">
          <h2>Transport</h2>
          <div class="transport-row">
            <button id="toggle-playback" disabled>Play</button>
            <select id="playback-rate" disabled>
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

const fileInput = mustElement<HTMLInputElement>("#replay-file");
const viewport = mustElement<HTMLDivElement>("#viewport");
const emptyState = mustElement<HTMLDivElement>("#empty-state");
const togglePlayback = mustElement<HTMLButtonElement>("#toggle-playback");
const playbackRate = mustElement<HTMLSelectElement>("#playback-rate");
const cameraMode = mustElement<HTMLSelectElement>("#camera-mode");
const trackedPlayer = mustElement<HTMLSelectElement>("#tracked-player");
const timeline = mustElement<HTMLInputElement>("#timeline");
const statusReadout = mustElement<HTMLElement>("#status-readout");
const teamsReadout = mustElement<HTMLElement>("#teams-readout");
const playersReadout = mustElement<HTMLElement>("#players-readout");
const timeReadout = mustElement<HTMLElement>("#time-readout");
const remainingReadout = mustElement<HTMLElement>("#remaining-readout");
const frameReadout = mustElement<HTMLElement>("#frame-readout");
const durationReadout = mustElement<HTMLElement>("#duration-readout");

let replayPlayer: ReplayPlayer | null = null;

function setControlsEnabled(enabled: boolean): void {
  togglePlayback.disabled = !enabled;
  playbackRate.disabled = !enabled;
  cameraMode.disabled = !enabled;
  trackedPlayer.disabled = !enabled;
  timeline.disabled = !enabled;
}

function renderSnapshot(snapshot: ReplayPlayerSnapshot): void {
  timeReadout.textContent = `${snapshot.currentTime.toFixed(2)}s`;
  frameReadout.textContent = `${snapshot.frameIndex}`;
  durationReadout.textContent = `${snapshot.duration.toFixed(2)}s`;
  timeline.value = `${snapshot.currentTime}`;
  togglePlayback.textContent = snapshot.playing ? "Pause" : "Play";
  cameraMode.value = snapshot.cameraMode;

  if (snapshot.trackedPlayerId) {
    trackedPlayer.value = snapshot.trackedPlayerId;
  }

  if (replayPlayer) {
    const metadata = replayPlayer.replay.frames[snapshot.frameIndex];
    remainingReadout.textContent =
      metadata === undefined ? "--" : `${metadata.secondsRemaining}s`;
  }
}

async function loadReplayFile(file: File): Promise<void> {
  statusReadout.textContent = "Parsing replay...";
  setControlsEnabled(false);

  replayPlayer?.dispose();
  replayPlayer = null;

  const bytes = new Uint8Array(await file.arrayBuffer());
  const { replay } = await loadReplayFromBytes(bytes);

  replayPlayer = new ReplayPlayer(viewport, replay, {
    initialCameraMode: "overview",
    initialTrackedPlayerId: replay.players[0]?.id,
  });
  replayPlayer.addEventListener("change", (event: Event) => {
    renderSnapshot((event as CustomEvent<ReplayPlayerSnapshot>).detail);
  });

  trackedPlayer.innerHTML = replay.players
    .map(
      (player) =>
        `<option value="${player.id}">${player.name} (${player.isTeamZero ? "Blue" : "Orange"})</option>`
    )
    .join("");

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

cameraMode.addEventListener("change", () => {
  replayPlayer?.setCameraMode(cameraMode.value as CameraMode);
});

trackedPlayer.addEventListener("change", () => {
  if (!trackedPlayer.value) {
    return;
  }

  replayPlayer?.setTrackedPlayer(trackedPlayer.value);
});

timeline.addEventListener("input", () => {
  replayPlayer?.seek(Number(timeline.value));
});
