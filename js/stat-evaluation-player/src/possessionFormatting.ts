import type { TeamStatsSnapshot } from "./statsTimeline.ts";

export type PossessionBreakdownClass = "possession_state" | "field_third";

interface PossessionRenderOptions {
  breakdownClasses?: PossessionBreakdownClass[];
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

function formatPossessionStateLabel(
  value: string,
  labelPerspective: PossessionRenderOptions["labelPerspective"],
): string {
  if (value === "neutral") {
    return "Neutral";
  }

  if (labelPerspective.kind === "shared") {
    return value === "own" ? "Blue control" : "Orange control";
  }

  return value === "own" ? "Team control" : "Opp control";
}

function getOrderedPossessionStates(
  labelPerspective: PossessionRenderOptions["labelPerspective"],
): string[] {
  if (labelPerspective.kind === "shared") {
    return ["own", "neutral", "opponent"];
  }

  return ["own", "neutral", "opponent"];
}

function formatFieldThirdLabel(
  value: string,
  labelPerspective: PossessionRenderOptions["labelPerspective"],
): string {
  if (value === "neutral_third") {
    return "Neutral third";
  }

  if (labelPerspective.kind === "shared") {
    return value === "defensive_third" ? "Blue third" : "Orange third";
  }

  return value === "defensive_third" ? "Own third" : "Opp third";
}

function getOrderedFieldThirds(
  labelPerspective: PossessionRenderOptions["labelPerspective"],
): string[] {
  if (labelPerspective.kind === "shared") {
    return ["defensive_third", "neutral_third", "offensive_third"];
  }

  return ["defensive_third", "neutral_third", "offensive_third"];
}

type PossessionBreakdownValueMap = Record<PossessionBreakdownClass, string>;

function compareBreakdownValues(
  left: PossessionBreakdownValueMap,
  right: PossessionBreakdownValueMap,
  classes: PossessionBreakdownClass[],
  labelPerspective: PossessionRenderOptions["labelPerspective"],
): number {
  for (const className of classes) {
    const valueOrder = className === "possession_state"
      ? getOrderedPossessionStates(labelPerspective)
      : getOrderedFieldThirds(labelPerspective);
    const leftIndex = valueOrder.indexOf(left[className]);
    const rightIndex = valueOrder.indexOf(right[className]);
    const normalizedLeftIndex = leftIndex === -1 ? Number.MAX_SAFE_INTEGER : leftIndex;
    const normalizedRightIndex = rightIndex === -1 ? Number.MAX_SAFE_INTEGER : rightIndex;
    if (normalizedLeftIndex !== normalizedRightIndex) {
      return normalizedLeftIndex - normalizedRightIndex;
    }
  }

  return 0;
}

function formatBreakdownLabel(
  values: PossessionBreakdownValueMap,
  classes: PossessionBreakdownClass[],
  labelPerspective: PossessionRenderOptions["labelPerspective"],
): string {
  const formatValue = (className: PossessionBreakdownClass, value: string): string =>
    className === "possession_state"
      ? formatPossessionStateLabel(value, labelPerspective)
      : formatFieldThirdLabel(value, labelPerspective);

  if (classes.length === 1) {
    const className = classes[0]!;
    return formatValue(className, values[className]);
  }

  return classes
    .map((className) => formatValue(className, values[className]))
    .join(" / ");
}

function renderPossessionBreakdownRows(
  possession: TeamStatsSnapshot["possession"],
  breakdownClasses: PossessionBreakdownClass[],
  trackedTime: number | undefined,
  labelPerspective: PossessionRenderOptions["labelPerspective"],
): string {
  if (breakdownClasses.length === 0) {
    return "";
  }

  const groups = new Map<string, { values: PossessionBreakdownValueMap; total: number }>();

  if (possession?.labeled_time?.entries?.length) {
    for (const entry of possession.labeled_time.entries) {
      const labelMap = new Map(entry.labels.map((label) => [label.key, label.value]));
      const values = {} as PossessionBreakdownValueMap;
      let complete = true;
      for (const className of breakdownClasses) {
        const value = labelMap.get(className);
        if (value === undefined) {
          complete = false;
          break;
        }
        values[className] = value;
      }
      if (!complete) {
        continue;
      }

      const key = breakdownClasses.map((className) => `${className}:${values[className]}`).join("|");
      const existing = groups.get(key);
      if (existing) {
        existing.total += entry.value;
      } else {
        groups.set(key, { values, total: entry.value });
      }
    }
  }

  if (groups.size === 0 && breakdownClasses.length === 1 && breakdownClasses[0] === "possession_state") {
    const totals = new Map<string, number>();
    if (possession) {
      totals.set("own", possession.possession_time);
      totals.set("neutral", possession.neutral_time ?? 0);
      totals.set("opponent", possession.opponent_possession_time);
    }

    if (!getOrderedPossessionStates(labelPerspective).some((state) => (totals.get(state) ?? 0) > 0)) {
      return "";
    }

    return getOrderedPossessionStates(labelPerspective)
      .filter((state) => totals.has(state))
      .map((state) => renderStatRow(
        formatPossessionStateLabel(state, labelPerspective),
        formatTimeShare(totals.get(state), trackedTime),
      ))
      .join("");
  }

  return [...groups.values()]
    .sort((left, right) => compareBreakdownValues(
      left.values,
      right.values,
      breakdownClasses,
      labelPerspective,
    ))
    .map((entry) => renderStatRow(
      formatBreakdownLabel(entry.values, breakdownClasses, labelPerspective),
      formatTimeShare(entry.total, trackedTime),
    ))
    .join("");
}

export function renderPossessionStats(
  possession: TeamStatsSnapshot["possession"],
  options: PossessionRenderOptions,
): string {
  const trackedTime = possession?.tracked_time;
  const breakdownClasses = normalizeBreakdownClasses(options.breakdownClasses);
  const breakdownRows = renderPossessionBreakdownRows(
    possession,
    breakdownClasses,
    trackedTime,
    options.labelPerspective,
  );

  return `
    ${renderStatRow("Tracked", formatNumber(trackedTime, 1, "s"))}
    ${breakdownRows}
  `;
}
