import type { PlayerStatsSnapshot, StatLabel } from "./statsTimeline.ts";

export type TouchBreakdownClass = "kind" | "height_band";

interface TouchRenderOptions {
  breakdownClasses?: TouchBreakdownClass[];
}

type TouchBreakdownValueMap = Record<TouchBreakdownClass, string>;

interface TouchBreakdownEntry {
  labels: StatLabel[];
  count: number;
}

const BREAKDOWN_CLASS_METADATA: Record<
  TouchBreakdownClass,
  {
    label: string;
    valueOrder: string[];
    formatValue: (value: string) => string;
  }
> = {
  kind: {
    label: "Kind",
    valueOrder: ["dribble", "control", "medium_hit", "hard_hit"],
    formatValue: (value) => ({
      dribble: "Dribble",
      control: "Control",
      medium_hit: "Medium",
      hard_hit: "Hard",
    }[value] ?? value),
  },
  height_band: {
    label: "Height",
    valueOrder: ["ground", "low_air", "high_air"],
    formatValue: (value) => ({
      ground: "Ground",
      low_air: "Low air",
      high_air: "High air",
    }[value] ?? value),
  },
};

function normalizeBreakdownClasses(
  breakdownClasses: TouchBreakdownClass[] | undefined,
): TouchBreakdownClass[] {
  const seen = new Set<TouchBreakdownClass>();
  const result: TouchBreakdownClass[] = [];

  for (const className of breakdownClasses ?? []) {
    if (!seen.has(className)) {
      seen.add(className);
      result.push(className);
    }
  }

  return result;
}

function formatInteger(value: number | undefined): string {
  if (value === undefined || Number.isNaN(value)) {
    return "?";
  }

  return `${Math.round(value)}`;
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
  left: TouchBreakdownValueMap,
  right: TouchBreakdownValueMap,
  classes: TouchBreakdownClass[],
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
  values: TouchBreakdownValueMap,
  classes: TouchBreakdownClass[],
): string {
  if (classes.length === 1) {
    const className = classes[0]!;
    return BREAKDOWN_CLASS_METADATA[className].formatValue(values[className]);
  }

  return classes
    .map((className) => BREAKDOWN_CLASS_METADATA[className].formatValue(values[className]))
    .join(" / ");
}

function labeledEntriesFromTouchSnapshot(
  touch: PlayerStatsSnapshot["touch"],
): TouchBreakdownEntry[] {
  return (touch?.labeled_touch_counts?.entries ?? []).map((entry) => ({
    labels: entry.labels,
    count: entry.count,
  }));
}

function renderTouchBreakdownRows(
  labeledEntries: TouchBreakdownEntry[],
  breakdownClasses: TouchBreakdownClass[],
): string {
  if (breakdownClasses.length === 0 || labeledEntries.length === 0) {
    return "";
  }

  const groups = new Map<string, { values: TouchBreakdownValueMap; count: number }>();

  for (const entry of labeledEntries) {
    const labelMap = new Map(entry.labels.map((label) => [label.key, label.value]));
    const values = {} as TouchBreakdownValueMap;
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
      existing.count += entry.count;
    } else {
      groups.set(key, { values, count: entry.count });
    }
  }

  return [...groups.values()]
    .sort((left, right) => compareBreakdownValues(left.values, right.values, breakdownClasses))
    .map((entry) => renderStatRow(
      formatBreakdownLabel(entry.values, breakdownClasses),
      formatInteger(entry.count),
    ))
    .join("");
}

function renderTouchBreakdownFallbackRows(
  touch: PlayerStatsSnapshot["touch"],
  breakdownClasses: TouchBreakdownClass[],
): string {
  if (!touch || breakdownClasses.length !== 1) {
    return "";
  }

  const [className] = breakdownClasses;
  if (className === "kind") {
    return [
      renderStatRow("Dribble", formatInteger(touch.dribble_touch_count)),
      renderStatRow("Control", formatInteger(touch.control_touch_count)),
      renderStatRow("Medium", formatInteger(touch.medium_hit_count)),
      renderStatRow("Hard", formatInteger(touch.hard_hit_count)),
    ].join("");
  }

  if (className === "height_band") {
    const highAir = touch.high_aerial_touch_count ?? 0;
    const lowAir = (touch.aerial_touch_count ?? 0) - highAir;
    const ground = (touch.touch_count ?? 0) - (touch.aerial_touch_count ?? 0);
    return [
      renderStatRow("Ground", formatInteger(ground)),
      renderStatRow("Low air", formatInteger(lowAir)),
      renderStatRow("High air", formatInteger(highAir)),
    ].join("");
  }

  return "";
}

export function renderTouchStats(
  touch: PlayerStatsSnapshot["touch"],
  options: TouchRenderOptions = {},
): string {
  const breakdownClasses = normalizeBreakdownClasses(options.breakdownClasses);
  const snapshotEntries = labeledEntriesFromTouchSnapshot(touch);
  const breakdownRows = renderTouchBreakdownRows(snapshotEntries, breakdownClasses)
    || renderTouchBreakdownFallbackRows(touch, breakdownClasses);

  return `
    ${renderStatRow("Touches", formatInteger(touch?.touch_count))}
    ${breakdownRows}
  `;
}
