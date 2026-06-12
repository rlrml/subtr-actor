import { timelineEventSeekTime, type ReplayTimelineEvent } from "@rlrml/player";
import type { StatsReplayPlayer } from "./statsReplayPlayer.ts";
import type { CameraControlsController } from "./cameraControls.ts";
import type { MechanicsReviewWindowController } from "./mechanicsReviewWindow.ts";
import type { StatsPlayerConfig } from "./playerConfig.ts";
import { getReplayPlayerStatePatchFromConfig } from "./playerConfigRuntime.ts";

export interface PlaybackActionController {
  watchGoalReplay(time: number, scorerId: string | null): void;
  cueGoalReplay(time: number): void;
  cueTimelineEvent(event: ReplayTimelineEvent): void;
  applyConfigToReplayPlayer(config: StatsPlayerConfig): void;
}

export interface PlaybackActionControllerOptions {
  readonly goalWatchLeadSeconds: number;
  getReplayPlayer(): StatsReplayPlayer | null;
  getCameraControlsController(): CameraControlsController | null;
  getMechanicsReviewController(): MechanicsReviewWindowController | null;
  getSkipPostGoalTransitions(): HTMLInputElement;
  getSkipKickoffs(): HTMLInputElement;
  syncBoostPadOverlayPlugin(): void;
  setupActiveModules(): void;
  renderModuleSummary(): void;
  renderModuleSettings(): void;
  renderStatsWindows(frameIndex: number): void;
  scheduleConfigUrlUpdate(): void;
}

export function createPlaybackActionController(
  options: PlaybackActionControllerOptions,
): PlaybackActionController {
  const resetReplayTransitionControls = () => {
    options.getSkipPostGoalTransitions().checked = false;
    options.getSkipKickoffs().checked = false;
  };

  const cueReplayAt = (time: number, playing: boolean) => {
    const replayPlayer = options.getReplayPlayer();
    if (!replayPlayer || !Number.isFinite(time)) {
      return;
    }

    options.getMechanicsReviewController()?.clearCurrentClip();
    resetReplayTransitionControls();
    replayPlayer.setState({
      currentTime: Math.max(0, time - options.goalWatchLeadSeconds),
      playing,
      skipPostGoalTransitionsEnabled: false,
      skipKickoffsEnabled: false,
    });
    options.scheduleConfigUrlUpdate();
  };

  return {
    watchGoalReplay(time, scorerId) {
      const replayPlayer = options.getReplayPlayer();
      if (!replayPlayer || !Number.isFinite(time)) {
        return;
      }

      options.getMechanicsReviewController()?.clearCurrentClip();

      const canFollowScorer =
        scorerId !== null && replayPlayer.replay.players.some((player) => player.id === scorerId);
      if (canFollowScorer) {
        replayPlayer.setAttachedPlayer(scorerId);
        replayPlayer.setCameraViewMode("follow");
        const cameraControlsController = options.getCameraControlsController();
        if (cameraControlsController) {
          cameraControlsController.freeCameraPreset = null;
        }
      }

      resetReplayTransitionControls();
      replayPlayer.setState({
        currentTime: Math.max(0, time - options.goalWatchLeadSeconds),
        playing: true,
        skipPostGoalTransitionsEnabled: false,
        skipKickoffsEnabled: false,
      });
      options.scheduleConfigUrlUpdate();
    },

    cueGoalReplay(time) {
      cueReplayAt(time, false);
    },

    cueTimelineEvent(event) {
      const replayPlayer = options.getReplayPlayer();
      if (!replayPlayer) {
        return;
      }

      options.getMechanicsReviewController()?.clearCurrentClip();
      resetReplayTransitionControls();
      replayPlayer.setState({
        currentTime: timelineEventSeekTime(event),
        skipPostGoalTransitionsEnabled: false,
        skipKickoffsEnabled: false,
      });
      options.scheduleConfigUrlUpdate();
    },

    applyConfigToReplayPlayer(config) {
      const replayPlayer = options.getReplayPlayer();
      if (!replayPlayer) {
        return;
      }

      replayPlayer.setState(
        getReplayPlayerStatePatchFromConfig(config.playback, config.camera, config),
      );
      const cameraControlsController = options.getCameraControlsController();
      if (cameraControlsController) {
        cameraControlsController.freeCameraPreset = config.camera.freePreset ?? null;
      }
      if (config.camera.mode === "free" && config.camera.freePreset) {
        replayPlayer.setFreeCameraPreset(config.camera.freePreset);
      }
      options.syncBoostPadOverlayPlugin();
      options.setupActiveModules();
      options.renderModuleSummary();
      options.renderModuleSettings();
      options.renderStatsWindows(replayPlayer.getState().frameIndex);
    },
  };
}
