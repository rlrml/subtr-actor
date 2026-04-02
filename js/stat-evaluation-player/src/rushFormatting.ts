import type { TeamStatsSnapshot } from "./statsTimeline.ts";

function formatInteger(value: number | undefined): string {
  if (value === undefined || Number.isNaN(value)) {
    return "?";
  }

  return `${Math.round(value)}`;
}

function renderStatRow(label: string, value: string): string {
  return `<div class="stat-row"><span class="label">${label}</span><span class="value">${value}</span></div>`;
}

export function renderRushStats(
  rush: TeamStatsSnapshot["rush"],
): string {
  return `
    ${renderStatRow("Rushes", formatInteger(rush?.count))}
    ${renderStatRow("2v1", formatInteger(rush?.two_v_one_count))}
    ${renderStatRow("2v2", formatInteger(rush?.two_v_two_count))}
    ${renderStatRow("2v3", formatInteger(rush?.two_v_three_count))}
    ${renderStatRow("3v1", formatInteger(rush?.three_v_one_count))}
    ${renderStatRow("3v2", formatInteger(rush?.three_v_two_count))}
    ${renderStatRow("3v3", formatInteger(rush?.three_v_three_count))}
  `;
}
