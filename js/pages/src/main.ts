import "./styles.css";
import { toBoostDisplayUnits } from "../../stat-evaluation-player/src/boostFormatting.ts";
import {
  formatReplayLoadProgress,
  loadReplayBundleInWorker,
} from "../../stat-evaluation-player/src/replayLoader.ts";
import {
  createStatRegistry,
  type StatDefinition,
  type StatScopeKind,
} from "../../stat-evaluation-player/src/statRegistry.ts";
import type {
  PlayerStatsSnapshot,
  StatsFrame,
  StatsTimeline,
  TeamStatsSnapshot,
} from "../../stat-evaluation-player/src/statsTimeline.ts";

type StatsTarget = PlayerStatsSnapshot | TeamStatsSnapshot;
type ChartKind = "bar" | "pie";
type ReportPageId = "overview" | "boost" | "territory" | "involvement" | "dump";

interface ReportState {
  fileName: string;
  statsTimeline: StatsTimeline;
}

interface ChartSpec {
  statId: string;
  kind: ChartKind;
  title: string;
}

interface NumberRow {
  label: string;
  value: number;
  color: string;
  formatted?: string;
}

interface StackedRow {
  label: string;
  segments: NumberRow[];
}

const TEAM_COLORS = ["#58a6ff", "#f39a37"];
const CHART_COLORS = [
  "#58a6ff",
  "#f39a37",
  "#65d6ad",
  "#d2a8ff",
  "#ff7b72",
  "#f2cc60",
  "#79c0ff",
  "#ffa657",
];

const BOOST_TANK_COLORS = {
  zero: "#ff7b72",
  low: "#f39a37",
  midLow: "#f2cc60",
  midHigh: "#65d6ad",
  high: "#58a6ff",
} as const;

const PAGES: readonly { id: ReportPageId; label: string }[] = [
  { id: "overview", label: "Overview" },
  { id: "boost", label: "Boost" },
  { id: "territory", label: "Possession & territory" },
  { id: "involvement", label: "Player involvement" },
  { id: "dump", label: "All stats" },
];

const OVERVIEW_CHARTS: ChartSpec[] = [
  { statId: "player:core.score", kind: "bar", title: "Score by player" },
  { statId: "player:core.shots", kind: "bar", title: "Shots by player" },
  { statId: "player:touch.touch_count", kind: "bar", title: "Touches by player" },
  { statId: "team:core.shots", kind: "pie", title: "Shot share" },
  { statId: "team:possession.possession_time", kind: "pie", title: "Possession share" },
  { statId: "team:pressure.offensive_pressure_time", kind: "bar", title: "Offensive pressure" },
];

const INVOLVEMENT_CHARTS: ChartSpec[] = [
  { statId: "player:touch.touch_count", kind: "bar", title: "Touches" },
  { statId: "player:touch.control_touch_count", kind: "bar", title: "Control touches" },
  { statId: "player:touch.hard_hit_count", kind: "bar", title: "Hard hits" },
  { statId: "player:demo.demos_inflicted", kind: "bar", title: "Demos inflicted" },
  { statId: "player:fifty_fifty.wins", kind: "bar", title: "50/50 wins" },
  { statId: "player:powerslide.total_duration", kind: "bar", title: "Powerslide time" },
];

function el<K extends keyof HTMLElementTagNameMap>(
  tagName: K,
  options: { className?: string; text?: string; id?: string } = {},
): HTMLElementTagNameMap[K] {
  const element = document.createElement(tagName);
  if (options.className) element.className = options.className;
  if (options.id) element.id = options.id;
  if (options.text !== undefined) element.textContent = options.text;
  return element;
}

function targetName(target: StatsTarget, scope: StatScopeKind, index: number): string {
  if (scope === "player") {
    return (target as PlayerStatsSnapshot).name || `Player ${index + 1}`;
  }
  return index === 0 ? "Blue" : "Orange";
}

function getTargets(frame: StatsFrame, scope: StatScopeKind): StatsTarget[] {
  return scope === "player" ? frame.players : [frame.team_zero, frame.team_one];
}

function getFinalFrame(statsTimeline: StatsTimeline): StatsFrame | null {
  return statsTimeline.frames.at(-1) ?? null;
}

function readNumber(definition: StatDefinition, target: StatsTarget): number | null {
  const value = definition.read(target);
  return typeof value === "number" && Number.isFinite(value) ? value : null;
}

function formatSeconds(value: number | null | undefined): string {
  return value == null || !Number.isFinite(value) ? "--" : `${Number(value.toFixed(1))}s`;
}

function formatPercent(value: number | null | undefined): string {
  return value == null || !Number.isFinite(value) ? "--" : `${Number(value.toFixed(1))}%`;
}

function formatShare(value: number, total: number): string {
  return total > 0 ? `${formatSeconds(value)} (${formatPercent((value / total) * 100)})` : "--";
}

function formatBoostAmount(raw: number | null | undefined): string {
  return raw == null || !Number.isFinite(raw) ? "--" : `${Number(toBoostDisplayUnits(raw).toFixed(0))}`;
}

function formatBoostPerMinute(raw: number, trackedTime: number): string {
  return trackedTime > 0
    ? `${Number(((toBoostDisplayUnits(raw) / trackedTime) * 60).toFixed(1))}/min`
    : "--";
}

function groupDefinitions(definitions: StatDefinition[]): Map<string, StatDefinition[]> {
  const groups = new Map<string, StatDefinition[]>();
  for (const definition of definitions) {
    const key = `${definition.scope}:${definition.category}`;
    const group = groups.get(key);
    if (group) {
      group.push(definition);
    } else {
      groups.set(key, [definition]);
    }
  }
  return new Map([...groups].sort(([left], [right]) => left.localeCompare(right)));
}

function formatSectionTitle(key: string): string {
  const [scope, category] = key.split(":");
  const prettyCategory = (category ?? "")
    .replace(/_/g, " ")
    .replace(/\b\w/g, (letter) => letter.toUpperCase());
  return `${scope === "player" ? "Player" : "Team"} ${prettyCategory}`;
}

function sectionId(key: string): string {
  return `stats-${key.replace(/[^a-z0-9]+/gi, "-").toLowerCase()}`;
}

function formatStatLabel(definition: StatDefinition): string {
  return definition.path.slice(1).join(".") || definition.label;
}

function isStatsReportDefinitionVisible(definition: StatDefinition): boolean {
  return !definition.path.includes("entries");
}

function createSummaryCard(label: string, value: string, detail?: string): HTMLElement {
  const card = el("section", { className: "stats-report-summary-card" });
  card.append(el("span", { text: label }), el("strong", { text: value }));
  if (detail) {
    card.append(el("small", { text: detail }));
  }
  return card;
}

function createSummary(state: ReportState, finalFrame: StatsFrame): HTMLElement {
  const summary = el("section", { className: "stats-report-summary" });
  const duration = finalFrame.time > 0 ? formatSeconds(finalFrame.time) : "--";
  summary.append(
    createSummaryCard("Replay", state.fileName),
    createSummaryCard("Frames", state.statsTimeline.frames.length.toLocaleString()),
    createSummaryCard("Duration", duration),
    createSummaryCard("Players", finalFrame.players.length.toLocaleString()),
  );
  return summary;
}

function createPageIntro(title: string, text: string): HTMLElement {
  const intro = el("section", { className: "stats-report-page-intro" });
  intro.append(el("h2", { text: title }), el("p", { text }));
  return intro;
}

function renderStatsTable(
  key: string,
  definitions: StatDefinition[],
  finalFrame: StatsFrame,
): HTMLElement {
  const scope = definitions[0]?.scope ?? "player";
  const targets = getTargets(finalFrame, scope);
  const section = el("section", {
    className: "stats-report-section",
    id: sectionId(key),
  });
  const header = el("header");
  header.append(
    el("h2", { text: formatSectionTitle(key) }),
    el("span", { text: `${definitions.length} stats` }),
  );

  const wrap = el("div", { className: "stats-report-table-wrap" });
  const table = el("table", { className: "stats-report-table" });
  const thead = el("thead");
  const headerRow = el("tr");
  headerRow.append(el("th", { text: "Statistic" }));
  targets.forEach((target, index) => {
    headerRow.append(el("th", { text: targetName(target, scope, index) }));
  });
  thead.append(headerRow);

  const tbody = el("tbody");
  definitions.forEach((definition) => {
    const row = el("tr");
    row.append(el("td", { text: formatStatLabel(definition) }));
    targets.forEach((target) => {
      row.append(el("td", { text: definition.format(definition.read(target)) }));
    });
    tbody.append(row);
  });

  table.append(thead, tbody);
  wrap.append(table);
  section.append(header, wrap);
  return section;
}

function getChartRows(definition: StatDefinition, finalFrame: StatsFrame): NumberRow[] {
  return getTargets(finalFrame, definition.scope)
    .map((target, index) => ({
      label: targetName(target, definition.scope, index),
      value: readNumber(definition, target) ?? 0,
      color: definition.scope === "team"
        ? TEAM_COLORS[index % TEAM_COLORS.length]!
        : CHART_COLORS[index % CHART_COLORS.length]!,
    }))
    .filter((row) => row.value > 0);
}

function renderBarChartRows(
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

function formatChartValue(definition: StatDefinition, value: number): string {
  const pathText = definition.path.join(".");
  if (definition.category === "boost" && (
    pathText.includes("amount_") ||
    pathText.includes("overfill") ||
    pathText.includes("boost_integral")
  )) {
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

function renderBarChart(definition: StatDefinition, finalFrame: StatsFrame): HTMLElement {
  return renderBarChartRows(
    getChartRows(definition, finalFrame),
    (value) => formatChartValue(definition, value),
  );
}

function describePieSegments(rows: NumberRow[]): string {
  const total = rows.reduce((sum, row) => sum + row.value, 0);
  if (total <= 0) {
    return "conic-gradient(rgba(255,255,255,0.12) 0 360deg)";
  }

  let cursor = 0;
  return `conic-gradient(${
    rows.map((row) => {
      const start = cursor;
      cursor += (row.value / total) * 360;
      return `${row.color} ${start}deg ${cursor}deg`;
    }).join(", ")
  })`;
}

function renderPieChartRows(rows: NumberRow[], format: (value: number) => string): HTMLElement {
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
  return renderPieChartRows(
    getChartRows(definition, finalFrame),
    (value) => formatChartValue(definition, value),
  );
}

function renderTerritoryShareChart(finalFrame: StatsFrame, title = "Territory share"): HTMLElement {
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

function renderChartCard(
  title: string,
  body: HTMLElement,
  detail?: string,
): HTMLElement {
  const card = el("section", { className: "stats-report-chart-card" });
  card.append(el("h3", { text: title }));
  if (detail) {
    card.append(el("p", { text: detail }));
  }
  card.append(body);
  return card;
}

function renderDefinitionChartCard(
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

function renderCharts(
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

function renderStackedRows(
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

function renderMetricGrid(cards: HTMLElement[]): HTMLElement {
  const grid = el("section", { className: "stats-report-metric-grid" });
  grid.append(...cards);
  return grid;
}

function getLeader(
  players: PlayerStatsSnapshot[],
  read: (player: PlayerStatsSnapshot) => number,
  format: (value: number) => string,
): HTMLElement {
  const leader = [...players].sort((left, right) => read(right) - read(left))[0];
  const value = leader ? read(leader) : 0;
  return createSummaryCard(leader?.name ?? "--", format(value));
}

function renderOverviewPage(
  state: ReportState,
  finalFrame: StatsFrame,
  definitions: StatDefinition[],
): HTMLElement {
  const page = el("div", { className: "stats-report-page" });
  page.append(createSummary(state, finalFrame));
  page.append(createPageIntro(
    "Featured stats",
    "A shorter readout of stable scoreboard, touch, boost, possession, and pressure signals. The raw export remains available in All stats.",
  ));

  const score = `${finalFrame.team_zero.core.goals}-${finalFrame.team_one.core.goals}`;
  page.append(renderMetricGrid([
    createSummaryCard("Final score", score, "Blue - Orange"),
    getLeader(finalFrame.players, (player) => player.touch.touch_count, (value) => `${value} touches`),
    getLeader(
      finalFrame.players,
      (player) => player.boost.tracked_time > 0
        ? toBoostDisplayUnits(player.boost.boost_integral / player.boost.tracked_time)
        : 0,
      (value) => `${Number(value.toFixed(0))} avg boost`,
    ),
    getLeader(finalFrame.players, (player) => player.core.score, (value) => `${value} score`),
  ]));

  const charts = renderCharts(definitions, finalFrame, OVERVIEW_CHARTS)
    ?? el("section", { className: "stats-report-charts" });
  charts.append(renderTerritoryShareChart(finalFrame));
  page.append(charts);
  return page;
}

function renderBoostPage(
  finalFrame: StatsFrame,
  definitions: StatDefinition[],
): HTMLElement {
  const page = el("div", { className: "stats-report-page" });
  page.append(createPageIntro(
    "Boost economy",
    "A focused view of boost usage, collection, pad mix, starvation, and waste. Values are shown in normal 0-100 boost units.",
  ));

  page.append(renderMetricGrid([
    getLeader(
      finalFrame.players,
      (player) => player.boost.amount_used,
      (value) => `${formatBoostAmount(value)} used`,
    ),
    getLeader(
      finalFrame.players,
      (player) => player.boost.amount_stolen,
      (value) => `${formatBoostAmount(value)} stolen`,
    ),
    getLeader(
      finalFrame.players,
      (player) => player.boost.overfill_total,
      (value) => `${formatBoostAmount(value)} overfill`,
    ),
    getLeader(
      finalFrame.players,
      (player) => player.boost.time_zero_boost,
      (value) => `${formatSeconds(value)} at zero`,
    ),
  ]));

  const charts = el("section", { className: "stats-report-charts" });
  charts.append(
    renderChartCard(
      "Boost used per minute",
      renderBarChartRows(
        finalFrame.players.map((player, index) => ({
          label: player.name || `Player ${index + 1}`,
          value: player.boost.tracked_time > 0
            ? toBoostDisplayUnits(player.boost.amount_used) / player.boost.tracked_time * 60
            : 0,
          color: CHART_COLORS[index % CHART_COLORS.length]!,
          formatted: formatBoostPerMinute(player.boost.amount_used, player.boost.tracked_time),
        })),
        (value) => `${Number(value.toFixed(1))}/min`,
      ),
    ),
    renderChartCard(
      "Pad collection mix",
      renderStackedRows(
        finalFrame.players.map((player, index) => ({
          label: player.name || `Player ${index + 1}`,
          segments: [
            {
              label: "Big",
              value: player.boost.amount_collected_big,
              color: CHART_COLORS[index % CHART_COLORS.length]!,
            },
            {
              label: "Small",
              value: player.boost.amount_collected_small,
              color: "#65d6ad",
            },
            {
              label: "Inactive",
              value: player.boost.amount_collected_inactive,
              color: "rgba(255,255,255,0.22)",
            },
          ],
        })),
        (value) => formatBoostAmount(value),
      ),
    ),
    renderChartCard(
      "Boost tank time",
      renderStackedRows(
        finalFrame.players.map((player, index) => ({
          label: player.name || `Player ${index + 1}`,
          segments: [
            { label: "0", value: player.boost.time_zero_boost, color: BOOST_TANK_COLORS.zero },
            { label: "0-25", value: player.boost.time_boost_0_25, color: BOOST_TANK_COLORS.low },
            { label: "25-50", value: player.boost.time_boost_25_50, color: BOOST_TANK_COLORS.midLow },
            { label: "50-75", value: player.boost.time_boost_50_75, color: BOOST_TANK_COLORS.midHigh },
            {
              label: "75-100",
              value: player.boost.time_boost_75_100 + player.boost.time_hundred_boost,
              color: BOOST_TANK_COLORS.high,
            },
          ],
        })),
        formatShare,
      ),
    ),
  );

  const byId = new Map(definitions.map((definition) => [definition.id, definition]));
  for (const spec of [
    { statId: "player:boost.amount_used", kind: "bar", title: "Total boost used" },
    { statId: "player:boost.overfill_total", kind: "bar", title: "Boost overfill" },
    { statId: "player:boost.amount_stolen", kind: "bar", title: "Stolen boost" },
  ] satisfies ChartSpec[]) {
    const definition = byId.get(spec.statId);
    const chart = definition ? renderDefinitionChartCard(spec, definition, finalFrame) : null;
    if (chart) charts.append(chart);
  }
  page.append(charts);
  page.append(renderBoostScorecard(finalFrame));
  return page;
}

function renderBoostScorecard(finalFrame: StatsFrame): HTMLElement {
  const section = el("section", { className: "stats-report-section" });
  const header = el("header");
  header.append(el("h2", { text: "Boost scorecard" }), el("span", { text: "display units" }));

  const metrics: {
    label: string;
    read(player: PlayerStatsSnapshot): string;
  }[] = [
    {
      label: "Average boost",
      read(player) {
        return player.boost.tracked_time > 0
          ? `${Number(toBoostDisplayUnits(player.boost.boost_integral / player.boost.tracked_time).toFixed(0))}`
          : "--";
      },
    },
    {
      label: "Used per minute",
      read(player) {
        return formatBoostPerMinute(player.boost.amount_used, player.boost.tracked_time);
      },
    },
    {
      label: "Collected",
      read(player) {
        return formatBoostAmount(player.boost.amount_collected);
      },
    },
    {
      label: "Stolen",
      read(player) {
        return formatBoostAmount(player.boost.amount_stolen);
      },
    },
    {
      label: "Overfill",
      read(player) {
        return formatBoostAmount(player.boost.overfill_total);
      },
    },
    {
      label: "Big pads",
      read(player) {
        return `${player.boost.big_pads_collected}`;
      },
    },
    {
      label: "Small pads",
      read(player) {
        return `${player.boost.small_pads_collected}`;
      },
    },
    {
      label: "Time at zero",
      read(player) {
        return formatShare(player.boost.time_zero_boost, player.boost.tracked_time);
      },
    },
  ];

  const wrap = el("div", { className: "stats-report-table-wrap" });
  const table = el("table", { className: "stats-report-table" });
  const thead = el("thead");
  const headerRow = el("tr");
  headerRow.append(el("th", { text: "Metric" }));
  finalFrame.players.forEach((player, index) => {
    headerRow.append(el("th", { text: player.name || `Player ${index + 1}` }));
  });
  thead.append(headerRow);

  const tbody = el("tbody");
  metrics.forEach((metric) => {
    const row = el("tr");
    row.append(el("td", { text: metric.label }));
    finalFrame.players.forEach((player) => {
      row.append(el("td", { text: metric.read(player) }));
    });
    tbody.append(row);
  });

  table.append(thead, tbody);
  wrap.append(table);
  section.append(header, wrap);
  return section;
}

function renderTerritoryPage(finalFrame: StatsFrame): HTMLElement {
  const page = el("div", { className: "stats-report-page" });
  page.append(createPageIntro(
    "Possession & territory",
    "Team control, field-half pressure, and where each player spent time relative to the field and the ball.",
  ));

  const possessionTotal = finalFrame.team_zero.possession.tracked_time;
  const pressureTotal = finalFrame.team_zero.pressure.tracked_time;
  page.append(renderMetricGrid([
    createSummaryCard(
      "Blue possession",
      formatShare(finalFrame.team_zero.possession.possession_time, possessionTotal),
    ),
    createSummaryCard(
      "Orange possession",
      formatShare(finalFrame.team_zero.possession.opponent_possession_time, possessionTotal),
    ),
    createSummaryCard(
      "Blue pressure",
      formatShare(finalFrame.team_zero.pressure.offensive_half_time, pressureTotal),
      "Time in Orange half",
    ),
    createSummaryCard(
      "Orange pressure",
      formatShare(finalFrame.team_zero.pressure.defensive_half_time, pressureTotal),
      "Time in Blue half",
    ),
  ]));

  const charts = el("section", { className: "stats-report-charts" });
  charts.append(
    renderChartCard(
      "Possession split",
      renderPieChartRows(
        [
          {
            label: "Blue control",
            value: finalFrame.team_zero.possession.possession_time,
            color: TEAM_COLORS[0]!,
          },
          {
            label: "Neutral",
            value: finalFrame.team_zero.possession.neutral_time,
            color: "#65d6ad",
          },
          {
            label: "Orange control",
            value: finalFrame.team_zero.possession.opponent_possession_time,
            color: TEAM_COLORS[1]!,
          },
        ],
        formatSeconds,
      ),
    ),
    renderTerritoryShareChart(finalFrame, "Field half pressure"),
    renderChartCard(
      "Player field thirds",
      renderStackedRows(
        finalFrame.players.map((player, index) => ({
          label: player.name || `Player ${index + 1}`,
          segments: [
            {
              label: "Def",
              value: player.positioning.time_defensive_third,
              color: player.is_team_0 ? TEAM_COLORS[0]! : TEAM_COLORS[1]!,
            },
            { label: "Mid", value: player.positioning.time_neutral_third, color: "#65d6ad" },
            {
              label: "Off",
              value: player.positioning.time_offensive_third,
              color: player.is_team_0 ? TEAM_COLORS[1]! : TEAM_COLORS[0]!,
            },
          ],
        })),
        formatShare,
      ),
    ),
    renderChartCard(
      "Role time",
      renderStackedRows(
        finalFrame.players.map((player, index) => ({
          label: player.name || `Player ${index + 1}`,
          segments: [
            { label: "Most back", value: player.positioning.time_most_back, color: "#58a6ff" },
            { label: "Mid", value: player.positioning.time_mid_role, color: "#65d6ad" },
            { label: "Most forward", value: player.positioning.time_most_forward, color: "#f39a37" },
            { label: "Other", value: player.positioning.time_other_role, color: "rgba(255,255,255,0.22)" },
          ],
        })),
        formatShare,
      ),
    ),
  );
  page.append(charts);
  return page;
}

function renderInvolvementPage(
  finalFrame: StatsFrame,
  definitions: StatDefinition[],
): HTMLElement {
  const page = el("div", { className: "stats-report-page" });
  page.append(createPageIntro(
    "Player involvement",
    "Interaction stats that are usually easier to trust at a glance: touches, hits, demos, 50/50 outcomes, movement, and powerslide usage.",
  ));

  const charts = renderCharts(definitions, finalFrame, INVOLVEMENT_CHARTS);
  if (charts) {
    page.append(charts);
  }

  const movementCharts = el("section", { className: "stats-report-charts" });
  movementCharts.append(
    renderChartCard(
      "Speed bands",
      renderStackedRows(
        finalFrame.players.map((player, index) => ({
          label: player.name || `Player ${index + 1}`,
          segments: [
            { label: "Slow", value: player.movement.time_slow_speed, color: "#58a6ff" },
            { label: "Boost", value: player.movement.time_boost_speed, color: "#f2cc60" },
            { label: "Supersonic", value: player.movement.time_supersonic_speed, color: "#f39a37" },
          ],
        })),
        formatShare,
      ),
    ),
    renderChartCard(
      "Aerial profile",
      renderStackedRows(
        finalFrame.players.map((player, index) => ({
          label: player.name || `Player ${index + 1}`,
          segments: [
            { label: "Ground", value: player.movement.time_on_ground, color: "#65d6ad" },
            { label: "Low air", value: player.movement.time_low_air, color: "#58a6ff" },
            { label: "High air", value: player.movement.time_high_air, color: "#d2a8ff" },
          ],
        })),
        formatShare,
      ),
    ),
  );
  page.append(movementCharts);
  page.append(el("p", {
    className: "stats-report-note",
    text: "Experimental mechanic detectors such as musty flicks, speed flips, dodge resets, and ceiling shots are kept in All stats until their precision is stronger.",
  }));
  return page;
}

function renderDumpPage(
  grouped: Map<string, StatDefinition[]>,
  finalFrame: StatsFrame,
): HTMLElement {
  const page = el("div", { className: "stats-report-page" });
  page.append(createPageIntro(
    "All stats dump",
    "Everything emitted by the current stats timeline, including experimental mechanic counters and low-level breakdowns.",
  ));

  const nav = el("nav", { className: "stats-report-jump-nav" });
  for (const key of grouped.keys()) {
    const link = el("a", { text: formatSectionTitle(key) });
    link.setAttribute("href", `#${sectionId(key)}`);
    nav.append(link);
  }
  page.append(nav);

  const grid = el("div", { className: "stats-report-grid" });
  for (const [key, group] of grouped) {
    grid.append(renderStatsTable(key, group, finalFrame));
  }
  page.append(grid);
  return page;
}

function getActivePageId(): ReportPageId {
  const raw = window.location.hash.replace(/^#/, "");
  return PAGES.some((page) => page.id === raw) ? raw as ReportPageId : "overview";
}

function renderPageTabs(activePage: ReportPageId, root: HTMLElement, state: ReportState): HTMLElement {
  const nav = el("nav", { className: "stats-report-tabs" });
  PAGES.forEach((page) => {
    const button = el("button", { text: page.label });
    button.type = "button";
    button.dataset.active = page.id === activePage ? "true" : "false";
    button.addEventListener("click", () => {
      if (getActivePageId() !== page.id) {
        window.history.replaceState(null, "", `#${page.id}`);
      }
      renderReport(root, state);
    });
    nav.append(button);
  });
  return nav;
}

function createHeader(statusText?: string): HTMLElement {
  const header = el("header", { className: "stats-report-header" });
  const title = el("div", { className: "stats-report-title" });
  title.append(
    el("h1", { text: "Replay Stats" }),
    el("p", {
      text: statusText
        ?? "Load a Rocket League replay to review curated stats pages, comparison graphs, and the complete raw stat dump.",
    }),
  );

  const actions = el("div", { className: "stats-report-actions" });
  const fileLabel = el("label", {
    className: "stats-report-file-label",
    text: "Load replay",
  });
  const fileInput = el("input");
  fileInput.type = "file";
  fileInput.accept = ".replay";
  fileInput.addEventListener("change", async () => {
    const file = fileInput.files?.[0];
    const root = document.querySelector("#app");
    if (file && root instanceof HTMLElement) {
      await loadReplayFile(root, file);
    }
  });
  fileLabel.append(fileInput);

  const playerLink = el("a", {
    className: "stats-report-link",
    text: "Open player",
  });
  playerLink.setAttribute("href", "../");
  actions.append(fileLabel, playerLink);
  header.append(title, actions);
  return header;
}

function renderReport(root: HTMLElement, state: ReportState): void {
  const finalFrame = getFinalFrame(state.statsTimeline);
  if (!finalFrame) {
    root.replaceChildren(el("main", {
      className: "stats-report-empty",
      text: "The replay did not produce any stats frames.",
    }));
    return;
  }

  const definitions = createStatRegistry(finalFrame).filter(isStatsReportDefinitionVisible);
  const grouped = groupDefinitions(definitions);
  const activePage = getActivePageId();
  const main = el("main", { className: "stats-report" });
  main.append(createHeader());
  main.append(renderPageTabs(activePage, root, state));

  if (activePage === "boost") {
    main.append(renderBoostPage(finalFrame, definitions));
  } else if (activePage === "territory") {
    main.append(renderTerritoryPage(finalFrame));
  } else if (activePage === "involvement") {
    main.append(renderInvolvementPage(finalFrame, definitions));
  } else if (activePage === "dump") {
    main.append(renderDumpPage(grouped, finalFrame));
  } else {
    main.append(renderOverviewPage(state, finalFrame, definitions));
  }

  root.replaceChildren(main);
}

function renderLoading(root: HTMLElement, message: string): void {
  const main = el("main", { className: "stats-report" });
  main.append(createHeader(message));
  main.append(el("p", { className: "stats-report-status", text: message }));
  root.replaceChildren(main);
}

async function loadReplayBytes(
  root: HTMLElement,
  bytes: Uint8Array,
  fileName: string,
): Promise<void> {
  renderLoading(root, `Loading ${fileName}...`);
  const bundle = await loadReplayBundleInWorker(bytes, {
    onProgress(progress) {
      renderLoading(root, formatReplayLoadProgress(progress));
    },
  });
  renderReport(root, {
    fileName,
    statsTimeline: bundle.statsTimeline,
  });
}

async function loadReplayFile(root: HTMLElement, file: File): Promise<void> {
  try {
    await loadReplayBytes(root, new Uint8Array(await file.arrayBuffer()), file.name);
  } catch (error) {
    renderLoading(root, error instanceof Error ? error.message : String(error));
  }
}

async function loadReplayUrl(root: HTMLElement, replayUrl: string): Promise<void> {
  try {
    renderLoading(root, `Fetching ${replayUrl}...`);
    const response = await fetch(replayUrl);
    if (!response.ok) {
      throw new Error(`Failed to fetch replay: ${response.status} ${response.statusText}`);
    }
    const pathname = new URL(replayUrl, window.location.href).pathname;
    const fileName = decodeURIComponent(pathname.split("/").pop() || "remote replay");
    await loadReplayBytes(root, new Uint8Array(await response.arrayBuffer()), fileName);
  } catch (error) {
    renderLoading(root, error instanceof Error ? error.message : String(error));
  }
}

function mountStatsReport(root: HTMLElement): void {
  const main = el("main", { className: "stats-report" });
  main.append(createHeader());
  main.append(el("section", {
    className: "stats-report-empty",
    text: "Load a replay to generate the stats report.",
  }));
  root.replaceChildren(main);

  const replayUrl = new URL(window.location.href).searchParams.get("replayUrl");
  if (replayUrl) {
    void loadReplayUrl(root, replayUrl);
  }
}

const root = document.querySelector("#app");
if (!(root instanceof HTMLElement)) {
  throw new Error("Missing #app mount element");
}
mountStatsReport(root);
