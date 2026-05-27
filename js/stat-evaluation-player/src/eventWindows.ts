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

const DEFAULT_UNSELECTED_EVENT_PLAYLIST_SOURCE_IDS = new Set(["module:touch", "module:powerslide"]);
const EVENT_PLAYLIST_PLAYER_COLORS = [
  "#3b82f6",
  "#06b6d4",
  "#22c55e",
  "#a855f7",
  "#f97316",
  "#ef4444",
  "#f59e0b",
  "#ec4899",
];
const EVENT_PLAYLIST_NEUTRAL_COLOR = "#d1d9e0";

export type { EventTimelineSource } from "./eventTimelineSources.ts";

interface EventPlaylistSource {
  id: string;
  group: string;
  label: string;
  events: ReplayTimelineEvent[];
}

interface EventPlaylistItem {
  key: string;
  sourceId: string;
  sourceLabel: string;
  event: ReplayTimelineEvent;
  color: string;
}

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
  let eventPlaylistActiveSourceIds: Set<string> | null = null;
  let eventPlaylistAutoFollow = true;
  let eventPlaylistLastActiveKey: string | null = null;

  function getEventTimelineSources(ctx: StatModuleContext | null): EventTimelineSource[] {
    return buildEventTimelineSources(ctx, deps);
  }

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

  function getEventPlaylistReplaySources(ctx: StatModuleContext): EventPlaylistSource[] {
    const replaySources: EventPlaylistSource[] = [
      {
        id: "replay:goals",
        group: "Replay",
        label: "Goals",
        events: ctx.replay.timelineEvents.filter((event) => event.kind === "goal"),
      },
    ];

    return replaySources.filter((source) => source.events.length > 0);
  }

  function getEventPlaylistSources(): EventPlaylistSource[] {
    const ctx = deps.getModuleContext();
    if (!ctx) {
      return [];
    }

    const eventSources = getEventTimelineSources(ctx)
      .map((source) => ({
        id: source.playlistId,
        group: source.group,
        label: source.label,
        events: source.buildPlaylistEvents(),
      }))
      .filter((source) => source.events.length > 0);

    return [...getEventPlaylistReplaySources(ctx), ...eventSources];
  }

  function getEventPlaylistSelectedSourceIds(sources: EventPlaylistSource[]): Set<string> {
    const sourceIds = sources.map((source) => source.id);
    if (eventPlaylistActiveSourceIds === null) {
      return new Set(
        sourceIds.filter((id) => !DEFAULT_UNSELECTED_EVENT_PLAYLIST_SOURCE_IDS.has(id)),
      );
    }
    return new Set(sourceIds.filter((id) => eventPlaylistActiveSourceIds?.has(id)));
  }

  function getEventPlaylistPlayerColor(event: ReplayTimelineEvent): string {
    const replayPlayer = deps.getReplayPlayer();
    const playerId = event.playerId ?? null;
    const playerIndex =
      playerId && replayPlayer
        ? replayPlayer.replay.players.findIndex((player) => player.id === playerId)
        : -1;
    if (playerIndex >= 0) {
      return EVENT_PLAYLIST_PLAYER_COLORS[playerIndex % EVENT_PLAYLIST_PLAYER_COLORS.length]!;
    }
    return event.color ?? EVENT_PLAYLIST_NEUTRAL_COLOR;
  }

  function buildEventPlaylistItems(sources: EventPlaylistSource[]): EventPlaylistItem[] {
    const selectedSourceIds = getEventPlaylistSelectedSourceIds(sources);
    return sources
      .filter((source) => selectedSourceIds.has(source.id))
      .flatMap((source) =>
        source.events.map((event, index) => ({
          key: `${source.id}:${event.id ?? `${event.kind}:${event.time}:${index}`}`,
          sourceId: source.id,
          sourceLabel: source.label,
          event,
          color: getEventPlaylistPlayerColor(event),
        })),
      )
      .sort((left, right) => {
        if (left.event.time !== right.event.time) {
          return left.event.time - right.event.time;
        }
        return (left.event.label ?? left.sourceLabel).localeCompare(
          right.event.label ?? right.sourceLabel,
        );
      });
  }

  function setEventPlaylistSourceSelection(
    sources: EventPlaylistSource[],
    updater: (selected: Set<string>) => void,
  ): void {
    const selected = getEventPlaylistSelectedSourceIds(sources);
    updater(selected);
    eventPlaylistActiveSourceIds = selected;
    eventPlaylistLastActiveKey = null;
    renderEventPlaylistWindow();
    const state = deps.getReplayPlayer()?.getState();
    if (state) {
      syncEventPlaylistTimeline(state);
    }
  }

  function renderEventPlaylistWindow(): void {
    const eventPlaylistWindowBody = deps.getPlaylistWindowBody();
    if (!eventPlaylistWindowBody) {
      return;
    }

    eventPlaylistWindowBody.replaceChildren();
    const sources = getEventPlaylistSources();
    if (sources.length === 0) {
      const empty = document.createElement("p");
      empty.className = "stat-window-empty";
      empty.textContent = deps.getReplayPlayer() ? "No events loaded." : "Load a replay to see events.";
      eventPlaylistWindowBody.append(empty);
      return;
    }

    const selectedSourceIds = getEventPlaylistSelectedSourceIds(sources);
    const items = buildEventPlaylistItems(sources);

    const toolbar = document.createElement("div");
    toolbar.className = "event-playlist-toolbar";

    const filters = document.createElement("details");
    filters.className = "event-playlist-filter";
    filters.dataset.noDrag = "true";

    const summary = document.createElement("summary");
    summary.textContent = `Filters ${selectedSourceIds.size}/${sources.length}`;
    filters.append(summary);

    const filterPanel = document.createElement("div");
    filterPanel.className = "event-playlist-filter-panel";

    const actions = document.createElement("div");
    actions.className = "event-playlist-filter-actions";

    const allButton = document.createElement("button");
    allButton.type = "button";
    allButton.textContent = "All";
    allButton.addEventListener("click", () => {
      eventPlaylistActiveSourceIds = new Set(sources.map((source) => source.id));
      eventPlaylistLastActiveKey = null;
      renderEventPlaylistWindow();
      const state = deps.getReplayPlayer()?.getState();
      if (state) syncEventPlaylistTimeline(state);
    });

    const noneButton = document.createElement("button");
    noneButton.type = "button";
    noneButton.textContent = "None";
    noneButton.addEventListener("click", () => {
      eventPlaylistActiveSourceIds = new Set();
      eventPlaylistLastActiveKey = null;
      renderEventPlaylistWindow();
    });

    actions.append(allButton, noneButton);
    filterPanel.append(actions);

    const sourcesByGroup = new Map<string, EventPlaylistSource[]>();
    for (const source of sources) {
      const group = sourcesByGroup.get(source.group) ?? [];
      group.push(source);
      sourcesByGroup.set(source.group, group);
    }

    for (const [group, groupSources] of sourcesByGroup) {
      const groupEl = document.createElement("section");
      groupEl.className = "event-playlist-filter-group";
      const heading = document.createElement("h3");
      heading.textContent = group;
      groupEl.append(heading);

      for (const source of groupSources) {
        const label = document.createElement("label");
        label.className = "toggle event-playlist-filter-option";

        const input = document.createElement("input");
        input.type = "checkbox";
        input.checked = selectedSourceIds.has(source.id);
        input.addEventListener("change", () => {
          setEventPlaylistSourceSelection(sources, (selected) => {
            if (input.checked) {
              selected.add(source.id);
            } else {
              selected.delete(source.id);
            }
          });
        });

        const text = document.createElement("span");
        text.textContent = `${source.label} (${source.events.length})`;
        label.append(input, text);
        groupEl.append(label);
      }

      filterPanel.append(groupEl);
    }

    filters.append(filterPanel);

    const followLabel = document.createElement("label");
    followLabel.className = "toggle event-playlist-follow";
    const followInput = document.createElement("input");
    followInput.type = "checkbox";
    followInput.checked = eventPlaylistAutoFollow;
    followInput.addEventListener("change", () => {
      eventPlaylistAutoFollow = followInput.checked;
      const state = deps.getReplayPlayer()?.getState();
      if (state) syncEventPlaylistTimeline(state, { forceScroll: true });
    });
    const followText = document.createElement("span");
    followText.textContent = "Auto-follow";
    followLabel.append(followInput, followText);

    toolbar.append(filters, followLabel);

    const list = document.createElement("div");
    list.className = "event-playlist-list";
    list.dataset.noDrag = "true";

    if (items.length === 0) {
      const empty = document.createElement("p");
      empty.className = "stat-window-empty";
      empty.textContent = "No event types selected.";
      list.append(empty);
    } else {
      for (const item of items) {
        const button = document.createElement("button");
        button.type = "button";
        button.className = "event-playlist-item";
        button.dataset.eventKey = item.key;
        button.dataset.eventTime = `${item.event.time}`;
        button.style.setProperty("--event-color", item.color);
        button.addEventListener("click", () => {
          deps.cueTimelineEvent(item.event);
        });

        const time = document.createElement("span");
        time.className = "event-playlist-time";
        time.textContent = deps.formatTime(item.event.time);

        const main = document.createElement("span");
        main.className = "event-playlist-main";
        const label = document.createElement("strong");
        label.textContent = item.event.label ?? item.sourceLabel;
        const meta = document.createElement("span");
        meta.textContent = [
          item.event.playerName ?? null,
          item.event.frame !== undefined ? `frame ${item.event.frame}` : null,
          item.sourceLabel,
        ]
          .filter((part): part is string => Boolean(part))
          .join(" · ");
        main.append(label, meta);

        button.append(time, main);
        list.append(button);
      }
    }

    eventPlaylistWindowBody.append(toolbar, list);
  }

  function getEventPlaylistActiveItem(list: HTMLElement, currentTime: number): HTMLElement | null {
    const items = [...list.querySelectorAll<HTMLElement>(".event-playlist-item")];
    if (items.length === 0) {
      return null;
    }

    let bestItem = items[0] ?? null;
    let bestDistance = Number.POSITIVE_INFINITY;
    for (const item of items) {
      const time = Number(item.dataset.eventTime);
      if (!Number.isFinite(time)) {
        continue;
      }
      const distance = Math.abs(time - currentTime);
      if (distance < bestDistance) {
        bestDistance = distance;
        bestItem = item;
      }
    }
    return bestItem;
  }

  function syncEventPlaylistTimeline(
    state: ReplayPlayerState,
    options: { forceScroll?: boolean } = {},
  ): void {
    const eventPlaylistWindowBody = deps.getPlaylistWindowBody();
    const list = eventPlaylistWindowBody?.querySelector<HTMLElement>(".event-playlist-list");
    if (!list) {
      return;
    }

    const activeItem = getEventPlaylistActiveItem(list, state.currentTime);
    const activeKey = activeItem?.dataset.eventKey ?? null;
    if (activeKey === eventPlaylistLastActiveKey && !options.forceScroll) {
      return;
    }

    list.querySelectorAll<HTMLElement>(".event-playlist-item[data-active='true']").forEach((item) => {
      item.dataset.active = "false";
    });

    if (activeItem) {
      activeItem.dataset.active = "true";
      if (eventPlaylistAutoFollow || options.forceScroll) {
        activeItem.scrollIntoView({ block: "nearest" });
      }
    }

    eventPlaylistLastActiveKey = activeKey;
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

  function resetPlaylistState(): void {
    eventPlaylistActiveSourceIds = null;
    eventPlaylistAutoFollow = true;
    eventPlaylistLastActiveKey = null;
  }

  return {
    countVisibleTimelineSources,
    getTimelineSources: getEventTimelineSources,
    renderPlaylistWindow: renderEventPlaylistWindow,
    renderTimelineControls: renderEventTimelineControls,
    resetPlaylistState,
    syncPlaylistTimeline: syncEventPlaylistTimeline,
  };
}
