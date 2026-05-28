import type { ReplayPlayer } from "@rlrml/player";
import type { StatEvaluationPlayerElements } from "./appElements.ts";
import type { FloatingWindowController } from "./floatingWindows.ts";
import type { WindowPlacementConfig } from "./playerConfig.ts";
import type { StatDefinition } from "./statRegistry.ts";
import { createStatsWindowsManager, type StatsWindowsManager } from "./statsWindows.ts";
import type { StatsFrame, StatsTimeline } from "./statsTimeline.ts";

interface AppStatsWindowsManagerDeps {
  readonly floatingWindows: FloatingWindowController;
  readonly goalWatchLeadSeconds: number;
  getElements(): StatEvaluationPlayerElements;
  getReplayPlayer(): ReplayPlayer | null;
  getStatRegistry(): readonly StatDefinition[];
  getStatsFrame(frameIndex: number): StatsFrame | null;
  getStatsTimeline(): StatsTimeline | null;
  scheduleConfigUrlUpdate(): void;
  setLauncherOpen(open: boolean): void;
  watchGoalReplay(time: number, scorerId: string | null): void;
}

export function createAppStatsWindowsManager(
  deps: AppStatsWindowsManagerDeps,
): StatsWindowsManager {
  return createStatsWindowsManager({
    getDefaultFrameIndex() {
      return deps.getReplayPlayer()?.getState().frameIndex ?? 0;
    },
    getReplayPlayer: deps.getReplayPlayer,
    getStatsFrame: deps.getStatsFrame,
    getStatsTimeline: deps.getStatsTimeline,
    getStatRegistry: deps.getStatRegistry,
    getWindowLayer() {
      return deps.getElements().statsWindowLayer;
    },
    applyWindowPlacement(windowEl: HTMLElement, placement: WindowPlacementConfig) {
      deps.floatingWindows.applyWindowPlacement(windowEl, placement);
    },
    bringWindowToFront(windowEl: HTMLElement) {
      deps.floatingWindows.bringWindowToFront(windowEl);
    },
    cueGoalReplay(time) {
      deps.getReplayPlayer()?.setState({
        currentTime: Math.max(0, time - deps.goalWatchLeadSeconds),
        playing: false,
        skipPostGoalTransitionsEnabled: false,
        skipKickoffsEnabled: false,
      });
      const elements = deps.getElements();
      elements.skipPostGoalTransitions.checked = false;
      elements.skipKickoffs.checked = false;
      deps.scheduleConfigUrlUpdate();
    },
    formatTime,
    readWindowPlacement(windowEl: HTMLElement) {
      return deps.floatingWindows.readWindowPlacement(windowEl);
    },
    scheduleConfigUrlUpdate: deps.scheduleConfigUrlUpdate,
    setLauncherOpen: deps.setLauncherOpen,
    watchGoalReplay: deps.watchGoalReplay,
  });
}

function formatTime(seconds: number): string {
  if (!Number.isFinite(seconds)) {
    return "--";
  }
  const minutes = Math.floor(Math.max(0, seconds) / 60);
  const remainingSeconds = Math.max(0, seconds) - minutes * 60;
  return `${minutes}:${remainingSeconds.toFixed(1).padStart(4, "0")}`;
}
