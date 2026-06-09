import type { RushEvent } from "./generated/RushEvent.ts";
import type { RushTeamStats } from "./generated/RushTeamStats.ts";
import type { StatsFrame, MaterializedStatsTimeline } from "./statsTimeline.ts";
import { statsEventPayloads } from "./statsTimeline.ts";

function defaultRushTeamStats(): RushTeamStats {
  return {
    count: 0,
    two_v_one_count: 0,
    two_v_two_count: 0,
    two_v_three_count: 0,
    three_v_one_count: 0,
    three_v_two_count: 0,
    three_v_three_count: 0,
  };
}

function sortRushEvents(events: readonly RushEvent[]): RushEvent[] {
  return events
    .map((event, index) => ({ event, index }))
    .sort((left, right) => {
      if (left.event.start_frame !== right.event.start_frame) {
        return left.event.start_frame - right.event.start_frame;
      }
      if (left.event.start_time !== right.event.start_time) {
        return left.event.start_time - right.event.start_time;
      }
      if (left.event.end_frame !== right.event.end_frame) {
        return left.event.end_frame - right.event.end_frame;
      }
      return left.index - right.index;
    })
    .map(({ event }) => event);
}

function applyRushEvent(stats: RushTeamStats, event: RushEvent): void {
  stats.count += 1;
  if (event.attackers === 2 && event.defenders === 1) {
    stats.two_v_one_count += 1;
  } else if (event.attackers === 2 && event.defenders === 2) {
    stats.two_v_two_count += 1;
  } else if (event.attackers === 2 && event.defenders === 3) {
    stats.two_v_three_count += 1;
  } else if (event.attackers === 3 && event.defenders === 1) {
    stats.three_v_one_count += 1;
  } else if (event.attackers === 3 && event.defenders === 2) {
    stats.three_v_two_count += 1;
  } else if (event.attackers === 3 && event.defenders === 3) {
    stats.three_v_three_count += 1;
  }
}

function assignRushTeamStats(target: RushTeamStats, source: RushTeamStats): void {
  Object.assign(target, source);
}

export function applyRushEventDerivedStats(
  timeline: MaterializedStatsTimeline,
): MaterializedStatsTimeline {
  const accumulator = createRushEventDerivedStatsAccumulator(timeline);

  for (const frame of timeline.frames) {
    accumulator.applyFrame(frame);
  }

  return timeline;
}

export function createRushEventDerivedStatsAccumulator(timeline: MaterializedStatsTimeline): {
  applyFrame(frame: StatsFrame): void;
} {
  const events = sortRushEvents(statsEventPayloads(timeline, "rush"));

  let eventIndex = 0;
  const teamZero = defaultRushTeamStats();
  const teamOne = defaultRushTeamStats();
  const minRetainedSeconds = timeline.config.rush_min_possession_retained_seconds;

  return {
    applyFrame(frame: StatsFrame): void {
      while (
        eventIndex < events.length &&
        frame.frame_number >= events[eventIndex]!.start_frame &&
        frame.time - events[eventIndex]!.start_time >= minRetainedSeconds
      ) {
        const event = events[eventIndex] as RushEvent;
        applyRushEvent(event.is_team_0 ? teamZero : teamOne, event);
        eventIndex += 1;
      }

      assignRushTeamStats(frame.team_zero.rush, teamZero);
      assignRushTeamStats(frame.team_one.rush, teamOne);
    },
  };
}
