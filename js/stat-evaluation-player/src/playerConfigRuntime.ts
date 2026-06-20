import type { CameraSettings } from "@rlrml/player";
import type { StatsReplayPlayer } from "./statsReplayPlayer.ts";
import { getConfigAdapterSnapshot, type StatsPlayerConfigAdapter } from "./configAdapters.ts";
import {
  DEFAULT_CUSTOM_CAMERA_SETTINGS,
  ballCamModeFromConfig,
  ballCamModeFromState,
  type CameraControlsController,
} from "./cameraControls.ts";
import type { RecordingConfig } from "./playerConfig.ts";
import {
  STATS_PLAYER_CONFIG_VERSION,
  type PlayerCameraConfig,
  type PlayerPlaybackConfig,
  type SingletonWindowConfig,
  type StatsPlayerConfig,
  type StatsPlayerConfigParamSnapshot,
  type StatsWindowConfig,
} from "./playerConfig.ts";
import type { StatModule } from "./statModules.ts";

export interface PlaybackConfigSnapshotOptions {
  replayPlayer: StatsReplayPlayer | null;
  playbackRate: HTMLSelectElement;
  skipPostGoalTransitions: HTMLInputElement;
  skipKickoffs: HTMLInputElement;
}

export interface CameraConfigSnapshotOptions {
  replayPlayer: StatsReplayPlayer | null;
  cameraControlsController: CameraControlsController | null;
}

export interface StatsPlayerConfigSnapshotOptions {
  playback: PlayerPlaybackConfig;
  camera: PlayerCameraConfig;
  activeTimelineEventSourceIds: ReadonlySet<string>;
  activeTimelineRangeModuleIds: ReadonlySet<string>;
  activeMechanicTimelineKinds: ReadonlySet<string>;
  activeRenderEffectModuleIds: ReadonlySet<string>;
  initialConfig: StatsPlayerConfig | null;
  replayPlayer: StatsReplayPlayer | null;
  boostPadOverlayEnabled: boolean;
  recording: RecordingConfig;
  singletonWindows: SingletonWindowConfig[];
  statsWindows: StatsWindowConfig[];
  moduleConfigs: Record<string, unknown>;
}

export function getDefaultCustomCameraSettings(): CameraSettings {
  return { ...DEFAULT_CUSTOM_CAMERA_SETTINGS };
}

export function getUsePlayerCameraSettingsFromConfig(camera?: PlayerCameraConfig | null): boolean {
  if (!camera) {
    return false;
  }
  if (camera.usePlayerCameraSettings !== undefined) {
    return camera.usePlayerCameraSettings;
  }
  return camera.customSettings === null;
}

export function getCustomCameraSettingsFromConfig(
  camera?: PlayerCameraConfig | null,
): CameraSettings | null {
  return getUsePlayerCameraSettingsFromConfig(camera)
    ? null
    : (camera?.customSettings ?? getDefaultCustomCameraSettings());
}

export function getConfigAdapters(modules: readonly StatModule[]): StatsPlayerConfigAdapter[] {
  return modules
    .filter((mod) => mod.getConfig || mod.applyConfig)
    .map((mod) => {
      const adapter: StatsPlayerConfigAdapter = {
        id: mod.id,
      };
      if (mod.id === "boost") {
        adapter.aliases = ["boost-pickup-animation"];
      }
      if (mod.getConfig) {
        adapter.getConfig = () => mod.getConfig?.();
      }
      if (mod.applyConfig) {
        adapter.applyConfig = (config: unknown) => mod.applyConfig?.(config);
      }
      return adapter;
    });
}

export function getModuleConfigSnapshot(modules: readonly StatModule[]): Record<string, unknown> {
  return getConfigAdapterSnapshot(getConfigAdapters(modules));
}

export function getPlaybackConfigSnapshot({
  replayPlayer,
  playbackRate,
  skipPostGoalTransitions,
  skipKickoffs,
}: PlaybackConfigSnapshotOptions): PlayerPlaybackConfig {
  const state = replayPlayer?.getState();
  return {
    currentTime: state?.currentTime,
    playing: state?.playing,
    rate: state?.speed ?? Number(playbackRate?.value ?? 1),
    skipPostGoalTransitions: replayPlayer
      ? state?.skipPostGoalTransitionsEnabled
      : skipPostGoalTransitions.checked,
    skipKickoffs: replayPlayer ? state?.skipKickoffsEnabled : skipKickoffs.checked,
  };
}

export function getCameraConfigSnapshot({
  replayPlayer,
  cameraControlsController,
}: CameraConfigSnapshotOptions): PlayerCameraConfig {
  const state = replayPlayer?.getState();
  const ballCamMode = state ? ballCamModeFromState(state) : cameraControlsController?.ballCamMode;
  return {
    mode: state?.cameraViewMode,
    freePreset: cameraControlsController?.freeCameraPreset ?? null,
    attachedPlayerId: state?.attachedPlayerId,
    ballCam: ballCamMode === undefined ? undefined : ballCamMode === "on",
    useReplayBallCam: ballCamMode === undefined ? undefined : ballCamMode === "player",
    usePlayerCameraSettings: state?.customCameraSettings === null,
    autoPossession: cameraControlsController?.autoPossessionEnabled,
    customSettings: state?.customCameraSettings,
    nameplateLiftUu: cameraControlsController?.nameplateLiftUu,
  };
}

export function getStatsPlayerConfigSnapshot({
  playback,
  camera,
  activeTimelineEventSourceIds,
  activeTimelineRangeModuleIds,
  activeMechanicTimelineKinds,
  activeRenderEffectModuleIds,
  initialConfig,
  replayPlayer,
  boostPadOverlayEnabled,
  recording,
  singletonWindows,
  statsWindows,
  moduleConfigs,
}: StatsPlayerConfigSnapshotOptions): StatsPlayerConfig {
  return {
    version: STATS_PLAYER_CONFIG_VERSION,
    playback,
    camera,
    overlays: {
      timelineEvents: [...activeTimelineEventSourceIds],
      timelineRanges: [...activeTimelineRangeModuleIds],
      mechanics: [...activeMechanicTimelineKinds],
      renderEffects: [...activeRenderEffectModuleIds],
      ...(initialConfig?.overlays.pluginRenderEffects !== undefined
        ? { pluginRenderEffects: [...initialConfig.overlays.pluginRenderEffects] }
        : {}),
      ...(initialConfig?.overlays.pluginHudOverlay !== undefined
        ? { pluginHudOverlay: initialConfig.overlays.pluginHudOverlay }
        : {}),
      followedPlayerHud: false,
      boostPads: boostPadOverlayEnabled,
      boostPickupAnimation: replayPlayer?.getState().boostPickupAnimationEnabled ?? false,
      hitboxWireframes: replayPlayer?.getState().hitboxWireframesEnabled ?? false,
      hitboxOnlyMode: replayPlayer?.getState().hitboxOnlyModeEnabled ?? false,
    },
    recording,
    singletonWindows,
    statsWindows,
    moduleConfigs,
  };
}

export function getReplayPlayerStatePatchFromConfig(
  playback: PlayerPlaybackConfig,
  camera: PlayerCameraConfig,
  config: StatsPlayerConfig,
): Parameters<StatsReplayPlayer["setState"]>[0] {
  return {
    currentTime: playback.currentTime,
    playing: playback.playing,
    speed: playback.rate,
    customCameraSettings: getCustomCameraSettingsFromConfig(camera),
    cameraViewMode: camera.mode,
    attachedPlayerId: camera.attachedPlayerId,
    // Tri-state ball cam: "player" follows the recorded toggle (useReplayBallCam),
    // otherwise force ball/car cam. Defaults to "player".
    ...(ballCamModeFromConfig(camera) === "player"
      ? { useReplayBallCam: true }
      : { ballCamEnabled: ballCamModeFromConfig(camera) === "on" }),
    boostPickupAnimationEnabled: config.overlays.boostPickupAnimation,
    hitboxWireframesEnabled: config.overlays.hitboxWireframes,
    hitboxOnlyModeEnabled: config.overlays.hitboxOnlyMode,
    skipPostGoalTransitionsEnabled: playback.skipPostGoalTransitions,
    skipKickoffsEnabled: playback.skipKickoffs,
  };
}

export function logStatsPlayerConfigLoadDebug(
  snapshot: StatsPlayerConfigParamSnapshot,
  config: StatsPlayerConfig | null,
  error: unknown,
): void {
  console.groupCollapsed("[subtr-actor] stats player cfg load");
  console.log("location.href", window.location.href);
  console.log("location.search", snapshot.search || "(empty)");
  console.log("location.hash", snapshot.hash || "(empty)");
  console.table([
    ...snapshot.searchParams.map(([name, value]) => ({
      source: "search",
      name,
      value,
    })),
    ...snapshot.hashParams.map(([name, value]) => ({
      source: "hash",
      name,
      value,
    })),
  ]);
  console.log("cfg selected source", snapshot.selectedSource ?? "(none)");
  console.log("cfg selected raw text", snapshot.selectedValue ?? "(none)");
  console.log("cfg selected raw length", snapshot.selectedValue?.length ?? 0);
  console.log("cfg search values", snapshot.searchValues);
  console.log("cfg hash values", snapshot.hashValues);
  if (snapshot.hashValues.length > 0 && snapshot.searchValues.length > 0) {
    console.warn("Both hash and search contain cfg; hash cfg is used.");
  }
  if (config) {
    console.log("cfg normalized JSON", JSON.stringify(config, null, 2));
    console.log("cfg normalized object", config);
  }
  if (error) {
    console.error("cfg decode/apply error", error);
  }
  console.groupEnd();
}
