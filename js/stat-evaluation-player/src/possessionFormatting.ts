import type { ExportedStat, StatsFrame } from "./statsTimeline.ts";

export type PossessionBreakdownClass = "possession_state";

interface PossessionRenderOptions {
  breakdownClasses?: PossessionBreakdownClass[];
  exportedStats?: ExportedStat[];
  isTeamZero: boolean;
}

function formatNumber(
  value: number | undefined,
  digits = 1,
  suffix = "",
): string {
  if (value === undefined || Number.isNaN(value)) {
    return "?";
  }

  return `${value.toFixed(digits)}${suffix}`;
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

function formatTimeShare(
  value: number | undefined,
  total: number | undefined,
  digits = 1,
): string {
  if (value === undefined || Number.isNaN(value)) {
    return "?";
  }

  const percentage = formatPercentage(value, total, digits);
  if (percentage === "?") {
    return `${value.toFixed(digits)}s`;
  }

  return `${value.toFixed(digits)}s (${percentage})`;
}

function escapeHtml(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}

function renderStatRow(label: string, value: string): string {
  return `<div class="stat-row"><span class="label">${escapeHtml(label)}</span><span class="value">${escapeHtml(value)}</span></div>`;
}

function normalizeBreakdownClasses(
  breakdownClasses: PossessionBreakdownClass[] | undefined,
): PossessionBreakdownClass[] {
  const seen = new Set<PossessionBreakdownClass>();
  const result: PossessionBreakdownClass[] = [];

  for (const className of breakdownClasses ?? []) {
    if (!seen.has(className)) {
      seen.add(className);
      result.push(className);
    }
  }

  return result;
}

function formatPossessionStateLabel(value: string, isTeamZero: boolean): string {
  if (value === "neutral") {
    return "Neutral";
  }

  const isOwnTeam = (value === "team_zero") === isTeamZero;
  return isOwnTeam ? "Team control" : "Opp control";
}

function renderPossessionBreakdownRows(
  exportedStats: ExportedStat[] | undefined,
  breakdownClasses: PossessionBreakdownClass[],
  trackedTime: number | undefined,
  isTeamZero: boolean,
): string {
  if (
    breakdownClasses.length === 0 ||
    !breakdownClasses.includes("possession_state") ||
    !exportedStats ||
    exportedStats.length === 0
  ) {
    return "";
  }

  const totals = new Map<string, number>();
  for (const stat of exportedStats) {
    if (
      stat.domain !== "possession" ||
      stat.name !== "time" ||
      stat.variant !== "labeled" ||
      stat.value_type !== "float" ||
      !Number.isFinite(stat.value)
    ) {
      continue;
    }

    const state = stat.labels?.find((label) => label.key === "possession_state")?.value;
    if (!state) {
      continue;
    }

    totals.set(state, (totals.get(state) ?? 0) + stat.value);
  }

  return ["team_zero", "team_one", "neutral"]
    .filter((state) => totals.has(state))
    .map((state) => renderStatRow(
      formatPossessionStateLabel(state, isTeamZero),
      formatTimeShare(totals.get(state), trackedTime),
    ))
    .join("");
}

export function renderPossessionStats(
  possession: StatsFrame["possession"],
  options: PossessionRenderOptions,
): string {
  const trackedTime = possession?.tracked_time;
  const breakdownClasses = normalizeBreakdownClasses(options.breakdownClasses);
  const breakdownRows = renderPossessionBreakdownRows(
    options.exportedStats,
    breakdownClasses,
    trackedTime,
    options.isTeamZero,
  );

  return `
    ${renderStatRow("Tracked", formatNumber(trackedTime, 1, "s"))}
    ${breakdownRows}
  `;
}
