import type { ReplayPlayer } from "@rlrml/player";
import type {
  StatsWindowConfig,
  StatsWindowKind,
  TeamScope,
  WindowPlacementConfig,
} from "./playerConfig.ts";
import { getStatDefinitionSearchMatches } from "./statSearch.ts";
import type { StatDefinition, StatScopeKind } from "./statRegistry.ts";
import { getTeamClass } from "./statModules.ts";
import { getStatsFrameForReplayFrame } from "./statsTimeline.ts";
import type {
  PlayerStatsSnapshot,
  StatsFrame,
  StatsFrameLookup,
  StatsTimeline,
  TeamStatsSnapshot,
} from "./statsTimeline.ts";
import { formatMechanicKind } from "./timelineMarkers.ts";
import { playerIdToString } from "./touchOverlay.ts";

interface SelectedStatEntry {
  key: string;
  statId: string;
  targetId?: string;
}

interface StatsWindowState {
  readonly id: string;
  readonly kind: StatsWindowKind;
  readonly entries: SelectedStatEntry[];
  playerId: string | null;
  team: TeamScope | null;
  pickerOpen: boolean;
  query: string;
  element: HTMLElement;
  body: HTMLElement;
}

export interface StatsWindowsControllerOptions {
  readonly layer: HTMLElement;
  getReplayPlayer(): ReplayPlayer | null;
  getStatsTimeline(): StatsTimeline | null;
  getStatsFrameLookup(): StatsFrameLookup | null;
  getStatRegistry(): StatDefinition[];
  readWindowPlacement(windowEl: HTMLElement): WindowPlacementConfig;
  applyWindowPlacement(windowEl: HTMLElement, placement: WindowPlacementConfig): void;
  bringWindowToFront(windowEl: HTMLElement): void;
  setLauncherOpen(open: boolean): void;
  requestConfigSync(): void;
  watchGoalReplay(time: number, scorerId: string | null): void;
  cueGoalReplay(time: number): void;
}

export interface RenderStatsWindowsOptions {
  preserveOpenPickers?: boolean;
}

export function formatTime(seconds: number): string {
  if (!Number.isFinite(seconds)) {
    return "--";
  }
  const minutes = Math.floor(Math.max(0, seconds) / 60);
  const remainingSeconds = Math.max(0, seconds) - minutes * 60;
  return `${minutes}:${remainingSeconds.toFixed(1).padStart(4, "0")}`;
}

export class StatsWindowsController {
  private readonly statsWindows = new Map<string, StatsWindowState>();
  private nextStatsWindowId = 1;

  constructor(private readonly options: StatsWindowsControllerOptions) {}

  getConfigs(): StatsWindowConfig[] {
    return [...this.statsWindows.values()].map((statsWindow) => ({
      id: statsWindow.id,
      kind: statsWindow.kind,
      placement: this.options.readWindowPlacement(statsWindow.element),
      playerId: statsWindow.playerId,
      team: statsWindow.team,
      entries: statsWindow.entries.map((entry) => ({
        statId: entry.statId,
        targetId: entry.targetId,
      })),
    }));
  }

  clear(): void {
    for (const statsWindow of this.statsWindows.values()) {
      statsWindow.element.remove();
    }
    this.statsWindows.clear();
    this.nextStatsWindowId = 1;
  }

  replaceFromConfig(configs: readonly StatsWindowConfig[]): void {
    this.clear();
    for (const config of configs) {
      this.create(config.kind, config);
    }
  }

  render(
    frameIndex = this.options.getReplayPlayer()?.getState().frameIndex ?? 0,
    options: RenderStatsWindowsOptions = {},
  ): void {
    for (const statsWindow of this.statsWindows.values()) {
      if (
        options.preserveOpenPickers &&
        (statsWindow.pickerOpen || statsWindow.element.contains(document.activeElement))
      ) {
        continue;
      }
      this.renderStatsWindow(statsWindow, frameIndex);
    }
  }

  create(kind: StatsWindowKind, config?: StatsWindowConfig): StatsWindowState {
    const id = config?.id ?? `stats-${this.nextStatsWindowId++}`;
    const idNumber = Number.parseInt(id.replace(/^stats-/, ""), 10);
    if (Number.isFinite(idNumber)) {
      this.nextStatsWindowId = Math.max(this.nextStatsWindowId, idNumber + 1);
    }
    const { x, y } = this.getStatsWindowDefaultPosition();
    const element = document.createElement("section");
    element.className = "stats-window";
    element.dataset.windowId = id;
    element.style.setProperty("--window-x", `${x}px`);
    element.style.setProperty("--window-y", `${y}px`);
    if (config) {
      this.options.applyWindowPlacement(element, config.placement);
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
    if (this.hasStatsWindowScopeSelector(kind)) {
      header.classList.add("stats-window-header-actions-only");
      header.append(actions);
    } else {
      const title = document.createElement("h2");
      title.textContent = this.getStatsWindowTitle(kind);
      header.append(title, actions);
    }

    const body = document.createElement("div");
    body.className = "stats-window-body";
    element.append(header, body);
    this.options.layer.append(element);

    const state: StatsWindowState = {
      id,
      kind,
      entries:
        config?.entries.map((entry) => ({
          key: `${id}:${entry.statId}:${entry.targetId ?? "scope"}`,
          statId: entry.statId,
          targetId: entry.targetId,
        })) ?? [],
      playerId: config?.playerId ?? this.options.getReplayPlayer()?.replay.players[0]?.id ?? null,
      team: config?.team ?? "blue",
      pickerOpen: false,
      query: "",
      element,
      body,
    };

    hideButton.addEventListener("click", () => {
      element.hidden = true;
      this.options.requestConfigSync();
    });

    this.statsWindows.set(id, state);
    if (!config) {
      this.options.bringWindowToFront(element);
    }
    this.options.setLauncherOpen(false);
    this.renderStatsWindow(state);
    this.options.requestConfigSync();
    return state;
  }

  private getStatById(statId: string): StatDefinition | null {
    return this.options.getStatRegistry().find((definition) => definition.id === statId) ?? null;
  }

  private getCurrentStatsFrame(frameIndex: number): StatsFrame | null {
    const lookup = this.options.getStatsFrameLookup();
    return lookup ? getStatsFrameForReplayFrame(lookup, frameIndex) : null;
  }

  private getTeamSnapshot(frame: StatsFrame, team: TeamScope): TeamStatsSnapshot | null {
    return team === "blue" ? (frame.team_zero ?? null) : (frame.team_one ?? null);
  }

  private getTeamLabel(team: TeamScope): string {
    return team === "blue" ? "Blue" : "Orange";
  }

  private getPlayerTeamClass(playerId: string | null | undefined): string | null {
    const player = this.options
      .getReplayPlayer()
      ?.replay.players.find((candidate) => candidate.id === playerId);
    return player ? getTeamClass(player.isTeamZero) : null;
  }

  private getTeamScopeClass(team: TeamScope): string {
    return getTeamClass(team === "blue");
  }

  private appendGroupedPlayerOptions(
    select: HTMLSelectElement,
    selectedPlayerId: string | null | undefined,
  ): void {
    const players = this.options.getReplayPlayer()?.replay.players ?? [];
    for (const team of ["blue", "orange"] as const) {
      const teamPlayers = players.filter((player) => player.isTeamZero === (team === "blue"));
      if (teamPlayers.length === 0) {
        continue;
      }

      const group = document.createElement("optgroup");
      group.label = `${this.getTeamLabel(team)} team`;
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

  private getStatsWindowScopeTeamClass(statsWindow: StatsWindowState): string | null {
    if (statsWindow.kind === "player") {
      return this.getPlayerTeamClass(statsWindow.playerId);
    }
    if (statsWindow.kind === "team") {
      return this.getTeamScopeClass(statsWindow.team ?? "blue");
    }
    return null;
  }

  private getStatTargetTeamClass(
    definition: StatDefinition,
    targetId: string | undefined,
  ): string | null {
    if (definition.scope === "player") {
      return this.getPlayerTeamClass(targetId);
    }
    return this.getTeamScopeClass(targetId === "orange" ? "orange" : "blue");
  }

  private getStatsWindowTitle(kind: StatsWindowKind): string {
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

  private hasStatsWindowScopeSelector(kind: StatsWindowKind): boolean {
    return kind === "player" || kind === "team";
  }

  private hasStatsWindowStatPicker(kind: StatsWindowKind): boolean {
    return kind !== "goals-overview";
  }

  private getStatsWindowAllowedScope(kind: StatsWindowKind): StatScopeKind | null {
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

  private getStatsWindowDefaultPosition(): { x: number; y: number } {
    const offset = this.statsWindows.size * 18;
    return {
      x: Math.max(12, Math.min(window.innerWidth - 360, 96 + offset)),
      y: Math.max(64, Math.min(window.innerHeight - 240, 96 + offset)),
    };
  }

  private renderStatsWindow(
    statsWindow: StatsWindowState,
    frameIndex = this.options.getReplayPlayer()?.getState().frameIndex ?? 0,
  ): void {
    const activeElement = document.activeElement;
    const searchFocused =
      activeElement instanceof HTMLInputElement &&
      activeElement.dataset.statsWindowSearch === statsWindow.id;
    const searchSelectionStart = searchFocused ? activeElement.selectionStart : null;
    const searchSelectionEnd = searchFocused ? activeElement.selectionEnd : null;
    const searchSelectionDirection = searchFocused ? activeElement.selectionDirection : null;

    statsWindow.body.replaceChildren();

    this.renderStatsWindowScope(statsWindow);
    if (this.hasStatsWindowStatPicker(statsWindow.kind)) {
      this.renderStatsWindowAddControl(statsWindow);
      this.renderStatsWindowPicker(statsWindow);
    }
    this.renderStatsWindowEntries(statsWindow, frameIndex);

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

  private renderStatsWindowScope(statsWindow: StatsWindowState): void {
    if (statsWindow.kind !== "player" && statsWindow.kind !== "team") {
      return;
    }

    const row = document.createElement("div");
    row.className = "stats-window-scope-row";

    const select = document.createElement("select");
    select.className = "stats-window-scope-select";
    const teamClass = this.getStatsWindowScopeTeamClass(statsWindow);
    if (teamClass) {
      select.classList.add(teamClass);
    }
    select.setAttribute(
      "aria-label",
      statsWindow.kind === "player" ? "Player stats target" : "Team stats target",
    );
    if (statsWindow.kind === "player") {
      this.appendGroupedPlayerOptions(select, statsWindow.playerId);
      select.value = statsWindow.playerId ?? "";
      select.addEventListener("change", () => {
        statsWindow.playerId = select.value || null;
        this.renderStatsWindow(statsWindow);
        this.options.requestConfigSync();
      });
    } else {
      select.append(
        new Option("Blue", "blue", statsWindow.team === "blue", statsWindow.team === "blue"),
        new Option("Orange", "orange", statsWindow.team === "orange", statsWindow.team === "orange"),
      );
      select.value = statsWindow.team ?? "blue";
      select.addEventListener("change", () => {
        statsWindow.team = select.value === "orange" ? "orange" : "blue";
        this.renderStatsWindow(statsWindow);
        this.options.requestConfigSync();
      });
    }

    row.append(select);
    statsWindow.body.append(row);
  }

  private renderStatsWindowAddControl(statsWindow: StatsWindowState): void {
    const button = document.createElement("button");
    button.type = "button";
    button.className = "stats-window-add-button";
    button.textContent = "+";
    button.title = "Add stat";
    button.setAttribute("aria-label", "Add stat");
    button.setAttribute("aria-expanded", String(statsWindow.pickerOpen));
    this.activateButton(button, () => {
      statsWindow.pickerOpen = !statsWindow.pickerOpen;
      this.renderStatsWindow(statsWindow);
    });

    if (this.hasStatsWindowScopeSelector(statsWindow.kind)) {
      const scopeRow = statsWindow.body.querySelector(".stats-window-scope-row");
      scopeRow?.append(button);
      return;
    }

    const toolbar = document.createElement("div");
    toolbar.className = "stats-window-toolbar";
    toolbar.append(button);
    statsWindow.body.append(toolbar);
  }

  private activateButton(button: HTMLButtonElement, callback: () => void): void {
    let pointerActivated = false;
    button.addEventListener("pointerdown", (event) => {
      if (button.disabled) {
        return;
      }
      pointerActivated = true;
      event.preventDefault();
      callback();
    });
    button.addEventListener("click", () => {
      if (pointerActivated) {
        pointerActivated = false;
        return;
      }
      if (!button.disabled) {
        callback();
      }
    });
  }

  private renderStatsWindowPicker(statsWindow: StatsWindowState): void {
    const picker = document.createElement("div");
    picker.className = "stats-window-picker";
    picker.hidden = !statsWindow.pickerOpen;
    if (picker.hidden) {
      statsWindow.body.append(picker);
      return;
    }

    const allowedScope = this.getStatsWindowAllowedScope(statsWindow.kind);
    const queryInput = document.createElement("input");
    queryInput.type = "search";
    queryInput.placeholder = "Search stats";
    queryInput.value = statsWindow.query;
    queryInput.dataset.statsWindowSearch = statsWindow.id;

    const list = document.createElement("div");
    list.className = "stats-window-picker-list";
    queryInput.addEventListener("input", () => {
      statsWindow.query = queryInput.value;
      this.renderStatsWindowPickerList(statsWindow, list, allowedScope);
    });

    this.renderStatsWindowPickerList(statsWindow, list, allowedScope);

    picker.append(queryInput, list);
    statsWindow.body.append(picker);
  }

  private renderStatsWindowPickerList(
    statsWindow: StatsWindowState,
    list: HTMLElement,
    allowedScope: StatScopeKind | null,
  ): void {
    list.replaceChildren();

    const statRegistry = this.options.getStatRegistry();
    const scopeDefinitions = allowedScope
      ? statRegistry.filter((definition) => definition.scope === allowedScope)
      : statRegistry;
    const definitions = getStatDefinitionSearchMatches(scopeDefinitions, statsWindow.query);

    const groupByCategory = new Map<string, StatDefinition[]>();
    for (const definition of definitions) {
      const group = groupByCategory.get(definition.category) ?? [];
      group.push(definition);
      groupByCategory.set(definition.category, group);
    }

    for (const [category, group] of groupByCategory) {
      if (group.length < 2) continue;
      const addGroup = document.createElement("button");
      addGroup.type = "button";
      addGroup.className = "stats-window-picker-item";
      addGroup.innerHTML = `<span>Add all ${category}</span><strong>${group.length}</strong>`;
      this.activateButton(addGroup, () => {
        for (const definition of group) {
          this.addStatToWindow(statsWindow, definition);
        }
        this.renderStatsWindow(statsWindow);
        this.options.requestConfigSync();
      });
      list.append(addGroup);
    }

    for (const definition of definitions) {
      const item = document.createElement("button");
      item.type = "button";
      item.className = "stats-window-picker-item";
      item.innerHTML = `<span>${definition.label}</span><strong>${definition.scope}</strong>`;
      item.disabled =
        statsWindow.kind !== "ad-hoc" &&
        statsWindow.entries.some((entry) => entry.statId === definition.id);
      this.activateButton(item, () => {
        this.addStatToWindow(statsWindow, definition);
        this.renderStatsWindow(statsWindow);
        this.options.requestConfigSync();
      });
      list.append(item);
    }

    if (definitions.length === 0) {
      const empty = document.createElement("p");
      empty.className = "stat-window-empty";
      empty.textContent = statRegistry.length === 0 ? "No stats available." : "No matching stats.";
      list.append(empty);
    }
  }

  private addStatToWindow(statsWindow: StatsWindowState, definition: StatDefinition): void {
    const targetId = statsWindow.kind === "ad-hoc" ? this.getDefaultAdHocTargetId(definition) : undefined;
    if (
      statsWindow.entries.some(
        (entry) => entry.statId === definition.id && entry.targetId === targetId,
      )
    ) {
      return;
    }
    statsWindow.entries.push({
      key: `${statsWindow.id}:${definition.id}:${targetId ?? "scope"}`,
      statId: definition.id,
      targetId,
    });
  }

  private getDefaultAdHocTargetId(definition: StatDefinition): string {
    if (definition.scope === "player") {
      return this.options.getReplayPlayer()?.replay.players[0]?.id ?? "";
    }
    return "blue";
  }

  private removeStatFromWindow(statsWindow: StatsWindowState, entryKey: string): void {
    const index = statsWindow.entries.findIndex((entry) => entry.key === entryKey);
    if (index >= 0) {
      statsWindow.entries.splice(index, 1);
    }
  }

  private renderStatsWindowEntries(statsWindow: StatsWindowState, frameIndex: number): void {
    if (statsWindow.kind === "goals-overview") {
      this.renderGoalLabelsOverview(statsWindow);
      return;
    }

    const allowedScope = this.getStatsWindowAllowedScope(statsWindow.kind);
    const entries = statsWindow.entries
      .map((entry) => ({ entry, definition: this.getStatById(entry.statId) }))
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

    const frame = this.getCurrentStatsFrame(frameIndex);
    if (!frame) {
      const empty = document.createElement("p");
      empty.className = "stat-window-empty";
      empty.textContent = "Load a replay to show stats.";
      statsWindow.body.append(empty);
      return;
    }

    if (statsWindow.kind === "all-players") {
      this.renderAllPlayersStats(statsWindow, frame, entries);
      return;
    }
    if (statsWindow.kind === "all-teams") {
      this.renderAllTeamsStats(statsWindow, frame, entries);
      return;
    }
    if (statsWindow.kind === "player") {
      const player = statsWindow.playerId
        ? (frame.players.find(
            (candidate) => playerIdToString(candidate.player_id) === statsWindow.playerId,
          ) ?? null)
        : null;
      this.renderScopedStatList(statsWindow, player, entries);
      return;
    }
    if (statsWindow.kind === "team") {
      this.renderScopedStatList(statsWindow, this.getTeamSnapshot(frame, statsWindow.team ?? "blue"), entries);
      return;
    }
    if (statsWindow.kind === "ad-hoc") {
      this.renderAdHocStats(statsWindow, frame, entries);
    }
  }

  private renderGoalLabelsOverview(statsWindow: StatsWindowState): void {
    const timeline = this.options.getStatsTimeline();
    const replay = this.options.getReplayPlayer()?.replay ?? null;
    if (!timeline || !replay) {
      this.appendStatsWindowEmpty(statsWindow, "Load a replay to show goal labels.");
      return;
    }

    const goalContexts = [...(timeline.events.goal_context ?? [])].sort(
      (left, right) => left.time - right.time,
    );
    const tagsByGoalIndex = new Map<number, typeof timeline.events.goal_tags>();
    for (const tag of timeline.events.goal_tags ?? []) {
      const group = tagsByGoalIndex.get(tag.goal_index) ?? [];
      group.push(tag);
      tagsByGoalIndex.set(tag.goal_index, group);
    }
    for (const group of tagsByGoalIndex.values()) {
      group.sort(
        (left, right) => left.kind.localeCompare(right.kind) || right.confidence - left.confidence,
      );
    }

    const goalIndexes = new Set<number>(goalContexts.map((_, index) => index));
    for (const index of tagsByGoalIndex.keys()) {
      goalIndexes.add(index);
    }
    const orderedGoalIndexes = [...goalIndexes].sort((left, right) => left - right);
    if (orderedGoalIndexes.length === 0) {
      this.appendStatsWindowEmpty(statsWindow, "No goals loaded.");
      return;
    }

    const list = document.createElement("div");
    list.className = "goal-label-list";
    for (const goalIndex of orderedGoalIndexes) {
      const context = goalContexts[goalIndex] ?? null;
      const tags = tagsByGoalIndex.get(goalIndex) ?? [];
      const firstTag = tags[0] ?? null;
      const time = context?.time ?? firstTag?.time ?? 0;
      const scorer = context?.scorer ?? firstTag?.scorer ?? null;
      const scorerId = scorer ? playerIdToString(scorer) : null;
      const scorerName = scorer
        ? (replay.players.find((player) => player.id === scorerId)?.name ?? scorerId)
        : "Unknown scorer";
      const isTeamZero = context?.scoring_team_is_team_0 ?? firstTag?.scoring_team_is_team_0 ?? null;

      const item = document.createElement("section");
      item.className = "goal-label-item";
      if (isTeamZero !== null) {
        item.classList.add(getTeamClass(isTeamZero));
      }

      const header = document.createElement("header");
      const title = document.createElement("h3");
      title.textContent = `Goal ${goalIndex + 1}`;
      const meta = document.createElement("span");
      meta.textContent = `${formatTime(time)} · ${scorerName}`;
      header.append(title, meta);

      const labels = document.createElement("div");
      labels.className = "goal-label-tags";
      if (tags.length === 0) {
        const empty = document.createElement("span");
        empty.className = "goal-label-tag goal-label-tag-empty";
        empty.textContent = "Unlabeled";
        labels.append(empty);
      } else {
        for (const tag of tags) {
          const chip = document.createElement("span");
          chip.className = "goal-label-tag";
          chip.textContent = `${formatMechanicKind(tag.kind)} ${Math.round(tag.confidence * 100)}%`;
          labels.append(chip);
        }
      }

      const actions = document.createElement("div");
      actions.className = "goal-label-actions";
      const watch = document.createElement("button");
      watch.type = "button";
      watch.className = "goal-label-watch";
      watch.textContent = "Watch";
      watch.addEventListener("click", () => {
        this.options.watchGoalReplay(time, scorerId);
      });
      const jump = document.createElement("button");
      jump.type = "button";
      jump.textContent = "Cue";
      jump.addEventListener("click", () => {
        this.options.cueGoalReplay(time);
      });
      actions.append(watch, jump);

      item.append(header, labels, actions);
      list.append(item);
    }
    statsWindow.body.append(list);
  }

  private appendStatsWindowEmpty(statsWindow: StatsWindowState, message: string): void {
    const empty = document.createElement("p");
    empty.className = "stat-window-empty";
    empty.textContent = message;
    statsWindow.body.append(empty);
  }

  private renderScopedStatList(
    statsWindow: StatsWindowState,
    target: PlayerStatsSnapshot | TeamStatsSnapshot | null,
    entries: Array<{ entry: SelectedStatEntry; definition: StatDefinition }>,
  ): void {
    const list = document.createElement("div");
    list.className = "stats-window-stat-list";
    for (const { entry, definition } of entries) {
      list.append(
        this.renderStatRow(
          statsWindow,
          entry,
          definition,
          target ? definition.format(definition.read(target)) : "--",
        ),
      );
    }
    statsWindow.body.append(list);
  }

  private renderAllPlayersStats(
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
      teamSection.className = `stats-window-team-group ${this.getTeamScopeClass(team)}`;

      const teamHeader = document.createElement("header");
      teamHeader.className = "stats-window-team-header";
      const teamTitle = document.createElement("h3");
      teamTitle.textContent = `${this.getTeamLabel(team)} team`;
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
            this.renderStatRow(statsWindow, entry, definition, definition.format(definition.read(player))),
          );
        }
        playerList.append(section);
      }
      teamSection.append(playerList);
      list.append(teamSection);
    }
    statsWindow.body.append(list);
  }

  private renderAllTeamsStats(
    statsWindow: StatsWindowState,
    frame: StatsFrame,
    entries: Array<{ entry: SelectedStatEntry; definition: StatDefinition }>,
  ): void {
    const list = document.createElement("div");
    list.className = "stats-window-entity-list";
    for (const team of ["blue", "orange"] as const) {
      const snapshot = this.getTeamSnapshot(frame, team);
      const section = document.createElement("section");
      section.className = `stats-window-entity ${this.getTeamScopeClass(team)}`;
      const title = document.createElement("h3");
      title.className = "stats-window-entity-title";
      title.textContent = this.getTeamLabel(team);
      section.append(title);
      for (const { entry, definition } of entries) {
        section.append(
          this.renderStatRow(
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

  private renderAdHocStats(
    statsWindow: StatsWindowState,
    frame: StatsFrame,
    entries: Array<{ entry: SelectedStatEntry; definition: StatDefinition }>,
  ): void {
    const list = document.createElement("div");
    list.className = "stats-window-stat-list";
    for (const { entry, definition } of entries) {
      const target = this.getAdHocTarget(frame, definition, entry.targetId);
      list.append(
        this.renderStatRow(
          statsWindow,
          entry,
          definition,
          target ? definition.format(definition.read(target)) : "--",
        ),
      );
    }
    statsWindow.body.append(list);
  }

  private getAdHocTarget(
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
    return this.getTeamSnapshot(frame, targetId === "orange" ? "orange" : "blue");
  }

  private renderStatRow(
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
      const teamClass = this.getStatTargetTeamClass(definition, entry.targetId);
      if (teamClass) {
        targetSelect.classList.add(teamClass);
      }
      if (definition.scope === "player") {
        this.appendGroupedPlayerOptions(targetSelect, entry.targetId);
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
          this.renderStatsWindow(statsWindow);
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
        this.renderStatsWindow(statsWindow);
        this.options.requestConfigSync();
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
      this.removeStatFromWindow(statsWindow, entry.key);
      this.renderStatsWindow(statsWindow);
      this.options.requestConfigSync();
    });
    row.append(name, valueEl, remove);
    return row;
  }
}

export function createStatsWindowsController(
  options: StatsWindowsControllerOptions,
): StatsWindowsController {
  return new StatsWindowsController(options);
}
