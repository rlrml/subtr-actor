import type { StatsFrame } from "./statsTimeline.ts";

function formatInteger(value: number | undefined): string {
  if (value === undefined || Number.isNaN(value)) {
    return "?";
  }

  return `${Math.round(value)}`;
}

function renderStatRow(label: string, value: string): string {
  return `<div class="stat-row"><span class="label">${label}</span><span class="value">${value}</span></div>`;
}

function teamRushCount(
  rush: StatsFrame["rush"],
  isTeamZero: boolean,
): number | undefined {
  return isTeamZero ? rush?.team_zero_count : rush?.team_one_count;
}

function matchupCount(
  rush: StatsFrame["rush"],
  isTeamZero: boolean,
  attackers: 2 | 3,
  defenders: 1 | 2 | 3,
): number | undefined {
  const prefix = isTeamZero ? "team_zero" : "team_one";
  const attackerLabel = attackers === 2 ? "two" : "three";
  const defenderLabel = defenders === 1 ? "one" : defenders === 2 ? "two" : "three";
  const key = `${prefix}_${attackerLabel}_v_${defenderLabel}_count`;
  return rush?.[key as keyof NonNullable<StatsFrame["rush"]>] as number | undefined;
}

export function renderRushStats(
  rush: StatsFrame["rush"],
  isTeamZero: boolean,
): string {
  return `
    ${renderStatRow("Rushes", formatInteger(teamRushCount(rush, isTeamZero)))}
    ${renderStatRow("2v1", formatInteger(matchupCount(rush, isTeamZero, 2, 1)))}
    ${renderStatRow("2v2", formatInteger(matchupCount(rush, isTeamZero, 2, 2)))}
    ${renderStatRow("2v3", formatInteger(matchupCount(rush, isTeamZero, 2, 3)))}
    ${renderStatRow("3v1", formatInteger(matchupCount(rush, isTeamZero, 3, 1)))}
    ${renderStatRow("3v2", formatInteger(matchupCount(rush, isTeamZero, 3, 2)))}
    ${renderStatRow("3v3", formatInteger(matchupCount(rush, isTeamZero, 3, 3)))}
  `;
}
