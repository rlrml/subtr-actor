import { toBoostDisplayUnits } from "./boostFormatting.ts";
import type { StatDefinition, StatScopeKind } from "./statRegistry.ts";
import {
  setStatsPlayerConfigOnUrl,
  STATS_PLAYER_CONFIG_VERSION,
  type StatsPlayerConfig,
} from "./playerConfig.ts";
import { playerIdToString } from "./touchOverlay.ts";
import type {
  PlayerStatsSnapshot,
  StatsFrame,
  StatsFrameLookup,
  StatsTimeline,
  TeamStatsSnapshot,
} from "./statsTimeline.ts";

export type StatsTarget = PlayerStatsSnapshot | TeamStatsSnapshot;
export type ChartKind = "bar" | "pie";
export type ReportPageId = "overview" | "goals" | "boost" | "territory" | "involvement" | "dump";
export type GoalContextEvent = StatsTimeline["events"]["goal_context"][number];
export type GoalTagEvent = StatsTimeline["events"]["goal_tags"][number];
export type GoalPlayerContext = GoalContextEvent["players"][number];
export type GoalContextPosition = NonNullable<GoalContextEvent["ball_position"]>;

export interface StatsReportData {
  fileName: string;
  replayUrl: URL | null;
  statsTimeline: StatsTimeline;
  statsFrameLookup?: StatsFrameLookup;
}

export interface StatsReportGoalWatchRequest {
  config: StatsPlayerConfig;
  href: string | null;
  goalTime: number;
  playerId: string | null;
}

export type ReportState = StatsReportData & { statsFrameLookup: StatsFrameLookup };

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

export const BOOST_TANK_COLORS = {
  zero: "#ff7b72",
  low: "#f39a37",
  midLow: "#f2cc60",
  midHigh: "#65d6ad",
  high: "#58a6ff",
} as const;

export const PAD_COLLECTION_COLORS = {
  big: "#f39a37",
  small: "#65d6ad",
} as const;

export const PAGES: readonly { id: ReportPageId; label: string }[] = [
  { id: "overview", label: "Overview" },
  { id: "goals", label: "Goals" },
  { id: "boost", label: "Boost" },
  { id: "territory", label: "Possession & territory" },
  { id: "involvement", label: "Player involvement" },
  { id: "dump", label: "All stats" },
];

export const OVERVIEW_CHARTS: ChartSpec[] = [
  { statId: "player:core.score", kind: "bar", title: "Score by player" },
  { statId: "player:core.shots", kind: "bar", title: "Shots by player" },
  { statId: "player:touch.touch_count", kind: "bar", title: "Touches by player" },
  { statId: "team:core.shots", kind: "pie", title: "Shot share" },
  { statId: "team:possession.possession_time", kind: "pie", title: "Possession share" },
  { statId: "team:pressure.offensive_pressure_time", kind: "bar", title: "Offensive pressure" },
];

export const INVOLVEMENT_CHARTS: ChartSpec[] = [
  { statId: "player:touch.touch_count", kind: "bar", title: "Touches" },
  { statId: "player:touch.control_touch_count", kind: "bar", title: "Control touches" },
  { statId: "player:touch.hard_hit_count", kind: "bar", title: "Hard hits" },
  { statId: "player:demo.demos_inflicted", kind: "bar", title: "Demos inflicted" },
  { statId: "player:fifty_fifty.wins", kind: "bar", title: "50/50 wins" },
  { statId: "player:powerslide.total_duration", kind: "bar", title: "Powerslide time" },
];

export function el<K extends keyof HTMLElementTagNameMap>(
  tagName: K,
  options: { className?: string; text?: string; id?: string } = {},
): HTMLElementTagNameMap[K] {
  const element = document.createElement(tagName);
  if (options.className) element.className = options.className;
  if (options.id) element.id = options.id;
  if (options.text !== undefined) element.textContent = options.text;
  return element;
}

export function targetName(target: StatsTarget, scope: StatScopeKind, index: number): string {
  if (scope === "player") {
    return (target as PlayerStatsSnapshot).name || `Player ${index + 1}`;
  }
  return index === 0 ? "Blue" : "Orange";
}

export function remoteIdKey(playerId: Record<string, unknown> | null | undefined): string | null {
  return playerId ? playerIdToString(playerId) : null;
}

export function playerNameForId(
  finalFrame: StatsFrame,
  playerId: Record<string, unknown> | null | undefined,
): string {
  const key = remoteIdKey(playerId);
  if (!key) return "--";
  return finalFrame.players.find((player) => remoteIdKey(player.player_id) === key)?.name ?? key;
}

export function teamLabel(isTeamZero: boolean | null | undefined): string {
  if (isTeamZero === true) return "Blue";
  if (isTeamZero === false) return "Orange";
  return "--";
}

export function getTargets(frame: StatsFrame, scope: StatScopeKind): StatsTarget[] {
  return scope === "player" ? frame.players : [frame.team_zero, frame.team_one];
}

export function getPlayerTeamColor(player: PlayerStatsSnapshot): string {
  return player.is_team_0 ? TEAM_COLORS[0]! : TEAM_COLORS[1]!;
}

export function getChartTargetColor(
  target: StatsTarget,
  scope: StatScopeKind,
  index: number,
): string {
  return scope === "player"
    ? getPlayerTeamColor(target as PlayerStatsSnapshot)
    : TEAM_COLORS[index % TEAM_COLORS.length]!;
}

export function getFinalFrame(
  statsTimeline: StatsTimeline,
  statsFrameLookup: StatsFrameLookup,
): StatsFrame | null {
  const finalFrame = statsTimeline.frames.at(-1);
  return finalFrame ? (statsFrameLookup.get(finalFrame.frame_number) ?? null) : null;
}

export function readNumber(definition: StatDefinition, target: StatsTarget): number | null {
  const value = definition.read(target);
  return typeof value === "number" && Number.isFinite(value) ? value : null;
}

export function formatSeconds(value: number | null | undefined): string {
  return value == null || !Number.isFinite(value) ? "--" : `${Number(value.toFixed(1))}s`;
}

export function formatPercent(value: number | null | undefined): string {
  return value == null || !Number.isFinite(value) ? "--" : `${Number(value.toFixed(1))}%`;
}

export function formatShare(value: number, total: number): string {
  return total > 0 ? `${formatSeconds(value)} (${formatPercent((value / total) * 100)})` : "--";
}

export function formatFieldPosition(position: GoalContextPosition | null | undefined): string {
  if (!position) return "--";
  return `x ${Math.round(position.x)}, y ${Math.round(position.y)}, z ${Math.round(position.z)}`;
}

export function formatBoostAmount(raw: number | null | undefined): string {
  return raw == null || !Number.isFinite(raw)
    ? "--"
    : `${Number(toBoostDisplayUnits(raw).toFixed(0))}`;
}

export function formatTime(seconds: number | null | undefined): string {
  if (seconds == null || !Number.isFinite(seconds)) return "--";
  const clamped = Math.max(0, seconds);
  const minutes = Math.floor(clamped / 60);
  const remainingSeconds = clamped - minutes * 60;
  return `${minutes}:${remainingSeconds.toFixed(1).padStart(4, "0")}`;
}

export function getPlayerUrlForGoal(
  replayUrl: URL | null,
  goalTime: number | null | undefined,
  scorer: Record<string, unknown> | null | undefined,
): string | null {
  if (!replayUrl || goalTime == null || !Number.isFinite(goalTime)) {
    return null;
  }

  const scorerId = remoteIdKey(scorer);
  const playerUrl = new URL("../", window.location.href);
  playerUrl.searchParams.set("replayUrl", replayUrl.href);
  return setStatsPlayerConfigOnUrl(playerUrl, getGoalWatchPlayerConfig(goalTime, scorerId)).href;
}

export function getGoalWatchRequest(
  replayUrl: URL | null,
  goalTime: number | null | undefined,
  scorer: Record<string, unknown> | null | undefined,
): StatsReportGoalWatchRequest | null {
  if (goalTime == null || !Number.isFinite(goalTime)) {
    return null;
  }
  const playerId = remoteIdKey(scorer);
  return {
    config: getGoalWatchPlayerConfig(goalTime, playerId),
    href: getPlayerUrlForGoal(replayUrl, goalTime, scorer),
    goalTime,
    playerId,
  };
}

export function getGoalWatchPlayerConfig(
  goalTime: number,
  scorerId: string | null,
): StatsPlayerConfig {
  return {
    version: STATS_PLAYER_CONFIG_VERSION,
    playback: {
      currentTime: Math.max(0, goalTime - 4),
      playing: true,
      rate: 1,
      skipPostGoalTransitions: false,
      skipKickoffs: false,
    },
    camera: scorerId
      ? {
          mode: "follow",
          attachedPlayerId: scorerId,
          ballCam: true,
        }
      : {
          mode: "free",
        },
    overlays: {
      timelineEvents: ["core"],
      timelineRanges: [],
      mechanics: [],
      renderEffects: [],
      followedPlayerHud: false,
      boostPads: true,
      boostPickupAnimation: false,
      hitboxWireframes: false,
    },
    recording: {},
    singletonWindows: [],
    statsWindows: [],
    moduleConfigs: {},
  };
}

export function formatBoostPerMinute(raw: number, trackedTime: number): string {
  return trackedTime > 0
    ? `${Number(((toBoostDisplayUnits(raw) / trackedTime) * 60).toFixed(1))}/min`
    : "--";
}

export function groupDefinitions(definitions: StatDefinition[]): Map<string, StatDefinition[]> {
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

export function formatSectionTitle(key: string): string {
  const [scope, category] = key.split(":");
  const prettyCategory = (category ?? "")
    .replace(/_/g, " ")
    .replace(/\b\w/g, (letter) => letter.toUpperCase());
  return `${scope === "player" ? "Player" : "Team"} ${prettyCategory}`;
}

export function sectionId(key: string): string {
  return `stats-${key.replace(/[^a-z0-9]+/gi, "-").toLowerCase()}`;
}

export function formatStatLabel(definition: StatDefinition): string {
  return definition.path.slice(1).join(".") || definition.label;
}

export function isStatsReportDefinitionVisible(definition: StatDefinition): boolean {
  return !definition.path.includes("entries");
}

export function createSummaryCard(label: string, value: string, detail?: string): HTMLElement {
  const card = el("section", { className: "stats-report-summary-card" });
  card.append(el("span", { text: label }), el("strong", { text: value }));
  if (detail) {
    card.append(el("small", { text: detail }));
  }
  return card;
}

export function createSummary(state: ReportState, finalFrame: StatsFrame): HTMLElement {
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

export function createPageIntro(title: string, text: string): HTMLElement {
  const intro = el("section", { className: "stats-report-page-intro" });
  intro.append(el("h2", { text: title }), el("p", { text }));
  return intro;
}

export function renderStatsTable(
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

export function getChartRows(definition: StatDefinition, finalFrame: StatsFrame): NumberRow[] {
  return getTargets(finalFrame, definition.scope)
    .map((target, index) => ({
      label: targetName(target, definition.scope, index),
      value: readNumber(definition, target) ?? 0,
      color: getChartTargetColor(target, definition.scope, index),
    }))
    .filter((row) => row.value > 0);
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

export function formatChartValue(definition: StatDefinition, value: number): string {
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

export function renderBarChart(definition: StatDefinition, finalFrame: StatsFrame): HTMLElement {
  return renderBarChartRows(getChartRows(definition, finalFrame), (value) =>
    formatChartValue(definition, value),
  );
}

export function describePieSegments(rows: NumberRow[]): string {
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

export function renderPieChart(definition: StatDefinition, finalFrame: StatsFrame): HTMLElement {
  return renderPieChartRows(getChartRows(definition, finalFrame), (value) =>
    formatChartValue(definition, value),
  );
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

export function getLeader(
  players: PlayerStatsSnapshot[],
  read: (player: PlayerStatsSnapshot) => number,
  format: (value: number) => string,
): HTMLElement {
  const leader = [...players].sort((left, right) => read(right) - read(left))[0];
  const value = leader ? read(leader) : 0;
  return createSummaryCard(leader?.name ?? "--", format(value));
}

export function renderDumpPage(
  grouped: Map<string, StatDefinition[]>,
  finalFrame: StatsFrame,
): HTMLElement {
  const page = el("div", { className: "stats-report-page" });
  page.append(
    createPageIntro(
      "All stats dump",
      "Everything emitted by the current stats timeline, including experimental mechanic counters and low-level breakdowns.",
    ),
  );

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
