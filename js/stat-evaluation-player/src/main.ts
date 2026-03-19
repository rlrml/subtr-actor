import "./styles.css";
import {
  createBallchasingOverlayPlugin,
  createBoostPadOverlayPlugin,
  createTimelineOverlayPlugin,
  ReplayPlayer,
} from "subtr-actor-player";
import type {
  ReplayPlayerState,
  ReplayPlayerTrack,
  TimelineOverlayPlugin,
} from "subtr-actor-player";
import { getAppTemplate } from "./appTemplate.ts";
import { createReplayLoadModal } from "./replayLoadModal.ts";
import type { ReplayLoadModalController } from "./replayLoadModal.ts";
import {
  createStatModules,
  getCurrentRole,
  getStatsPlayerSnapshot,
  getTeamClass,
  RELATIVE_POSITIONING_MODULE_ID,
  ROLE_LABELS,
} from "./statModules.ts";
import type { StatModule, StatModuleContext } from "./statModules.ts";
import { createStatsFrameLookup } from "./statsTimeline.ts";
import type { StatsFrame, StatsTimeline } from "./statsTimeline.ts";
import {
  countEnabledTimelineEvents,
  filterReplayTimelineEvents,
} from "./timelineMarkers.ts";
import {
  formatReplayLoadProgress,
  loadReplayBundleInWorker,
} from "./replayLoader.ts";

const DEFAULT_CAMERA_DISTANCE_SCALE = 2.25;

let replayPlayer: ReplayPlayer | null = null;
let timelineOverlay: TimelineOverlayPlugin | null = null;
let statsTimeline: StatsTimeline | null = null;
let statsFrameLookup: Map<number, StatsFrame> | null = null;
let unsubscribe: (() => void) | null = null;
let removeRenderHook: (() => void) | null = null;

const timelineSourceRemovers = new Map<string, () => void>();
const timelineRangeSourceRemovers = new Map<string, () => void>();

const MODULES = createStatModules({
  rerenderCurrentState() {
    if (!replayPlayer) {
      return;
    }

    const state = replayPlayer.getState();
    renderStats(state.frameIndex);
    renderFocusedPlayerOverlay(state);
  },
});

let activeModules: StatModule[] = [];
let activeModuleIds = new Set<string>();

export interface StatEvaluationPlayerHandle {
  readonly root: HTMLElement;
  destroy(): void;
}

let appRoot: HTMLElement | null = null;
let fileInput!: HTMLInputElement;
let viewport!: HTMLDivElement;
let emptyState!: HTMLDivElement;
let togglePlayback!: HTMLButtonElement;
let followedPlayerOverlay!: HTMLDivElement;
let playbackRate!: HTMLSelectElement;
let attachedPlayer!: HTMLSelectElement;
let cameraDistance!: HTMLInputElement;
let cameraDistanceReadout!: HTMLElement;
let ballCam!: HTMLInputElement;
let showFollowedPlayerOverlay!: HTMLInputElement;
let moduleSummaryEl!: HTMLDivElement;
let moduleSettingsEl!: HTMLDivElement;
let timeReadout!: HTMLElement;
let frameReadout!: HTMLElement;
let durationReadout!: HTMLElement;
let playbackStatusReadout!: HTMLElement;
let statusReadout!: HTMLElement;
let playersReadout!: HTMLElement;
let framesReadout!: HTMLElement;
let eventsReadout!: HTMLElement;
let playerStatsEl!: HTMLDivElement;
let cameraProfileReadout!: HTMLElement;
let cameraFovReadout!: HTMLElement;
let cameraHeightReadout!: HTMLElement;
let cameraPitchReadout!: HTMLElement;
let cameraBaseDistanceReadout!: HTMLElement;
let cameraStiffnessReadout!: HTMLElement;
let skipPostGoalTransitions!: HTMLInputElement;
let replayLoadModal: ReplayLoadModalController | null = null;
let skipKickoffs!: HTMLInputElement;
let currentMountCleanup: (() => void) | null = null;

function getModuleContext(): StatModuleContext | null {
  if (!replayPlayer || !statsTimeline || !statsFrameLookup) {
    return null;
  }

  return {
    player: replayPlayer,
    replay: replayPlayer.replay,
    statsTimeline,
    statsFrameLookup,
    fieldScale: replayPlayer.options.fieldScale ?? 1,
  };
}

function setupActiveModules(): void {
  teardownActiveModules();

  const ctx = getModuleContext();
  if (!ctx) return;

  activeModules = MODULES.filter((mod) => activeModuleIds.has(mod.id));
  for (const mod of activeModules) {
    mod.setup(ctx);
  }

  removeRenderHook = ctx.player.onBeforeRender((info) => {
    for (const mod of activeModules) {
      mod.onBeforeRender(info);
    }
  });

  syncTimelineEvents();
  syncTimelineRanges();
}

function teardownActiveModules(): void {
  removeRenderHook?.();
  removeRenderHook = null;
  clearTimelineEventSources();
  clearTimelineRangeSources();

  for (const mod of activeModules) {
    mod.teardown();
  }
  activeModules = [];
}

function toggleModule(id: string, enabled: boolean): void {
  if (enabled) {
    activeModuleIds.add(id);
  } else {
    activeModuleIds.delete(id);
  }

  setupActiveModules();
  renderModuleSummary();
  renderModuleSettings();
  if (replayPlayer) {
    const state = replayPlayer.getState();
    renderStats(state.frameIndex);
    renderFocusedPlayerOverlay(state);
  }
  renderTimelineEventCount();
}

function clearTimelineEventSources(): void {
  for (const removeSource of timelineSourceRemovers.values()) {
    removeSource();
  }
  timelineSourceRemovers.clear();
}

function clearTimelineRangeSources(): void {
  for (const removeSource of timelineRangeSourceRemovers.values()) {
    removeSource();
  }
  timelineRangeSourceRemovers.clear();
}

function syncTimelineEvents(): void {
  clearTimelineEventSources();

  const ctx = getModuleContext();
  if (!timelineOverlay || !ctx) {
    return;
  }

  for (const mod of activeModules) {
    const events = mod.getTimelineEvents?.(ctx);
    if (!events || events.length === 0) {
      continue;
    }

    timelineSourceRemovers.set(mod.id, timelineOverlay.addEventSource(events));
  }

  timelineOverlay.refreshEvents();
}

function syncTimelineRanges(): void {
  clearTimelineRangeSources();

  const ctx = getModuleContext();
  if (!timelineOverlay || !ctx) {
    return;
  }

  for (const mod of activeModules) {
    const ranges = mod.getTimelineRanges?.(ctx);
    if (!ranges || ranges.length === 0) {
      continue;
    }

    timelineRangeSourceRemovers.set(mod.id, timelineOverlay.addRangeSource(ranges));
  }

  timelineOverlay.refreshRanges();
}

function renderTimelineEventCount(): void {
  if (!replayPlayer || !statsTimeline) {
    eventsReadout.textContent = "--";
    return;
  }

  eventsReadout.textContent = `${countEnabledTimelineEvents(
    activeModuleIds,
    replayPlayer.replay,
    statsTimeline,
  )}`;
}

function mustElement<T extends HTMLElement>(
  root: ParentNode,
  selector: string,
): T {
  const element = root.querySelector(selector);
  if (!(element instanceof HTMLElement)) {
    throw new Error(`Missing element for selector: ${selector}`);
  }

  return element as T;
}

function renderModuleSummary(): void {
  moduleSummaryEl.replaceChildren();

  for (const mod of MODULES) {
    const active = activeModuleIds.has(mod.id);
    const item = document.createElement("button");
    item.type = "button";
    item.className = "module-summary-item";
    item.dataset.active = active ? "true" : "false";
    item.setAttribute("aria-pressed", active ? "true" : "false");
    item.addEventListener("click", () => {
      toggleModule(mod.id, !activeModuleIds.has(mod.id));
    });

    const name = document.createElement("span");
    name.textContent = mod.label;

    const state = document.createElement("strong");
    state.textContent = active ? "On" : "Off";

    item.append(name, state);
    moduleSummaryEl.append(item);
  }
}

function renderModuleSettings(): void {
  moduleSettingsEl.replaceChildren();

  const ctx = getModuleContext();
  const panels = activeModules
    .map((mod) => mod.renderSettings?.(ctx) ?? null)
    .filter((panel): panel is HTMLElement => panel instanceof HTMLElement);

  if (panels.length === 0) {
    moduleSettingsEl.hidden = true;
    return;
  }

  moduleSettingsEl.hidden = false;
  moduleSettingsEl.append(...panels);
}

function formatSetting(
  value: number | undefined,
  suffix = "",
  digits = 0,
): string {
  if (value === undefined || Number.isNaN(value)) {
    return "--";
  }

  return `${value.toFixed(digits)}${suffix}`;
}

function setTransportEnabled(enabled: boolean): void {
  togglePlayback.disabled = !enabled;
  playbackRate.disabled = !enabled;
  attachedPlayer.disabled = !enabled;
}

function syncCameraControlAvailability(state?: ReplayPlayerState): void {
  const attached = state?.attachedPlayerId ?? null;
  const hasAttachedCamera = replayPlayer !== null && attached !== null;
  cameraDistance.disabled = !hasAttachedCamera;
  ballCam.disabled = !hasAttachedCamera;
}

function populateAttachedPlayerOptions(players: ReplayPlayerTrack[]): void {
  attachedPlayer.replaceChildren();
  attachedPlayer.append(new Option("Free camera", ""));

  for (const player of players) {
    attachedPlayer.append(
      new Option(
        `${player.name} (${player.isTeamZero ? "Blue" : "Orange"})`,
        player.id,
      ),
    );
  }
}

function renderCameraProfile(attachedPlayerId: string | null): void {
  if (!replayPlayer || attachedPlayerId === null) {
    cameraProfileReadout.textContent = "Free camera";
    cameraFovReadout.textContent = "--";
    cameraHeightReadout.textContent = "--";
    cameraPitchReadout.textContent = "--";
    cameraBaseDistanceReadout.textContent = "--";
    cameraStiffnessReadout.textContent = "--";
    return;
  }

  const player = replayPlayer.replay.players.find(
    (candidate) => candidate.id === attachedPlayerId,
  );
  if (!player) {
    cameraProfileReadout.textContent = "Unknown";
    cameraFovReadout.textContent = "--";
    cameraHeightReadout.textContent = "--";
    cameraPitchReadout.textContent = "--";
    cameraBaseDistanceReadout.textContent = "--";
    cameraStiffnessReadout.textContent = "--";
    return;
  }

  const { cameraSettings } = player;
  cameraProfileReadout.textContent = player.name;
  cameraFovReadout.textContent = formatSetting(cameraSettings.fov, "", 0);
  cameraHeightReadout.textContent = formatSetting(cameraSettings.height, "", 0);
  cameraPitchReadout.textContent = formatSetting(cameraSettings.pitch, "", 0);
  cameraBaseDistanceReadout.textContent = formatSetting(
    cameraSettings.distance,
    "",
    0,
  );
  cameraStiffnessReadout.textContent = formatSetting(
    cameraSettings.stiffness,
    "",
    2,
  );
}

function renderStats(frameIndex: number): void {
  const ctx = getModuleContext();
  if (!ctx) return;

  const sections = activeModules
    .map((mod) => {
      const html = mod.renderStats(frameIndex, ctx);
      if (!html) return "";
      return `<section class="stat-module-section">
        <div class="stat-module-label">${mod.label}</div>
        <div class="player-stats-grid">${html}</div>
      </section>`;
    })
    .filter(Boolean);

  playerStatsEl.innerHTML = sections.length > 0
    ? sections.join("")
    : "No stat modules active.";
}

function renderFocusedPlayerOverlay(state?: ReplayPlayerState): void {
  const ctx = getModuleContext();
  if (!ctx || !state || !showFollowedPlayerOverlay.checked) {
    followedPlayerOverlay.hidden = true;
    followedPlayerOverlay.innerHTML = "";
    return;
  }

  const attachedPlayerId = state.attachedPlayerId;
  if (!attachedPlayerId) {
    followedPlayerOverlay.hidden = true;
    followedPlayerOverlay.innerHTML = "";
    return;
  }

  const player = getStatsPlayerSnapshot(ctx, state.frameIndex, attachedPlayerId);
  if (!player) {
    followedPlayerOverlay.hidden = true;
    followedPlayerOverlay.innerHTML = "";
    return;
  }

  const sections = activeModules.map((mod) => {
    const body = mod.renderFocusedPlayerStats(
      attachedPlayerId,
      state.frameIndex,
      ctx,
    );
    if (!body) return "";

    return `<section class="focused-player-module">
      <div class="focused-player-module-label">${mod.label}</div>
      <div class="focused-player-module-body">${body}</div>
    </section>`;
  }).filter(Boolean);

  if (sections.length === 0) {
    followedPlayerOverlay.hidden = true;
    followedPlayerOverlay.innerHTML = "";
    return;
  }

  const showRoleIndicator = activeModuleIds.has(RELATIVE_POSITIONING_MODULE_ID);
  const role = showRoleIndicator
    ? getCurrentRole(ctx.replay, attachedPlayerId, state.frameIndex)
    : null;
  followedPlayerOverlay.innerHTML = `
    <div class="followed-player-overlay-card ${getTeamClass(player.is_team_0)}">
      <div class="followed-player-overlay-header">
        <div class="followed-player-overlay-title">
          <p class="followed-player-overlay-eyebrow">Follow cam</p>
          <div class="followed-player-overlay-name-row">
            <span class="player-name">${player.name}</span>
            ${role ? `<span class="role-indicator role-${role}">${ROLE_LABELS[role]}</span>` : ""}
          </div>
        </div>
        <strong class="followed-player-overlay-team">
          ${player.is_team_0 ? "Blue" : "Orange"}
        </strong>
      </div>
      <div class="followed-player-overlay-body">${sections.join("")}</div>
    </div>
  `;
  followedPlayerOverlay.hidden = false;
}

function renderSnapshot(state: ReplayPlayerState): void {
  timeReadout.textContent = `${state.currentTime.toFixed(2)}s`;
  frameReadout.textContent = `${state.frameIndex}`;
  durationReadout.textContent = `${state.duration.toFixed(2)}s`;
  playbackStatusReadout.textContent = state.playing ? "Playing" : "Paused";
  togglePlayback.textContent = state.playing ? "Pause" : "Play";
  playbackRate.value = `${state.speed}`;
  cameraDistance.value = `${state.cameraDistanceScale}`;
  cameraDistanceReadout.textContent = `${state.cameraDistanceScale.toFixed(2)}x`;
  ballCam.checked = state.ballCamEnabled;
  attachedPlayer.value = state.attachedPlayerId ?? "";
  skipPostGoalTransitions.checked = state.skipPostGoalTransitionsEnabled;
  skipKickoffs.checked = state.skipKickoffsEnabled;
  emptyState.hidden = true;

  syncCameraControlAvailability(state);
  renderCameraProfile(state.attachedPlayerId);
  renderStats(state.frameIndex);
  renderFocusedPlayerOverlay(state);
}

async function loadReplay(file: File): Promise<void> {
  statusReadout.textContent = "Parsing replay...";
  fileInput.disabled = true;
  replayLoadModal?.show(file.name, "Parsing replay...");
  setTransportEnabled(false);
  syncCameraControlAvailability();
  emptyState.hidden = false;

  if (unsubscribe) {
    unsubscribe();
    unsubscribe = null;
  }

  teardownActiveModules();
  replayPlayer?.destroy();
  replayPlayer = null;
  timelineOverlay = null;
  statsTimeline = null;
  statsFrameLookup = null;
  clearTimelineEventSources();
  clearTimelineRangeSources();
  renderTimelineEventCount();
  renderModuleSettings();

  try {
    const bytes = new Uint8Array(await file.arrayBuffer());
    const loadedReplay = await loadReplayBundleInWorker(bytes, {
      reportEveryNFrames: 500,
      onProgress(progress) {
        statusReadout.textContent = formatReplayLoadProgress(progress);
        replayLoadModal?.update(progress);
      },
    });
    const { replay } = loadedReplay;
    statsTimeline = loadedReplay.statsTimeline;
    statsFrameLookup = createStatsFrameLookup(statsTimeline);

    timelineOverlay = createTimelineOverlayPlugin({
      replayEvents: (context) =>
        filterReplayTimelineEvents(context.replay, activeModuleIds),
    });

    replayPlayer = new ReplayPlayer(viewport, replay, {
      initialCameraDistanceScale: DEFAULT_CAMERA_DISTANCE_SCALE,
      initialAttachedPlayerId: null,
      initialBallCamEnabled: false,
      initialSkipPostGoalTransitionsEnabled: skipPostGoalTransitions.checked,
      initialSkipKickoffsEnabled: skipKickoffs.checked,
      plugins: [
        createBallchasingOverlayPlugin(),
        createBoostPadOverlayPlugin(),
        timelineOverlay,
      ],
    });

    setupActiveModules();
    unsubscribe = replayPlayer.subscribe(renderSnapshot);

    populateAttachedPlayerOptions(replay.players);
    emptyState.hidden = true;
    statusReadout.textContent = `Loaded ${file.name}`;
    playersReadout.textContent = replay.players.map((player) => player.name)
      .join(", ");
    framesReadout.textContent = `${replay.frameCount}`;
    renderTimelineEventCount();
    setTransportEnabled(true);
    syncCameraControlAvailability(replayPlayer.getState());
    renderSnapshot(replayPlayer.getState());
    renderModuleSettings();
    replayLoadModal?.hide();
  } catch (error) {
    replayLoadModal?.hide();
    throw error;
  } finally {
    fileInput.disabled = false;
  }
}

export function mountStatEvaluationPlayer(
  root: HTMLElement,
): StatEvaluationPlayerHandle {
  currentMountCleanup?.();

  root.innerHTML = getAppTemplate(DEFAULT_CAMERA_DISTANCE_SCALE);
  appRoot = root;
  replayLoadModal = createReplayLoadModal(root);

  fileInput = mustElement<HTMLInputElement>(root, "#replay-file");
  viewport = mustElement<HTMLDivElement>(root, "#viewport");
  emptyState = mustElement<HTMLDivElement>(root, "#empty-state");
  togglePlayback = mustElement<HTMLButtonElement>(root, "#toggle-playback");
  followedPlayerOverlay = mustElement<HTMLDivElement>(
    root,
    "#followed-player-overlay",
  );
  playbackRate = mustElement<HTMLSelectElement>(root, "#playback-rate");
  attachedPlayer = mustElement<HTMLSelectElement>(root, "#attached-player");
  cameraDistance = mustElement<HTMLInputElement>(root, "#camera-distance");
  cameraDistanceReadout = mustElement<HTMLElement>(
    root,
    "#camera-distance-readout",
  );
  ballCam = mustElement<HTMLInputElement>(root, "#ball-cam");
  showFollowedPlayerOverlay = mustElement<HTMLInputElement>(
    root,
    "#show-followed-player-overlay",
  );
  moduleSummaryEl = mustElement<HTMLDivElement>(root, "#module-summary");
  moduleSettingsEl = mustElement<HTMLDivElement>(root, "#module-settings");
  timeReadout = mustElement<HTMLElement>(root, "#time-readout");
  frameReadout = mustElement<HTMLElement>(root, "#frame-readout");
  durationReadout = mustElement<HTMLElement>(root, "#duration-readout");
  playbackStatusReadout = mustElement<HTMLElement>(
    root,
    "#playback-status-readout",
  );
  statusReadout = mustElement<HTMLElement>(root, "#status-readout");
  playersReadout = mustElement<HTMLElement>(root, "#players-readout");
  framesReadout = mustElement<HTMLElement>(root, "#frames-readout");
  eventsReadout = mustElement<HTMLElement>(root, "#events-readout");
  playerStatsEl = mustElement<HTMLDivElement>(root, "#player-stats");
  cameraProfileReadout = mustElement<HTMLElement>(
    root,
    "#camera-profile-readout",
  );
  cameraFovReadout = mustElement<HTMLElement>(root, "#camera-fov-readout");
  cameraHeightReadout = mustElement<HTMLElement>(root, "#camera-height-readout");
  cameraPitchReadout = mustElement<HTMLElement>(root, "#camera-pitch-readout");
  cameraBaseDistanceReadout = mustElement<HTMLElement>(
    root,
    "#camera-base-distance-readout",
  );
  cameraStiffnessReadout = mustElement<HTMLElement>(
    root,
    "#camera-stiffness-readout",
  );
  skipPostGoalTransitions = mustElement<HTMLInputElement>(
    root,
    "#skip-post-goal-transitions",
  );
  skipKickoffs = mustElement<HTMLInputElement>(root, "#skip-kickoffs");

  const listeners = new AbortController();
  const cleanup = () => {
    listeners.abort();
    unsubscribe?.();
    unsubscribe = null;
    teardownActiveModules();
    replayPlayer?.destroy();
    replayPlayer = null;
    timelineOverlay = null;
    statsTimeline = null;
    statsFrameLookup = null;
    clearTimelineEventSources();
    clearTimelineRangeSources();
    activeModules = [];
    replayLoadModal?.destroy();
    replayLoadModal = null;
    activeModuleIds = new Set<string>();
    removeRenderHook = null;
    if (appRoot === root) {
      appRoot = null;
      root.replaceChildren();
    }
    if (currentMountCleanup === cleanup) {
      currentMountCleanup = null;
    }
  };
  currentMountCleanup = cleanup;

  fileInput.addEventListener("change", async () => {
    const file = fileInput.files?.[0];
    if (!file) return;

    try {
      await loadReplay(file);
    } catch (error) {
      console.error("Failed to load replay:", error);
      statusReadout.textContent =
        error instanceof Error ? error.message : "Failed to load replay";
    }
  }, { signal: listeners.signal });

  togglePlayback.addEventListener("click", () => {
    replayPlayer?.togglePlayback();
  }, { signal: listeners.signal });

  playbackRate.addEventListener("change", () => {
    replayPlayer?.setPlaybackRate(Number(playbackRate.value));
  }, { signal: listeners.signal });

  cameraDistance.addEventListener("input", () => {
    replayPlayer?.setCameraDistanceScale(Number(cameraDistance.value));
  }, { signal: listeners.signal });

  attachedPlayer.addEventListener("change", () => {
    replayPlayer?.setAttachedPlayer(attachedPlayer.value || null);
  }, { signal: listeners.signal });

  ballCam.addEventListener("change", () => {
    replayPlayer?.setBallCamEnabled(ballCam.checked);
  }, { signal: listeners.signal });

  showFollowedPlayerOverlay.addEventListener("change", () => {
    renderFocusedPlayerOverlay(replayPlayer?.getState());
  }, { signal: listeners.signal });

  skipPostGoalTransitions.addEventListener("change", () => {
    replayPlayer?.setSkipPostGoalTransitionsEnabled(
      skipPostGoalTransitions.checked,
    );
  }, { signal: listeners.signal });

  skipKickoffs.addEventListener("change", () => {
    replayPlayer?.setSkipKickoffsEnabled(skipKickoffs.checked);
  }, { signal: listeners.signal });

  renderModuleSummary();
  renderModuleSettings();
  renderCameraProfile(null);
  renderTimelineEventCount();

  return {
    root,
    destroy: cleanup,
  };
}
