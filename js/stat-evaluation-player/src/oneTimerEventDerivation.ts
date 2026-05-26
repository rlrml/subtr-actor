import type { OneTimerEvent } from "./generated/OneTimerEvent.ts";
import type { OneTimerPlayerStats } from "./generated/OneTimerPlayerStats.ts";
import type { OneTimerTeamStats } from "./generated/OneTimerTeamStats.ts";
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

function defaultOneTimerPlayerStats(): OneTimerPlayerStats {
  return {
    count: 0,
    total_ball_speed: 0,
    fastest_ball_speed: 0,
    total_pass_distance: 0,
    is_last_one_timer: false,
    last_one_timer_time: null,
    last_one_timer_frame: null,
    time_since_last_one_timer: null,
    frames_since_last_one_timer: null,
  };
}

function sortOneTimerEvents(events: readonly OneTimerEvent[]): OneTimerEvent[] {
  return events
    .map((event, index) => ({ event, index }))
    .sort((left, right) => {
      if (left.event.frame !== right.event.frame) {
        return left.event.frame - right.event.frame;
      }
      if (left.event.time !== right.event.time) {
        return left.event.time - right.event.time;
      }
      return left.index - right.index;
    })
    .map(({ event }) => event);
}

function advanceOneTimerFrame(
  stats: OneTimerPlayerStats,
  frameNumber: number,
  frameTime: number,
  isLastOneTimerPlayer: boolean,
): void {
  stats.is_last_one_timer = isLastOneTimerPlayer;
  stats.time_since_last_one_timer =
    stats.last_one_timer_time == null ? null : Math.max(0, frameTime - stats.last_one_timer_time);
  stats.frames_since_last_one_timer =
    stats.last_one_timer_frame == null
      ? null
      : Math.max(0, frameNumber - stats.last_one_timer_frame);
}

function applyOneTimerEvent(
  stats: OneTimerPlayerStats,
  event: OneTimerEvent,
  frameNumber: number,
  frameTime: number,
): void {
  stats.count += 1;
  stats.total_ball_speed += event.ball_speed;
  stats.fastest_ball_speed = Math.max(stats.fastest_ball_speed, event.ball_speed);
  stats.total_pass_distance += event.pass_travel_distance;
  stats.last_one_timer_time = event.time;
  stats.last_one_timer_frame = event.frame;
  stats.time_since_last_one_timer = Math.max(0, frameTime - event.time);
  stats.frames_since_last_one_timer = Math.max(0, frameNumber - event.frame);
}

function assignOneTimerPlayerStats(
  target: OneTimerPlayerStats,
  source: OneTimerPlayerStats | undefined,
): void {
  Object.assign(target, source ?? defaultOneTimerPlayerStats());
}

function assignOneTimerTeamStats(target: OneTimerTeamStats, source: OneTimerTeamStats): void {
  Object.assign(target, source);
}

export function applyOneTimerEventDerivedStats(
  timeline: MaterializedStatsTimeline,
): MaterializedStatsTimeline {
  const accumulator = createOneTimerEventDerivedStatsAccumulator(timeline);

  for (const frame of timeline.frames) {
    accumulator.applyFrame(frame);
  }

  return timeline;
}

export function createOneTimerEventDerivedStatsAccumulator(timeline: MaterializedStatsTimeline): {
  applyFrame(frame: StatsFrame): void;
} {
  const events = sortOneTimerEvents(timeline.events.one_timer ?? []);

  let eventIndex = 0;
  let lastOneTimerPlayer: string | null = null;
  const players = new Map<string, OneTimerPlayerStats>();
  const teamZero: OneTimerTeamStats = { count: 0, total_ball_speed: 0, fastest_ball_speed: 0 };
  const teamOne: OneTimerTeamStats = { count: 0, total_ball_speed: 0, fastest_ball_speed: 0 };

  return {
    applyFrame(frame: StatsFrame): void {
      for (const [playerKey, stats] of players) {
        advanceOneTimerFrame(
          stats,
          frame.frame_number,
          frame.time,
          frame.is_live_play && playerKey === lastOneTimerPlayer,
        );
      }

      if (!frame.is_live_play) {
        lastOneTimerPlayer = null;
      } else {
        let processedEvent = false;
        while (eventIndex < events.length && events[eventIndex]!.frame <= frame.frame_number) {
          const event = events[eventIndex] as OneTimerEvent;
          const playerKey = remoteIdKey(event.player);
          const stats = players.get(playerKey) ?? defaultOneTimerPlayerStats();
          players.set(playerKey, stats);
          applyOneTimerEvent(stats, event, frame.frame_number, frame.time);

          const teamStats = event.is_team_0 ? teamZero : teamOne;
          teamStats.count += 1;
          teamStats.total_ball_speed += event.ball_speed;
          teamStats.fastest_ball_speed = Math.max(teamStats.fastest_ball_speed, event.ball_speed);

          lastOneTimerPlayer = playerKey;
          processedEvent = true;
          eventIndex += 1;
        }

        if (processedEvent) {
          for (const stats of players.values()) {
            stats.is_last_one_timer = false;
          }
        }
        if (lastOneTimerPlayer != null) {
          const stats = players.get(lastOneTimerPlayer);
          if (stats) {
            stats.is_last_one_timer = true;
          }
        }
      }

      assignOneTimerTeamStats(frame.team_zero.one_timer, teamZero);
      assignOneTimerTeamStats(frame.team_one.one_timer, teamOne);
      for (const player of frame.players) {
        assignOneTimerPlayerStats(player.one_timer, players.get(remoteIdKey(player.player_id)));
      }
    },
  };
}
