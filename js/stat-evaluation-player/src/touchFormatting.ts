import type { ExportedStat, PlayerStatsSnapshot } from "./statsTimeline.ts";

export type TouchBreakdownClass = "kind" | "aerial" | "high_aerial";

interface TouchRenderOptions {
  breakdownClasses?: TouchBreakdownClass[];
  exportedStats?: ExportedStat[];
}

type TouchBreakdownValueMap = Record<TouchBreakdownClass, string>;

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
  aerial: {
    label: "Aerial",
    valueOrder: ["true", "false"],
    formatValue: (value) => value === "true" ? "Aerial" : "Ground",
  },
  high_aerial: {
    label: "High Aerial",
    valueOrder: ["true", "false"],
    formatValue: (value) => value === "true" ? "High aerial" : "Not high aerial",
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

function renderTouchBreakdownRows(
  exportedStats: ExportedStat[] | undefined,
  breakdownClasses: TouchBreakdownClass[],
): string {
  if (breakdownClasses.length === 0 || !exportedStats || exportedStats.length === 0) {
    return "";
  }

  const groups = new Map<string, { values: TouchBreakdownValueMap; count: number }>();

  for (const stat of exportedStats) {
    if (
      stat.domain !== "touch" ||
      stat.name !== "touch_count" ||
      stat.variant !== "labeled" ||
      stat.value_type !== "unsigned" ||
      !Number.isFinite(stat.value)
    ) {
      continue;
    }

    const labelMap = new Map((stat.labels ?? []).map((label) => [label.key, label.value]));
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
    const count = Math.round(stat.value);
    const existing = groups.get(key);
    if (existing) {
      existing.count += count;
    } else {
      groups.set(key, { values, count });
    }
  }

  const rows = [...groups.values()]
    .sort((left, right) => compareBreakdownValues(left.values, right.values, breakdownClasses))
    .map((entry) => renderStatRow(
      formatBreakdownLabel(entry.values, breakdownClasses),
      formatInteger(entry.count),
    ));

  return rows.join("");
}

export function renderTouchStats(
  touch: PlayerStatsSnapshot["touch"],
  options: TouchRenderOptions = {},
): string {
  const breakdownClasses = normalizeBreakdownClasses(options.breakdownClasses);
  const breakdownRows = renderTouchBreakdownRows(options.exportedStats, breakdownClasses);

  return `
    ${renderStatRow("Touches", formatInteger(touch?.touch_count))}
    ${breakdownRows}
    ${renderStatRow("Current", touch?.is_last_touch ? "Yes" : "No")}
    ${renderStatRow("Touch time", formatNumber(touch?.last_touch_time, 2, "s"))}
    ${renderStatRow("Touch frame", formatInteger(touch?.last_touch_frame))}
    ${renderStatRow("Since touch", formatNumber(touch?.time_since_last_touch, 2, "s"))}
    ${renderStatRow("Frames since", formatInteger(touch?.frames_since_last_touch))}
    ${renderStatRow("Last change", formatNumber(touch?.last_ball_speed_change, 1))}
    ${renderStatRow("Avg change", formatNumber(touch?.average_ball_speed_change, 1))}
    ${renderStatRow("Max change", formatNumber(touch?.max_ball_speed_change, 1))}
  `;
}
