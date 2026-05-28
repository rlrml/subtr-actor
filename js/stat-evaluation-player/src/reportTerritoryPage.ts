import { createPageIntro, createSummaryCard } from "./reportLayout.ts";
import {
  TEAM_COLORS,
  renderChartCard,
  renderMetricGrid,
  renderPieChartRows,
  renderStackedRows,
  renderTerritoryShareChart,
} from "./reportCharts.ts";
import { el } from "./reportDom.ts";
import { formatSeconds, formatShare } from "./reportFormat.ts";
import type { StatsFrame } from "./statsTimeline.ts";

export function renderTerritoryPage(finalFrame: StatsFrame): HTMLElement {
  const page = el("div", { className: "stats-report-page" });
  page.append(
    createPageIntro(
      "Possession & territory",
      "Team control, field-half pressure, and where each player spent time relative to the field and the ball.",
    ),
  );

  const possessionTotal = finalFrame.team_zero.possession.tracked_time;
  const pressureTotal = finalFrame.team_zero.pressure.tracked_time;
  page.append(
    renderMetricGrid([
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
    ]),
  );

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
            {
              label: "Most forward",
              value: player.positioning.time_most_forward,
              color: "#f39a37",
            },
            {
              label: "Other",
              value: player.positioning.time_other_role,
              color: "rgba(255,255,255,0.22)",
            },
          ],
        })),
        formatShare,
      ),
    ),
  );
  page.append(charts);
  return page;
}
