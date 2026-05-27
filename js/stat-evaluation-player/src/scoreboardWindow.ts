import { getTeamClass } from "./statModules.ts";
import type { StatsFrame } from "./statsTimeline.ts";

function formatScoreboardInteger(value: number | null | undefined): string {
  return typeof value === "number" && Number.isFinite(value) ? `${Math.round(value)}` : "--";
}

function createScoreboardDivider(): HTMLElement {
  const divider = document.createElement("span");
  divider.className = "scoreboard-divider";
  divider.textContent = "-";
  return divider;
}

function createScoreboardGoalValue(
  goals: number | null | undefined,
  isTeamZero: boolean,
): HTMLElement {
  const score = document.createElement("strong");
  score.className = `scoreboard-goal-value ${getTeamClass(isTeamZero)}`;
  score.textContent = formatScoreboardInteger(goals);
  return score;
}

export function renderScoreboardWindow(
  body: HTMLElement,
  frame: StatsFrame | null,
  hasReplay: boolean,
): void {
  body.replaceChildren();
  if (!frame || !hasReplay) {
    const empty = document.createElement("p");
    empty.className = "scoreboard-empty";
    empty.textContent = "Load a replay to show the scoreboard.";
    body.append(empty);
    return;
  }

  const header = document.createElement("div");
  header.className = "scoreboard-scoreline";
  header.append(
    createScoreboardGoalValue(frame.team_zero?.core.goals, true),
    createScoreboardDivider(),
    createScoreboardGoalValue(frame.team_one?.core.goals, false),
  );

  body.append(header);
}
