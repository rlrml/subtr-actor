import type {
  ReplayPlayer,
  ReplayPlayerState,
  ReplayTimelineEvent,
} from "@rlrml/player";
import type { StatModule, StatModuleContext } from "./statModules.ts";
import {
  buildEventTimelineSources,
  type EventTimelineSource,
} from "./eventTimelineSources.ts";
import { createEventPlaylistWindowController } from "./eventPlaylistWindow.ts";

export type { EventTimelineSource } from "./eventTimelineSources.ts";

interface EventWindowsManagerDeps {
  cueTimelineEvent(event: ReplayTimelineEvent): void;
  formatTime(seconds: number): string;
  getActiveMechanicTimelineKinds(): Set<string>;
  getActiveTimelineEventSourceIds(): Set<string>;
  getModuleContext(): StatModuleContext | null;
  getModules(): readonly StatModule[];
  getPlaylistWindowBody(): HTMLElement | null;
  getReplayPlayer(): ReplayPlayer | null;
  getTimelineWindowBody(): HTMLElement | null;
  renderModuleSettings(): void;
  renderModuleSummary(): void;
  renderTimelineEventCount(): void;
  scheduleConfigUrlUpdate(): void;
  setMechanicTimelineKind(kind: string, enabled: boolean): void;
  setupActiveModules(): void;
  syncTimelineEvents(): void;
  syncTimelineRanges(): void;
  toggleCapability(id: string, kind: "events", enabled: boolean): void;
}

export interface EventWindowsManager {
  countVisibleTimelineSources(ctx: StatModuleContext): number;
  getTimelineSources(ctx: StatModuleContext | null): EventTimelineSource[];
  renderPlaylistWindow(): void;
  renderTimelineControls(): void;
  resetPlaylistState(): void;
  syncPlaylistTimeline(state: ReplayPlayerState, options?: { forceScroll?: boolean }): void;
}

export function createEventWindowsManager(deps: EventWindowsManagerDeps): EventWindowsManager {
  function getEventTimelineSources(ctx: StatModuleContext | null): EventTimelineSource[] {
    return buildEventTimelineSources(ctx, deps);
  }

  const eventPlaylistWindow = createEventPlaylistWindowController({
    cueTimelineEvent: deps.cueTimelineEvent,
    formatTime: deps.formatTime,
    getEventTimelineSources,
    getModuleContext: deps.getModuleContext,
    getPlaylistWindowBody: deps.getPlaylistWindowBody,
    getReplayPlayer: deps.getReplayPlayer,
  });

  function renderEventTimelineControls(): void {
    const mechanicsTimelineWindowBody = deps.getTimelineWindowBody();
    if (!mechanicsTimelineWindowBody) {
      return;
    }
    mechanicsTimelineWindowBody.replaceChildren();

    const ctx = deps.getModuleContext();
    const sources = getEventTimelineSources(ctx);

    if (sources.length === 0) {
      const empty = document.createElement("p");
      empty.className = "stat-window-empty";
      empty.textContent = "No events loaded.";
      mechanicsTimelineWindowBody.append(empty);
      return;
    }

    const actions = document.createElement("div");
    actions.className = "mechanics-actions";

    const allButton = document.createElement("button");
    allButton.type = "button";
    allButton.className = "module-summary-item";
    allButton.addEventListener("click", () => {
      for (const source of sources) {
        source.setActive(true);
      }
      deps.setupActiveModules();
      deps.syncTimelineEvents();
      deps.syncTimelineRanges();
      renderEventTimelineControls();
      deps.renderModuleSummary();
      deps.renderModuleSettings();
      deps.renderTimelineEventCount();
      deps.scheduleConfigUrlUpdate();
    });
    const allName = document.createElement("span");
    allName.textContent = "All events";
    const allCount = document.createElement("strong");
    allCount.textContent = `${sources.length}`;
    allButton.append(allName, allCount);

    const noneButton = document.createElement("button");
    noneButton.type = "button";
    noneButton.className = "module-summary-item";
    noneButton.addEventListener("click", () => {
      for (const source of sources) {
        source.setActive(false);
      }
      deps.setupActiveModules();
      deps.syncTimelineEvents();
      deps.syncTimelineRanges();
      renderEventTimelineControls();
      deps.renderModuleSummary();
      deps.renderModuleSettings();
      deps.renderTimelineEventCount();
      deps.scheduleConfigUrlUpdate();
    });
    const noneName = document.createElement("span");
    noneName.textContent = "No events";
    const noneState = document.createElement("strong");
    noneState.textContent = "Off";
    noneButton.append(noneName, noneState);

    actions.append(allButton, noneButton);
    mechanicsTimelineWindowBody.append(actions);

    const list = renderEventSourceList(sources);
    if (list) {
      mechanicsTimelineWindowBody.append(list);
    }
  }

  function renderEventSourceList(sources: EventTimelineSource[]): HTMLElement | null {
    if (sources.length === 0) {
      return null;
    }

    const list = document.createElement("div");
    list.className = "module-list mechanics-list mechanics-event-list";
    list.style.setProperty(
      "--event-source-columns",
      `${getEventSourceColumnCount(sources.length)}`,
    );

    for (const source of sources) {
      const item = document.createElement("button");
      item.type = "button";
      item.className = "module-summary-item";
      item.dataset.active = source.active ? "true" : "false";
      item.setAttribute("aria-pressed", source.active ? "true" : "false");
      item.addEventListener("click", () => {
        source.setActive(!source.active);
        deps.syncTimelineEvents();
        deps.syncTimelineRanges();
        renderEventTimelineControls();
        deps.renderTimelineEventCount();
      });

      const name = document.createElement("span");
      name.textContent = source.label;

      const state = document.createElement("strong");
      state.textContent = `${source.active ? "On" : "Off"} ${source.count}`;

      item.append(name, state);
      list.append(item);
    }

    return list;
  }

  function getEventSourceColumnCount(sourceCount: number): number {
    if (window.innerWidth < 640) {
      return 1;
    }
    if (window.innerWidth < 900) {
      return sourceCount >= 7 ? 2 : 1;
    }
    if (sourceCount >= 13) {
      return 3;
    }
    if (sourceCount >= 7) {
      return 2;
    }
    return 1;
  }

  function countVisibleTimelineSources(ctx: StatModuleContext): number {
    const goalCount = ctx.replay.timelineEvents.filter((event) => event.kind === "goal").length;
    return (
      goalCount +
      getEventTimelineSources(ctx)
        .filter((source) => source.active)
        .reduce((count, source) => count + source.count, 0)
    );
  }

  return {
    countVisibleTimelineSources,
    getTimelineSources: getEventTimelineSources,
    renderPlaylistWindow: eventPlaylistWindow.renderWindow,
    renderTimelineControls: renderEventTimelineControls,
    resetPlaylistState: eventPlaylistWindow.resetState,
    syncPlaylistTimeline: eventPlaylistWindow.syncTimeline,
  };
}
