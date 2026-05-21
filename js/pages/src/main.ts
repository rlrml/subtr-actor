import "./styles.css";
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

interface ReportState {
  fileName: string;
  statsTimeline: StatsTimeline;
}

interface ChartSpec {
  statId: string;
  kind: ChartKind;
  title: string;
}

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

const CHART_SPECS: ChartSpec[] = [
  { statId: "player:core.score", kind: "bar", title: "Score by player" },
  { statId: "player:core.goals", kind: "bar", title: "Goals by player" },
  { statId: "player:core.shots", kind: "bar", title: "Shots by player" },
  { statId: "player:core.saves", kind: "bar", title: "Saves by player" },
  { statId: "player:touch.touch_count", kind: "bar", title: "Touches by player" },
  { statId: "player:boost.bpm", kind: "bar", title: "Boost used per minute" },
  { statId: "team:core.goals", kind: "pie", title: "Goal share" },
  { statId: "team:core.shots", kind: "pie", title: "Shot share" },
  { statId: "team:possession.possession_time", kind: "pie", title: "Possession share" },
  { statId: "team:pressure.offensive_pressure_time", kind: "pie", title: "Pressure share" },
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

function createSummaryCard(label: string, value: string): HTMLElement {
  const card = el("section", { className: "stats-report-summary-card" });
  card.append(el("span", { text: label }), el("strong", { text: value }));
  return card;
}

function createSummary(state: ReportState, finalFrame: StatsFrame): HTMLElement {
  const summary = el("section", { className: "stats-report-summary" });
  const duration = finalFrame.time > 0 ? `${Math.round(finalFrame.time)}s` : "--";
  summary.append(
    createSummaryCard("Replay", state.fileName),
    createSummaryCard("Frames", state.statsTimeline.frames.length.toLocaleString()),
    createSummaryCard("Duration", duration),
    createSummaryCard("Players", finalFrame.players.length.toLocaleString()),
  );
  return summary;
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

function getChartRows(definition: StatDefinition, finalFrame: StatsFrame) {
  return getTargets(finalFrame, definition.scope)
    .map((target, index) => ({
      label: targetName(target, definition.scope, index),
      value: readNumber(definition, target) ?? 0,
      color: CHART_COLORS[index % CHART_COLORS.length]!,
    }))
    .filter((row) => row.value > 0);
}

function renderBarChart(definition: StatDefinition, finalFrame: StatsFrame): HTMLElement {
  const rows = getChartRows(definition, finalFrame);
  const max = Math.max(...rows.map((row) => row.value), 1);
  const body = el("div", { className: "stats-report-bar-chart" });
  rows.forEach((row) => {
    const item = el("div", { className: "stats-report-bar-row" });
    item.style.setProperty("--bar-color", row.color);
    item.style.setProperty("--bar-width", `${Math.max(2, (row.value / max) * 100)}%`);
    item.append(
      el("span", { className: "stats-report-bar-label", text: row.label }),
      el("span", { className: "stats-report-bar-track" }),
      el("strong", { text: definition.format(row.value) }),
    );
    body.append(item);
  });
  return body;
}

function describePieSegments(rows: ReturnType<typeof getChartRows>): string {
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

function renderPieChart(definition: StatDefinition, finalFrame: StatsFrame): HTMLElement {
  const rows = getChartRows(definition, finalFrame);
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
      el("strong", { text: `${definition.format(row.value)} (${percentage})` }),
    );
    legend.append(item);
  });

  body.append(pie, legend);
  return body;
}

function renderChartCard(
  spec: ChartSpec,
  definition: StatDefinition,
  finalFrame: StatsFrame,
): HTMLElement | null {
  const rows = getChartRows(definition, finalFrame);
  if (rows.length === 0) {
    return null;
  }

  const card = el("section", { className: "stats-report-chart-card" });
  card.append(el("h3", { text: spec.title }));
  card.append(
    spec.kind === "pie"
      ? renderPieChart(definition, finalFrame)
      : renderBarChart(definition, finalFrame),
  );
  return card;
}

function renderCharts(definitions: StatDefinition[], finalFrame: StatsFrame): HTMLElement | null {
  const byId = new Map(definitions.map((definition) => [definition.id, definition]));
  const charts = el("section", { className: "stats-report-charts" });
  CHART_SPECS.forEach((spec) => {
    const definition = byId.get(spec.statId);
    if (!definition) {
      return;
    }
    const chart = renderChartCard(spec, definition, finalFrame);
    if (chart) {
      charts.append(chart);
    }
  });
  return charts.childElementCount > 0 ? charts : null;
}

function createHeader(statusText?: string): HTMLElement {
  const header = el("header", { className: "stats-report-header" });
  const title = el("div", { className: "stats-report-title" });
  title.append(
    el("h1", { text: "Replay Stats Report" }),
    el("p", {
      text: statusText
        ?? "Load a Rocket League replay to generate final player and team stats with comparison charts.",
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
  const main = el("main", { className: "stats-report" });
  main.append(createHeader());
  main.append(createSummary(state, finalFrame));

  const charts = renderCharts(definitions, finalFrame);
  if (charts) {
    main.append(charts);
  }

  const nav = el("nav", { className: "stats-report-nav" });
  for (const key of grouped.keys()) {
    const link = el("a", { text: formatSectionTitle(key) });
    link.setAttribute("href", `#${sectionId(key)}`);
    nav.append(link);
  }
  main.append(nav);

  const grid = el("div", { className: "stats-report-grid" });
  for (const [key, group] of grouped) {
    grid.append(renderStatsTable(key, group, finalFrame));
  }
  main.append(grid);
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
