import type { PlayerStatsSnapshot, StatsFrame, TeamStatsSnapshot } from "./statsTimeline.ts";
import type { StatDefinition, StatScopeKind } from "./statRegistry.ts";
import { el } from "./reportDom.ts";
import { formatBoostAmount, formatSeconds } from "./reportFormat.ts";

type StatsTarget = PlayerStatsSnapshot | TeamStatsSnapshot;

export type ChartKind = "bar" | "pie";

export interface ChartSpec {
  statId: string;
  kind: ChartKind;
  title: string;
}

export interface NumberRow {
  label: string;
  value: number;
  color: string;
  formatted?: string;
}

export interface StackedRow {
  label: string;
  segments: NumberRow[];
}

export const TEAM_COLORS = ["#58a6ff", "#f39a37"];
export const CHART_COLORS = [
  "#58a6ff",
  "#f39a37",
  "#65d6ad",
  "#d2a8ff",
  "#ff7b72",
  "#f2cc60",
  "#79c0ff",
  "#ffa657",
];

function targetName(target: StatsTarget, scope: StatScopeKind, index: number): string {
  if (scope === "player") {
    return (target as PlayerStatsSnapshot).name || `Player ${index + 1}`;
  }
  return index === 0 ? "Blue" : "Orange";
}

function getTargets(frame: StatsFrame, scope: StatScopeKind): StatsTarget[] {
  return scope === "player" ? frame.players : [frame.team_zero, frame.team_one];
}

export function getPlayerTeamColor(player: PlayerStatsSnapshot): string {
  return player.is_team_0 ? TEAM_COLORS[0]! : TEAM_COLORS[1]!;
}

function getChartTargetColor(target: StatsTarget, scope: StatScopeKind, index: number): string {
  return scope === "player"
    ? getPlayerTeamColor(target as PlayerStatsSnapshot)
    : TEAM_COLORS[index % TEAM_COLORS.length]!;
}

function readNumber(definition: StatDefinition, target: StatsTarget): number | null {
  const value = definition.read(target);
  return typeof value === "number" && Number.isFinite(value) ? value : null;
}

function getChartRows(definition: StatDefinition, finalFrame: StatsFrame): NumberRow[] {
  return getTargets(finalFrame, definition.scope)
    .map((target, index) => ({
      label: targetName(target, definition.scope, index),
      value: readNumber(definition, target) ?? 0,
      color: getChartTargetColor(target, definition.scope, index),
    }))
    .filter((row) => row.value > 0);
}

function formatChartValue(definition: StatDefinition, value: number): string {
  const pathText = definition.path.join(".");
  if (
    definition.category === "boost" &&
    (pathText.includes("amount_") ||
      pathText.includes("overfill") ||
      pathText.includes("boost_integral"))
  ) {
    return formatBoostAmount(value);
  }
  if (
    pathText.endsWith("_time") ||
    pathText.startsWith("time_") ||
    pathText.includes(".time_") ||
    pathText.endsWith("_duration") ||
    pathText === "active_game_time" ||
    pathText === "tracked_time"
  ) {
    return formatSeconds(value);
  }
  return definition.format(value);
}

export function renderBarChartRows(
  rows: NumberRow[],
  format: (value: number) => string,
): HTMLElement {
  const max = Math.max(...rows.map((row) => row.value), 1);
  const body = el("div", { className: "stats-report-bar-chart" });
  rows.forEach((row) => {
    const item = el("div", { className: "stats-report-bar-row" });
    item.style.setProperty("--bar-color", row.color);
    item.style.setProperty("--bar-width", `${Math.max(2, (row.value / max) * 100)}%`);
    item.append(
      el("span", { className: "stats-report-bar-label", text: row.label }),
      el("span", { className: "stats-report-bar-track" }),
      el("strong", { text: row.formatted ?? format(row.value) }),
    );
    body.append(item);
  });
  return body;
}

function renderBarChart(definition: StatDefinition, finalFrame: StatsFrame): HTMLElement {
  return renderBarChartRows(getChartRows(definition, finalFrame), (value) =>
    formatChartValue(definition, value),
  );
}

function describePieSegments(rows: NumberRow[]): string {
  const total = rows.reduce((sum, row) => sum + row.value, 0);
  if (total <= 0) {
    return "conic-gradient(rgba(255,255,255,0.12) 0 360deg)";
  }

  let cursor = 0;
  return `conic-gradient(${rows
    .map((row) => {
      const start = cursor;
      cursor += (row.value / total) * 360;
      return `${row.color} ${start}deg ${cursor}deg`;
    })
    .join(", ")})`;
}

export function renderPieChartRows(
  rows: NumberRow[],
  format: (value: number) => string,
): HTMLElement {
  const total = rows.reduce((sum, row) => sum + row.value, 0);
  const body = el("div", { className: "stats-report-pie-chart" });
  const pie = el("div", { className: "stats-report-pie" });
  pie.style.background = describePieSegments(rows);

  const legend = el("div", { className: "stats-report-pie-legend" });
  rows.forEach((row) => {
    const item = el("div");
    item.style.setProperty("--legend-color", row.color);
    const percentage = total > 0 ? `${Math.round((row.value / total) * 100)}%` : "--";
    item.append(
      el("span", { text: row.label }),
      el("strong", { text: `${row.formatted ?? format(row.value)} (${percentage})` }),
    );
    legend.append(item);
  });

  body.append(pie, legend);
  return body;
}

function renderPieChart(definition: StatDefinition, finalFrame: StatsFrame): HTMLElement {
  return renderPieChartRows(getChartRows(definition, finalFrame), (value) =>
    formatChartValue(definition, value),
  );
}

export function renderChartCard(title: string, body: HTMLElement, detail?: string): HTMLElement {
  const card = el("section", { className: "stats-report-chart-card" });
  card.append(el("h3", { text: title }));
  if (detail) {
    card.append(el("p", { text: detail }));
  }
  card.append(body);
  return card;
}

export function renderDefinitionChartCard(
  spec: ChartSpec,
  definition: StatDefinition,
  finalFrame: StatsFrame,
): HTMLElement | null {
  const rows = getChartRows(definition, finalFrame);
  if (rows.length === 0) {
    return null;
  }

  return renderChartCard(
    spec.title,
    spec.kind === "pie"
      ? renderPieChart(definition, finalFrame)
      : renderBarChart(definition, finalFrame),
  );
}

export function renderCharts(
  definitions: StatDefinition[],
  finalFrame: StatsFrame,
  specs: readonly ChartSpec[],
): HTMLElement | null {
  const byId = new Map(definitions.map((definition) => [definition.id, definition]));
  const charts = el("section", { className: "stats-report-charts" });
  specs.forEach((spec) => {
    const definition = byId.get(spec.statId);
    if (!definition) {
      return;
    }
    const chart = renderDefinitionChartCard(spec, definition, finalFrame);
    if (chart) {
      charts.append(chart);
    }
  });
  return charts.childElementCount > 0 ? charts : null;
}

export function renderTerritoryShareChart(
  finalFrame: StatsFrame,
  title = "Territory share",
): HTMLElement {
  return renderChartCard(
    title,
    renderPieChartRows(
      [
        {
          label: "Blue half",
          value: finalFrame.team_zero.pressure.defensive_half_time,
          color: TEAM_COLORS[0]!,
        },
        {
          label: "Neutral",
          value: finalFrame.team_zero.pressure.neutral_time,
          color: "#65d6ad",
        },
        {
          label: "Orange half",
          value: finalFrame.team_zero.pressure.offensive_half_time,
          color: TEAM_COLORS[1]!,
        },
      ],
      formatSeconds,
    ),
  );
}

export function renderStackedRows(
  rows: StackedRow[],
  format: (value: number, total: number) => string,
): HTMLElement {
  const body = el("div", { className: "stats-report-stacked-chart" });
  rows.forEach((row) => {
    const total = row.segments.reduce((sum, segment) => sum + Math.max(0, segment.value), 0);
    const item = el("div", { className: "stats-report-stacked-row" });
    const track = el("div", { className: "stats-report-stacked-track" });
    row.segments.forEach((segment) => {
      const part = el("span");
      part.style.setProperty("--segment-color", segment.color);
      part.style.setProperty(
        "--segment-width",
        `${total > 0 ? Math.max(1.5, (segment.value / total) * 100) : 0}%`,
      );
      part.title = `${segment.label}: ${format(segment.value, total)}`;
      track.append(part);
    });

    const legend = el("div", { className: "stats-report-stacked-legend" });
    row.segments.forEach((segment) => {
      const entry = el("span", { text: `${segment.label}: ${format(segment.value, total)}` });
      entry.style.setProperty("--legend-color", segment.color);
      legend.append(entry);
    });
    item.append(el("strong", { text: row.label }), track, legend);
    body.append(item);
  });
  return body;
}

export function renderMetricGrid(cards: HTMLElement[]): HTMLElement {
  const grid = el("section", { className: "stats-report-metric-grid" });
  grid.append(...cards);
  return grid;
}
