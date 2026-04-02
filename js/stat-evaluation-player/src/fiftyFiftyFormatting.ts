import type {
  PlayerStatsSnapshot,
  TeamStatsSnapshot,
} from "./statsTimeline.ts";

interface FiftyFiftySummaryOptions {
  kind: "shared";
}

type FiftyFiftySummaryPerspective =
  | FiftyFiftySummaryOptions
  | { kind: "team" };

function formatInteger(value: number | undefined): string {
  if (value === undefined || Number.isNaN(value)) {
    return "?";
  }

  return `${Math.round(value)}`;
}

function formatPercentage(
  numerator: number | undefined,
  denominator: number | undefined,
  digits = 1,
): string {
  if (
    numerator === undefined ||
    denominator === undefined ||
    Number.isNaN(numerator) ||
    Number.isNaN(denominator) ||
    denominator <= 0
  ) {
    return "?";
  }

  return `${((numerator * 100) / denominator).toFixed(digits)}%`;
}

function renderStatRow(label: string, value: string): string {
  return `<div class="stat-row"><span class="label">${label}</span><span class="value">${value}</span></div>`;
}

export function renderFiftyFiftySummary(
  stats: TeamStatsSnapshot["fifty_fifty"],
  perspective: FiftyFiftySummaryPerspective,
): string {
  if (perspective.kind === "shared") {
    return `
      ${renderStatRow("50s", formatInteger(stats?.count))}
      ${renderStatRow("Blue wins", `${formatInteger(stats?.wins)} (${formatPercentage(stats?.wins, stats?.count)})`)}
      ${renderStatRow("Orange wins", `${formatInteger(stats?.losses)} (${formatPercentage(stats?.losses, stats?.count)})`)}
      ${renderStatRow("Neutral", formatInteger(stats?.neutral_outcomes))}
      ${renderStatRow("Blue poss after", formatInteger(stats?.possession_after_count))}
      ${renderStatRow("Orange poss after", formatInteger(stats?.opponent_possession_after_count))}
      ${renderStatRow("Kickoff 50s", formatInteger(stats?.kickoff_count))}
      ${renderStatRow("Blue kickoff wins", formatInteger(stats?.kickoff_wins))}
      ${renderStatRow("Orange kickoff wins", formatInteger(stats?.kickoff_losses))}
      ${renderStatRow("Blue kickoff poss", formatInteger(stats?.kickoff_possession_after_count))}
      ${renderStatRow("Orange kickoff poss", formatInteger(stats?.kickoff_opponent_possession_after_count))}
    `;
  }

  return `
    ${renderStatRow("50s", formatInteger(stats?.count))}
    ${renderStatRow("Wins", `${formatInteger(stats?.wins)} (${formatPercentage(stats?.wins, stats?.count)})`)}
    ${renderStatRow("Losses", formatInteger(stats?.losses))}
    ${renderStatRow("Neutral", formatInteger(stats?.neutral_outcomes))}
    ${renderStatRow("Poss after", formatInteger(stats?.possession_after_count))}
    ${renderStatRow("Kickoff 50s", formatInteger(stats?.kickoff_count))}
    ${renderStatRow("Kickoff wins", formatInteger(stats?.kickoff_wins))}
    ${renderStatRow("Kickoff poss", formatInteger(stats?.kickoff_possession_after_count))}
  `;
}

export function renderPlayerFiftyFiftyStats(
  stats: PlayerStatsSnapshot["fifty_fifty"],
): string {
  return `
    <div class="stat-row"><span class="label">50s</span><span class="value">${formatInteger(stats?.count)}</span></div>
    <div class="stat-row"><span class="label">Wins</span><span class="value">${formatInteger(stats?.wins)} (${formatPercentage(stats?.wins, stats?.count)})</span></div>
    <div class="stat-row"><span class="label">Losses</span><span class="value">${formatInteger(stats?.losses)}</span></div>
    <div class="stat-row"><span class="label">Neutral</span><span class="value">${formatInteger(stats?.neutral_outcomes)}</span></div>
    <div class="stat-row"><span class="label">Poss after</span><span class="value">${formatInteger(stats?.possession_after_count)}</span></div>
    <div class="stat-row"><span class="label">Kickoff 50s</span><span class="value">${formatInteger(stats?.kickoff_count)}</span></div>
    <div class="stat-row"><span class="label">Kickoff wins</span><span class="value">${formatInteger(stats?.kickoff_wins)}</span></div>
    <div class="stat-row"><span class="label">Kickoff poss</span><span class="value">${formatInteger(stats?.kickoff_possession_after_count)}</span></div>
  `;
}
