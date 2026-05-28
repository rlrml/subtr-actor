import type { ReplayPlayer } from "@rlrml/player";
import type { StatDefinition } from "./statRegistry.ts";
import { getTeamClass } from "./statModules.ts";
import {
  appendGroupedPlayerOptions,
  getStatTargetTeamClass,
  getStatsWindowAllowedScope,
  getTeamLabel,
  getTeamScopeClass,
  getTeamSnapshot,
} from "./statsWindowScope.ts";
import { renderGoalLabelsOverview } from "./statsWindowGoalLabels.ts";
import type { SelectedStatEntry, StatsWindowState } from "./statsWindowTypes.ts";
import type {
  PlayerStatsSnapshot,
  StatsFrame,
  StatsTimeline,
  TeamStatsSnapshot,
} from "./statsTimeline.ts";
import { playerIdToString } from "./touchOverlay.ts";

interface StatsWindowEntriesRenderDeps {
  getReplayPlayer(): ReplayPlayer | null;
  getStatById(statId: string): StatDefinition | null;
  getStatsFrame(frameIndex: number): StatsFrame | null;
  getStatsTimeline(): StatsTimeline | null;
  renderStatsWindow(statsWindow: StatsWindowState): void;
  scheduleConfigUrlUpdate(): void;
  cueGoalReplay(time: number): void;
  formatTime(seconds: number): string;
  watchGoalReplay(time: number, scorerId: string | null): void;
}

export function renderStatsWindowEntries(
  statsWindow: StatsWindowState,
  frameIndex: number,
  deps: StatsWindowEntriesRenderDeps,
): void {
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
    .map((entry) => ({ entry, definition: deps.getStatById(entry.statId) }))
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
    renderAllPlayersStats(statsWindow, frame, entries, deps);
    return;
  }
  if (statsWindow.kind === "all-teams") {
    renderAllTeamsStats(statsWindow, frame, entries, deps);
    return;
  }
  if (statsWindow.kind === "player") {
    const player = statsWindow.playerId
      ? (frame.players.find(
          (candidate) => playerIdToString(candidate.player_id) === statsWindow.playerId,
        ) ?? null)
      : null;
    renderScopedStatList(statsWindow, player, entries, deps);
    return;
  }
  if (statsWindow.kind === "team") {
    renderScopedStatList(
      statsWindow,
      getTeamSnapshot(frame, statsWindow.team ?? "blue"),
      entries,
      deps,
    );
    return;
  }
  if (statsWindow.kind === "ad-hoc") {
    renderAdHocStats(statsWindow, frame, entries, deps);
  }
}

function renderScopedStatList(
  statsWindow: StatsWindowState,
  target: PlayerStatsSnapshot | TeamStatsSnapshot | null,
  entries: Array<{ entry: SelectedStatEntry; definition: StatDefinition }>,
  deps: StatsWindowEntriesRenderDeps,
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
        deps,
      ),
    );
  }
  statsWindow.body.append(list);
}

function renderAllPlayersStats(
  statsWindow: StatsWindowState,
  frame: StatsFrame,
  entries: Array<{ entry: SelectedStatEntry; definition: StatDefinition }>,
  deps: StatsWindowEntriesRenderDeps,
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
            deps,
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
  deps: StatsWindowEntriesRenderDeps,
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
          deps,
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
  deps: StatsWindowEntriesRenderDeps,
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
        deps,
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
  deps: StatsWindowEntriesRenderDeps,
): HTMLElement {
  const row = document.createElement("div");
  row.className = "stats-window-stat-row";
  const name = document.createElement("span");
  name.className = "stats-window-stat-name";
  name.textContent = definition.label;
  if (statsWindow.kind === "ad-hoc") {
    const targetSelect = document.createElement("select");
    targetSelect.className = "stats-window-stat-target";
    const teamClass = getStatTargetTeamClass(deps.getReplayPlayer(), definition, entry.targetId);
    if (teamClass) {
      targetSelect.classList.add(teamClass);
    }
    if (definition.scope === "player") {
      appendGroupedPlayerOptions(deps.getReplayPlayer(), targetSelect, entry.targetId);
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
        deps.renderStatsWindow(statsWindow);
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
      deps.renderStatsWindow(statsWindow);
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
    deps.renderStatsWindow(statsWindow);
    deps.scheduleConfigUrlUpdate();
  });
  row.append(name, valueEl, remove);
  return row;
}

function removeStatFromWindow(statsWindow: StatsWindowState, entryKey: string): void {
  const index = statsWindow.entries.findIndex((entry) => entry.key === entryKey);
  if (index >= 0) {
    statsWindow.entries.splice(index, 1);
  }
}
