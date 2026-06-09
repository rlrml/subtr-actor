import "./report.css";
import { toBoostDisplayUnits } from "./boostFormatting.ts";
import { formatReplayLoadProgress, loadReplayBundleInWorker } from "./replayLoader.ts";
import { createStatRegistry, type StatDefinition } from "./statRegistry.ts";
import {
  formatGoalTagPerformer,
  formatMechanicKind,
  isScorerGoalTag,
  isTeammatePerformedGoalTag,
} from "./timelineMarkers.ts";
import { createStatsFrameLookup, statsEventPayloads } from "./statsTimeline.ts";
import type { PlayerStatsSnapshot, StatsFrame } from "./statsTimeline.ts";
import {
  BOOST_TANK_COLORS,
  CHART_COLORS,
  INVOLVEMENT_CHARTS,
  OVERVIEW_CHARTS,
  PAD_COLLECTION_COLORS,
  PAGES,
  TEAM_COLORS,
  createPageIntro,
  createSummary,
  createSummaryCard,
  el,
  formatBoostAmount,
  formatBoostPerMinute,
  formatFieldPosition,
  formatSeconds,
  formatShare,
  formatTime,
  getGoalWatchRequest,
  getFinalFrame,
  getLeader,
  getPlayerTeamColor,
  groupDefinitions,
  isStatsReportDefinitionVisible,
  playerNameForId,
  renderBarChartRows,
  renderChartCard,
  renderCharts,
  renderDefinitionChartCard,
  renderDumpPage,
  renderMetricGrid,
  renderPieChartRows,
  renderStackedRows,
  renderTerritoryShareChart,
  teamLabel,
  type ChartSpec,
  type GoalContextEvent,
  type GoalPlayerContext,
  type GoalTag,
  type NumberRow,
  type ReportPageId,
  type ReportState,
  type StatsReportData,
  type StatsReportGoalWatchRequest,
} from "./reportShared.ts";

export type { StatsReportData } from "./reportShared.ts";

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

let currentReportRoot: HTMLElement | null = null;
let currentReportOptions: StatsReportMountOptions = {};

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

function orderedGoalTags(tags: readonly GoalTag[] | null | undefined): GoalTag[] {
  return [...(tags ?? [])].sort(
    (left, right) =>
      left.kind.localeCompare(right.kind) || right.metadata.confidence - left.metadata.confidence,
  );
}

function getGoalTagCounts(goalTags: GoalTag[]): NumberRow[] {
  const counts = new Map<string, number>();
  for (const tag of goalTags) {
    counts.set(tag.kind, (counts.get(tag.kind) ?? 0) + 1);
  }
  return [...counts.entries()]
    .sort(
      ([leftKind, leftCount], [rightKind, rightCount]) =>
        rightCount - leftCount ||
        formatMechanicKind(leftKind).localeCompare(formatMechanicKind(rightKind)),
    )
    .map(([kind, count], index) => ({
      label: formatMechanicKind(kind),
      value: count,
      color: CHART_COLORS[index % CHART_COLORS.length]!,
      formatted: count.toLocaleString(),
    }));
}

function createDetailList(items: { label: string; value: string }[]): HTMLElement {
  const list = el("dl", { className: "stats-report-detail-list" });
  for (const item of items) {
    const detail = el("div", { className: "stats-report-detail-item" });
    detail.append(el("dt", { text: item.label }), el("dd", { text: item.value }));
    list.append(detail);
  }
  return list;
}

function renderGoalTagChips(tags: GoalTag[]): HTMLElement {
  const list = el("div", { className: "stats-report-goal-tags" });
  if (tags.length === 0) {
    list.append(
      el("span", {
        className: "stats-report-goal-tag stats-report-goal-tag-empty",
        text: "Unlabeled",
      }),
    );
    return list;
  }

  for (const tag of tags) {
    const metadata = tag.metadata;
    const performer = formatGoalTagPerformer(tag);
    const modifiers = performer ? ` - ${performer}` : "";
    list.append(
      el("span", {
        className: "stats-report-goal-tag",
        text: `${formatMechanicKind(tag.kind)} ${Math.round(metadata.confidence * 100)}%${modifiers}`,
      }),
    );
  }
  return list;
}

function renderGoalPlayerContextTable(
  finalFrame: StatsFrame,
  players: GoalPlayerContext[],
): HTMLElement | null {
  if (players.length === 0) return null;

  const section = el("div", { className: "stats-report-goal-subsection" });
  section.append(el("h3", { text: "Player context" }));
  const wrap = el("div", { className: "stats-report-table-wrap" });
  const table = el("table", { className: "stats-report-table" });
  const thead = el("thead");
  const headerRow = el("tr");
  ["Player", "Team", "Boost", "Leadup avg", "Leadup min", "Role", "Position"].forEach((label) => {
    headerRow.append(el("th", { text: label }));
  });
  thead.append(headerRow);

  const tbody = el("tbody");
  for (const player of players) {
    const row = el("tr");
    row.append(
      el("td", { text: playerNameForId(finalFrame, player.player) }),
      el("td", { text: teamLabel(player.is_team_0) }),
      el("td", { text: formatBoostAmount(player.boost_amount) }),
      el("td", { text: formatBoostAmount(player.average_boost_in_leadup) }),
      el("td", { text: formatBoostAmount(player.min_boost_in_leadup) }),
      el("td", { text: player.is_most_back ? "Most back" : "--" }),
      el("td", { text: formatFieldPosition(player.position) }),
    );
    tbody.append(row);
  }
  table.append(thead, tbody);
  wrap.append(table);
  section.append(wrap);
  return section;
}

function renderGoalCard(
  finalFrame: StatsFrame,
  replayUrl: URL | null,
  goalIndex: number,
  context: GoalContextEvent | null,
  tags: GoalTag[],
): HTMLElement {
  const scoringTeamIsTeamZero = context?.scoring_team_is_team_0 ?? null;
  const scorer = context?.scorer ?? null;
  const time = context?.time ?? null;
  const frame = context?.frame ?? null;
  const watchRequest = getGoalWatchRequest(replayUrl, time, scorer);
  const card = el("section", { className: "stats-report-goal-card" });
  if (scoringTeamIsTeamZero !== null) {
    card.dataset.team = scoringTeamIsTeamZero ? "blue" : "orange";
  }

  const header = el("header");
  const heading = el("div", { className: "stats-report-goal-heading" });
  heading.append(
    el("h2", { text: `Goal ${goalIndex + 1}` }),
    el("span", {
      text: `${teamLabel(scoringTeamIsTeamZero)} - ${playerNameForId(finalFrame, scorer)} - ${formatTime(time)}`,
    }),
  );
  header.append(heading);
  if (watchRequest) {
    if (currentReportOptions.onWatchGoal) {
      const watchButton = el("button", {
        className: "stats-report-goal-watch",
        text: "Watch",
      });
      watchButton.type = "button";
      watchButton.addEventListener("click", () => {
        currentReportOptions.onWatchGoal?.(watchRequest);
      });
      header.append(watchButton);
    } else if (watchRequest.href) {
      const watchLink = el("a", {
        className: "stats-report-goal-watch",
        text: "Watch",
      });
      watchLink.setAttribute("href", watchRequest.href);
      watchLink.setAttribute("target", "_blank");
      watchLink.setAttribute("rel", "noreferrer");
      header.append(watchLink);
    }
  }
  card.append(header);
  card.append(renderGoalTagChips(tags));

  const detailItems = [
    { label: "Scoring team", value: teamLabel(scoringTeamIsTeamZero) },
    { label: "Scorer", value: playerNameForId(finalFrame, scorer) },
    { label: "Time", value: formatTime(time) },
    { label: "Frame", value: frame == null ? "--" : frame.toLocaleString() },
    {
      label: "Scorer last touch",
      value: context?.scorer_last_touch
        ? `${playerNameForId(finalFrame, context.scorer_last_touch.player)} at ${formatTime(context.scorer_last_touch.time)}`
        : "--",
    },
    {
      label: "Scoring most back",
      value: playerNameForId(finalFrame, context?.scoring_team_most_back_player),
    },
    {
      label: "Defending most back",
      value: playerNameForId(finalFrame, context?.defending_team_most_back_player),
    },
    { label: "Ball position", value: formatFieldPosition(context?.ball_position) },
    {
      label: "Last touch ball",
      value: formatFieldPosition(context?.scorer_last_touch?.ball_position),
    },
    {
      label: "Last touch player",
      value: formatFieldPosition(context?.scorer_last_touch?.player_position),
    },
  ];
  card.append(createDetailList(detailItems));

  const playerContext = renderGoalPlayerContextTable(finalFrame, context?.players ?? []);
  if (playerContext) card.append(playerContext);
  return card;
}

function renderGoalsPage(state: ReportState, finalFrame: StatsFrame): HTMLElement {
  const page = el("div", { className: "stats-report-page" });
  page.append(
    createPageIntro(
      "Goal metadata",
      "Goal-by-goal scorer, timing, context, tag confidence, and lead-up player state from the stats timeline event stream.",
    ),
  );

  const goalContexts = [...statsEventPayloads(state.statsTimeline, "goal_context")].sort(
    (left, right) => left.time - right.time,
  );
  const goalTags = goalContexts.flatMap((goal) => goal.tags ?? []);
  const scorerGoalTags = goalTags.filter(isScorerGoalTag);
  const assistGoalTags = goalTags.filter(isTeammatePerformedGoalTag);
  const goalIndexes = goalContexts.map((_, index) => index);
  const taggedGoalCount = goalContexts.filter((goal) => (goal.tags ?? []).length > 0).length;
  const scorerTagRows = getGoalTagCounts(scorerGoalTags);
  const assistTagRows = getGoalTagCounts(assistGoalTags);
  const topTag = scorerTagRows[0];

  page.append(
    renderMetricGrid([
      createSummaryCard("Goals found", goalIndexes.length.toLocaleString()),
      createSummaryCard("Tagged goals", taggedGoalCount.toLocaleString()),
      createSummaryCard("Scorer goal tags", scorerGoalTags.length.toLocaleString()),
      createSummaryCard("Assist goal tags", assistGoalTags.length.toLocaleString()),
      createSummaryCard("Top tag", topTag ? `${topTag.label} (${topTag.value})` : "--"),
    ]),
  );

  if (goalIndexes.length === 0) {
    page.append(
      el("section", {
        className: "stats-report-empty",
        text: "No goal metadata was emitted for this replay.",
      }),
    );
    return page;
  }

  const charts = el("section", { className: "stats-report-charts" });
  charts.append(
    renderChartCard(
      "Scorer goal tags by type",
      scorerTagRows.length > 0
        ? renderBarChartRows(scorerTagRows, (value) => value.toLocaleString())
        : el("p", { className: "stats-report-note", text: "No scorer goal tags emitted." }),
    ),
    renderChartCard(
      "Assist goal tags by type",
      assistTagRows.length > 0
        ? renderBarChartRows(assistTagRows, (value) => value.toLocaleString())
        : el("p", { className: "stats-report-note", text: "No assist goal tags emitted." }),
    ),
    renderChartCard(
      "Goal timing",
      renderBarChartRows(
        goalIndexes.map((goalIndex) => {
          const context = goalContexts[goalIndex] ?? null;
          const value = context?.time ?? 0;
          const scoringTeamIsTeamZero = context?.scoring_team_is_team_0 ?? true;
          return {
            label: `Goal ${goalIndex + 1}`,
            value,
            color: scoringTeamIsTeamZero ? TEAM_COLORS[0]! : TEAM_COLORS[1]!,
            formatted: formatTime(value),
          };
        }),
        formatTime,
      ),
    ),
  );
  page.append(charts);

  const list = el("div", { className: "stats-report-goal-list" });
  for (const goalIndex of goalIndexes) {
    list.append(
      renderGoalCard(
        finalFrame,
        state.replayUrl,
        goalIndex,
        goalContexts[goalIndex] ?? null,
        orderedGoalTags(goalContexts[goalIndex]?.tags),
      ),
    );
  }
  page.append(list);
  return page;
}

function renderBoostPage(finalFrame: StatsFrame, definitions: StatDefinition[]): HTMLElement {
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

function renderTerritoryPage(finalFrame: StatsFrame): HTMLElement {
  const page = el("div", { className: "stats-report-page" });
  page.append(
    createPageIntro(
      "Possession & territory",
      "Team control, field-half pressure, and where each player spent time relative to the field and the ball.",
    ),
  );

  const possessionTotal = finalFrame.team_zero.possession.tracked_time;
  const ballHalfTotal = finalFrame.team_zero.ball_half.tracked_time;
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
        formatShare(finalFrame.team_zero.ball_half.offensive_half_time, ballHalfTotal),
        "Time in Orange half",
      ),
      createSummaryCard(
        "Orange pressure",
        formatShare(finalFrame.team_zero.ball_half.defensive_half_time, ballHalfTotal),
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
      text: "Experimental event detectors such as musty flicks, speed flips, dodge refreshes, and ceiling shots are kept in All stats until their precision is stronger.",
    }),
  );
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
    main.append(renderGoalsPage(state, finalFrame));
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
