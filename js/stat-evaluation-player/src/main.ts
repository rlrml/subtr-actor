import "./styles.css";
import {
  ReplayPlayer,
  loadReplayFromBytes,
} from "../../player/src/lib.ts";
import type {
  ReplayPlayerState,
} from "../../player/src/lib.ts";
import { RoleOverlay, ThresholdLineOverlay, createZoneBoundaryLines } from "./overlays.ts";

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
      <span id="time-readout" class="readout">0.00s</span>
      <span id="frame-readout" class="readout">0</span>
      <input id="timeline" type="range" min="0" max="0" step="0.01" value="0" disabled style="flex:1" />
    </div>
    <div id="viewport" class="viewport"></div>
    <div id="stats-panel" class="stats-panel">
      <div id="player-stats" class="player-stats-grid">Load a replay to see stats.</div>
    </div>
  </div>
`;

let replayPlayer: ReplayPlayer | null = null;
let statsTimeline: StatsTimeline | null = null;
let unsubscribe: (() => void) | null = null;
let roleOverlay: RoleOverlay | null = null;
let thresholdLineOverlay: ThresholdLineOverlay | null = null;
let removeRenderHook: (() => void) | null = null;

const MOST_BACK_FORWARD_THRESHOLD_Y = 118.0;

const fileInput = document.getElementById("replay-file") as HTMLInputElement;
const viewport = document.getElementById("viewport")!;
const togglePlayback = document.getElementById("toggle-playback") as HTMLButtonElement;
const playbackRate = document.getElementById("playback-rate") as HTMLSelectElement;
const timeline = document.getElementById("timeline") as HTMLInputElement;
const timeReadout = document.getElementById("time-readout")!;
const frameReadout = document.getElementById("frame-readout")!;
const playerStatsEl = document.getElementById("player-stats")!;

type Role = "back" | "forward" | "even" | "mid";

function getCurrentRole(
  playerId: string,
  frameIndex: number,
): Role {
  if (!replayPlayer) return "even";
  const replay = replayPlayer.replay;

  const player = replay.players.find((p) => p.id === playerId);
  if (!player) return "even";
  const frame = player.frames[frameIndex];
  if (!frame?.position) return "even";

  const isTeamZero = player.isTeamZero;
  const teammates = replay.players.filter(
    (p) => p.isTeamZero === isTeamZero && p.id !== playerId,
  );

  const normalizedY = isTeamZero ? frame.position.y : -frame.position.y;

  const allYs = [normalizedY];
  for (const t of teammates) {
    const tf = t.frames[frameIndex];
    if (!tf?.position) continue;
    allYs.push(isTeamZero ? tf.position.y : -tf.position.y);
  }

  const minY = Math.min(...allYs);
  const maxY = Math.max(...allYs);
  const spread = maxY - minY;

  if (spread <= MOST_BACK_FORWARD_THRESHOLD_Y) return "even";

  const nearBack = (normalizedY - minY) <= MOST_BACK_FORWARD_THRESHOLD_Y;
  const nearFront = (maxY - normalizedY) <= MOST_BACK_FORWARD_THRESHOLD_Y;

  if (nearBack && !nearFront) return "back";
  if (nearFront && !nearBack) return "forward";
  return "mid";
}

const ROLE_LABELS: Record<Role, string> = {
  back: "Back",
  forward: "Fwd",
  even: "Even",
  mid: "Mid",
};

function renderStats(frameIndex: number): void {
  if (!statsTimeline) return;

  const statsFrame = statsTimeline.frames[frameIndex];
  if (!statsFrame) return;

  const bluePlayers = statsFrame.players.filter((p) => p.is_team_0);
  const orangePlayers = statsFrame.players.filter((p) => !p.is_team_0);

  function renderTeam(players: PlayerStatsSnapshot[], teamClass: string): string {
    return players.map((player) => {
      const pos = player.positioning;
      const pid = Object.entries(player.player_id).map(([k, v]) => `${k}:${v}`).join(",");
      const role = getCurrentRole(pid, frameIndex);
      return `<div class="player-card ${teamClass}">
        <div class="player-card-header">
          <span class="player-name">${player.name}</span>
          <span class="role-indicator role-${role}">${ROLE_LABELS[role]}</span>
        </div>
        <div class="stat-row"><span class="label">Back</span><span class="value">${pos?.time_most_back?.toFixed(1) ?? "?"}s</span></div>
        <div class="stat-row"><span class="label">Forward</span><span class="value">${pos?.time_most_forward?.toFixed(1) ?? "?"}s</span></div>
        <div class="stat-row"><span class="label">Even</span><span class="value">${pos?.time_even?.toFixed(1) ?? "?"}s</span></div>
      </div>`;
    }).join("");
  }

  playerStatsEl.innerHTML =
    renderTeam(bluePlayers, "team-blue") +
    renderTeam(orangePlayers, "team-orange");
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
  thresholdLineOverlay?.dispose();
  removeRenderHook?.();

  // Set up overlays
  const fs = replayPlayer.options.fieldScale ?? 1;
  roleOverlay = new RoleOverlay(replayPlayer.sceneState, replay);
  thresholdLineOverlay = new ThresholdLineOverlay(
    replayPlayer.sceneState.scene,
    replay,
    fs,
  );
  removeRenderHook = replayPlayer.onBeforeRender((info) => {
    roleOverlay?.update(info);
    thresholdLineOverlay?.update(info, fs);
  });

  createZoneBoundaryLines(replayPlayer.sceneState.scene, fs);

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
