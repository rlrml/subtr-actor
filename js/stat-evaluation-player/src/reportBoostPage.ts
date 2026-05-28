import { toBoostDisplayUnits } from "./boostFormatting.ts";
import {
  getPlayerTeamColor,
  renderBarChartRows,
  renderChartCard,
  renderDefinitionChartCard,
  renderMetricGrid,
  renderStackedRows,
  type ChartSpec,
} from "./reportCharts.ts";
import { el } from "./reportDom.ts";
import { formatBoostAmount, formatSeconds, formatShare } from "./reportFormat.ts";
import { createPageIntro, getLeader } from "./reportLayout.ts";
import type { StatDefinition } from "./statRegistry.ts";
import type { PlayerStatsSnapshot, StatsFrame } from "./statsTimeline.ts";

const BOOST_TANK_COLORS = {
  zero: "#ff7b72",
  low: "#f39a37",
  midLow: "#f2cc60",
  midHigh: "#65d6ad",
  high: "#58a6ff",
} as const;

const PAD_COLLECTION_COLORS = {
  big: "#f39a37",
  small: "#65d6ad",
} as const;

function formatBoostPerMinute(raw: number, trackedTime: number): string {
  return trackedTime > 0
    ? `${Number(((toBoostDisplayUnits(raw) / trackedTime) * 60).toFixed(1))}/min`
    : "--";
}

export function renderBoostPage(
  finalFrame: StatsFrame,
  definitions: StatDefinition[],
): HTMLElement {
  const page = el("div", { className: "stats-report-page" });
  page.append(
    createPageIntro(
      "Boost economy",
      "A focused view of boost usage, collection, pad mix, starvation, and waste. Values are shown in normal 0-100 boost units.",
    ),
  );

  page.append(
    renderMetricGrid([
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
    ]),
  );

  const charts = el("section", { className: "stats-report-charts" });
  charts.append(
    renderChartCard(
      "Boost used per minute",
      renderBarChartRows(
        finalFrame.players.map((player, index) => ({
          label: player.name || `Player ${index + 1}`,
          value:
            player.boost.tracked_time > 0
              ? (toBoostDisplayUnits(player.boost.amount_used) / player.boost.tracked_time) * 60
              : 0,
          color: getPlayerTeamColor(player),
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
              color: PAD_COLLECTION_COLORS.big,
            },
            {
              label: "Small",
              value: player.boost.amount_collected_small,
              color: PAD_COLLECTION_COLORS.small,
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
            {
              label: "25-50",
              value: player.boost.time_boost_25_50,
              color: BOOST_TANK_COLORS.midLow,
            },
            {
              label: "50-75",
              value: player.boost.time_boost_50_75,
              color: BOOST_TANK_COLORS.midHigh,
            },
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
