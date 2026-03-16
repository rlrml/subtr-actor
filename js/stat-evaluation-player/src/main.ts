import "./styles.css";
import {
  createBallchasingOverlayPlugin,
  ReplayPlayer,
  loadReplayFromBytes,
} from "../../player/src/lib.ts";
import type {
  ReplayModel,
  ReplayPlayerState,
  FrameRenderInfo,
} from "../../player/src/lib.ts";
import type { ReplayScene } from "../../player/src/scene.ts";
import { RoleOverlay, ThresholdZoneOverlay, createZoneBoundaryLines } from "./overlays.ts";

// WASM module - init is the default export, get_stats_timeline is named
import wasmInit, {
  get_stats_timeline,
} from "../../pkg/rl_replay_subtr_actor.js";

// --- Stat types ---

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
  boost?: {
    amount_collected: number;
    amount_collected_big: number;
    amount_collected_small: number;
    amount_stolen: number;
    big_pads_collected: number;
    small_pads_collected: number;
    amount_used_while_supersonic: number;
    time_zero_boost: number;
    time_hundred_boost: number;
    boost_integral: number;
    tracked_time: number;
    [key: string]: unknown;
  };
  [key: string]: unknown;
}

// --- Stat Module interface ---

interface StatModuleContext {
  player: ReplayPlayer;
  replay: ReplayModel;
  statsTimeline: StatsTimeline;
  fieldScale: number;
}

interface StatModule {
  readonly id: string;
  readonly label: string;
  setup(ctx: StatModuleContext): void;
  teardown(): void;
  onBeforeRender(info: FrameRenderInfo): void;
  renderStats(frameIndex: number, ctx: StatModuleContext): string;
}

// --- Positioning module ---

const MOST_BACK_FORWARD_THRESHOLD_Y = 118.0;
type Role = "back" | "forward" | "even" | "mid";
const ROLE_LABELS: Record<Role, string> = {
  back: "Back",
  forward: "Fwd",
  even: "Even",
  mid: "Mid",
};

function getCurrentRole(
  replay: ReplayModel,
  playerId: string,
  frameIndex: number,
): Role {
  const player = replay.players.find((p) => p.id === playerId);
  if (!player) return "mid";
  const frame = player.frames[frameIndex];
  if (!frame?.position) return "mid";

  const isTeamZero = player.isTeamZero;
  const teamRosterCount = replay.players.filter((p) => p.isTeamZero === isTeamZero).length;
  const allYs: number[] = [];
  let normalizedY = 0;

  for (const p of replay.players) {
    if (p.isTeamZero !== isTeamZero) continue;
    const f = p.frames[frameIndex];
    if (!f?.position) continue;
    const ny = isTeamZero ? f.position.y : -f.position.y;
    allYs.push(ny);
    if (p.id === playerId) normalizedY = ny;
  }

  if (teamRosterCount < 2 || allYs.length !== teamRosterCount) return "mid";

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

function createPositioningModule(): StatModule {
  let roleOverlay: RoleOverlay | null = null;
  let thresholdZoneOverlay: ThresholdZoneOverlay | null = null;
  let fieldScale = 1;

  return {
    id: "positioning",
    label: "Positioning",

    setup(ctx) {
      fieldScale = ctx.fieldScale;
      roleOverlay = new RoleOverlay(ctx.player.sceneState, ctx.replay);
      thresholdZoneOverlay = new ThresholdZoneOverlay(
        ctx.player.sceneState.scene,
        ctx.replay,
        fieldScale,
      );
      createZoneBoundaryLines(ctx.player.sceneState.scene, fieldScale);
    },

    teardown() {
      roleOverlay?.dispose();
      thresholdZoneOverlay?.dispose();
      roleOverlay = null;
      thresholdZoneOverlay = null;
    },

    onBeforeRender(info) {
      roleOverlay?.update(info);
      thresholdZoneOverlay?.update(info, fieldScale);
    },

    renderStats(frameIndex, ctx) {
      const statsFrame = ctx.statsTimeline.frames[frameIndex];
      if (!statsFrame) return "";

      return statsFrame.players.map((player) => {
        const pos = player.positioning;
        const pid = Object.entries(player.player_id).map(([k, v]) => `${k}:${v}`).join(",");
        const role = getCurrentRole(ctx.replay, pid, frameIndex);
        const teamClass = player.is_team_0 ? "team-blue" : "team-orange";
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
    },
  };
}

// --- Boost module ---

function createBoostModule(): StatModule {
  return {
    id: "boost",
    label: "Boost",

    setup(ctx) {
      ctx.player.setBoostMeterEnabled(true);
    },

    teardown() {
      // boost meter is on the base player; we'll disable it
    },

    onBeforeRender() {
      // boost meter updates are handled by the base player
    },

    renderStats(frameIndex, ctx) {
      const statsFrame = ctx.statsTimeline.frames[frameIndex];
      if (!statsFrame) return "";

      return statsFrame.players.map((player) => {
        const b = player.boost;
        const teamClass = player.is_team_0 ? "team-blue" : "team-orange";
        const avgBoost = (b && b.tracked_time > 0)
          ? (b.boost_integral / b.tracked_time * 100 / 255).toFixed(0)
          : "?";
        return `<div class="player-card ${teamClass}">
          <div class="player-card-header">
            <span class="player-name">${player.name}</span>
          </div>
          <div class="stat-row"><span class="label">Collected</span><span class="value">${b?.amount_collected?.toFixed(0) ?? "?"}</span></div>
          <div class="stat-row"><span class="label">Amount from big pads</span><span class="value">${b?.amount_collected_big?.toFixed(0) ?? "?"}</span></div>
          <div class="stat-row"><span class="label">Amount from small pads</span><span class="value">${b?.amount_collected_small?.toFixed(0) ?? "?"}</span></div>
          <div class="stat-row"><span class="label">Big pads collected</span><span class="value">${b?.big_pads_collected ?? "?"}</span></div>
          <div class="stat-row"><span class="label">Small pads collected</span><span class="value">${b?.small_pads_collected ?? "?"}</span></div>
          <div class="stat-row"><span class="label">Stolen</span><span class="value">${b?.amount_stolen?.toFixed(0) ?? "?"}</span></div>
          <div class="stat-row"><span class="label">Avg boost</span><span class="value">${avgBoost}%</span></div>
          <div class="stat-row"><span class="label">Time @ 0</span><span class="value">${b?.time_zero_boost?.toFixed(1) ?? "?"}s</span></div>
          <div class="stat-row"><span class="label">Time @ 100</span><span class="value">${b?.time_hundred_boost?.toFixed(1) ?? "?"}s</span></div>
        </div>`;
      }).join("");
    },
  };
}

// --- Module registry and state ---

const ALL_MODULES = [createPositioningModule, createBoostModule];

let activeModules: StatModule[] = [];
let activeModuleIds = new Set<string>(["positioning"]); // positioning on by default
let removeRenderHook: (() => void) | null = null;

let replayPlayer: ReplayPlayer | null = null;
let statsTimeline: StatsTimeline | null = null;
let unsubscribe: (() => void) | null = null;

function getModuleContext(): StatModuleContext | null {
  if (!replayPlayer || !statsTimeline) return null;
  return {
    player: replayPlayer,
    replay: replayPlayer.replay,
    statsTimeline,
    fieldScale: replayPlayer.options.fieldScale ?? 1,
  };
}

function setupActiveModules(): void {
  teardownActiveModules();

  const ctx = getModuleContext();
  if (!ctx) return;

  activeModules = ALL_MODULES
    .filter((factory) => {
      const tmp = factory();
      return activeModuleIds.has(tmp.id);
    })
    .map((factory) => {
      const mod = factory();
      mod.setup(ctx);
      return mod;
    });

  removeRenderHook = ctx.player.onBeforeRender((info) => {
    for (const mod of activeModules) {
      mod.onBeforeRender(info);
    }
  });
}

function teardownActiveModules(): void {
  removeRenderHook?.();
  removeRenderHook = null;
  for (const mod of activeModules) {
    mod.teardown();
  }
  // disable boost meter if boost module was active
  replayPlayer?.setBoostMeterEnabled(false);
  activeModules = [];
}

function toggleModule(id: string, enabled: boolean): void {
  if (enabled) {
    activeModuleIds.add(id);
  } else {
    activeModuleIds.delete(id);
  }
  setupActiveModules();
  // re-render stats immediately
  if (replayPlayer) {
    renderStats(replayPlayer.getState().frameIndex);
  }
}

// --- DOM ---

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
    <div class="module-toggles" id="module-toggles"></div>
    <div id="viewport" class="viewport"></div>
    <div id="stats-panel" class="stats-panel">
      <div id="player-stats" class="player-stats-grid">Load a replay to see stats.</div>
    </div>
  </div>
`;

// Build module toggle checkboxes
const moduleTogglesEl = document.getElementById("module-toggles")!;
for (const factory of ALL_MODULES) {
  const mod = factory();
  const label = document.createElement("label");
  label.className = "module-toggle";
  const checkbox = document.createElement("input");
  checkbox.type = "checkbox";
  checkbox.checked = activeModuleIds.has(mod.id);
  checkbox.addEventListener("change", () => {
    toggleModule(mod.id, checkbox.checked);
  });
  label.appendChild(checkbox);
  label.appendChild(document.createTextNode(` ${mod.label}`));
  moduleTogglesEl.appendChild(label);
}

const fileInput = document.getElementById("replay-file") as HTMLInputElement;
const viewport = document.getElementById("viewport")!;
const togglePlayback = document.getElementById("toggle-playback") as HTMLButtonElement;
const playbackRate = document.getElementById("playback-rate") as HTMLSelectElement;
const timeline = document.getElementById("timeline") as HTMLInputElement;
const timeReadout = document.getElementById("time-readout")!;
const frameReadout = document.getElementById("frame-readout")!;
const playerStatsEl = document.getElementById("player-stats")!;

function renderStats(frameIndex: number): void {
  const ctx = getModuleContext();
  if (!ctx) return;

  const sections = activeModules.map((mod) => {
    const html = mod.renderStats(frameIndex, ctx);
    if (!html) return "";
    return `<div class="stat-module-section">
      <div class="stat-module-label">${mod.label}</div>
      <div class="player-stats-grid">${html}</div>
    </div>`;
  }).filter(Boolean);

  playerStatsEl.innerHTML = sections.length > 0
    ? sections.join("")
    : "No stat modules active.";
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
  teardownActiveModules();
  replayPlayer?.destroy();
  replayPlayer = null;

  await wasmInit();
  const bytes = new Uint8Array(await file.arrayBuffer());

  const { replay } = await loadReplayFromBytes(bytes);
  statsTimeline = get_stats_timeline(bytes) as unknown as StatsTimeline;

  replayPlayer = new ReplayPlayer(viewport, replay, {
    initialCameraDistanceScale: 2.25,
    plugins: [createBallchasingOverlayPlugin()],
  });

  setupActiveModules();

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
