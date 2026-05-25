import type { HalfVolleyEvent } from "./generated/HalfVolleyEvent.ts";
import type { HalfVolleyPlayerStats } from "./generated/HalfVolleyPlayerStats.ts";
import type { HalfVolleyTeamStats } from "./generated/HalfVolleyTeamStats.ts";
import type { StatsTimeline } from "./statsTimeline.ts";

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

function defaultHalfVolleyPlayerStats(): HalfVolleyPlayerStats {
  return {
    count: 0,
    total_ball_speed: 0,
    fastest_ball_speed: 0,
    is_last_half_volley: false,
    last_half_volley_time: null,
    last_half_volley_frame: null,
    time_since_last_half_volley: null,
    frames_since_last_half_volley: null,
  };
}

function sortHalfVolleyEvents(events: readonly HalfVolleyEvent[]): HalfVolleyEvent[] {
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

function advanceHalfVolleyFrame(
  stats: HalfVolleyPlayerStats,
  frameNumber: number,
  frameTime: number,
  isLastHalfVolleyPlayer: boolean,
): void {
  stats.is_last_half_volley = isLastHalfVolleyPlayer;
  stats.time_since_last_half_volley =
    stats.last_half_volley_time == null
      ? null
      : Math.max(0, frameTime - stats.last_half_volley_time);
  stats.frames_since_last_half_volley =
    stats.last_half_volley_frame == null
      ? null
      : Math.max(0, frameNumber - stats.last_half_volley_frame);
}

function applyHalfVolleyEvent(
  stats: HalfVolleyPlayerStats,
  event: HalfVolleyEvent,
  frameNumber: number,
  frameTime: number,
): void {
  stats.count += 1;
  stats.total_ball_speed += event.ball_speed;
  stats.fastest_ball_speed = Math.max(stats.fastest_ball_speed, event.ball_speed);
  stats.last_half_volley_time = event.time;
  stats.last_half_volley_frame = event.frame;
  stats.time_since_last_half_volley = Math.max(0, frameTime - event.time);
  stats.frames_since_last_half_volley = Math.max(0, frameNumber - event.frame);
}

function assignHalfVolleyPlayerStats(
  target: HalfVolleyPlayerStats,
  source: HalfVolleyPlayerStats | undefined,
): void {
  Object.assign(target, source ?? defaultHalfVolleyPlayerStats());
}

function assignHalfVolleyTeamStats(
  target: HalfVolleyTeamStats,
  source: HalfVolleyTeamStats,
): void {
  Object.assign(target, source);
}

export function applyHalfVolleyEventDerivedStats(timeline: StatsTimeline): StatsTimeline {
  const events = sortHalfVolleyEvents(timeline.events.half_volley ?? []);

  let eventIndex = 0;
  let lastHalfVolleyPlayer: string | null = null;
  const players = new Map<string, HalfVolleyPlayerStats>();
  const teamZero: HalfVolleyTeamStats = { count: 0, total_ball_speed: 0, fastest_ball_speed: 0 };
  const teamOne: HalfVolleyTeamStats = { count: 0, total_ball_speed: 0, fastest_ball_speed: 0 };

  for (const frame of timeline.frames) {
    for (const [playerKey, stats] of players) {
      advanceHalfVolleyFrame(
        stats,
        frame.frame_number,
        frame.time,
        frame.is_live_play && playerKey === lastHalfVolleyPlayer,
      );
    }

    if (!frame.is_live_play) {
      lastHalfVolleyPlayer = null;
    } else {
      let processedEvent = false;
      while (eventIndex < events.length && events[eventIndex]!.frame <= frame.frame_number) {
        const event = events[eventIndex] as HalfVolleyEvent;
        const playerKey = remoteIdKey(event.player);
        const stats = players.get(playerKey) ?? defaultHalfVolleyPlayerStats();
        players.set(playerKey, stats);
        applyHalfVolleyEvent(stats, event, frame.frame_number, frame.time);

        const teamStats = event.is_team_0 ? teamZero : teamOne;
        teamStats.count += 1;
        teamStats.total_ball_speed += event.ball_speed;
        teamStats.fastest_ball_speed = Math.max(teamStats.fastest_ball_speed, event.ball_speed);

        lastHalfVolleyPlayer = playerKey;
        processedEvent = true;
        eventIndex += 1;
      }

      if (processedEvent) {
        for (const stats of players.values()) {
          stats.is_last_half_volley = false;
        }
      }
      if (lastHalfVolleyPlayer != null) {
        const stats = players.get(lastHalfVolleyPlayer);
        if (stats) {
          stats.is_last_half_volley = true;
        }
      }
    }

    assignHalfVolleyTeamStats(frame.team_zero.half_volley, teamZero);
    assignHalfVolleyTeamStats(frame.team_one.half_volley, teamOne);
    for (const player of frame.players) {
      assignHalfVolleyPlayerStats(
        player.half_volley,
        players.get(remoteIdKey(player.player_id)),
      );
    }
  }

  return timeline;
}
