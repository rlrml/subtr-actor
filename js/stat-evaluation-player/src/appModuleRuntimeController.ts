import {
  timelineEventSeekTime,
  type ReplayPlayer,
  type TimelineOverlayPlugin,
} from "@rlrml/player";
import type { EventWindowsManager } from "./eventWindows.ts";
import {
  createModuleRuntimeController,
  type ModuleRuntimeController,
} from "./moduleRuntimeController.ts";
import type { StatsWindowsManager } from "./statsWindows.ts";
import type { StatsFrameLookup, StatsTimeline } from "./statsTimeline.ts";

interface AppModuleRuntimeControllerDeps {
  readonly statsWindowManager: StatsWindowsManager;
  getEventWindowsManager(): EventWindowsManager;
  getReplayPlayer(): ReplayPlayer | null;
  getStatsFrameLookup(): StatsFrameLookup | null;
  getStatsTimeline(): StatsTimeline | null;
  getTimelineOverlay(): TimelineOverlayPlugin | null;
  renderModuleRuntimeViews(): void;
  renderTimelineEventCountValue(value: string): void;
  requestConfigSync(): void;
}

export function createAppModuleRuntimeController(
  deps: AppModuleRuntimeControllerDeps,
): ModuleRuntimeController {
  return createModuleRuntimeController({
    getEventWindowsManager: deps.getEventWindowsManager,
    getReplayPlayer: deps.getReplayPlayer,
    getStatsFrameLookup: deps.getStatsFrameLookup,
    getStatsTimeline: deps.getStatsTimeline,
    getTimelineOverlay: deps.getTimelineOverlay,
    renderTimelineEvent(event) {
      return {
        ...event,
        seekTime: timelineEventSeekTime(event),
      };
    },
    rerenderStatsWindow() {
      const replayPlayer = deps.getReplayPlayer();
      if (!replayPlayer) {
        return;
      }

      const state = replayPlayer.getState();
      deps.statsWindowManager.render(state.frameIndex);
    },
    renderModuleRuntimeViews: deps.renderModuleRuntimeViews,
    renderTimelineEventCountValue: deps.renderTimelineEventCountValue,
    requestConfigSync: deps.requestConfigSync,
  });
}
