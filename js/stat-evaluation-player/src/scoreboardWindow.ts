import { getTeamClass } from "./statModules.ts";
import { getStatsFrameForReplayFrame, type StatsFrameLookup } from "./statsTimeline.ts";
import type { StatsReplayPlayer } from "./statsReplayPlayer.ts";

export interface ScoreboardWindowOptions {
  readonly body: HTMLDivElement;
  getReplayPlayer(): StatsReplayPlayer | null;
  getStatsFrameLookup(): StatsFrameLookup | null;
}

export class ScoreboardWindowController {
  constructor(private readonly options: ScoreboardWindowOptions) {}

  render(frameIndex = this.options.getReplayPlayer()?.getState().frameIndex ?? 0): void {
    const { body } = this.options;
    body.replaceChildren();
    const statsFrameLookup = this.options.getStatsFrameLookup();
    const frame = statsFrameLookup
      ? getStatsFrameForReplayFrame(statsFrameLookup, frameIndex)
      : null;
    const replay = this.options.getReplayPlayer()?.replay ?? null;
    if (!frame || !replay) {
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
}

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

export function createScoreboardWindowController(
  options: ScoreboardWindowOptions,
): ScoreboardWindowController {
  return new ScoreboardWindowController(options);
}
