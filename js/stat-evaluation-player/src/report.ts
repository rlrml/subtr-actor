import "./report.css";
import { formatReplayLoadProgress, loadReplayBundleInWorker } from "./replayLoader.ts";
import { createStatRegistry, type StatDefinition } from "./statRegistry.ts";
import { createStatsFrameLookup } from "./statsTimeline.ts";
import { renderBoostPage } from "./reportBoostPage.ts";
import { el } from "./reportDom.ts";
import { renderDumpPage } from "./reportDumpPage.ts";
import { renderGoalsPage, type StatsReportGoalWatchRequest } from "./reportGoalsPage.ts";
import { renderInvolvementPage } from "./reportInvolvementPage.ts";
import { renderOverviewPage } from "./reportOverviewPage.ts";
import { renderTerritoryPage } from "./reportTerritoryPage.ts";
import type { StatsFrame, StatsFrameLookup, StatsTimeline } from "./statsTimeline.ts";

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

function getFinalFrame(
  statsTimeline: StatsTimeline,
  statsFrameLookup: StatsFrameLookup,
): StatsFrame | null {
  const finalFrame = statsTimeline.frames.at(-1);
  return finalFrame ? (statsFrameLookup.get(finalFrame.frame_number) ?? null) : null;
}

function isStatsReportDefinitionVisible(definition: StatDefinition): boolean {
  return !definition.path.includes("entries");
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
    main.append(renderDumpPage(definitions, finalFrame));
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
