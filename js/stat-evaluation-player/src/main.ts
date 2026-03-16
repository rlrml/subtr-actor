import "./styles.css";
import {
  ReplayPlayer,
  loadReplayFromBytes,
} from "../../player/src/lib.ts";
import type {
  ReplayPlayerState,
} from "../../player/src/lib.ts";
import { RoleOverlay, createZoneBoundaryLines } from "./overlays.ts";

// WASM module - init is the default export, get_stats_timeline is named
import wasmInit, {
  get_stats_timeline,
} from "../../pkg/rl_replay_subtr_actor.js";

interface StatsTimeline {
  replay_meta: unknown;
  timeline_events: unknown[];
  frames: StatsFrame[];
}

interface StatsFrame {
  frame_number: number;
  time: number;
  dt: number;
  players: PlayerStatsSnapshot[];
  [key: string]: unknown;
}

interface PlayerStatsSnapshot {
  player_id: Record<string, string>;
  name: string;
  is_team_0: boolean;
  positioning?: {
    time_most_back: number;
    time_most_forward: number;
    time_even: number;
    [key: string]: unknown;
  };
  [key: string]: unknown;
}

const app = document.getElementById("app")!;

app.innerHTML = `
  <div class="shell">
    <div class="header">
      <h1>Stat Evaluation Player</h1>
      <input id="replay-file" type="file" accept=".replay" />
      <button id="toggle-playback" disabled>Play</button>
      <select id="playback-rate" disabled>
        <option value="0.25">0.25x</option>
        <option value="0.5">0.5x</option>
        <option value="1" selected>1.0x</option>
        <option value="2">2.0x</option>
      </select>
      <input id="timeline" type="range" min="0" max="0" step="0.01" value="0" disabled style="flex:1" />
    </div>
    <div id="viewport" class="viewport"></div>
    <aside class="sidebar">
      <div class="panel">
        <h2>Playback</h2>
        <div class="stat-row">
          <span class="label">Time</span>
          <span class="value" id="time-readout">0.00s</span>
        </div>
        <div class="stat-row">
          <span class="label">Frame</span>
          <span class="value" id="frame-readout">0</span>
        </div>
      </div>
      <div id="stats-container" class="panel">
        <h2>Player Stats</h2>
        <div id="player-stats">Load a replay to see stats.</div>
      </div>
    </aside>
  </div>
`;

let replayPlayer: ReplayPlayer | null = null;
let statsTimeline: StatsTimeline | null = null;
let unsubscribe: (() => void) | null = null;
let roleOverlay: RoleOverlay | null = null;
let removeRenderHook: (() => void) | null = null;

const fileInput = document.getElementById("replay-file") as HTMLInputElement;
const viewport = document.getElementById("viewport")!;
const togglePlayback = document.getElementById("toggle-playback") as HTMLButtonElement;
const playbackRate = document.getElementById("playback-rate") as HTMLSelectElement;
const timeline = document.getElementById("timeline") as HTMLInputElement;
const timeReadout = document.getElementById("time-readout")!;
const frameReadout = document.getElementById("frame-readout")!;
const playerStatsEl = document.getElementById("player-stats")!;

function renderStats(frameIndex: number): void {
  if (!statsTimeline) return;

  const statsFrame = statsTimeline.frames[frameIndex];
  if (!statsFrame) return;

  const lines: string[] = [];
  for (const player of statsFrame.players) {
    const pos = player.positioning;
    lines.push(`<div class="player-stats-group">`);
    lines.push(`<h3>${player.name} ${player.is_team_0 ? "(Blue)" : "(Orange)"}</h3>`);
    if (pos) {
      lines.push(`<div class="stat-row"><span class="label">Most back</span><span class="value">${pos.time_most_back?.toFixed(1) ?? "?"}s</span></div>`);
      lines.push(`<div class="stat-row"><span class="label">Most forward</span><span class="value">${pos.time_most_forward?.toFixed(1) ?? "?"}s</span></div>`);
      lines.push(`<div class="stat-row"><span class="label">Even</span><span class="value">${pos.time_even?.toFixed(1) ?? "?"}s</span></div>`);
    }
    lines.push(`</div>`);
  }
  playerStatsEl.innerHTML = lines.join("\n");
}

function onStateChange(state: ReplayPlayerState): void {
  timeReadout.textContent = `${state.currentTime.toFixed(2)}s`;
  frameReadout.textContent = `${state.frameIndex}`;
  timeline.value = `${state.currentTime}`;
  togglePlayback.textContent = state.playing ? "Pause" : "Play";
  renderStats(state.frameIndex);
}

async function loadReplay(file: File): Promise<void> {
  if (unsubscribe) {
    unsubscribe();
    unsubscribe = null;
  }
  replayPlayer?.destroy();
  replayPlayer = null;

  await wasmInit();
  const bytes = new Uint8Array(await file.arrayBuffer());

  const { replay } = await loadReplayFromBytes(bytes);
  statsTimeline = get_stats_timeline(bytes) as unknown as StatsTimeline;

  replayPlayer = new ReplayPlayer(viewport, replay, {
    initialCameraDistanceScale: 2.25,
  });

  // Clean up previous overlays
  roleOverlay?.dispose();
  removeRenderHook?.();

  // Set up overlays
  roleOverlay = new RoleOverlay(replayPlayer.sceneState, replay);
  removeRenderHook = replayPlayer.onBeforeRender((info) => {
    roleOverlay?.update(info);
  });

  createZoneBoundaryLines(
    replayPlayer.sceneState.scene,
    replayPlayer.options.fieldScale ?? 1,
  );

  unsubscribe = replayPlayer.subscribe(onStateChange);

  timeline.min = "0";
  timeline.max = `${replay.duration}`;

  togglePlayback.disabled = false;
  playbackRate.disabled = false;
  timeline.disabled = false;
}

fileInput.addEventListener("change", async () => {
  const file = fileInput.files?.[0];
  if (file) {
    try {
      await loadReplay(file);
    } catch (error) {
      console.error("Failed to load replay:", error);
    }
  }
});

togglePlayback.addEventListener("click", () => replayPlayer?.togglePlayback());
playbackRate.addEventListener("change", () => replayPlayer?.setPlaybackRate(Number(playbackRate.value)));
timeline.addEventListener("input", () => replayPlayer?.seek(Number(timeline.value)));
