import type { ReplayPlayer } from "@rlrml/player";
import type { StatEvaluationPlayerElements } from "./appElements.ts";
import type { CameraControls } from "./cameraControls.ts";
import type { FloatingWindowController } from "./floatingWindows.ts";
import type { ModuleRuntimeController } from "./moduleRuntimeController.ts";
import {
  STATS_PLAYER_CONFIG_VERSION,
  type PlayerCameraConfig,
  type PlayerPlaybackConfig,
  type SingletonWindowId,
  type StatsPlayerConfig,
} from "./playerConfig.ts";
import type { RecordingControls } from "./recordingControls.ts";
import type { StatsWindowsManager } from "./statsWindows.ts";

interface PlaybackConfigSnapshotOptions {
  readonly elements: StatEvaluationPlayerElements;
  readonly replayPlayer: ReplayPlayer | null;
}

interface StatsPlayerConfigSnapshotOptions extends PlaybackConfigSnapshotOptions {
  readonly boostPadOverlayEnabled: boolean;
  readonly cameraControls: CameraControls | null;
  readonly floatingWindows: FloatingWindowController;
  readonly moduleRuntimeController: ModuleRuntimeController;
  readonly recordingControls: RecordingControls | null;
  readonly singletonWindowIds: SingletonWindowId[];
  readonly statsWindowManager: StatsWindowsManager;
}

export function getPlaybackConfigSnapshot({
  elements,
  replayPlayer,
}: PlaybackConfigSnapshotOptions): PlayerPlaybackConfig {
  const state = replayPlayer?.getState();
  return {
    currentTime: state?.currentTime,
    playing: state?.playing,
    rate: state?.speed ?? Number(elements.playbackRate?.value ?? 1),
    skipPostGoalTransitions: replayPlayer
      ? state?.skipPostGoalTransitionsEnabled
      : elements.skipPostGoalTransitions.checked,
    skipKickoffs: replayPlayer
      ? state?.skipKickoffsEnabled
      : elements.skipKickoffs.checked,
  };
}

export function getStatsPlayerConfigSnapshot(
  options: StatsPlayerConfigSnapshotOptions,
): StatsPlayerConfig {
  const { replayPlayer } = options;
  return {
    version: STATS_PLAYER_CONFIG_VERSION,
    playback: getPlaybackConfigSnapshot(options),
    camera: options.cameraControls?.getConfigSnapshot() ?? {},
    overlays: {
      ...options.moduleRuntimeController.getOverlayConfigSnapshot(),
      followedPlayerHud: false,
      boostPads: options.boostPadOverlayEnabled,
      boostPickupAnimation: replayPlayer?.getState().boostPickupAnimationEnabled ?? false,
    },
    recording: options.recordingControls?.getConfigSnapshot() ?? {},
    singletonWindows: options.floatingWindows.getSingletonWindowConfigs(
      options.singletonWindowIds,
    ),
    statsWindows: options.statsWindowManager.getConfigs(),
    moduleConfigs: options.moduleRuntimeController.getModuleConfigSnapshot(),
  };
}

export function getReplayPlayerStatePatchFromConfig(
  playback: PlayerPlaybackConfig,
  camera: PlayerCameraConfig,
  config: StatsPlayerConfig,
): Parameters<ReplayPlayer["setState"]>[0] {
  return {
    currentTime: playback.currentTime,
    playing: playback.playing,
    speed: playback.rate,
    cameraDistanceScale: camera.distanceScale,
    customCameraSettings: camera.customSettings,
    cameraViewMode: camera.mode,
    attachedPlayerId: camera.attachedPlayerId,
    ballCamEnabled: camera.ballCam,
    boostPickupAnimationEnabled: config.overlays.boostPickupAnimation,
    skipPostGoalTransitionsEnabled: playback.skipPostGoalTransitions,
    skipKickoffsEnabled: playback.skipKickoffs,
  };
}
