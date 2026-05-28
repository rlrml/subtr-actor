import type { ReplayPlayer, ReplayTimelineEvent } from "@rlrml/player";
import type { StatEvaluationPlayerElements } from "./appElements.ts";
import {
  createEventWindowsManager,
  type EventWindowsManager,
} from "./eventWindows.ts";
import type { ModuleRuntimeController } from "./moduleRuntimeController.ts";

interface AppEventWindowsManagerDeps {
  cueTimelineEvent(event: ReplayTimelineEvent): void;
  formatTime(seconds: number): string;
  getElements(): StatEvaluationPlayerElements;
  getModuleRuntimeController(): ModuleRuntimeController;
  getReplayPlayer(): ReplayPlayer | null;
  renderModuleSettings(): void;
  renderModuleSummary(): void;
  scheduleConfigUrlUpdate(): void;
}

export function createAppEventWindowsManager(
  deps: AppEventWindowsManagerDeps,
): EventWindowsManager {
  return createEventWindowsManager({
    cueTimelineEvent: deps.cueTimelineEvent,
    formatTime: deps.formatTime,
    getActiveMechanicTimelineKinds() {
      return deps.getModuleRuntimeController().getActiveMechanicTimelineKinds();
    },
    getActiveTimelineEventSourceIds() {
      return deps.getModuleRuntimeController().getActiveTimelineEventSourceIds();
    },
    getModuleContext: () => deps.getModuleRuntimeController().getContext(),
    getModules() {
      return deps.getModuleRuntimeController().modules;
    },
    getPlaylistWindowBody() {
      return deps.getElements().eventPlaylistWindowBody;
    },
    getReplayPlayer: deps.getReplayPlayer,
    getTimelineWindowBody() {
      return deps.getElements().mechanicsTimelineWindowBody;
    },
    renderModuleSettings: deps.renderModuleSettings,
    renderModuleSummary: deps.renderModuleSummary,
    renderTimelineEventCount: () =>
      deps.getModuleRuntimeController().renderTimelineEventCount(),
    scheduleConfigUrlUpdate: deps.scheduleConfigUrlUpdate,
    setMechanicTimelineKind(kind, enabled) {
      deps.getModuleRuntimeController().setMechanicTimelineKind(kind, enabled);
    },
    setupActiveModules: () => deps.getModuleRuntimeController().setupActiveModules(),
    syncTimelineEvents: () => deps.getModuleRuntimeController().syncTimelineEvents(),
    syncTimelineRanges: () => deps.getModuleRuntimeController().syncTimelineRanges(),
    toggleCapability: (id, kind, enabled) =>
      deps.getModuleRuntimeController().toggleCapability(id, kind, enabled),
  });
}
