import type { ExportedStat, StatsFrame } from "./statsTimeline.ts";
import {
  getExportedStatDomain,
  getExportedStatLabels,
  getExportedStatName,
  getExportedStatValue,
  getExportedStatValueType,
  getExportedStatVariant,
} from "./exportedStats.ts";

export type PossessionBreakdownClass = "possession_state";

interface PossessionRenderOptions {
  breakdownClasses?: PossessionBreakdownClass[];
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
    return value === "team_zero" ? "Blue control" : "Orange control";
  }

  const isOwnTeam = (value === "team_zero") === labelPerspective.isTeamZero;
  return isOwnTeam ? "Team control" : "Opp control";
}

function getOrderedPossessionStates(
  labelPerspective: PossessionRenderOptions["labelPerspective"],
): string[] {
  if (labelPerspective.kind === "shared") {
    return ["team_zero", "neutral", "team_one"];
  }

  return labelPerspective.isTeamZero
    ? ["team_zero", "neutral", "team_one"]
    : ["team_one", "neutral", "team_zero"];
}

function renderPossessionBreakdownRows(
  possession: StatsFrame["possession"],
  exportedStats: ExportedStat[] | undefined,
  breakdownClasses: PossessionBreakdownClass[],
  trackedTime: number | undefined,
  labelPerspective: PossessionRenderOptions["labelPerspective"],
): string {
  if (breakdownClasses.length === 0 || !breakdownClasses.includes("possession_state")) {
    return "";
  }

  const totals = new Map<string, number>();
  if (possession) {
    totals.set("team_zero", possession.team_zero_time);
    totals.set("neutral", possession.neutral_time ?? 0);
    totals.set("team_one", possession.team_one_time);
  }

  if (possession?.labeled_time?.entries?.length) {
    totals.clear();
    for (const entry of possession.labeled_time.entries) {
      const state = entry.labels.find((label) => label.key === "possession_state")?.value;
      if (!state) {
        continue;
      }

      totals.set(state, (totals.get(state) ?? 0) + entry.value);
    }
  } else if (exportedStats?.length) {
    totals.clear();
    for (const stat of exportedStats ?? []) {
      const domain = getExportedStatDomain(stat);
      const name = getExportedStatName(stat);
      const variant = getExportedStatVariant(stat);
      const valueType = getExportedStatValueType(stat);
      const value = getExportedStatValue(stat);
      if (
        domain !== "possession" ||
        name !== "time" ||
        variant !== "labeled" ||
        valueType !== "float" ||
        value === undefined
      ) {
        continue;
      }

      const state = getExportedStatLabels(stat).find((label) => label.key === "possession_state")?.value;
      if (!state) {
        continue;
      }

      totals.set(state, (totals.get(state) ?? 0) + value);
    }
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

export function renderPossessionStats(
  possession: StatsFrame["possession"],
  options: PossessionRenderOptions,
): string {
  const trackedTime = possession?.tracked_time;
  const breakdownClasses = normalizeBreakdownClasses(options.breakdownClasses);
  const breakdownRows = renderPossessionBreakdownRows(
    possession,
    options.exportedStats,
    breakdownClasses,
    trackedTime,
    options.labelPerspective,
  );

  return `
    ${renderStatRow("Tracked", formatNumber(trackedTime, 1, "s"))}
    ${breakdownRows}
  `;
}
