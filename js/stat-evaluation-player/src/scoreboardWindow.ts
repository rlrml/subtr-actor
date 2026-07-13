import { getTeamClass } from "./statModules.ts";
import { getStatsFrameForReplayFrame, type StatsFrameLookup } from "./statsTimeline.ts";
import type { StatsReplayPlayer } from "./statsReplayPlayer.ts";

export function normalizeThreatProbability(value: number | null | undefined): number | null {
  if (typeof value !== "number" || !Number.isFinite(value)) {
    return null;
  }
  return Math.max(0, Math.min(1, value));
}

export function formatThreatProbability(value: number | null | undefined): string {
  const normalized = normalizeThreatProbability(value);
  return normalized === null ? "--" : `${(normalized * 100).toFixed(1)}%`;
}

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
    if (replay.players.length === 4) {
      body.append(
        createThreatReadout(
          frame.team_zero?.expected_goals.current_threat,
          frame.team_one?.expected_goals.current_threat,
          frame.team_zero?.expected_goals.incident_xg,
          frame.team_one?.expected_goals.incident_xg,
        ),
      );
    }
  }
}

function createThreatReadout(
  teamZeroThreat: number | null | undefined,
  teamOneThreat: number | null | undefined,
  teamZeroIncidentXg: number | null | undefined,
  teamOneIncidentXg: number | null | undefined,
): HTMLElement {
  const readout = document.createElement("section");
  readout.className = "scoreboard-threat";
  readout.title =
    "Current values are each team's probability of scoring within five seconds. " +
    "Incident xG counts one calibrated peak per dangerous incident and excludes the " +
    "goal-result window beginning shortly before the scoring team's final touch.";

  const values = document.createElement("div");
  values.className = "scoreboard-threat-values";
  values.append(
    createThreatValue(teamZeroThreat, true),
    createThreatLabel(),
    createThreatValue(teamOneThreat, false),
  );

  const meter = document.createElement("div");
  meter.className = "scoreboard-threat-meter";
  meter.setAttribute("aria-hidden", "true");
  meter.append(
    createThreatMeterHalf(teamZeroThreat, true),
    createThreatMeterHalf(teamOneThreat, false),
  );

  const accumulated = document.createElement("div");
  accumulated.className = "scoreboard-threat-accumulated";
  accumulated.append(
    createIncidentXgValue(teamZeroIncidentXg, true),
    createIncidentXgLabel(),
    createIncidentXgValue(teamOneIncidentXg, false),
  );

  readout.append(values, meter, accumulated);
  return readout;
}

function createThreatValue(value: number | null | undefined, isTeamZero: boolean): HTMLElement {
  const output = document.createElement("strong");
  output.className = `scoreboard-threat-value ${getTeamClass(isTeamZero)}`;
  output.textContent = formatThreatProbability(value);
  output.setAttribute(
    "aria-label",
    `${isTeamZero ? "Blue" : "Orange"} goal probability ${output.textContent}`,
  );
  return output;
}

function createThreatLabel(): HTMLElement {
  const label = document.createElement("span");
  label.className = "scoreboard-threat-label";
  label.textContent = "Goal in 5s";
  return label;
}

function createThreatMeterHalf(value: number | null | undefined, isTeamZero: boolean): HTMLElement {
  const half = document.createElement("span");
  half.className = `scoreboard-threat-meter-half ${getTeamClass(isTeamZero)}`;
  const fill = document.createElement("span");
  fill.className = "scoreboard-threat-meter-fill";
  fill.style.setProperty("--threat-value", `${normalizeThreatProbability(value) ?? 0}`);
  half.append(fill);
  return half;
}

function createIncidentXgLabel(): HTMLElement {
  const label = document.createElement("span");
  label.className = "scoreboard-threat-accumulated-label";
  label.textContent = "Incident xG";
  return label;
}

function createIncidentXgValue(value: number | null | undefined, isTeamZero: boolean): HTMLElement {
  const output = document.createElement("span");
  output.className = `scoreboard-threat-accumulated-value ${getTeamClass(isTeamZero)}`;
  output.textContent = formatIncidentXg(value);
  return output;
}

export function formatIncidentXg(value: number | null | undefined): string {
  return typeof value === "number" && Number.isFinite(value) ? value.toFixed(2) : "--";
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
