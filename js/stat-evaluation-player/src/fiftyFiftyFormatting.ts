import type {
  PlayerStatsSnapshot,
  StatsFrame,
} from "./statsTimeline.ts";

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

export function renderFiftyFiftySummary(
  stats: StatsFrame["fifty_fifty"],
  isTeamZero: boolean,
): string {
  const wins = isTeamZero ? stats?.team_zero_wins : stats?.team_one_wins;
  const losses = isTeamZero ? stats?.team_one_wins : stats?.team_zero_wins;
  const possession = isTeamZero
    ? stats?.team_zero_possession_after_count
    : stats?.team_one_possession_after_count;
  const kickoffWins = isTeamZero
    ? stats?.kickoff_team_zero_wins
    : stats?.kickoff_team_one_wins;
  const kickoffPossession = isTeamZero
    ? stats?.kickoff_team_zero_possession_after_count
    : stats?.kickoff_team_one_possession_after_count;

  return `
    <div class="stat-row"><span class="label">50s</span><span class="value">${formatInteger(stats?.count)}</span></div>
    <div class="stat-row"><span class="label">Wins</span><span class="value">${formatInteger(wins)} (${formatPercentage(wins, stats?.count)})</span></div>
    <div class="stat-row"><span class="label">Losses</span><span class="value">${formatInteger(losses)}</span></div>
    <div class="stat-row"><span class="label">Neutral</span><span class="value">${formatInteger(stats?.neutral_outcomes)}</span></div>
    <div class="stat-row"><span class="label">Poss after</span><span class="value">${formatInteger(possession)}</span></div>
    <div class="stat-row"><span class="label">Kickoff 50s</span><span class="value">${formatInteger(stats?.kickoff_count)}</span></div>
    <div class="stat-row"><span class="label">Kickoff wins</span><span class="value">${formatInteger(kickoffWins)}</span></div>
    <div class="stat-row"><span class="label">Kickoff poss</span><span class="value">${formatInteger(kickoffPossession)}</span></div>
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
