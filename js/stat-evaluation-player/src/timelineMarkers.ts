import type {
  ReplayModel,
  ReplayTimelineEvent,
  ReplayTimelineEventKind,
} from "../../player/src/types.ts";
import {
  buildFiftyFiftyMarkers,
} from "./fiftyFiftyOverlay.ts";
import type { StatsTimeline } from "./statsTimeline.ts";

const BLUE_TIMELINE_COLOR = "#3b82f6";
const ORANGE_TIMELINE_COLOR = "#f59e0b";
const NEUTRAL_TIMELINE_COLOR = "#d1d9e0";

export function getReplayTimelineEventKinds(
  activeModuleIds: Iterable<string>,
): ReplayTimelineEventKind[] {
  const active = new Set(activeModuleIds);
  const allowedKinds = new Set<ReplayTimelineEventKind>(["goal"]);

  if (active.has("core")) {
    allowedKinds.add("save");
    allowedKinds.add("shot");
  }

  if (active.has("demo")) {
    allowedKinds.add("demo");
  }

  return [...allowedKinds];
}

export function filterReplayTimelineEvents(
  replay: ReplayModel,
  activeModuleIds: Iterable<string>,
): ReplayTimelineEvent[] {
  const allowedKinds = new Set(getReplayTimelineEventKinds(activeModuleIds));
  return replay.timelineEvents.filter((event) => allowedKinds.has(event.kind));
}

export function buildFiftyFiftyTimelineEvents(
  statsTimeline: StatsTimeline,
  replay: ReplayModel,
): ReplayTimelineEvent[] {
  return buildFiftyFiftyMarkers(statsTimeline, replay).map((marker) => ({
    id: marker.id,
    time: marker.time,
    kind: "fifty-fifty",
    label: marker.label,
    shortLabel: marker.label.startsWith("Kickoff 50/50") ? "KO" : "50",
    isTeamZero: marker.winnerIsTeamZero,
    color: marker.winnerIsTeamZero === null
      ? NEUTRAL_TIMELINE_COLOR
      : marker.winnerIsTeamZero
        ? BLUE_TIMELINE_COLOR
        : ORANGE_TIMELINE_COLOR,
  }));
}

export function countEnabledTimelineEvents(
  activeModuleIds: Iterable<string>,
  replay: ReplayModel,
  statsTimeline: StatsTimeline,
): number {
  const active = new Set(activeModuleIds);
  let count = filterReplayTimelineEvents(replay, active).length;

  if (active.has("fifty-fifty")) {
    count += buildFiftyFiftyTimelineEvents(statsTimeline, replay).length;
  }

  return count;
}
