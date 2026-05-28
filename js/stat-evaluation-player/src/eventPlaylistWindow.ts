import type { ReplayPlayer, ReplayPlayerState, ReplayTimelineEvent } from "@rlrml/player";
import type { StatModuleContext } from "./statModules.ts";
import type { EventTimelineSource } from "./eventTimelineSources.ts";

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

interface EventPlaylistWindowControllerDeps {
  cueTimelineEvent(event: ReplayTimelineEvent): void;
  formatTime(seconds: number): string;
  getEventTimelineSources(ctx: StatModuleContext | null): EventTimelineSource[];
  getModuleContext(): StatModuleContext | null;
  getPlaylistWindowBody(): HTMLElement | null;
  getReplayPlayer(): ReplayPlayer | null;
}

export interface EventPlaylistWindowController {
  renderWindow(): void;
  resetState(): void;
  syncTimeline(state: ReplayPlayerState, options?: { forceScroll?: boolean }): void;
}

export function createEventPlaylistWindowController(
  deps: EventPlaylistWindowControllerDeps,
): EventPlaylistWindowController {
  let activeSourceIds: Set<string> | null = null;
  let autoFollow = true;
  let lastActiveKey: string | null = null;

  function getReplaySources(ctx: StatModuleContext): EventPlaylistSource[] {
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

  function getSources(): EventPlaylistSource[] {
    const ctx = deps.getModuleContext();
    if (!ctx) {
      return [];
    }

    const eventSources = deps
      .getEventTimelineSources(ctx)
      .map((source) => ({
        id: source.playlistId,
        group: source.group,
        label: source.label,
        events: source.buildPlaylistEvents(),
      }))
      .filter((source) => source.events.length > 0);

    return [...getReplaySources(ctx), ...eventSources];
  }

  function getSelectedSourceIds(sources: EventPlaylistSource[]): Set<string> {
    const sourceIds = sources.map((source) => source.id);
    if (activeSourceIds === null) {
      return new Set(
        sourceIds.filter((id) => !DEFAULT_UNSELECTED_EVENT_PLAYLIST_SOURCE_IDS.has(id)),
      );
    }
    return new Set(sourceIds.filter((id) => activeSourceIds?.has(id)));
  }

  function getPlayerColor(event: ReplayTimelineEvent): string {
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

  function buildItems(sources: EventPlaylistSource[]): EventPlaylistItem[] {
    const selectedSourceIds = getSelectedSourceIds(sources);
    return sources
      .filter((source) => selectedSourceIds.has(source.id))
      .flatMap((source) =>
        source.events.map((event, index) => ({
          key: `${source.id}:${event.id ?? `${event.kind}:${event.time}:${index}`}`,
          sourceId: source.id,
          sourceLabel: source.label,
          event,
          color: getPlayerColor(event),
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

  function setSourceSelection(
    sources: EventPlaylistSource[],
    updater: (selected: Set<string>) => void,
  ): void {
    const selected = getSelectedSourceIds(sources);
    updater(selected);
    activeSourceIds = selected;
    lastActiveKey = null;
    renderWindow();
    const state = deps.getReplayPlayer()?.getState();
    if (state) {
      syncTimeline(state);
    }
  }

  function renderWindow(): void {
    const eventPlaylistWindowBody = deps.getPlaylistWindowBody();
    if (!eventPlaylistWindowBody) {
      return;
    }

    eventPlaylistWindowBody.replaceChildren();
    const sources = getSources();
    if (sources.length === 0) {
      const empty = document.createElement("p");
      empty.className = "stat-window-empty";
      empty.textContent = deps.getReplayPlayer()
        ? "No events loaded."
        : "Load a replay to see events.";
      eventPlaylistWindowBody.append(empty);
      return;
    }

    const selectedSourceIds = getSelectedSourceIds(sources);
    const items = buildItems(sources);

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
      activeSourceIds = new Set(sources.map((source) => source.id));
      lastActiveKey = null;
      renderWindow();
      const state = deps.getReplayPlayer()?.getState();
      if (state) syncTimeline(state);
    });

    const noneButton = document.createElement("button");
    noneButton.type = "button";
    noneButton.textContent = "None";
    noneButton.addEventListener("click", () => {
      activeSourceIds = new Set();
      lastActiveKey = null;
      renderWindow();
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
          setSourceSelection(sources, (selected) => {
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
    followInput.checked = autoFollow;
    followInput.addEventListener("change", () => {
      autoFollow = followInput.checked;
      const state = deps.getReplayPlayer()?.getState();
      if (state) syncTimeline(state, { forceScroll: true });
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

  function getActiveItem(list: HTMLElement, currentTime: number): HTMLElement | null {
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

  function syncTimeline(state: ReplayPlayerState, options: { forceScroll?: boolean } = {}): void {
    const eventPlaylistWindowBody = deps.getPlaylistWindowBody();
    const list = eventPlaylistWindowBody?.querySelector<HTMLElement>(".event-playlist-list");
    if (!list) {
      return;
    }

    const activeItem = getActiveItem(list, state.currentTime);
    const activeKey = activeItem?.dataset.eventKey ?? null;
    if (activeKey === lastActiveKey && !options.forceScroll) {
      return;
    }

    list
      .querySelectorAll<HTMLElement>(".event-playlist-item[data-active='true']")
      .forEach((item) => {
        item.dataset.active = "false";
      });

    if (activeItem) {
      activeItem.dataset.active = "true";
      if (autoFollow || options.forceScroll) {
        activeItem.scrollIntoView({ block: "nearest" });
      }
    }

    lastActiveKey = activeKey;
  }

  function resetState(): void {
    activeSourceIds = null;
    autoFollow = true;
    lastActiveKey = null;
  }

  return {
    renderWindow,
    resetState,
    syncTimeline,
  };
}
