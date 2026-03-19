import type {
  ReplayModel,
  ReplayTimelineEvent,
  ReplayTimelineEventKind,
} from "subtr-actor-player";
import {
  buildFiftyFiftyMarkers,
} from "./fiftyFiftyOverlay.ts";
import type { StatsTimeline } from "./statsTimeline.ts";

const BLUE_TIMELINE_COLOR = "#3b82f6";
const ORANGE_TIMELINE_COLOR = "#f59e0b";
const NEUTRAL_TIMELINE_COLOR = "#d1d9e0";
const RUSH_MATCHUPS = [
  { suffix: "two_v_one_count", shortLabel: "2v1" },
  { suffix: "two_v_two_count", shortLabel: "2v2" },
  { suffix: "two_v_three_count", shortLabel: "2v3" },
  { suffix: "three_v_one_count", shortLabel: "3v1" },
  { suffix: "three_v_two_count", shortLabel: "3v2" },
  { suffix: "three_v_three_count", shortLabel: "3v3" },
] as const;

type RushTeamPrefix = "team_zero" | "team_one";

function getRushCount(
  rush: StatsTimeline["frames"][number]["rush"] | undefined,
  key: string,
): number {
  const value = rush?.[key as keyof NonNullable<StatsTimeline["frames"][number]["rush"]>];
  return typeof value === "number" && Number.isFinite(value) ? value : 0;
}

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

export function buildRushTimelineEvents(
  statsTimeline: StatsTimeline,
  replay?: ReplayModel,
): ReplayTimelineEvent[] {
  const events: ReplayTimelineEvent[] = [];
  const previousCounts = new Map<string, number>();

  for (const frame of statsTimeline.frames) {
    const eventTime = replay?.frames[frame.frame_number]?.time ?? frame.time;
    if (!Number.isFinite(eventTime)) {
      continue;
    }

    for (const [isTeamZero, teamPrefix, teamLabel, color] of [
      [true, "team_zero", "Blue", BLUE_TIMELINE_COLOR],
      [false, "team_one", "Orange", ORANGE_TIMELINE_COLOR],
    ] as const) {
      const totalKey = `${teamPrefix}_count`;
      const currentTotal = getRushCount(frame.rush, totalKey);
      const previousTotal = previousCounts.get(totalKey) ?? 0;
      const totalDelta = Math.max(0, currentTotal - previousTotal);
      previousCounts.set(totalKey, currentTotal);

      const matchupLabels: string[] = [];
      for (const matchup of RUSH_MATCHUPS) {
        const matchupKey = `${teamPrefix}_${matchup.suffix}`;
        const currentMatchupCount = getRushCount(frame.rush, matchupKey);
        const previousMatchupCount = previousCounts.get(matchupKey) ?? 0;
        const matchupDelta = Math.max(0, currentMatchupCount - previousMatchupCount);
        previousCounts.set(matchupKey, currentMatchupCount);
        for (let index = 0; index < matchupDelta; index += 1) {
          matchupLabels.push(matchup.shortLabel);
        }
      }

      for (let index = 0; index < totalDelta; index += 1) {
        const matchupLabel = matchupLabels[index];
        events.push({
          id: `rush:${frame.frame_number}:${teamPrefix}:${index}:${matchupLabel ?? "total"}`,
          time: eventTime,
          frame: frame.frame_number,
          kind: "rush",
          label: matchupLabel ? `${teamLabel} rush ${matchupLabel}` : `${teamLabel} rush`,
          shortLabel: matchupLabel ?? "R",
          isTeamZero,
          color,
        });
      }
    }
  }

  return events;
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

  if (active.has("rush")) {
    count += buildRushTimelineEvents(statsTimeline, replay).length;
  }

  return count;
}
