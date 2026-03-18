import type { ExportedStat, StatsFrame } from "./statsTimeline.ts";

export type PressureBreakdownClass = "field_half";

interface PressureRenderOptions {
  breakdownClasses?: PressureBreakdownClass[];
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
  breakdownClasses: PressureBreakdownClass[] | undefined,
): PressureBreakdownClass[] {
  const seen = new Set<PressureBreakdownClass>();
  const result: PressureBreakdownClass[] = [];

  for (const className of breakdownClasses ?? []) {
    if (!seen.has(className)) {
      seen.add(className);
      result.push(className);
    }
  }

  return result;
}

function formatFieldHalfLabel(value: string, isTeamZero: boolean): string {
  const isOwnHalf = (value === "team_zero_side") === isTeamZero;
  return isOwnHalf ? "Own half" : "Opp half";
}

function renderPressureBreakdownRows(
  exportedStats: ExportedStat[] | undefined,
  breakdownClasses: PressureBreakdownClass[],
  trackedTime: number | undefined,
  isTeamZero: boolean,
): string {
  if (
    breakdownClasses.length === 0 ||
    !breakdownClasses.includes("field_half") ||
    !exportedStats ||
    exportedStats.length === 0
  ) {
    return "";
  }

  const totals = new Map<string, number>();
  for (const stat of exportedStats) {
    if (
      stat.domain !== "pressure" ||
      stat.name !== "time" ||
      stat.variant !== "labeled" ||
      stat.value_type !== "float" ||
      !Number.isFinite(stat.value)
    ) {
      continue;
    }

    const half = stat.labels?.find((label) => label.key === "field_half")?.value;
    if (!half) {
      continue;
    }

    totals.set(half, (totals.get(half) ?? 0) + stat.value);
  }

  return ["team_zero_side", "team_one_side"]
    .filter((half) => totals.has(half))
    .map((half) => renderStatRow(
      formatFieldHalfLabel(half, isTeamZero),
      formatTimeShare(totals.get(half), trackedTime),
    ))
    .join("");
}

export function renderPressureStats(
  pressure: StatsFrame["pressure"],
  options: PressureRenderOptions,
): string {
  const trackedTime = pressure?.tracked_time;
  const breakdownClasses = normalizeBreakdownClasses(options.breakdownClasses);
  const breakdownRows = renderPressureBreakdownRows(
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
