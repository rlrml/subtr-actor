import type { ReplayPlayer } from "@rlrml/player";
import type { StatsWindowConfig, StatsWindowKind, WindowPlacementConfig } from "./playerConfig.ts";
import type { StatDefinition } from "./statRegistry.ts";
import {
  getDefaultAdHocTargetId,
  getStatsWindowAllowedScope,
  getStatsWindowTitle,
  hasStatsWindowScopeSelector,
  hasStatsWindowStatPicker,
  renderStatsWindowScope,
} from "./statsWindowScope.ts";
import { renderStatsWindowAddControl, renderStatsWindowPicker } from "./statsWindowPicker.ts";
import { renderStatsWindowEntries } from "./statsWindowEntries.ts";
import type { StatsWindowState } from "./statsWindowTypes.ts";
import type { StatsFrame, StatsTimeline } from "./statsTimeline.ts";

interface StatsWindowsManagerDeps {
  getDefaultFrameIndex(): number;
  getReplayPlayer(): ReplayPlayer | null;
  getStatsFrame(frameIndex: number): StatsFrame | null;
  getStatsTimeline(): StatsTimeline | null;
  getStatRegistry(): readonly StatDefinition[];
  getWindowLayer(): HTMLElement | null;
  applyWindowPlacement(windowEl: HTMLElement, placement: WindowPlacementConfig): void;
  bringWindowToFront(windowEl: HTMLElement): void;
  cueGoalReplay(time: number): void;
  formatTime(seconds: number): string;
  readWindowPlacement(windowEl: HTMLElement): WindowPlacementConfig;
  scheduleConfigUrlUpdate(): void;
  setLauncherOpen(open: boolean): void;
  watchGoalReplay(time: number, scorerId: string | null): void;
}

export interface StatsWindowsManager {
  clear(): void;
  create(kind: StatsWindowKind, config?: StatsWindowConfig): StatsWindowState;
  getConfigs(): StatsWindowConfig[];
  render(frameIndex?: number, options?: { preserveOpenPickers?: boolean }): void;
  replaceFromConfig(configs: readonly StatsWindowConfig[]): void;
}

export function createStatsWindowsManager(deps: StatsWindowsManagerDeps): StatsWindowsManager {
  const statsWindows = new Map<string, StatsWindowState>();
  let nextStatsWindowId = 1;

  function getStatById(statId: string): StatDefinition | null {
    return deps.getStatRegistry().find((definition) => definition.id === statId) ?? null;
  }

  function getStatsWindowDefaultPosition(): { x: number; y: number } {
    const offset = statsWindows.size * 18;
    return {
      x: Math.max(12, Math.min(window.innerWidth - 360, 96 + offset)),
      y: Math.max(64, Math.min(window.innerHeight - 240, 96 + offset)),
    };
  }

  function getStatsWindowConfig(statsWindow: StatsWindowState): StatsWindowConfig {
    return {
      id: statsWindow.id,
      kind: statsWindow.kind,
      placement: deps.readWindowPlacement(statsWindow.element),
      playerId: statsWindow.playerId,
      team: statsWindow.team,
      entries: statsWindow.entries.map((entry) => ({
        statId: entry.statId,
        targetId: entry.targetId,
      })),
    };
  }

  function renderStatsWindows(
    frameIndex = deps.getDefaultFrameIndex(),
    options: { preserveOpenPickers?: boolean } = {},
  ): void {
    for (const statsWindow of statsWindows.values()) {
      if (
        options.preserveOpenPickers &&
        (statsWindow.pickerOpen || statsWindow.element.contains(document.activeElement))
      ) {
        continue;
      }
      renderStatsWindow(statsWindow, frameIndex);
    }
  }

  function createStatsWindow(kind: StatsWindowKind, config?: StatsWindowConfig): StatsWindowState {
    const statsWindowLayer = deps.getWindowLayer();
    if (!statsWindowLayer) {
      throw new Error("Stats window layer is not mounted.");
    }

    const id = config?.id ?? `stats-${nextStatsWindowId++}`;
    const idNumber = Number.parseInt(id.replace(/^stats-/, ""), 10);
    if (Number.isFinite(idNumber)) {
      nextStatsWindowId = Math.max(nextStatsWindowId, idNumber + 1);
    }
    const { x, y } = getStatsWindowDefaultPosition();
    const element = document.createElement("section");
    element.className = "stats-window";
    element.dataset.windowId = id;
    element.style.setProperty("--window-x", `${x}px`);
    element.style.setProperty("--window-y", `${y}px`);
    if (config) {
      deps.applyWindowPlacement(element, config.placement);
    }

    const header = document.createElement("header");
    header.className = "stats-window-header";

    const actions = document.createElement("div");
    actions.className = "stats-window-actions";
    const hideButton = document.createElement("button");
    hideButton.type = "button";
    hideButton.className = "stats-window-action";
    hideButton.textContent = "Hide";
    actions.append(hideButton);
    if (hasStatsWindowScopeSelector(kind)) {
      header.classList.add("stats-window-header-actions-only");
      header.append(actions);
    } else {
      const title = document.createElement("h2");
      title.textContent = getStatsWindowTitle(kind);
      header.append(title, actions);
    }

    const body = document.createElement("div");
    body.className = "stats-window-body";
    element.append(header, body);
    statsWindowLayer.append(element);

    const state: StatsWindowState = {
      id,
      kind,
      entries:
        config?.entries.map((entry) => ({
          key: `${id}:${entry.statId}:${entry.targetId ?? "scope"}`,
          statId: entry.statId,
          targetId: entry.targetId,
        })) ?? [],
      playerId: config?.playerId ?? deps.getReplayPlayer()?.replay.players[0]?.id ?? null,
      team: config?.team ?? "blue",
      pickerOpen: false,
      query: "",
      element,
      body,
    };

    hideButton.addEventListener("click", () => {
      element.hidden = true;
      deps.scheduleConfigUrlUpdate();
    });

    statsWindows.set(id, state);
    if (!config) {
      deps.bringWindowToFront(element);
    }
    deps.setLauncherOpen(false);
    renderStatsWindow(state);
    deps.scheduleConfigUrlUpdate();
    return state;
  }

  function replaceStatsWindowsFromConfig(configs: readonly StatsWindowConfig[]): void {
    clear();
    for (const config of configs) {
      createStatsWindow(config.kind, config);
    }
  }

  function renderStatsWindow(
    statsWindow: StatsWindowState,
    frameIndex = deps.getDefaultFrameIndex(),
  ): void {
    const activeElement = document.activeElement;
    const searchFocused =
      activeElement instanceof HTMLInputElement &&
      activeElement.dataset.statsWindowSearch === statsWindow.id;
    const searchSelectionStart = searchFocused ? activeElement.selectionStart : null;
    const searchSelectionEnd = searchFocused ? activeElement.selectionEnd : null;
    const searchSelectionDirection = searchFocused ? activeElement.selectionDirection : null;

    statsWindow.body.replaceChildren();

    renderStatsWindowScope(statsWindow, {
      getReplayPlayer: deps.getReplayPlayer,
      renderStatsWindow,
      scheduleConfigUrlUpdate: deps.scheduleConfigUrlUpdate,
    });
    if (hasStatsWindowStatPicker(statsWindow.kind)) {
      renderStatsWindowAddControl(statsWindow, {
        getAllowedScope: getStatsWindowAllowedScope,
        getDefaultAdHocTargetId: (definition) =>
          getDefaultAdHocTargetId(deps.getReplayPlayer(), definition),
        getStatRegistry: deps.getStatRegistry,
        hasScopeSelector: hasStatsWindowScopeSelector,
        renderStatsWindow,
        scheduleConfigUrlUpdate: deps.scheduleConfigUrlUpdate,
      });
      renderStatsWindowPicker(statsWindow, {
        getAllowedScope: getStatsWindowAllowedScope,
        getDefaultAdHocTargetId: (definition) =>
          getDefaultAdHocTargetId(deps.getReplayPlayer(), definition),
        getStatRegistry: deps.getStatRegistry,
        hasScopeSelector: hasStatsWindowScopeSelector,
        renderStatsWindow,
        scheduleConfigUrlUpdate: deps.scheduleConfigUrlUpdate,
      });
    }
    renderStatsWindowEntries(statsWindow, frameIndex, {
      getReplayPlayer: deps.getReplayPlayer,
      getStatById,
      getStatsFrame: deps.getStatsFrame,
      getStatsTimeline: deps.getStatsTimeline,
      renderStatsWindow,
      scheduleConfigUrlUpdate: deps.scheduleConfigUrlUpdate,
      cueGoalReplay: deps.cueGoalReplay,
      formatTime: deps.formatTime,
      watchGoalReplay: deps.watchGoalReplay,
    });

    if (searchFocused) {
      const searchInput = statsWindow.body.querySelector<HTMLInputElement>(
        `input[data-stats-window-search="${statsWindow.id}"]`,
      );
      searchInput?.focus({ preventScroll: true });
      if (searchInput && searchSelectionStart !== null && searchSelectionEnd !== null) {
        searchInput.setSelectionRange(
          searchSelectionStart,
          searchSelectionEnd,
          searchSelectionDirection ?? "none",
        );
      }
    }
  }

  function clear(): void {
    for (const statsWindow of statsWindows.values()) {
      statsWindow.element.remove();
    }
    statsWindows.clear();
    nextStatsWindowId = 1;
  }

  return {
    clear,
    create: createStatsWindow,
    getConfigs() {
      return [...statsWindows.values()].map(getStatsWindowConfig);
    },
    render: renderStatsWindows,
    replaceFromConfig: replaceStatsWindowsFromConfig,
  };
}
