import type { PassEvent } from "./generated/PassEvent.ts";
import type { PassPlayerStats } from "./generated/PassPlayerStats.ts";
import type { PassTeamStats } from "./generated/PassTeamStats.ts";
import type { StatsFrame, MaterializedStatsTimeline } from "./statsTimeline.ts";

function remoteIdKey(playerId: unknown): string {
  if (!playerId || typeof playerId !== "object") {
    return String(playerId);
  }
  const [kind, value] = Object.entries(playerId as Record<string, unknown>)[0] ?? [
    "Unknown",
    "unknown",
  ];
  return `${kind}:${typeof value === "string" ? value : JSON.stringify(value)}`;
}

function defaultPassPlayerStats(): PassPlayerStats {
  return {
    completed_pass_count: 0,
    received_pass_count: 0,
    total_pass_distance: 0,
    total_pass_advance: 0,
    longest_pass_distance: 0,
    is_last_completed_pass: false,
    last_completed_pass_time: null,
    last_completed_pass_frame: null,
    time_since_last_completed_pass: null,
    frames_since_last_completed_pass: null,
  };
}

function sortPassEvents(events: readonly PassEvent[]): PassEvent[] {
  return events
    .map((event, index) => ({ event, index }))
    .sort((left, right) => {
      const leftSampleFrame = left.event.sample_frame ?? left.event.frame;
      const rightSampleFrame = right.event.sample_frame ?? right.event.frame;
      if (leftSampleFrame !== rightSampleFrame) {
        return leftSampleFrame - rightSampleFrame;
      }
      const leftSampleTime = left.event.sample_time ?? left.event.time;
      const rightSampleTime = right.event.sample_time ?? right.event.time;
      if (leftSampleTime !== rightSampleTime) {
        return leftSampleTime - rightSampleTime;
      }
      return left.index - right.index;
    })
    .map(({ event }) => event);
}

function advancePassFrame(
  stats: PassPlayerStats,
  frameNumber: number,
  frameTime: number,
  isLastCompletedPassPlayer: boolean,
): void {
  stats.is_last_completed_pass = isLastCompletedPassPlayer;
  stats.time_since_last_completed_pass =
    stats.last_completed_pass_time == null
      ? null
      : Math.max(0, frameTime - stats.last_completed_pass_time);
  stats.frames_since_last_completed_pass =
    stats.last_completed_pass_frame == null
      ? null
      : Math.max(0, frameNumber - stats.last_completed_pass_frame);
}

function applyCompletedPassEvent(
  stats: PassPlayerStats,
  event: PassEvent,
  frameNumber: number,
  frameTime: number,
): void {
  stats.completed_pass_count += 1;
  stats.total_pass_distance += event.ball_travel_distance;
  stats.total_pass_advance += event.ball_advance_distance;
  stats.longest_pass_distance = Math.max(stats.longest_pass_distance, event.ball_travel_distance);
  stats.last_completed_pass_time = event.time;
  stats.last_completed_pass_frame = event.frame;
  stats.time_since_last_completed_pass = Math.max(0, frameTime - event.time);
  stats.frames_since_last_completed_pass = Math.max(0, frameNumber - event.frame);
}

function assignPassPlayerStats(target: PassPlayerStats, source: PassPlayerStats | undefined): void {
  Object.assign(target, source ?? defaultPassPlayerStats());
}

function assignPassTeamStats(target: PassTeamStats, source: PassTeamStats): void {
  Object.assign(target, source);
}

export function applyPassEventDerivedStats(
  timeline: MaterializedStatsTimeline,
): MaterializedStatsTimeline {
  const accumulator = createPassEventDerivedStatsAccumulator(timeline);

  for (const frame of timeline.frames) {
    accumulator.applyFrame(frame);
  }

  return timeline;
}

export function createPassEventDerivedStatsAccumulator(timeline: MaterializedStatsTimeline): {
  applyFrame(frame: StatsFrame): void;
} {
  const events = sortPassEvents(timeline.events.pass ?? []);

  let eventIndex = 0;
  let lastCompletedPassPlayer: string | null = null;
  const players = new Map<string, PassPlayerStats>();
  const teamZero: PassTeamStats = {
    completed_pass_count: 0,
    total_pass_distance: 0,
    total_pass_advance: 0,
    longest_pass_distance: 0,
  };
  const teamOne: PassTeamStats = {
    completed_pass_count: 0,
    total_pass_distance: 0,
    total_pass_advance: 0,
    longest_pass_distance: 0,
  };

  return {
    applyFrame(frame: StatsFrame): void {
      for (const [playerKey, stats] of players) {
        advancePassFrame(
          stats,
          frame.frame_number,
          frame.time,
          frame.is_live_play && playerKey === lastCompletedPassPlayer,
        );
      }

      if (!frame.is_live_play) {
        lastCompletedPassPlayer = null;
      } else {
        let processedEvent = false;
        while (
          eventIndex < events.length &&
          (events[eventIndex]!.sample_frame ?? events[eventIndex]!.frame) <= frame.frame_number
        ) {
          const event = events[eventIndex] as PassEvent;
          const passerKey = remoteIdKey(event.passer);
          const passerStats = players.get(passerKey) ?? defaultPassPlayerStats();
          players.set(passerKey, passerStats);
          applyCompletedPassEvent(passerStats, event, frame.frame_number, frame.time);

          const receiverKey = remoteIdKey(event.receiver);
          const receiverStats = players.get(receiverKey) ?? defaultPassPlayerStats();
          players.set(receiverKey, receiverStats);
          receiverStats.received_pass_count += 1;

          const teamStats = event.is_team_0 ? teamZero : teamOne;
          teamStats.completed_pass_count += 1;
          teamStats.total_pass_distance += event.ball_travel_distance;
          teamStats.total_pass_advance += event.ball_advance_distance;
          teamStats.longest_pass_distance = Math.max(
            teamStats.longest_pass_distance,
            event.ball_travel_distance,
          );

          lastCompletedPassPlayer = passerKey;
          processedEvent = true;
          eventIndex += 1;
        }

        if (processedEvent) {
          for (const stats of players.values()) {
            stats.is_last_completed_pass = false;
          }
        }
        if (lastCompletedPassPlayer != null) {
          const lastStats = players.get(lastCompletedPassPlayer);
          if (lastStats) {
            lastStats.is_last_completed_pass = true;
          }
        }
      }

      assignPassTeamStats(frame.team_zero.pass, teamZero);
      assignPassTeamStats(frame.team_one.pass, teamOne);
      for (const player of frame.players) {
        assignPassPlayerStats(player.pass, players.get(remoteIdKey(player.player_id)));
      }
    },
  };
}
