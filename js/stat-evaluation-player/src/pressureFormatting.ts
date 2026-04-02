import type { TeamStatsSnapshot } from "./statsTimeline.ts";

interface PressureRenderOptions {
  labelPerspective:
    | {
      kind: "shared";
    }
    | {
      kind: "team";
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
    return value === "defensive_half" ? "Blue side" : "Orange side";
  }

  return value === "defensive_half" ? "Own half" : "Opp half";
}

function renderPressureBreakdownRows(
  pressure: TeamStatsSnapshot["pressure"],
  trackedTime: number | undefined,
  labelPerspective: PressureRenderOptions["labelPerspective"],
): string {
  const totals = new Map<string, number>();

  if (pressure) {
    totals.set("defensive_half", pressure.defensive_half_time);
    totals.set("neutral", pressure.neutral_time ?? 0);
    totals.set("offensive_half", pressure.offensive_half_time);
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
  }

  const orderedHalves = ["defensive_half", "neutral", "offensive_half"];
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
  pressure: TeamStatsSnapshot["pressure"],
  options: PressureRenderOptions,
): string {
  const trackedTime = pressure?.tracked_time;
  const breakdownRows = renderPressureBreakdownRows(
    pressure,
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
