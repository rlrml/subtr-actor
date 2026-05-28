import type { ReplayPlayer } from "@rlrml/player";
import type { StatsWindowKind, TeamScope } from "./playerConfig.ts";
import type { StatDefinition, StatScopeKind } from "./statRegistry.ts";
import { getTeamClass } from "./statModules.ts";
import type { StatsFrame, TeamStatsSnapshot } from "./statsTimeline.ts";
import type { StatsWindowState } from "./statsWindowTypes.ts";

export interface StatsWindowScopeRenderDeps {
  getReplayPlayer(): ReplayPlayer | null;
  renderStatsWindow(statsWindow: StatsWindowState): void;
  scheduleConfigUrlUpdate(): void;
}

export function getTeamSnapshot(frame: StatsFrame, team: TeamScope): TeamStatsSnapshot | null {
  return team === "blue" ? (frame.team_zero ?? null) : (frame.team_one ?? null);
}

export function getTeamLabel(team: TeamScope): string {
  return team === "blue" ? "Blue" : "Orange";
}

export function getPlayerTeamClass(
  replayPlayer: ReplayPlayer | null,
  playerId: string | null | undefined,
): string | null {
  const player = replayPlayer?.replay.players.find((candidate) => candidate.id === playerId);
  return player ? getTeamClass(player.isTeamZero) : null;
}

export function getTeamScopeClass(team: TeamScope): string {
  return getTeamClass(team === "blue");
}

export function appendGroupedPlayerOptions(
  replayPlayer: ReplayPlayer | null,
  select: HTMLSelectElement,
  selectedPlayerId: string | null | undefined,
): void {
  const players = replayPlayer?.replay.players ?? [];
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

export function getStatsWindowScopeTeamClass(
  replayPlayer: ReplayPlayer | null,
  statsWindow: StatsWindowState,
): string | null {
  if (statsWindow.kind === "player") {
    return getPlayerTeamClass(replayPlayer, statsWindow.playerId);
  }
  if (statsWindow.kind === "team") {
    return getTeamScopeClass(statsWindow.team ?? "blue");
  }
  return null;
}

export function getStatTargetTeamClass(
  replayPlayer: ReplayPlayer | null,
  definition: StatDefinition,
  targetId: string | undefined,
): string | null {
  if (definition.scope === "player") {
    return getPlayerTeamClass(replayPlayer, targetId);
  }
  return getTeamScopeClass(targetId === "orange" ? "orange" : "blue");
}

export function getStatsWindowTitle(kind: StatsWindowKind): string {
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

export function hasStatsWindowScopeSelector(kind: StatsWindowKind): boolean {
  return kind === "player" || kind === "team";
}

export function hasStatsWindowStatPicker(kind: StatsWindowKind): boolean {
  return kind !== "goals-overview";
}

export function getStatsWindowAllowedScope(kind: StatsWindowKind): StatScopeKind | null {
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

export function getDefaultAdHocTargetId(
  replayPlayer: ReplayPlayer | null,
  definition: StatDefinition,
): string {
  if (definition.scope === "player") {
    return replayPlayer?.replay.players[0]?.id ?? "";
  }
  return "blue";
}

export function renderStatsWindowScope(
  statsWindow: StatsWindowState,
  deps: StatsWindowScopeRenderDeps,
): void {
  if (statsWindow.kind !== "player" && statsWindow.kind !== "team") {
    return;
  }

  const row = document.createElement("div");
  row.className = "stats-window-scope-row";

  const select = document.createElement("select");
  select.className = "stats-window-scope-select";
  const replayPlayer = deps.getReplayPlayer();
  const teamClass = getStatsWindowScopeTeamClass(replayPlayer, statsWindow);
  if (teamClass) {
    select.classList.add(teamClass);
  }
  select.setAttribute(
    "aria-label",
    statsWindow.kind === "player" ? "Player stats target" : "Team stats target",
  );
  if (statsWindow.kind === "player") {
    appendGroupedPlayerOptions(replayPlayer, select, statsWindow.playerId);
    select.value = statsWindow.playerId ?? "";
    select.addEventListener("change", () => {
      statsWindow.playerId = select.value || null;
      deps.renderStatsWindow(statsWindow);
      deps.scheduleConfigUrlUpdate();
    });
  } else {
    select.append(
      new Option("Blue", "blue", statsWindow.team === "blue", statsWindow.team === "blue"),
      new Option("Orange", "orange", statsWindow.team === "orange", statsWindow.team === "orange"),
    );
    select.value = statsWindow.team ?? "blue";
    select.addEventListener("change", () => {
      statsWindow.team = select.value === "orange" ? "orange" : "blue";
      deps.renderStatsWindow(statsWindow);
      deps.scheduleConfigUrlUpdate();
    });
  }

  row.append(select);
  statsWindow.body.append(row);
}
