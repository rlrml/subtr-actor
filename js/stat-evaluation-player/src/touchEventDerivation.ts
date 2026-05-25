import type { StatLabel } from "./generated/StatLabel.ts";
import type { TouchBallMovementEvent } from "./generated/TouchBallMovementEvent.ts";
import type { TouchLastTouchEvent } from "./generated/TouchLastTouchEvent.ts";
import type { TouchStats } from "./generated/TouchStats.ts";
import type { TouchStatsEvent } from "./generated/TouchStatsEvent.ts";
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

function defaultTouchStats(): TouchStats {
  return {
    touch_count: 0,
    control_touch_count: 0,
    medium_hit_count: 0,
    hard_hit_count: 0,
    aerial_touch_count: 0,
    high_aerial_touch_count: 0,
    wall_touch_count: 0,
    is_last_touch: false,
    last_touch_time: null,
    last_touch_frame: null,
    time_since_last_touch: null,
    frames_since_last_touch: null,
    last_ball_speed_change: null,
    max_ball_speed_change: 0,
    cumulative_ball_speed_change: 0,
    total_ball_travel_distance: 0,
    total_ball_advance_distance: 0,
    total_ball_retreat_distance: 0,
    labeled_touch_counts: { entries: [] },
  };
}

function sortBySample<T extends { time: number; frame: number; sample_time?: number; sample_frame?: number }>(
  events: readonly T[],
): T[] {
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

function sortByFrame<T extends { time: number; frame: number }>(events: readonly T[]): T[] {
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

function incrementLabeledTouchCount(stats: TouchStats, labels: StatLabel[]): void {
  labels.sort((left, right) =>
    left.key === right.key ? left.value.localeCompare(right.value) : left.key.localeCompare(right.key),
  );
  const entries = stats.labeled_touch_counts?.entries ?? [];
  stats.labeled_touch_counts = { entries };
  const entry = entries.find(
    (candidate) =>
      candidate.labels.length === labels.length &&
      candidate.labels.every(
        (label, index) => label.key === labels[index]?.key && label.value === labels[index]?.value,
      ),
  );
  if (entry) {
    entry.count += 1;
  } else {
    entries.push({ labels, count: 1 });
    entries.sort((left, right) => JSON.stringify(left.labels).localeCompare(JSON.stringify(right.labels)));
  }
}

function applyTouchStatsEvent(
  stats: TouchStats,
  event: TouchStatsEvent,
  frame: StatsTimeline["frames"][number],
): void {
  stats.touch_count += 1;
  if (event.kind === "control") {
    stats.control_touch_count += 1;
  } else if (event.kind === "medium_hit") {
    stats.medium_hit_count += 1;
  } else if (event.kind === "hard_hit") {
    stats.hard_hit_count += 1;
  }

  if (event.height_band === "low_air") {
    stats.aerial_touch_count += 1;
  } else if (event.height_band === "high_air") {
    stats.aerial_touch_count += 1;
    stats.high_aerial_touch_count += 1;
  }
  if (event.surface === "wall") {
    stats.wall_touch_count += 1;
  }

  incrementLabeledTouchCount(stats, [
    { key: "kind", value: event.kind },
    { key: "height_band", value: event.height_band },
    { key: "surface", value: event.surface },
    { key: "dodge_state", value: event.dodge_state },
  ]);
  stats.last_touch_time = event.time;
  stats.last_touch_frame = event.frame;
  stats.time_since_last_touch = Math.max(0, frame.time - event.time);
  stats.frames_since_last_touch = Math.max(0, frame.frame_number - event.frame);
  stats.last_ball_speed_change = event.ball_speed_change;
  stats.max_ball_speed_change = Math.max(stats.max_ball_speed_change, event.ball_speed_change);
  stats.cumulative_ball_speed_change += event.ball_speed_change;
}

function assignTouchStats(target: TouchStats, source: TouchStats | undefined): void {
  Object.assign(target, source ?? defaultTouchStats());
}

export function applyTouchEventDerivedStats(timeline: StatsTimeline): StatsTimeline {
  const touchEvents = sortBySample(timeline.events.touch ?? []);
  const lastTouchEvents = sortBySample(timeline.events.touch_last_touch ?? []);
  const movementEvents = sortByFrame(timeline.events.touch_ball_movement ?? []);

  let touchEventIndex = 0;
  let lastTouchEventIndex = 0;
  let movementEventIndex = 0;
  let currentLastTouchPlayerKey: string | null = null;
  const players = new Map<string, TouchStats>();

  for (const frame of timeline.frames) {
    if (!frame.is_live_play) {
      currentLastTouchPlayerKey = null;
    } else {
      for (const stats of players.values()) {
        stats.is_last_touch = false;
        if (stats.last_touch_time != null) {
          stats.time_since_last_touch = Math.max(0, frame.time - stats.last_touch_time);
        }
        if (stats.last_touch_frame != null) {
          stats.frames_since_last_touch = Math.max(0, frame.frame_number - stats.last_touch_frame);
        }
      }

      while (
        touchEventIndex < touchEvents.length &&
        (touchEvents[touchEventIndex]!.sample_frame ?? touchEvents[touchEventIndex]!.frame) <=
          frame.frame_number
      ) {
        const event = touchEvents[touchEventIndex] as TouchStatsEvent;
        const playerKey = remoteIdKey(event.player);
        const stats = players.get(playerKey) ?? defaultTouchStats();
        players.set(playerKey, stats);
        applyTouchStatsEvent(stats, event, frame);
        touchEventIndex += 1;
      }

      while (
        lastTouchEventIndex < lastTouchEvents.length &&
        (lastTouchEvents[lastTouchEventIndex]!.sample_frame ??
          lastTouchEvents[lastTouchEventIndex]!.frame) <= frame.frame_number
      ) {
        const event = lastTouchEvents[lastTouchEventIndex] as TouchLastTouchEvent;
        currentLastTouchPlayerKey = event.player == null ? null : remoteIdKey(event.player);
        lastTouchEventIndex += 1;
      }

      if (currentLastTouchPlayerKey != null) {
        const stats = players.get(currentLastTouchPlayerKey);
        if (stats) {
          stats.is_last_touch = true;
        }
      }
    }

    while (
      movementEventIndex < movementEvents.length &&
      movementEvents[movementEventIndex]!.frame <= frame.frame_number
    ) {
      const event = movementEvents[movementEventIndex] as TouchBallMovementEvent;
      const playerKey = remoteIdKey(event.player);
      const stats = players.get(playerKey) ?? defaultTouchStats();
      players.set(playerKey, stats);
      stats.total_ball_travel_distance += event.travel_distance;
      stats.total_ball_advance_distance += event.advance_distance;
      stats.total_ball_retreat_distance += event.retreat_distance;
      movementEventIndex += 1;
    }

    for (const player of frame.players) {
      assignTouchStats(player.touch, players.get(remoteIdKey(player.player_id)));
    }
  }

  return timeline;
}
