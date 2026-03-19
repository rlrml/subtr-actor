import type { ExportedStat, StatsFrame } from "./statsTimeline.ts";
import {
  getExportedStatDomain,
  getExportedStatLabels,
  getExportedStatName,
  getExportedStatValue,
  getExportedStatValueType,
  getExportedStatVariant,
} from "./exportedStats.ts";

interface PressureRenderOptions {
  exportedStats?: ExportedStat[];
  labelPerspective:
    | {
      kind: "shared";
    }
    | {
      kind: "team";
      isTeamZero: boolean;
    };
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

function formatFieldHalfLabel(
  value: string,
  labelPerspective: PressureRenderOptions["labelPerspective"],
): string {
  if (value === "neutral") {
    return "Neutral zone";
  }

  if (labelPerspective.kind === "shared") {
    return value === "team_zero_side" ? "Blue side" : "Orange side";
  }

  const isOwnHalf = (value === "team_zero_side") === labelPerspective.isTeamZero;
  return isOwnHalf ? "Own half" : "Opp half";
}

function renderPressureBreakdownRows(
  pressure: StatsFrame["pressure"],
  exportedStats: ExportedStat[] | undefined,
  trackedTime: number | undefined,
  labelPerspective: PressureRenderOptions["labelPerspective"],
): string {
  const totals = new Map<string, number>();

  if (pressure) {
    totals.set("team_zero_side", pressure.team_zero_side_time);
    totals.set("neutral", pressure.neutral_time ?? 0);
    totals.set("team_one_side", pressure.team_one_side_time);
  }

  if (pressure?.labeled_time?.entries?.length) {
    totals.clear();
    for (const entry of pressure.labeled_time.entries) {
      const half = entry.labels.find((label) => label.key === "field_half")?.value;
      if (!half) {
        continue;
      }

      totals.set(half, (totals.get(half) ?? 0) + entry.value);
    }
  } else if (totals.size === 0) {
    for (const stat of exportedStats ?? []) {
      const domain = getExportedStatDomain(stat);
      const name = getExportedStatName(stat);
      const variant = getExportedStatVariant(stat);
      const valueType = getExportedStatValueType(stat);
      const value = getExportedStatValue(stat);
      if (
        domain !== "pressure" ||
        name !== "time" ||
        variant !== "labeled" ||
        valueType !== "float" ||
        value === undefined
      ) {
        continue;
      }

      const half = getExportedStatLabels(stat).find((label) => label.key === "field_half")?.value;
      if (!half) {
        continue;
      }

      totals.set(half, (totals.get(half) ?? 0) + value);
    }
  }

  const orderedHalves = ["team_zero_side", "neutral", "team_one_side"];
  if (!orderedHalves.some((half) => (totals.get(half) ?? 0) > 0)) {
    return "";
  }

  return orderedHalves
    .filter((half) => totals.has(half))
    .map((half) => renderStatRow(
      formatFieldHalfLabel(half, labelPerspective),
      formatTimeShare(totals.get(half), trackedTime),
    ))
    .join("");
}

export function renderPressureStats(
  pressure: StatsFrame["pressure"],
  options: PressureRenderOptions,
): string {
  const trackedTimeStat = options.exportedStats?.find((stat) =>
    getExportedStatDomain(stat) === "pressure"
    && getExportedStatName(stat) === "time"
    && getExportedStatVariant(stat) !== "labeled"
    && getExportedStatValueType(stat) === "float"
    && getExportedStatValue(stat) !== undefined
  );
  const trackedTime = pressure?.tracked_time
    ?? (trackedTimeStat ? getExportedStatValue(trackedTimeStat) : undefined);
  const breakdownRows = renderPressureBreakdownRows(
    pressure,
    options.exportedStats,
    trackedTime,
    options.labelPerspective,
  );
  const trackedRow = breakdownRows.length === 0
    ? renderStatRow("Tracked", formatNumber(trackedTime, 1, "s"))
    : "";

  return `
    ${trackedRow}
    ${breakdownRows}
  `;
}
