import type { ExportedStat, PlayerStatsSnapshot } from "./statsTimeline.ts";
import {
  getExportedStatDomain,
  getExportedStatLabels,
  getExportedStatName,
  getExportedStatValue,
  getExportedStatValueType,
  getExportedStatVariant,
} from "./exportedStats.ts";

export type MovementBreakdownClass = "speed_band" | "height_band";

interface MovementRenderOptions {
  breakdownClasses?: MovementBreakdownClass[];
  exportedStats?: ExportedStat[];
}

type MovementBreakdownValueMap = Record<MovementBreakdownClass, string>;

const BREAKDOWN_CLASS_METADATA: Record<
  MovementBreakdownClass,
  {
    valueOrder: string[];
    formatValue: (value: string) => string;
  }
> = {
  speed_band: {
    valueOrder: ["slow", "boost", "supersonic"],
    formatValue: (value) => ({
      slow: "Slow",
      boost: "Boost",
      supersonic: "Supersonic",
    }[value] ?? value),
  },
  height_band: {
    valueOrder: ["ground", "low_air", "high_air"],
    formatValue: (value) => ({
      ground: "Ground",
      low_air: "Low air",
      high_air: "High air",
    }[value] ?? value),
  },
};

function normalizeBreakdownClasses(
  breakdownClasses: MovementBreakdownClass[] | undefined,
): MovementBreakdownClass[] {
  const seen = new Set<MovementBreakdownClass>();
  const result: MovementBreakdownClass[] = [];

  for (const className of breakdownClasses ?? []) {
    if (!seen.has(className)) {
      seen.add(className);
      result.push(className);
    }
  }

  return result;
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

function formatTimeShare(
  value: number | undefined,
  total: number | undefined,
  digits = 1,
): string {
  if (value === undefined || Number.isNaN(value)) {
    return "?";
  }

  if (total === undefined || Number.isNaN(total) || total <= 0) {
    return `${value.toFixed(digits)}s`;
  }

  return `${value.toFixed(digits)}s (${(value * 100 / total).toFixed(digits)}%)`;
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

function compareBreakdownValues(
  left: MovementBreakdownValueMap,
  right: MovementBreakdownValueMap,
  classes: MovementBreakdownClass[],
): number {
  for (const className of classes) {
    const { valueOrder } = BREAKDOWN_CLASS_METADATA[className];
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
  values: MovementBreakdownValueMap,
  classes: MovementBreakdownClass[],
): string {
  if (classes.length === 1) {
    const className = classes[0]!;
    return BREAKDOWN_CLASS_METADATA[className].formatValue(values[className]);
  }

  return classes
    .map((className) => BREAKDOWN_CLASS_METADATA[className].formatValue(values[className]))
    .join(" / ");
}

function renderMovementBreakdownRows(
  movement: PlayerStatsSnapshot["movement"],
  exportedStats: ExportedStat[] | undefined,
  breakdownClasses: MovementBreakdownClass[],
  trackedTime: number | undefined,
): string {
  if (breakdownClasses.length === 0) {
    return "";
  }
  if (!movement?.labeled_tracked_time?.entries?.length && (!exportedStats || exportedStats.length === 0)) {
    return "";
  }

  const groups = new Map<string, { values: MovementBreakdownValueMap; total: number }>();

  const snapshotEntries = movement?.labeled_tracked_time?.entries ?? [];
  if (snapshotEntries.length > 0) {
    for (const entry of snapshotEntries) {
      const labelMap = new Map(entry.labels.map((label) => [label.key, label.value]));
      const values = {} as MovementBreakdownValueMap;
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
  } else {
    for (const stat of exportedStats ?? []) {
      const domain = getExportedStatDomain(stat);
      const name = getExportedStatName(stat);
      const variant = getExportedStatVariant(stat);
      const valueType = getExportedStatValueType(stat);
      const value = getExportedStatValue(stat);
      if (
        domain !== "movement" ||
        name !== "tracked_time" ||
        variant !== "labeled" ||
        valueType !== "float" ||
        value === undefined
      ) {
        continue;
      }

      const labelMap = new Map(getExportedStatLabels(stat).map((label) => [label.key, label.value]));
      const values = {} as MovementBreakdownValueMap;
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
        existing.total += value;
      } else {
        groups.set(key, { values, total: value });
      }
    }
  }

  return [...groups.values()]
    .sort((left, right) => compareBreakdownValues(left.values, right.values, breakdownClasses))
    .map((entry) => renderStatRow(
      formatBreakdownLabel(entry.values, breakdownClasses),
      formatTimeShare(entry.total, trackedTime),
    ))
    .join("");
}

export function renderMovementStats(
  movement: PlayerStatsSnapshot["movement"],
  options: MovementRenderOptions = {},
): string {
  const trackedTime = movement?.tracked_time;
  const averageSpeed = movement && trackedTime && trackedTime > 0
    ? movement.speed_integral / trackedTime
    : trackedTime === 0
      ? 0
      : undefined;
  const breakdownClasses = normalizeBreakdownClasses(options.breakdownClasses);
  const breakdownRows = renderMovementBreakdownRows(
    movement,
    options.exportedStats,
    breakdownClasses,
    trackedTime,
  );

  return `
    ${renderStatRow("Tracked", formatNumber(trackedTime, 1, "s"))}
    ${renderStatRow("Distance", formatNumber(movement?.total_distance, 0, " uu"))}
    ${renderStatRow("Avg speed", formatNumber(averageSpeed, 0, " uu/s"))}
    ${breakdownRows}
  `;
}
