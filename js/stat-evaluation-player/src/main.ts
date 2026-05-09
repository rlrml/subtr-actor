import "./styles.css";
import {
  createBallchasingOverlayPlugin,
  createBoostPadOverlayPlugin,
  createBoostPickupAnimationPlugin,
  createTimelineOverlayPlugin,
  ReplayPlayer,
} from "subtr-actor-player";
import type {
  BoostPickupAnimationPickup,
  CameraSettings,
  ReplayCameraViewMode,
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
  hasBoostPickupAnimationTimelineMatch,
  RELATIVE_POSITIONING_MODULE_ID,
  ROLE_LABELS,
} from "./statModules.ts";
import type { StatModule, StatModuleContext } from "./statModules.ts";
import {
  createStatsFrameLookup,
  getStatsFrameForReplayFrame,
} from "./statsTimeline.ts";
import type { StatsFrame, StatsTimeline } from "./statsTimeline.ts";
import {
  countEnabledTimelineEvents,
  filterReplayTimelineEvents,
} from "./timelineMarkers.ts";
import {
  formatReplayLoadProgress,
  loadReplayBundleInWorker,
} from "./replayLoader.ts";
import {
  getReplayFileNameFromUrl,
  getReplayUrlFromSearch,
} from "./replayUrl.ts";
import { StatMonitor } from "./statMonitor.ts";

const DEFAULT_CAMERA_DISTANCE_SCALE = 2.25;
const CAMERA_VIEW_MODES: ReplayCameraViewMode[] = [
  "free",
  "follow",
];

let replayPlayer: ReplayPlayer | null = null;
let timelineOverlay: TimelineOverlayPlugin | null = null;
let statsTimeline: StatsTimeline | null = null;
let statsFrameLookup: Map<number, StatsFrame> | null = null;
let unsubscribe: (() => void) | null = null;
let removeRenderHook: (() => void) | null = null;
let statMonitor: StatMonitor | null = null;

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
  refreshTimelineRanges() {
    syncTimelineRanges();
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
let statMonitorOverlay!: HTMLDivElement;
let playbackRate!: HTMLSelectElement;
let attachedPlayer!: HTMLSelectElement;
let cameraViewFreeButton!: HTMLButtonElement;
let cameraViewFollowButton!: HTMLButtonElement;
let cameraViewOverheadButton!: HTMLButtonElement;
let cameraViewSideButton!: HTMLButtonElement;
let cameraDistance!: HTMLInputElement;
let cameraDistanceReadout!: HTMLElement;
let customCameraSettings!: HTMLInputElement;
let customCameraFov!: HTMLInputElement;
let customCameraHeight!: HTMLInputElement;
let customCameraPitch!: HTMLInputElement;
let customCameraDistance!: HTMLInputElement;
let customCameraStiffness!: HTMLInputElement;
let customCameraSwivelSpeed!: HTMLInputElement;
let customCameraTransitionSpeed!: HTMLInputElement;
let customCameraFovReadout!: HTMLElement;
let customCameraHeightReadout!: HTMLElement;
let customCameraPitchReadout!: HTMLElement;
let customCameraDistanceReadout!: HTMLElement;
let customCameraStiffnessReadout!: HTMLElement;
let customCameraSwivelSpeedReadout!: HTMLElement;
let customCameraTransitionSpeedReadout!: HTMLElement;
let ballCam!: HTMLInputElement;
let showFollowedPlayerOverlay!: HTMLInputElement;
let showBoostPickupAnimation!: HTMLInputElement;
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
let statMonitorSlotZero!: HTMLElement;
let statMonitorSlotOne!: HTMLElement;
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
let statsRenderCacheKey: string | null = null;
let focusedPlayerOverlayCacheKey: string | null = null;

interface ReplayInputSource {
  name: string;
  preparingStatus: string;
  readBytes(): Promise<Uint8Array>;
}

function getActiveModuleSignature(): string {
  return activeModules.map((mod) => mod.id).join("|");
}

function clearRenderCaches(): void {
  statsRenderCacheKey = null;
  focusedPlayerOverlayCacheKey = null;
}

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
  clearRenderCaches();
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
  clearRenderCaches();
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
    if (!mod.getTimelineRanges) {
      continue;
    }

    timelineRangeSourceRemovers.set(
      mod.id,
      timelineOverlay.addRangeSource(() => mod.getTimelineRanges?.(ctx) ?? []),
    );
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

function getFallbackCameraSettings(): Required<CameraSettings> {
  return {
    fov: 110,
    height: 100,
    pitch: -4,
    distance: 270,
    stiffness: 0,
    swivelSpeed: 1,
    transitionSpeed: 1,
  };
}

function getAttachedPlayerCameraSettings(
  attachedPlayerId: string | null,
): CameraSettings | null {
  if (!replayPlayer || attachedPlayerId === null) {
    return null;
  }

  return replayPlayer.replay.players.find(
    (candidate) => candidate.id === attachedPlayerId,
  )?.cameraSettings ?? null;
}

function getEffectiveCameraSettings(state: ReplayPlayerState): CameraSettings {
  return {
    ...getFallbackCameraSettings(),
    ...(getAttachedPlayerCameraSettings(state.attachedPlayerId) ?? {}),
    ...(state.customCameraSettings ?? {}),
  };
}

function readCustomCameraSettings(): CameraSettings {
  return {
    fov: Number(customCameraFov.value),
    height: Number(customCameraHeight.value),
    pitch: Number(customCameraPitch.value),
    distance: Number(customCameraDistance.value),
    stiffness: Number(customCameraStiffness.value),
    swivelSpeed: Number(customCameraSwivelSpeed.value),
    transitionSpeed: Number(customCameraTransitionSpeed.value),
  };
}

function setCameraSettingControlsEnabled(enabled: boolean): void {
  customCameraFov.disabled = !enabled;
  customCameraHeight.disabled = !enabled;
  customCameraPitch.disabled = !enabled;
  customCameraDistance.disabled = !enabled;
  customCameraStiffness.disabled = !enabled;
  customCameraSwivelSpeed.disabled = !enabled;
  customCameraTransitionSpeed.disabled = !enabled;
}

function syncCustomCameraSettingControls(settings: CameraSettings): void {
  const fallback = getFallbackCameraSettings();
  const fov = settings.fov ?? fallback.fov;
  const height = settings.height ?? fallback.height;
  const pitch = settings.pitch ?? fallback.pitch;
  const distance = settings.distance ?? fallback.distance;
  const stiffness = settings.stiffness ?? fallback.stiffness;
  const swivelSpeed = settings.swivelSpeed ?? fallback.swivelSpeed;
  const transitionSpeed = settings.transitionSpeed ?? fallback.transitionSpeed;

  customCameraFov.value = `${fov}`;
  customCameraHeight.value = `${height}`;
  customCameraPitch.value = `${pitch}`;
  customCameraDistance.value = `${distance}`;
  customCameraStiffness.value = `${stiffness}`;
  customCameraSwivelSpeed.value = `${swivelSpeed}`;
  customCameraTransitionSpeed.value = `${transitionSpeed}`;

  customCameraFovReadout.textContent = formatSetting(fov, "", 0);
  customCameraHeightReadout.textContent = formatSetting(height, "", 0);
  customCameraPitchReadout.textContent = formatSetting(pitch, "", 0);
  customCameraDistanceReadout.textContent = formatSetting(distance, "", 0);
  customCameraStiffnessReadout.textContent = formatSetting(stiffness, "", 2);
  customCameraSwivelSpeedReadout.textContent = formatSetting(swivelSpeed, "", 1);
  customCameraTransitionSpeedReadout.textContent = formatSetting(
    transitionSpeed,
    "",
    2,
  );
}

function setTransportEnabled(enabled: boolean): void {
  togglePlayback.disabled = !enabled;
  playbackRate.disabled = !enabled;
  attachedPlayer.disabled = !enabled;
  syncCameraModeButtons(enabled ? replayPlayer?.getState() : undefined);
}

function getCameraViewButton(
  mode: ReplayCameraViewMode,
): HTMLButtonElement {
  switch (mode) {
    case "free":
      return cameraViewFreeButton;
    case "follow":
      return cameraViewFollowButton;
  }
}

function syncCameraModeButtons(state?: ReplayPlayerState): void {
  const activeMode = state?.cameraViewMode ?? "free";
  const hasReplay = replayPlayer !== null && state !== undefined;
  const canFollow = (state?.attachedPlayerId ?? null) !== null;

  for (const mode of CAMERA_VIEW_MODES) {
    const button = getCameraViewButton(mode);
    button.disabled = !hasReplay || (mode === "follow" && !canFollow);
    const isActive = mode === activeMode;
    button.dataset.active = isActive ? "true" : "false";
    button.setAttribute("aria-pressed", isActive ? "true" : "false");
  }

  cameraViewOverheadButton.disabled = !hasReplay;
  cameraViewSideButton.disabled = !hasReplay;
  cameraViewOverheadButton.dataset.active = "false";
  cameraViewSideButton.dataset.active = "false";
  cameraViewOverheadButton.setAttribute("aria-pressed", "false");
  cameraViewSideButton.setAttribute("aria-pressed", "false");
}

function syncCameraControlAvailability(state?: ReplayPlayerState): void {
  syncCameraModeButtons(state);
  const hasAttachedCamera = replayPlayer !== null &&
    state?.cameraViewMode === "follow" &&
    (state.attachedPlayerId ?? null) !== null;
  cameraDistance.disabled = !hasAttachedCamera;
  customCameraSettings.disabled = !hasAttachedCamera;
  setCameraSettingControlsEnabled(
    hasAttachedCamera && state?.customCameraSettings !== null,
  );
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

function renderCameraProfile(state?: ReplayPlayerState): void {
  const attachedPlayerId = state?.attachedPlayerId ?? null;
  if (!replayPlayer || state?.cameraViewMode !== "follow" || attachedPlayerId === null) {
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

  const cameraSettings = getEffectiveCameraSettings(state);
  cameraProfileReadout.textContent = state.customCameraSettings === null
    ? player.name
    : `${player.name} custom`;
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

  const cacheKey = `${frameIndex}:${getActiveModuleSignature()}`;
  if (statsRenderCacheKey === cacheKey) {
    return;
  }

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
  statsRenderCacheKey = cacheKey;
}

function renderFocusedPlayerOverlay(state?: ReplayPlayerState): void {
  const ctx = getModuleContext();
  if (!ctx || !state || !showFollowedPlayerOverlay.checked) {
    followedPlayerOverlay.hidden = true;
    followedPlayerOverlay.innerHTML = "";
    focusedPlayerOverlayCacheKey = "hidden";
    return;
  }

  const attachedPlayerId = state.attachedPlayerId;
  if (!attachedPlayerId) {
    followedPlayerOverlay.hidden = true;
    followedPlayerOverlay.innerHTML = "";
    focusedPlayerOverlayCacheKey = "hidden";
    return;
  }

  const player = getStatsPlayerSnapshot(ctx, state.frameIndex, attachedPlayerId);
  if (!player) {
    followedPlayerOverlay.hidden = true;
    followedPlayerOverlay.innerHTML = "";
    focusedPlayerOverlayCacheKey = "hidden";
    return;
  }

  const showRoleIndicator = activeModuleIds.has(RELATIVE_POSITIONING_MODULE_ID);
  const role = showRoleIndicator
    ? getCurrentRole(ctx.replay, attachedPlayerId, state.frameIndex)
    : null;
  const cacheKey = [
    state.frameIndex,
    attachedPlayerId,
    getActiveModuleSignature(),
    showRoleIndicator ? role ?? "none" : "off",
  ].join(":");
  if (focusedPlayerOverlayCacheKey === cacheKey) {
    followedPlayerOverlay.hidden = false;
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
    focusedPlayerOverlayCacheKey = "hidden";
    return;
  }

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
  focusedPlayerOverlayCacheKey = cacheKey;
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
  customCameraSettings.checked = state.customCameraSettings !== null;
  syncCustomCameraSettingControls(getEffectiveCameraSettings(state));
  ballCam.checked = state.ballCamEnabled;
  showBoostPickupAnimation.checked = state.boostPickupAnimationEnabled;
  attachedPlayer.value = state.attachedPlayerId ?? "";
  skipPostGoalTransitions.checked = state.skipPostGoalTransitionsEnabled;
  skipKickoffs.checked = state.skipKickoffsEnabled;
  emptyState.hidden = true;

  syncCameraControlAvailability(state);
  renderCameraProfile(state);
  renderStats(state.frameIndex);
  renderFocusedPlayerOverlay(state);
  statMonitor?.renderFrame(
    statsFrameLookup
      ? getStatsFrameForReplayFrame(statsFrameLookup, state.frameIndex)
      : null,
  );
}

function includeBoostPickupAnimationPickup(
  pickup: BoostPickupAnimationPickup,
): boolean {
  const boostModule = activeModules.find((mod) => mod.id === "boost");
  return boostModule?.includeBoostPickupAnimationPickup?.(pickup) ??
    hasBoostPickupAnimationTimelineMatch(pickup, statsTimeline);
}

function createFileReplaySource(file: File): ReplayInputSource {
  return {
    name: file.name,
    preparingStatus: "Preparing replay...",
    async readBytes() {
      return new Uint8Array(await file.arrayBuffer());
    },
  };
}

function createUrlReplaySource(
  url: URL,
  signal: AbortSignal,
): ReplayInputSource {
  return {
    name: getReplayFileNameFromUrl(url),
    preparingStatus: "Fetching replay...",
    async readBytes() {
      const response = await fetch(url, { signal });
      if (!response.ok) {
        const statusText = response.statusText
          ? ` ${response.statusText}`
          : "";
        throw new Error(
          `Failed to fetch replay from ${url.href} (${response.status}${statusText})`,
        );
      }
      return new Uint8Array(await response.arrayBuffer());
    },
  };
}

async function loadReplay(source: ReplayInputSource): Promise<void> {
  statusReadout.textContent = source.preparingStatus;
  fileInput.disabled = true;
  replayLoadModal?.show(source.name, source.preparingStatus);
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
  statMonitor?.setStatsTimeline(null);
  statMonitor?.renderFrame(null);
  clearTimelineEventSources();
  clearTimelineRangeSources();
  clearRenderCaches();
  renderTimelineEventCount();
  renderModuleSettings();

  try {
    const bytes = await source.readBytes();
    statusReadout.textContent = "Parsing replay...";
    replayLoadModal?.show(source.name, "Parsing replay...");
    const loadedReplay = await loadReplayBundleInWorker(bytes, {
      reportEveryNFrames: 100,
      onProgress(progress) {
        statusReadout.textContent = formatReplayLoadProgress(progress);
        replayLoadModal?.update(progress);
      },
    });
    const { replay } = loadedReplay;
    statsTimeline = loadedReplay.statsTimeline;
    statsFrameLookup = createStatsFrameLookup(statsTimeline);
    statMonitor?.setStatsTimeline(statsTimeline);

    timelineOverlay = createTimelineOverlayPlugin({
      replayEvents: (context) =>
        filterReplayTimelineEvents(context.replay, activeModuleIds),
    });

    replayPlayer = new ReplayPlayer(viewport, replay, {
      initialCameraDistanceScale: DEFAULT_CAMERA_DISTANCE_SCALE,
      initialCustomCameraSettings: null,
      initialAttachedPlayerId: null,
      initialBallCamEnabled: false,
      initialBoostPickupAnimationEnabled: showBoostPickupAnimation.checked,
      initialSkipPostGoalTransitionsEnabled: skipPostGoalTransitions.checked,
      initialSkipKickoffsEnabled: skipKickoffs.checked,
      plugins: [
        createBallchasingOverlayPlugin(),
        createBoostPadOverlayPlugin(),
        createBoostPickupAnimationPlugin({
          includePickup: includeBoostPickupAnimationPickup,
        }),
        timelineOverlay,
      ],
    });

    setupActiveModules();
    unsubscribe = replayPlayer.subscribe(renderSnapshot);

    populateAttachedPlayerOptions(replay.players);
    emptyState.hidden = true;
    statusReadout.textContent = `Loaded ${source.name}`;
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

function loadReplayFromLocation(signal: AbortSignal): void {
  let replayUrl: URL | null;
  try {
    replayUrl = getReplayUrlFromSearch(
      window.location.search,
      window.location.href,
    );
  } catch (error) {
    console.error("Invalid replay URL:", error);
    statusReadout.textContent = error instanceof Error
      ? error.message
      : "Invalid replay URL";
    return;
  }

  if (!replayUrl) {
    return;
  }

  void loadReplay(createUrlReplaySource(replayUrl, signal)).catch((error) => {
    if (signal.aborted) {
      return;
    }
    console.error("Failed to load replay URL:", error);
    statusReadout.textContent = error instanceof Error
      ? error.message
      : "Failed to load replay URL";
  });
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
  statMonitorOverlay = mustElement<HTMLDivElement>(
    root,
    "#stat-monitor-overlay",
  );
  playbackRate = mustElement<HTMLSelectElement>(root, "#playback-rate");
  attachedPlayer = mustElement<HTMLSelectElement>(root, "#attached-player");
  cameraViewFreeButton = mustElement<HTMLButtonElement>(
    root,
    "#camera-view-free",
  );
  cameraViewFollowButton = mustElement<HTMLButtonElement>(
    root,
    "#camera-view-follow",
  );
  cameraViewOverheadButton = mustElement<HTMLButtonElement>(
    root,
    "#camera-view-overhead",
  );
  cameraViewSideButton = mustElement<HTMLButtonElement>(root, "#camera-view-side");
  cameraDistance = mustElement<HTMLInputElement>(root, "#camera-distance");
  cameraDistanceReadout = mustElement<HTMLElement>(
    root,
    "#camera-distance-readout",
  );
  customCameraSettings = mustElement<HTMLInputElement>(
    root,
    "#custom-camera-settings",
  );
  customCameraFov = mustElement<HTMLInputElement>(root, "#custom-camera-fov");
  customCameraHeight = mustElement<HTMLInputElement>(
    root,
    "#custom-camera-height",
  );
  customCameraPitch = mustElement<HTMLInputElement>(root, "#custom-camera-pitch");
  customCameraDistance = mustElement<HTMLInputElement>(
    root,
    "#custom-camera-distance",
  );
  customCameraStiffness = mustElement<HTMLInputElement>(
    root,
    "#custom-camera-stiffness",
  );
  customCameraSwivelSpeed = mustElement<HTMLInputElement>(
    root,
    "#custom-camera-swivel-speed",
  );
  customCameraTransitionSpeed = mustElement<HTMLInputElement>(
    root,
    "#custom-camera-transition-speed",
  );
  customCameraFovReadout = mustElement<HTMLElement>(
    root,
    "#custom-camera-fov-readout",
  );
  customCameraHeightReadout = mustElement<HTMLElement>(
    root,
    "#custom-camera-height-readout",
  );
  customCameraPitchReadout = mustElement<HTMLElement>(
    root,
    "#custom-camera-pitch-readout",
  );
  customCameraDistanceReadout = mustElement<HTMLElement>(
    root,
    "#custom-camera-distance-readout",
  );
  customCameraStiffnessReadout = mustElement<HTMLElement>(
    root,
    "#custom-camera-stiffness-readout",
  );
  customCameraSwivelSpeedReadout = mustElement<HTMLElement>(
    root,
    "#custom-camera-swivel-speed-readout",
  );
  customCameraTransitionSpeedReadout = mustElement<HTMLElement>(
    root,
    "#custom-camera-transition-speed-readout",
  );
  ballCam = mustElement<HTMLInputElement>(root, "#ball-cam");
  showFollowedPlayerOverlay = mustElement<HTMLInputElement>(
    root,
    "#show-followed-player-overlay",
  );
  showBoostPickupAnimation = mustElement<HTMLInputElement>(
    root,
    "#show-boost-pickup-animation",
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
  statMonitorSlotZero = mustElement<HTMLElement>(root, "#stat-monitor-slot-0");
  statMonitorSlotOne = mustElement<HTMLElement>(root, "#stat-monitor-slot-1");
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
  statMonitor = new StatMonitor(
    [statMonitorSlotZero, statMonitorSlotOne],
    statMonitorOverlay,
  );

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
    statMonitor?.destroy();
    statMonitor = null;
    clearTimelineEventSources();
    clearTimelineRangeSources();
    clearRenderCaches();
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
      await loadReplay(createFileReplaySource(file));
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

  customCameraSettings.addEventListener("change", () => {
    replayPlayer?.setCustomCameraSettings(
      customCameraSettings.checked ? readCustomCameraSettings() : null,
    );
  }, { signal: listeners.signal });

  for (const input of [
    customCameraFov,
    customCameraHeight,
    customCameraPitch,
    customCameraDistance,
    customCameraStiffness,
    customCameraSwivelSpeed,
    customCameraTransitionSpeed,
  ]) {
    input.addEventListener("input", () => {
      const settings = readCustomCameraSettings();
      syncCustomCameraSettingControls(settings);
      replayPlayer?.setCustomCameraSettings(settings);
    }, { signal: listeners.signal });
  }

  attachedPlayer.addEventListener("change", () => {
    replayPlayer?.setAttachedPlayer(attachedPlayer.value || null);
  }, { signal: listeners.signal });

  cameraViewFreeButton.addEventListener("click", () => {
    replayPlayer?.setCameraViewMode("free");
  }, { signal: listeners.signal });

  cameraViewFollowButton.addEventListener("click", () => {
    replayPlayer?.setCameraViewMode("follow");
  }, { signal: listeners.signal });

  cameraViewOverheadButton.addEventListener("click", () => {
    replayPlayer?.setFreeCameraPreset("overhead");
  }, { signal: listeners.signal });

  cameraViewSideButton.addEventListener("click", () => {
    replayPlayer?.setFreeCameraPreset("side");
  }, { signal: listeners.signal });

  ballCam.addEventListener("change", () => {
    replayPlayer?.setBallCamEnabled(ballCam.checked);
  }, { signal: listeners.signal });

  showFollowedPlayerOverlay.addEventListener("change", () => {
    renderFocusedPlayerOverlay(replayPlayer?.getState());
  }, { signal: listeners.signal });

  showBoostPickupAnimation.addEventListener("change", () => {
    replayPlayer?.setBoostPickupAnimationEnabled(
      showBoostPickupAnimation.checked,
    );
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
  renderCameraProfile();
  syncCameraModeButtons();
  renderTimelineEventCount();
  loadReplayFromLocation(listeners.signal);

  return {
    root,
    destroy: cleanup,
  };
}
