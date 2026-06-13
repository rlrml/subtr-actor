import type { ReplayPlayerState, ReplayTimelineEvent } from "@rlrml/player";
import type { StatsReplayPlayer } from "./statsReplayPlayer.ts";
import {
  buildEventPlaylistItems,
  getEventPlaylistSelectedSourceIds,
  type EventPlaylistSource,
} from "./eventTimelineSources.ts";

export interface EventPlaylistWindowControllerOptions {
  readonly body: HTMLElement;
  getReplayPlayer(): StatsReplayPlayer | null;
  getSources(): EventPlaylistSource[];
  cueTimelineEvent(event: ReplayTimelineEvent): void;
  formatTime(seconds: number): string;
}

export interface SyncEventPlaylistTimelineOptions {
  forceScroll?: boolean;
}

export class EventPlaylistWindowController {
  private activeSourceIds: Set<string> | null = null;
  private autoFollow = true;
  private lastActiveKey: string | null = null;
  private activeItem: HTMLElement | null = null;
  private renderedItems: { key: string; time: number; element: HTMLElement }[] = [];

  constructor(private readonly options: EventPlaylistWindowControllerOptions) {}

  reset(): void {
    this.activeSourceIds = null;
    this.lastActiveKey = null;
    this.activeItem = null;
    this.renderedItems = [];
  }

  render(): void {
    this.options.body.replaceChildren();
    this.lastActiveKey = null;
    this.activeItem = null;
    this.renderedItems = [];
    const sources = this.options.getSources();
    if (sources.length === 0) {
      const empty = document.createElement("p");
      empty.className = "stat-window-empty";
      empty.textContent = this.options.getReplayPlayer()
        ? "No events loaded."
        : "Load a replay to see events.";
      this.options.body.append(empty);
      return;
    }

    const selectedSourceIds = getEventPlaylistSelectedSourceIds(sources, this.activeSourceIds);
    const items = buildEventPlaylistItems({
      sources,
      activeSourceIds: this.activeSourceIds,
      replayPlayers: this.options.getReplayPlayer()?.replay.players ?? [],
    });

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
      this.activeSourceIds = new Set(sources.map((source) => source.id));
      this.lastActiveKey = null;
      this.render();
      const state = this.options.getReplayPlayer()?.getState();
      if (state) this.syncTimeline(state);
    });

    const noneButton = document.createElement("button");
    noneButton.type = "button";
    noneButton.textContent = "None";
    noneButton.addEventListener("click", () => {
      this.activeSourceIds = new Set();
      this.lastActiveKey = null;
      this.render();
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
          this.setSourceSelection(sources, (selected) => {
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
    followInput.checked = this.autoFollow;
    followInput.addEventListener("change", () => {
      this.autoFollow = followInput.checked;
      const state = this.options.getReplayPlayer()?.getState();
      if (state) this.syncTimeline(state, { forceScroll: true });
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
      if (selectedSourceIds.size === 0) {
        empty.textContent = "No event types selected.";
      } else {
        empty.textContent = "No events in selected event types.";
      }
      list.append(empty);
    } else {
      for (const item of items) {
        const button = document.createElement("button");
        button.type = "button";
        button.className = "event-playlist-item";
        button.dataset.eventKey = item.key;
        button.dataset.eventTime = `${item.event.time}`;
        button.style.setProperty("--event-color", item.color);
        if (Number.isFinite(item.event.time)) {
          this.renderedItems.push({
            key: item.key,
            time: item.event.time,
            element: button,
          });
        }
        button.addEventListener("click", () => {
          this.options.cueTimelineEvent(item.event);
        });

        const time = document.createElement("span");
        time.className = "event-playlist-time";
        time.textContent = this.options.formatTime(item.event.time);

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

    this.options.body.append(toolbar, list);
  }

  syncTimeline(state: ReplayPlayerState, options: SyncEventPlaylistTimelineOptions = {}): void {
    const list = this.options.body.querySelector<HTMLElement>(".event-playlist-list");
    if (!list) {
      return;
    }

    const activeItem = this.getActiveItem(state.currentTime);
    const activeKey = activeItem?.dataset.eventKey ?? null;
    if (activeKey === this.lastActiveKey && !options.forceScroll) {
      return;
    }

    if (this.activeItem?.isConnected) {
      this.activeItem.dataset.active = "false";
    } else if (this.activeItem) {
      this.activeItem = null;
    }

    if (activeItem) {
      activeItem.dataset.active = "true";
      this.activeItem = activeItem;
      if (this.autoFollow || options.forceScroll) {
        activeItem.scrollIntoView({ block: "nearest" });
      }
    } else {
      this.activeItem = null;
    }

    this.lastActiveKey = activeKey;
  }

  private setSourceSelection(
    sources: EventPlaylistSource[],
    updater: (selected: Set<string>) => void,
  ): void {
    const selected = getEventPlaylistSelectedSourceIds(sources, this.activeSourceIds);
    updater(selected);
    this.activeSourceIds = selected;
    this.lastActiveKey = null;
    this.render();
    const state = this.options.getReplayPlayer()?.getState();
    if (state) {
      this.syncTimeline(state);
    }
  }

  private getActiveItem(currentTime: number): HTMLElement | null {
    const items = this.renderedItems;
    if (items.length === 0) {
      return null;
    }

    let low = 0;
    let high = items.length - 1;
    while (low < high) {
      const mid = Math.floor((low + high) / 2);
      if (items[mid]!.time < currentTime) {
        low = mid + 1;
      } else {
        high = mid;
      }
    }

    const right = items[low]!;
    const left = items[low - 1] ?? null;
    if (!left) {
      return right.element;
    }
    return Math.abs(left.time - currentTime) <= Math.abs(right.time - currentTime)
      ? left.element
      : right.element;
  }
}

export function createEventPlaylistWindowController(
  options: EventPlaylistWindowControllerOptions,
): EventPlaylistWindowController {
  return new EventPlaylistWindowController(options);
}
