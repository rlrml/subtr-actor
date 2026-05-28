import {
  setStatsPlayerConfigOnUrl,
  STATS_PLAYER_CONFIG_VERSION,
  type StatsPlayerConfig,
} from "./playerConfig.ts";
import { el } from "./reportDom.ts";
import { createPageIntro, createSummaryCard } from "./reportLayout.ts";
import { formatBoostAmount, formatFieldPosition, formatTime } from "./reportFormat.ts";
import {
  CHART_COLORS,
  TEAM_COLORS,
  renderBarChartRows,
  renderChartCard,
  renderMetricGrid,
  type NumberRow,
} from "./reportCharts.ts";
import type { StatsFrame, StatsTimeline } from "./statsTimeline.ts";
import { formatMechanicKind } from "./timelineMarkers.ts";
import { playerIdToString } from "./touchOverlay.ts";

type GoalContextEvent = StatsTimeline["events"]["goal_context"][number];
type GoalTagEvent = StatsTimeline["events"]["goal_tags"][number];
type GoalPlayerContext = GoalContextEvent["players"][number];

export interface StatsReportGoalPageState {
  replayUrl: URL | null;
  statsTimeline: StatsTimeline;
}

export interface StatsReportGoalWatchRequest {
  config: StatsPlayerConfig;
  href: string | null;
  goalTime: number;
  playerId: string | null;
}

export type StatsReportGoalWatchHandler = (request: StatsReportGoalWatchRequest) => void;

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
  onWatchGoal: StatsReportGoalWatchHandler | undefined,
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
    if (onWatchGoal) {
      const watchButton = el("button", {
        className: "stats-report-goal-watch",
        text: "Watch",
      });
      watchButton.type = "button";
      watchButton.addEventListener("click", () => {
        onWatchGoal(watchRequest);
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

export function renderGoalsPage(
  state: StatsReportGoalPageState,
  finalFrame: StatsFrame,
  onWatchGoal?: StatsReportGoalWatchHandler,
): HTMLElement {
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
        onWatchGoal,
      ),
    );
  }
  page.append(list);
  return page;
}
