import type { TerritorialPressureEvent } from "./generated/TerritorialPressureEvent.ts";
import type { TerritorialPressureTeamStats } from "./generated/TerritorialPressureTeamStats.ts";
import type { StatsFrame, MaterializedStatsTimeline } from "./statsTimeline.ts";
import { statsEventPayloads } from "./statsTimeline.ts";

function f32(value: number): number {
  return Math.fround(value);
}

function addF32(left: number, right: number): number {
  return f32(f32(left) + f32(right));
}

function defaultTerritorialPressureTeamStats(): TerritorialPressureTeamStats {
  return {
    tracked_time: 0,
    session_count: 0,
    opponent_session_count: 0,
    session_time: 0,
    opponent_session_time: 0,
    offensive_half_time: 0,
    offensive_third_time: 0,
    longest_session_time: 0,
    opponent_longest_session_time: 0,
    average_session_time: 0,
  };
}

function sortTerritorialPressureEvents(
  events: readonly TerritorialPressureEvent[],
): TerritorialPressureEvent[] {
  return events
    .map((event, index) => ({ event, index }))
    .sort((left, right) => {
      if (left.event.end_frame !== right.event.end_frame) {
        return left.event.end_frame - right.event.end_frame;
      }
      if (left.event.end_time !== right.event.end_time) {
        return left.event.end_time - right.event.end_time;
      }
      return left.index - right.index;
    })
    .map(({ event }) => event);
}

function applyCompletedSession(
  own: TerritorialPressureTeamStats,
  opponent: TerritorialPressureTeamStats,
  event: TerritorialPressureEvent,
): void {
  own.session_count += 1;
  own.session_time = addF32(own.session_time, event.duration);
  own.offensive_half_time = addF32(own.offensive_half_time, event.offensive_half_time);
  own.offensive_third_time = addF32(own.offensive_third_time, event.offensive_third_time);
  own.longest_session_time = Math.max(own.longest_session_time, event.duration);
  own.average_session_time =
    own.session_count === 0 ? 0 : f32(own.session_time / own.session_count);

  opponent.opponent_session_count += 1;
  opponent.opponent_session_time = addF32(opponent.opponent_session_time, event.duration);
  opponent.opponent_longest_session_time = Math.max(
    opponent.opponent_longest_session_time,
    event.duration,
  );
}

function assignTerritorialPressureStats(
  target: TerritorialPressureTeamStats,
  source: TerritorialPressureTeamStats,
): void {
  Object.assign(target, source);
}

export function applyTerritorialPressureEventDerivedStats(
  timeline: MaterializedStatsTimeline,
): MaterializedStatsTimeline {
  const accumulator = createTerritorialPressureEventDerivedStatsAccumulator(timeline);

  for (const frame of timeline.frames) {
    accumulator.applyFrame(frame);
  }

  return timeline;
}

export function createTerritorialPressureEventDerivedStatsAccumulator(
  timeline: MaterializedStatsTimeline,
): { applyFrame(frame: StatsFrame): void } {
  const events = sortTerritorialPressureEvents(
    statsEventPayloads(timeline, "territorial_pressure"),
  );
  let eventIndex = 0;
  const teamZero = defaultTerritorialPressureTeamStats();
  const teamOne = defaultTerritorialPressureTeamStats();

  return {
    applyFrame(frame: StatsFrame): void {
      while (eventIndex < events.length && frame.frame_number >= events[eventIndex]!.end_frame) {
        const event = events[eventIndex] as TerritorialPressureEvent;
        applyCompletedSession(
          event.team_is_team_0 ? teamZero : teamOne,
          event.team_is_team_0 ? teamOne : teamZero,
          event,
        );
        eventIndex += 1;
      }

      assignTerritorialPressureStats(frame.team_zero.territorial_pressure, teamZero);
      assignTerritorialPressureStats(frame.team_one.territorial_pressure, teamOne);
    },
  };
}
