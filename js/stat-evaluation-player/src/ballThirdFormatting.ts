import type { TeamStatsSnapshot } from "./statsTimeline.ts";

interface BallThirdRenderOptions {
  labelPerspective:
    | {
        kind: "shared";
      }
    | {
        kind: "team";
      };
}

function formatNumber(value: number | undefined, digits = 1, suffix = ""): string {
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

function formatTimeShare(value: number | undefined, total: number | undefined, digits = 1): string {
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

function formatFieldThirdLabel(
  value: string,
  labelPerspective: BallThirdRenderOptions["labelPerspective"],
): string {
  if (value === "neutral_third") {
    return "Neutral third";
  }

  if (labelPerspective.kind === "shared") {
    return value === "defensive_third" ? "Blue third" : "Orange third";
  }

  return value === "defensive_third" ? "Own third" : "Opp third";
}

function renderBallThirdBreakdownRows(
  ballThird: TeamStatsSnapshot["ball_third"],
  trackedTime: number | undefined,
  labelPerspective: BallThirdRenderOptions["labelPerspective"],
): string {
  const totals = new Map<string, number>();

  if (ballThird) {
    totals.set("defensive_third", ballThird.defensive_third_time);
    totals.set("neutral_third", ballThird.neutral_third_time ?? 0);
    totals.set("offensive_third", ballThird.offensive_third_time);
  }

  if (ballThird?.labeled_time?.entries?.length) {
    totals.clear();
    for (const entry of ballThird.labeled_time.entries) {
      const third = entry.labels.find((label) => label.key === "field_third")?.value;
      if (!third) {
        continue;
      }

      totals.set(third, (totals.get(third) ?? 0) + entry.value);
    }
  }

  const orderedThirds = ["defensive_third", "neutral_third", "offensive_third"];
  if (!orderedThirds.some((third) => (totals.get(third) ?? 0) > 0)) {
    return "";
  }

  return orderedThirds
    .filter((third) => totals.has(third))
    .map((third) =>
      renderStatRow(
        formatFieldThirdLabel(third, labelPerspective),
        formatTimeShare(totals.get(third), trackedTime),
      ),
    )
    .join("");
}

export function renderBallThirdStats(
  ballThird: TeamStatsSnapshot["ball_third"],
  options: BallThirdRenderOptions,
): string {
  const trackedTime = ballThird?.tracked_time;
  const breakdownRows = renderBallThirdBreakdownRows(
    ballThird,
    trackedTime,
    options.labelPerspective,
  );
  const trackedRow =
    breakdownRows.length === 0 ? renderStatRow("Tracked", formatNumber(trackedTime, 1, "s")) : "";

  return `
    ${trackedRow}
    ${breakdownRows}
  `;
}
