import { toBoostDisplayUnits } from "./boostFormatting.ts";
import type { StatDefinition } from "./statRegistry.ts";
import {
  renderCharts,
  renderMetricGrid,
  renderTerritoryShareChart,
  type ChartSpec,
} from "./reportCharts.ts";
import { el } from "./reportDom.ts";
import { formatSeconds } from "./reportFormat.ts";
import { createPageIntro, createSummaryCard, getLeader } from "./reportLayout.ts";
import type { StatsFrame, StatsTimeline } from "./statsTimeline.ts";

interface ReportOverviewState {
  fileName: string;
  statsTimeline: StatsTimeline;
}

const OVERVIEW_CHARTS: ChartSpec[] = [
  { statId: "player:core.score", kind: "bar", title: "Score by player" },
  { statId: "player:core.shots", kind: "bar", title: "Shots by player" },
  { statId: "player:touch.touch_count", kind: "bar", title: "Touches by player" },
  { statId: "team:core.shots", kind: "pie", title: "Shot share" },
  { statId: "team:possession.possession_time", kind: "pie", title: "Possession share" },
  { statId: "team:pressure.offensive_pressure_time", kind: "bar", title: "Offensive pressure" },
];

function createSummary(state: ReportOverviewState, finalFrame: StatsFrame): HTMLElement {
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

export function renderOverviewPage(
  state: ReportOverviewState,
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
