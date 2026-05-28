import "./report.css";
import { toBoostDisplayUnits } from "./boostFormatting.ts";
import { formatReplayLoadProgress, loadReplayBundleInWorker } from "./replayLoader.ts";
import { createStatRegistry, type StatDefinition, type StatScopeKind } from "./statRegistry.ts";
import { createStatsFrameLookup } from "./statsTimeline.ts";
import { renderBoostPage } from "./reportBoostPage.ts";
import { el } from "./reportDom.ts";
import { formatSeconds, formatShare, formatTime } from "./reportFormat.ts";
import { renderGoalsPage, type StatsReportGoalWatchRequest } from "./reportGoalsPage.ts";
import { createPageIntro, createSummaryCard, getLeader } from "./reportLayout.ts";
import {
  renderChartCard,
  renderCharts,
  renderMetricGrid,
  renderStackedRows,
  renderTerritoryShareChart,
  type ChartSpec,
} from "./reportCharts.ts";
import { renderTerritoryPage } from "./reportTerritoryPage.ts";
import type {
  PlayerStatsSnapshot,
  StatsFrame,
  StatsFrameLookup,
  StatsTimeline,
  TeamStatsSnapshot,
} from "./statsTimeline.ts";

type StatsTarget = PlayerStatsSnapshot | TeamStatsSnapshot;
type ReportPageId = "overview" | "goals" | "boost" | "territory" | "involvement" | "dump";

export type { StatsReportGoalWatchRequest } from "./reportGoalsPage.ts";

export interface StatsReportData {
  fileName: string;
  replayUrl: URL | null;
  statsTimeline: StatsTimeline;
  statsFrameLookup?: StatsFrameLookup;
}

export interface StatsReportMountOptions {
  initialData?: StatsReportData | null;
  showStandaloneActions?: boolean;
  onWatchGoal?: (request: StatsReportGoalWatchRequest) => void;
}

export interface StatsReportHandle {
  readonly root: HTMLElement;
  render(data: StatsReportData): void;
  destroy(): void;
}

type ReportState = StatsReportData & { statsFrameLookup: StatsFrameLookup };

let currentReportRoot: HTMLElement | null = null;
let currentReportOptions: StatsReportMountOptions = {};

const PAGES: readonly { id: ReportPageId; label: string }[] = [
  { id: "overview", label: "Overview" },
  { id: "goals", label: "Goals" },
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

function targetName(target: StatsTarget, scope: StatScopeKind, index: number): string {
  if (scope === "player") {
    return (target as PlayerStatsSnapshot).name || `Player ${index + 1}`;
  }
  return index === 0 ? "Blue" : "Orange";
}

function getTargets(frame: StatsFrame, scope: StatScopeKind): StatsTarget[] {
  return scope === "player" ? frame.players : [frame.team_zero, frame.team_one];
}

function getFinalFrame(
  statsTimeline: StatsTimeline,
  statsFrameLookup: StatsFrameLookup,
): StatsFrame | null {
  const finalFrame = statsTimeline.frames.at(-1);
  return finalFrame ? (statsFrameLookup.get(finalFrame.frame_number) ?? null) : null;
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

function renderOverviewPage(
  state: ReportState,
  finalFrame: StatsFrame,
  definitions: StatDefinition[],
): HTMLElement {
  const page = el("div", { className: "stats-report-page" });
  page.append(createSummary(state, finalFrame));
  page.append(
    createPageIntro(
      "Featured stats",
      "A shorter readout of stable scoreboard, touch, boost, possession, and pressure signals. The raw export remains available in All stats.",
    ),
  );

  const score = `${finalFrame.team_zero.core.goals}-${finalFrame.team_one.core.goals}`;
  page.append(
    renderMetricGrid([
      createSummaryCard("Final score", score, "Blue - Orange"),
      getLeader(
        finalFrame.players,
        (player) => player.touch.touch_count,
        (value) => `${value} touches`,
      ),
      getLeader(
        finalFrame.players,
        (player) =>
          player.boost.tracked_time > 0
            ? toBoostDisplayUnits(player.boost.boost_integral / player.boost.tracked_time)
            : 0,
        (value) => `${Number(value.toFixed(0))} avg boost`,
      ),
      getLeader(
        finalFrame.players,
        (player) => player.core.score,
        (value) => `${value} score`,
      ),
    ]),
  );

  const charts =
    renderCharts(definitions, finalFrame, OVERVIEW_CHARTS) ??
    el("section", { className: "stats-report-charts" });
  charts.append(renderTerritoryShareChart(finalFrame));
  page.append(charts);
  return page;
}

function renderInvolvementPage(finalFrame: StatsFrame, definitions: StatDefinition[]): HTMLElement {
  const page = el("div", { className: "stats-report-page" });
  page.append(
    createPageIntro(
      "Player involvement",
      "Interaction stats that are usually easier to trust at a glance: touches, hits, demos, 50/50 outcomes, movement, and powerslide usage.",
    ),
  );

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
  page.append(
    el("p", {
      className: "stats-report-note",
      text: "Experimental mechanic detectors such as musty flicks, speed flips, dodge refreshes, and ceiling shots are kept in All stats until their precision is stronger.",
    }),
  );
  return page;
}

function renderDumpPage(
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

function getActivePageId(): ReportPageId {
  const raw = window.location.hash.replace(/^#/, "");
  return PAGES.some((page) => page.id === raw) ? (raw as ReportPageId) : "overview";
}

function renderPageTabs(
  activePage: ReportPageId,
  root: HTMLElement,
  state: ReportState,
): HTMLElement {
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
      text:
        statusText ??
        "Load a Rocket League replay to review curated stats pages, comparison graphs, and the complete raw stat dump.",
    }),
  );

  if (currentReportOptions.showStandaloneActions !== false) {
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
      const root = currentReportRoot;
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
  } else {
    header.append(title);
  }
  return header;
}

function renderReport(root: HTMLElement, state: ReportState): void {
  const finalFrame = getFinalFrame(state.statsTimeline, state.statsFrameLookup);
  if (!finalFrame) {
    root.replaceChildren(
      el("main", {
        className: "stats-report-empty",
        text: "The replay did not produce any stats frames.",
      }),
    );
    return;
  }

  const definitions = createStatRegistry(finalFrame).filter(isStatsReportDefinitionVisible);
  const grouped = groupDefinitions(definitions);
  const activePage = getActivePageId();
  const main = el("main", { className: "stats-report" });
  main.append(createHeader());
  main.append(renderPageTabs(activePage, root, state));

  if (activePage === "goals") {
    main.append(renderGoalsPage(state, finalFrame, currentReportOptions.onWatchGoal));
  } else if (activePage === "boost") {
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

function normalizeStatsReportData(data: StatsReportData): ReportState {
  return {
    ...data,
    statsFrameLookup: data.statsFrameLookup ?? createStatsFrameLookup(data.statsTimeline),
  };
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
  replayUrl: URL | null,
): Promise<void> {
  renderLoading(root, `Loading ${fileName}...`);
  const bundle = await loadReplayBundleInWorker(bytes, {
    onProgress(progress) {
      renderLoading(root, formatReplayLoadProgress(progress));
    },
  });
  renderReport(root, {
    fileName,
    replayUrl,
    statsTimeline: bundle.statsTimeline,
    statsFrameLookup: bundle.statsFrameLookup,
  });
}

async function loadReplayFile(root: HTMLElement, file: File): Promise<void> {
  try {
    await loadReplayBytes(root, new Uint8Array(await file.arrayBuffer()), file.name, null);
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
    await loadReplayBytes(
      root,
      new Uint8Array(await response.arrayBuffer()),
      fileName,
      response.url ? new URL(response.url) : new URL(replayUrl, window.location.href),
    );
  } catch (error) {
    renderLoading(root, error instanceof Error ? error.message : String(error));
  }
}

export function mountStatsReport(
  root: HTMLElement,
  options: StatsReportMountOptions = {},
): StatsReportHandle {
  currentReportRoot = root;
  currentReportOptions = options;

  if (options.initialData) {
    renderReport(root, normalizeStatsReportData(options.initialData));
  } else {
    const main = el("main", { className: "stats-report" });
    main.append(createHeader());
    main.append(
      el("section", {
        className: "stats-report-empty",
        text: "Load a replay to generate the stats report.",
      }),
    );
    root.replaceChildren(main);
  }

  const replayUrl = new URL(window.location.href).searchParams.get("replayUrl");
  if (!options.initialData && replayUrl) {
    void loadReplayUrl(root, replayUrl);
  }

  return {
    root,
    render(data) {
      renderReport(root, normalizeStatsReportData(data));
    },
    destroy() {
      if (currentReportRoot === root) {
        currentReportRoot = null;
        currentReportOptions = {};
      }
      root.replaceChildren();
    },
  };
}
