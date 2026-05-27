import "./report.css";
import { toBoostDisplayUnits } from "./boostFormatting.ts";
import { formatReplayLoadProgress, loadReplayBundleInWorker } from "./replayLoader.ts";
import { createStatRegistry, type StatDefinition, type StatScopeKind } from "./statRegistry.ts";
import { formatMechanicKind } from "./timelineMarkers.ts";
import {
  setStatsPlayerConfigOnUrl,
  STATS_PLAYER_CONFIG_VERSION,
  type StatsPlayerConfig,
} from "./playerConfig.ts";
import { playerIdToString } from "./touchOverlay.ts";
import { createStatsFrameLookup } from "./statsTimeline.ts";
import { el } from "./reportDom.ts";
import {
  formatBoostAmount,
  formatFieldPosition,
  formatPercent,
  formatSeconds,
  formatShare,
  formatTime,
} from "./reportFormat.ts";
import {
  CHART_COLORS,
  TEAM_COLORS,
  renderBarChartRows,
  renderChartCard,
  renderCharts,
  renderDefinitionChartCard,
  renderMetricGrid,
  renderPieChartRows,
  renderStackedRows,
  renderTerritoryShareChart,
  getPlayerTeamColor,
  type ChartSpec,
  type NumberRow,
  type StackedRow,
} from "./reportCharts.ts";
import type {
  PlayerStatsSnapshot,
  StatsFrame,
  StatsFrameLookup,
  StatsTimeline,
  TeamStatsSnapshot,
} from "./statsTimeline.ts";

type StatsTarget = PlayerStatsSnapshot | TeamStatsSnapshot;
type ReportPageId = "overview" | "goals" | "boost" | "territory" | "involvement" | "dump";
type GoalContextEvent = StatsTimeline["events"]["goal_context"][number];
type GoalTagEvent = StatsTimeline["events"]["goal_tags"][number];
type GoalPlayerContext = GoalContextEvent["players"][number];
type GoalContextPosition = NonNullable<GoalContextEvent["ball_position"]>;

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

function remoteIdKey(playerId: Record<string, unknown> | null | undefined): string | null {
  return playerId ? playerIdToString(playerId) : null;
}

function playerNameForId(
  finalFrame: StatsFrame,
  playerId: Record<string, unknown> | null | undefined,
): string {
  const key = remoteIdKey(playerId);
  if (!key) return "--";
  return finalFrame.players.find((player) => remoteIdKey(player.player_id) === key)?.name ?? key;
}

function teamLabel(isTeamZero: boolean | null | undefined): string {
  if (isTeamZero === true) return "Blue";
  if (isTeamZero === false) return "Orange";
  return "--";
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

function getPlayerUrlForGoal(
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

function getGoalWatchRequest(
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

function getGoalWatchPlayerConfig(goalTime: number, scorerId: string | null): StatsPlayerConfig {
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
    },
    recording: {},
    singletonWindows: [],
    statsWindows: [],
    moduleConfigs: {},
  };
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

function groupGoalTagsByGoalIndex(goalTags: GoalTagEvent[]): Map<number, GoalTagEvent[]> {
  const groups = new Map<number, GoalTagEvent[]>();
  for (const tag of goalTags) {
    const group = groups.get(tag.goal_index) ?? [];
    group.push(tag);
    groups.set(tag.goal_index, group);
  }
  for (const group of groups.values()) {
    group.sort(
      (left, right) => left.kind.localeCompare(right.kind) || right.confidence - left.confidence,
    );
  }
  return groups;
}

function getOrderedGoalIndexes(
  goalContexts: GoalContextEvent[],
  tagsByGoalIndex: Map<number, GoalTagEvent[]>,
): number[] {
  const goalIndexes = new Set<number>(goalContexts.map((_, index) => index));
  for (const goalIndex of tagsByGoalIndex.keys()) {
    goalIndexes.add(goalIndex);
  }
  return [...goalIndexes].sort((left, right) => left - right);
}

function getGoalTagCounts(goalTags: GoalTagEvent[]): NumberRow[] {
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

function renderGoalTagChips(tags: GoalTagEvent[]): HTMLElement {
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
    const modifiers =
      tag.modifiers.length > 0 ? ` - ${tag.modifiers.map(formatMechanicKind).join(", ")}` : "";
    list.append(
      el("span", {
        className: "stats-report-goal-tag",
        text: `${formatMechanicKind(tag.kind)} ${Math.round(tag.confidence * 100)}%${modifiers}`,
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
  tags: GoalTagEvent[],
): HTMLElement {
  const firstTag = tags[0] ?? null;
  const scoringTeamIsTeamZero =
    context?.scoring_team_is_team_0 ?? firstTag?.scoring_team_is_team_0 ?? null;
  const scorer = context?.scorer ?? firstTag?.scorer ?? null;
  const time = context?.time ?? firstTag?.time ?? null;
  const frame = context?.frame ?? firstTag?.frame ?? null;
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

  const goalContexts = [...(state.statsTimeline.events.goal_context ?? [])].sort(
    (left, right) => left.time - right.time,
  );
  const goalTags = [...(state.statsTimeline.events.goal_tags ?? [])];
  const tagsByGoalIndex = groupGoalTagsByGoalIndex(goalTags);
  const goalIndexes = getOrderedGoalIndexes(goalContexts, tagsByGoalIndex);
  const taggedGoalCount = [...tagsByGoalIndex.values()].filter((tags) => tags.length > 0).length;
  const tagRows = getGoalTagCounts(goalTags);
  const topTag = tagRows[0];

  page.append(
    renderMetricGrid([
      createSummaryCard("Goals found", goalIndexes.length.toLocaleString()),
      createSummaryCard("Tagged goals", taggedGoalCount.toLocaleString()),
      createSummaryCard("Goal tags", goalTags.length.toLocaleString()),
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
      "Goal tags by type",
      tagRows.length > 0
        ? renderBarChartRows(tagRows, (value) => value.toLocaleString())
        : el("p", { className: "stats-report-note", text: "No goal tags emitted." }),
    ),
    renderChartCard(
      "Goal timing",
      renderBarChartRows(
        goalIndexes.map((goalIndex) => {
          const context = goalContexts[goalIndex] ?? null;
          const firstTag = tagsByGoalIndex.get(goalIndex)?.[0] ?? null;
          const value = context?.time ?? firstTag?.time ?? 0;
          const scoringTeamIsTeamZero =
            context?.scoring_team_is_team_0 ?? firstTag?.scoring_team_is_team_0 ?? true;
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
        tagsByGoalIndex.get(goalIndex) ?? [],
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
