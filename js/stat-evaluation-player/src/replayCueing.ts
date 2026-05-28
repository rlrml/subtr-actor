import { timelineEventSeekTime, type ReplayPlayer, type ReplayTimelineEvent } from "@rlrml/player";
import type { StatEvaluationPlayerElements } from "./appElements.ts";
import type { CameraControls } from "./cameraControls.ts";
import type { MechanicsReviewController } from "./mechanicsReviewController.ts";

const GOAL_WATCH_LEAD_SECONDS = 4;

interface ReplayCueingControllerDeps {
  readonly elements: Pick<StatEvaluationPlayerElements, "skipPostGoalTransitions" | "skipKickoffs">;
  getCameraControls(): CameraControls | null;
  getMechanicsReviewController(): MechanicsReviewController | null;
  getReplayPlayer(): ReplayPlayer | null;
  scheduleConfigUrlUpdate(): void;
}

export interface ReplayCueingController {
  cueTimelineEvent(event: ReplayTimelineEvent): void;
  watchGoalReplay(time: number, scorerId: string | null): void;
}

export function createReplayCueingController(
  deps: ReplayCueingControllerDeps,
): ReplayCueingController {
  function resetTransitionSkipControls(): void {
    deps.elements.skipPostGoalTransitions.checked = false;
    deps.elements.skipKickoffs.checked = false;
  }

  return {
    cueTimelineEvent(event): void {
      const replayPlayer = deps.getReplayPlayer();
      if (!replayPlayer) {
        return;
      }

      deps.getMechanicsReviewController()?.clearCurrentClip();
      resetTransitionSkipControls();
      replayPlayer.setState({
        currentTime: timelineEventSeekTime(event),
        skipPostGoalTransitionsEnabled: false,
        skipKickoffsEnabled: false,
      });
      deps.scheduleConfigUrlUpdate();
    },

    watchGoalReplay(time, scorerId): void {
      const replayPlayer = deps.getReplayPlayer();
      if (!replayPlayer || !Number.isFinite(time)) {
        return;
      }

      deps.getMechanicsReviewController()?.clearCurrentClip();

      const canFollowScorer =
        scorerId !== null && replayPlayer.replay.players.some((player) => player.id === scorerId);
      if (canFollowScorer) {
        replayPlayer.setAttachedPlayer(scorerId);
        replayPlayer.setCameraViewMode("follow");
        deps.getCameraControls()?.clearFreePreset();
      }

      resetTransitionSkipControls();
      replayPlayer.setState({
        currentTime: Math.max(0, time - GOAL_WATCH_LEAD_SECONDS),
        playing: true,
        skipPostGoalTransitionsEnabled: false,
        skipKickoffsEnabled: false,
      });
      deps.scheduleConfigUrlUpdate();
    },
  };
}
