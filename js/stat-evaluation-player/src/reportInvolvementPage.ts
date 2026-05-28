import type { StatDefinition } from "./statRegistry.ts";
import {
  renderChartCard,
  renderCharts,
  renderStackedRows,
  type ChartSpec,
} from "./reportCharts.ts";
import { el } from "./reportDom.ts";
import { formatShare } from "./reportFormat.ts";
import { createPageIntro } from "./reportLayout.ts";
import type { StatsFrame } from "./statsTimeline.ts";

const INVOLVEMENT_CHARTS: ChartSpec[] = [
  { statId: "player:touch.touch_count", kind: "bar", title: "Touches" },
  { statId: "player:touch.control_touch_count", kind: "bar", title: "Control touches" },
  { statId: "player:touch.hard_hit_count", kind: "bar", title: "Hard hits" },
  { statId: "player:demo.demos_inflicted", kind: "bar", title: "Demos inflicted" },
  { statId: "player:fifty_fifty.wins", kind: "bar", title: "50/50 wins" },
  { statId: "player:powerslide.total_duration", kind: "bar", title: "Powerslide time" },
];

export function renderInvolvementPage(
  finalFrame: StatsFrame,
  definitions: StatDefinition[],
): HTMLElement {
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
