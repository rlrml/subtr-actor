import { getTeamClass } from "./statModules.ts";
import type { StatsTimeline } from "./statsTimeline.ts";
import { formatMechanicKind } from "./timelineMechanics.ts";
import { playerIdToString } from "./touchOverlay.ts";

interface GoalLabelsReplay {
  players: Array<{
    id: string;
    name: string;
  }>;
}

export interface GoalLabelsOverviewDeps {
  getStatsTimeline(): StatsTimeline | null;
  getReplay(): GoalLabelsReplay | null;
  formatTime(seconds: number): string;
  watchGoalReplay(time: number, scorerId: string | null): void;
  cueGoalReplay(time: number): void;
}

export function renderGoalLabelsOverview(body: HTMLElement, deps: GoalLabelsOverviewDeps): void {
  const timeline = deps.getStatsTimeline();
  const replay = deps.getReplay();
  if (!timeline || !replay) {
    appendStatsWindowEmpty(body, "Load a replay to show goal labels.");
    return;
  }

  const goalContexts = [...(timeline.events.goal_context ?? [])].sort(
    (left, right) => left.time - right.time,
  );
  const tagsByGoalIndex = new Map<number, typeof timeline.events.goal_tags>();
  for (const tag of timeline.events.goal_tags ?? []) {
    const group = tagsByGoalIndex.get(tag.goal_index) ?? [];
    group.push(tag);
    tagsByGoalIndex.set(tag.goal_index, group);
  }
  for (const group of tagsByGoalIndex.values()) {
    group.sort(
      (left, right) => left.kind.localeCompare(right.kind) || right.confidence - left.confidence,
    );
  }

  const goalIndexes = new Set<number>(goalContexts.map((_, index) => index));
  for (const index of tagsByGoalIndex.keys()) {
    goalIndexes.add(index);
  }
  const orderedGoalIndexes = [...goalIndexes].sort((left, right) => left - right);
  if (orderedGoalIndexes.length === 0) {
    appendStatsWindowEmpty(body, "No goals loaded.");
    return;
  }

  const list = document.createElement("div");
  list.className = "goal-label-list";
  for (const goalIndex of orderedGoalIndexes) {
    list.append(renderGoalLabelItem(goalIndex, goalContexts, tagsByGoalIndex, replay, deps));
  }
  body.append(list);
}

function renderGoalLabelItem(
  goalIndex: number,
  goalContexts: NonNullable<StatsTimeline["events"]["goal_context"]>,
  tagsByGoalIndex: Map<number, StatsTimeline["events"]["goal_tags"]>,
  replay: GoalLabelsReplay,
  deps: GoalLabelsOverviewDeps,
): HTMLElement {
  const context = goalContexts[goalIndex] ?? null;
  const tags = tagsByGoalIndex.get(goalIndex) ?? [];
  const firstTag = tags[0] ?? null;
  const time = context?.time ?? firstTag?.time ?? 0;
  const scorer = context?.scorer ?? firstTag?.scorer ?? null;
  const scorerId = scorer ? playerIdToString(scorer) : null;
  const scorerName = scorer
    ? (replay.players.find((player) => player.id === scorerId)?.name ?? scorerId)
    : "Unknown scorer";
  const isTeamZero = context?.scoring_team_is_team_0 ?? firstTag?.scoring_team_is_team_0 ?? null;

  const item = document.createElement("section");
  item.className = "goal-label-item";
  if (isTeamZero !== null) {
    item.classList.add(getTeamClass(isTeamZero));
  }

  const header = document.createElement("header");
  const title = document.createElement("h3");
  title.textContent = `Goal ${goalIndex + 1}`;
  const meta = document.createElement("span");
  meta.textContent = `${deps.formatTime(time)} · ${scorerName}`;
  header.append(title, meta);

  const labels = document.createElement("div");
  labels.className = "goal-label-tags";
  if (tags.length === 0) {
    const empty = document.createElement("span");
    empty.className = "goal-label-tag goal-label-tag-empty";
    empty.textContent = "Unlabeled";
    labels.append(empty);
  } else {
    for (const tag of tags) {
      const chip = document.createElement("span");
      chip.className = "goal-label-tag";
      chip.textContent = `${formatMechanicKind(tag.kind)} ${Math.round(tag.confidence * 100)}%`;
      labels.append(chip);
    }
  }

  const actions = document.createElement("div");
  actions.className = "goal-label-actions";
  const watch = document.createElement("button");
  watch.type = "button";
  watch.className = "goal-label-watch";
  watch.textContent = "Watch";
  watch.addEventListener("click", () => {
    deps.watchGoalReplay(time, scorerId);
  });
  const jump = document.createElement("button");
  jump.type = "button";
  jump.textContent = "Cue";
  jump.addEventListener("click", () => {
    deps.cueGoalReplay(time);
  });
  actions.append(watch, jump);

  item.append(header, labels, actions);
  return item;
}

function appendStatsWindowEmpty(body: HTMLElement, message: string): void {
  const empty = document.createElement("p");
  empty.className = "stat-window-empty";
  empty.textContent = message;
  body.append(empty);
}
