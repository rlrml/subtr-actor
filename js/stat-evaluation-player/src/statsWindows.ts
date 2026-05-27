import type { ReplayPlayer } from "@rlrml/player";
import type {
  StatsWindowConfig,
  StatsWindowKind,
  TeamScope,
  WindowPlacementConfig,
} from "./playerConfig.ts";
import type { StatDefinition, StatScopeKind } from "./statRegistry.ts";
import { getTeamClass } from "./statModules.ts";
import {
  renderStatsWindowAddControl,
  renderStatsWindowPicker,
} from "./statsWindowPicker.ts";
import { renderGoalLabelsOverview } from "./statsWindowGoalLabels.ts";
import type { SelectedStatEntry, StatsWindowState } from "./statsWindowTypes.ts";
import type {
  PlayerStatsSnapshot,
  StatsFrame,
  StatsTimeline,
  TeamStatsSnapshot,
} from "./statsTimeline.ts";
import { playerIdToString } from "./touchOverlay.ts";

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

  function getTeamSnapshot(frame: StatsFrame, team: TeamScope): TeamStatsSnapshot | null {
    return team === "blue" ? (frame.team_zero ?? null) : (frame.team_one ?? null);
  }

  function getTeamLabel(team: TeamScope): string {
    return team === "blue" ? "Blue" : "Orange";
  }

  function getPlayerTeamClass(playerId: string | null | undefined): string | null {
    const player = deps.getReplayPlayer()?.replay.players.find((candidate) => candidate.id === playerId);
    return player ? getTeamClass(player.isTeamZero) : null;
  }

  function getTeamScopeClass(team: TeamScope): string {
    return getTeamClass(team === "blue");
  }

  function appendGroupedPlayerOptions(
    select: HTMLSelectElement,
    selectedPlayerId: string | null | undefined,
  ): void {
    const players = deps.getReplayPlayer()?.replay.players ?? [];
    for (const team of ["blue", "orange"] as const) {
      const teamPlayers = players.filter((player) => player.isTeamZero === (team === "blue"));
      if (teamPlayers.length === 0) {
        continue;
      }

      const group = document.createElement("optgroup");
      group.label = `${getTeamLabel(team)} team`;
      for (const player of teamPlayers) {
        group.append(
          new Option(
            player.name,
            player.id,
            player.id === selectedPlayerId,
            player.id === selectedPlayerId,
          ),
        );
      }
      select.append(group);
    }
  }

  function getStatsWindowScopeTeamClass(statsWindow: StatsWindowState): string | null {
    if (statsWindow.kind === "player") {
      return getPlayerTeamClass(statsWindow.playerId);
    }
    if (statsWindow.kind === "team") {
      return getTeamScopeClass(statsWindow.team ?? "blue");
    }
    return null;
  }

  function getStatTargetTeamClass(
    definition: StatDefinition,
    targetId: string | undefined,
  ): string | null {
    if (definition.scope === "player") {
      return getPlayerTeamClass(targetId);
    }
    return getTeamScopeClass(targetId === "orange" ? "orange" : "blue");
  }

  function getStatsWindowTitle(kind: StatsWindowKind): string {
    switch (kind) {
      case "player":
        return "Player stats";
      case "team":
        return "Team stats";
      case "all-players":
        return "All players stats";
      case "all-teams":
        return "All teams stats";
      case "goals-overview":
        return "Goal labels";
      case "ad-hoc":
        return "Ad hoc stats";
    }
  }

  function hasStatsWindowScopeSelector(kind: StatsWindowKind): boolean {
    return kind === "player" || kind === "team";
  }

  function hasStatsWindowStatPicker(kind: StatsWindowKind): boolean {
    return kind !== "goals-overview";
  }

  function getStatsWindowAllowedScope(kind: StatsWindowKind): StatScopeKind | null {
    switch (kind) {
      case "player":
      case "all-players":
        return "player";
      case "team":
      case "all-teams":
        return "team";
      case "goals-overview":
        return null;
      case "ad-hoc":
        return null;
    }
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

    renderStatsWindowScope(statsWindow);
    if (hasStatsWindowStatPicker(statsWindow.kind)) {
      renderStatsWindowAddControl(statsWindow, {
        getAllowedScope: getStatsWindowAllowedScope,
        getDefaultAdHocTargetId,
        getStatRegistry: deps.getStatRegistry,
        hasScopeSelector: hasStatsWindowScopeSelector,
        renderStatsWindow,
        scheduleConfigUrlUpdate: deps.scheduleConfigUrlUpdate,
      });
      renderStatsWindowPicker(statsWindow, {
        getAllowedScope: getStatsWindowAllowedScope,
        getDefaultAdHocTargetId,
        getStatRegistry: deps.getStatRegistry,
        hasScopeSelector: hasStatsWindowScopeSelector,
        renderStatsWindow,
        scheduleConfigUrlUpdate: deps.scheduleConfigUrlUpdate,
      });
    }
    renderStatsWindowEntries(statsWindow, frameIndex);

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

  function renderStatsWindowScope(statsWindow: StatsWindowState): void {
    if (statsWindow.kind !== "player" && statsWindow.kind !== "team") {
      return;
    }

    const row = document.createElement("div");
    row.className = "stats-window-scope-row";

    const select = document.createElement("select");
    select.className = "stats-window-scope-select";
    const teamClass = getStatsWindowScopeTeamClass(statsWindow);
    if (teamClass) {
      select.classList.add(teamClass);
    }
    select.setAttribute(
      "aria-label",
      statsWindow.kind === "player" ? "Player stats target" : "Team stats target",
    );
    if (statsWindow.kind === "player") {
      appendGroupedPlayerOptions(select, statsWindow.playerId);
      select.value = statsWindow.playerId ?? "";
      select.addEventListener("change", () => {
        statsWindow.playerId = select.value || null;
        renderStatsWindow(statsWindow);
        deps.scheduleConfigUrlUpdate();
      });
    } else {
      select.append(
        new Option("Blue", "blue", statsWindow.team === "blue", statsWindow.team === "blue"),
        new Option(
          "Orange",
          "orange",
          statsWindow.team === "orange",
          statsWindow.team === "orange",
        ),
      );
      select.value = statsWindow.team ?? "blue";
      select.addEventListener("change", () => {
        statsWindow.team = select.value === "orange" ? "orange" : "blue";
        renderStatsWindow(statsWindow);
        deps.scheduleConfigUrlUpdate();
      });
    }

    row.append(select);
    statsWindow.body.append(row);
  }

  function getDefaultAdHocTargetId(definition: StatDefinition): string {
    if (definition.scope === "player") {
      return deps.getReplayPlayer()?.replay.players[0]?.id ?? "";
    }
    return "blue";
  }

  function removeStatFromWindow(statsWindow: StatsWindowState, entryKey: string): void {
    const index = statsWindow.entries.findIndex((entry) => entry.key === entryKey);
    if (index >= 0) {
      statsWindow.entries.splice(index, 1);
    }
  }

  function renderStatsWindowEntries(statsWindow: StatsWindowState, frameIndex: number): void {
    if (statsWindow.kind === "goals-overview") {
      renderGoalLabelsOverview(statsWindow.body, {
        getStatsTimeline: deps.getStatsTimeline,
        getReplay: () => deps.getReplayPlayer()?.replay ?? null,
        formatTime: deps.formatTime,
        watchGoalReplay: deps.watchGoalReplay,
        cueGoalReplay: deps.cueGoalReplay,
      });
      return;
    }

    const allowedScope = getStatsWindowAllowedScope(statsWindow.kind);
    const entries = statsWindow.entries
      .map((entry) => ({ entry, definition: getStatById(entry.statId) }))
      .filter(
        (item): item is { entry: SelectedStatEntry; definition: StatDefinition } =>
          item.definition !== null && (!allowedScope || item.definition.scope === allowedScope),
      );

    if (entries.length === 0) {
      const empty = document.createElement("p");
      empty.className = "stat-window-empty";
      empty.textContent = "No stats added.";
      statsWindow.body.append(empty);
      return;
    }

    const frame = deps.getStatsFrame(frameIndex);
    if (!frame) {
      const empty = document.createElement("p");
      empty.className = "stat-window-empty";
      empty.textContent = "Load a replay to show stats.";
      statsWindow.body.append(empty);
      return;
    }

    if (statsWindow.kind === "all-players") {
      renderAllPlayersStats(statsWindow, frame, entries);
      return;
    }
    if (statsWindow.kind === "all-teams") {
      renderAllTeamsStats(statsWindow, frame, entries);
      return;
    }
    if (statsWindow.kind === "player") {
      const player = statsWindow.playerId
        ? (frame.players.find(
            (candidate) => playerIdToString(candidate.player_id) === statsWindow.playerId,
          ) ?? null)
        : null;
      renderScopedStatList(statsWindow, player, entries);
      return;
    }
    if (statsWindow.kind === "team") {
      renderScopedStatList(statsWindow, getTeamSnapshot(frame, statsWindow.team ?? "blue"), entries);
      return;
    }
    if (statsWindow.kind === "ad-hoc") {
      renderAdHocStats(statsWindow, frame, entries);
    }
  }

  function appendStatsWindowEmpty(statsWindow: StatsWindowState, message: string): void {
    const empty = document.createElement("p");
    empty.className = "stat-window-empty";
    empty.textContent = message;
    statsWindow.body.append(empty);
  }

  function renderScopedStatList(
    statsWindow: StatsWindowState,
    target: PlayerStatsSnapshot | TeamStatsSnapshot | null,
    entries: Array<{ entry: SelectedStatEntry; definition: StatDefinition }>,
  ): void {
    const list = document.createElement("div");
    list.className = "stats-window-stat-list";
    for (const { entry, definition } of entries) {
      list.append(
        renderStatRow(
          statsWindow,
          entry,
          definition,
          target ? definition.format(definition.read(target)) : "--",
        ),
      );
    }
    statsWindow.body.append(list);
  }

  function renderAllPlayersStats(
    statsWindow: StatsWindowState,
    frame: StatsFrame,
    entries: Array<{ entry: SelectedStatEntry; definition: StatDefinition }>,
  ): void {
    const list = document.createElement("div");
    list.className = "stats-window-team-list";
    for (const team of ["blue", "orange"] as const) {
      const players = frame.players.filter((player) => player.is_team_0 === (team === "blue"));
      if (players.length === 0) {
        continue;
      }

      const teamSection = document.createElement("section");
      teamSection.className = `stats-window-team-group ${getTeamScopeClass(team)}`;

      const teamHeader = document.createElement("header");
      teamHeader.className = "stats-window-team-header";
      const teamTitle = document.createElement("h3");
      teamTitle.textContent = `${getTeamLabel(team)} team`;
      const teamMeta = document.createElement("span");
      teamMeta.textContent = `${players.length} player${players.length === 1 ? "" : "s"}`;
      teamHeader.append(teamTitle, teamMeta);
      teamSection.append(teamHeader);

      const playerList = document.createElement("div");
      playerList.className = "stats-window-entity-list";
      for (const player of players) {
        const section = document.createElement("section");
        section.className = `stats-window-entity ${getTeamClass(player.is_team_0)}`;
        const title = document.createElement("h4");
        title.className = "stats-window-entity-title";
        title.textContent = player.name;
        section.append(title);
        for (const { entry, definition } of entries) {
          section.append(
            renderStatRow(
              statsWindow,
              entry,
              definition,
              definition.format(definition.read(player)),
            ),
          );
        }
        playerList.append(section);
      }
      teamSection.append(playerList);
      list.append(teamSection);
    }
    statsWindow.body.append(list);
  }

  function renderAllTeamsStats(
    statsWindow: StatsWindowState,
    frame: StatsFrame,
    entries: Array<{ entry: SelectedStatEntry; definition: StatDefinition }>,
  ): void {
    const list = document.createElement("div");
    list.className = "stats-window-entity-list";
    for (const team of ["blue", "orange"] as const) {
      const snapshot = getTeamSnapshot(frame, team);
      const section = document.createElement("section");
      section.className = `stats-window-entity ${getTeamScopeClass(team)}`;
      const title = document.createElement("h3");
      title.className = "stats-window-entity-title";
      title.textContent = getTeamLabel(team);
      section.append(title);
      for (const { entry, definition } of entries) {
        section.append(
          renderStatRow(
            statsWindow,
            entry,
            definition,
            snapshot ? definition.format(definition.read(snapshot)) : "--",
          ),
        );
      }
      list.append(section);
    }
    statsWindow.body.append(list);
  }

  function renderAdHocStats(
    statsWindow: StatsWindowState,
    frame: StatsFrame,
    entries: Array<{ entry: SelectedStatEntry; definition: StatDefinition }>,
  ): void {
    const list = document.createElement("div");
    list.className = "stats-window-stat-list";
    for (const { entry, definition } of entries) {
      const target = getAdHocTarget(frame, definition, entry.targetId);
      list.append(
        renderStatRow(
          statsWindow,
          entry,
          definition,
          target ? definition.format(definition.read(target)) : "--",
        ),
      );
    }
    statsWindow.body.append(list);
  }

  function getAdHocTarget(
    frame: StatsFrame,
    definition: StatDefinition,
    targetId: string | undefined,
  ): PlayerStatsSnapshot | TeamStatsSnapshot | null {
    if (definition.scope === "player") {
      return (
        frame.players.find((player) => playerIdToString(player.player_id) === targetId) ??
        frame.players[0] ??
        null
      );
    }
    return getTeamSnapshot(frame, targetId === "orange" ? "orange" : "blue");
  }

  function renderStatRow(
    statsWindow: StatsWindowState,
    entry: SelectedStatEntry,
    definition: StatDefinition,
    value: string,
  ): HTMLElement {
    const row = document.createElement("div");
    row.className = "stats-window-stat-row";
    const name = document.createElement("span");
    name.className = "stats-window-stat-name";
    name.textContent = definition.label;
    if (statsWindow.kind === "ad-hoc") {
      const targetSelect = document.createElement("select");
      targetSelect.className = "stats-window-stat-target";
      const teamClass = getStatTargetTeamClass(definition, entry.targetId);
      if (teamClass) {
        targetSelect.classList.add(teamClass);
      }
      if (definition.scope === "player") {
        appendGroupedPlayerOptions(targetSelect, entry.targetId);
      } else {
        targetSelect.append(
          new Option("Blue", "blue", entry.targetId === "blue", entry.targetId === "blue"),
          new Option("Orange", "orange", entry.targetId === "orange", entry.targetId === "orange"),
        );
      }
      targetSelect.value = entry.targetId ?? "";
      targetSelect.addEventListener("change", () => {
        const nextTargetId = targetSelect.value;
        if (
          statsWindow.entries.some(
            (candidate) =>
              candidate !== entry &&
              candidate.statId === entry.statId &&
              candidate.targetId === nextTargetId,
          )
        ) {
          renderStatsWindow(statsWindow);
          return;
        }
        const index = statsWindow.entries.findIndex((candidate) => candidate.key === entry.key);
        if (index >= 0) {
          statsWindow.entries[index] = {
            key: `${statsWindow.id}:${entry.statId}:${nextTargetId}`,
            statId: entry.statId,
            targetId: nextTargetId,
          };
        }
        renderStatsWindow(statsWindow);
        deps.scheduleConfigUrlUpdate();
      });
      name.append(" ", targetSelect);
    }
    const valueEl = document.createElement("span");
    valueEl.className = "stats-window-stat-value";
    valueEl.textContent = value;
    const remove = document.createElement("button");
    remove.type = "button";
    remove.className = "stats-window-stat-remove";
    remove.textContent = "x";
    remove.addEventListener("click", () => {
      removeStatFromWindow(statsWindow, entry.key);
      renderStatsWindow(statsWindow);
      deps.scheduleConfigUrlUpdate();
    });
    row.append(name, valueEl, remove);
    return row;
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
