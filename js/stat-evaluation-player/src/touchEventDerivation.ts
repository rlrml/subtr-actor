import type { StatLabel } from "./generated/StatLabel.ts";
import type { TouchBallMovement } from "./generated/TouchBallMovement.ts";
import type { TouchStats } from "./generated/TouchStats.ts";
import type { TouchClassificationEvent } from "./generated/TouchClassificationEvent.ts";
import type { StatsFrame, MaterializedStatsTimeline } from "./statsTimeline.ts";
import { statsEventPayloads } from "./statsTimeline.ts";

const TOUCH_KINDS = ["control", "hard_hit", "medium_hit"] as const;
const TOUCH_HEIGHT_BANDS = ["ground", "high_air", "low_air"] as const;
const TOUCH_SURFACES = ["air", "ground", "wall"] as const;
const TOUCH_DODGE_STATES = ["dodge", "no_dodge"] as const;

interface TouchAccumulator {
  stats: TouchStats;
  labeledCountsVersion: number;
  labeledCountsSnapshot: TouchStats["labeled_touch_counts"];
  labeledCountsSnapshotVersion: number;
}

interface TouchMovementCredit {
  player: TouchClassificationEvent["player"];
  movement: TouchBallMovement;
}

function f32(value: number): number {
  return Math.fround(value);
}

function addF32(left: number, right: number): number {
  return f32(f32(left) + f32(right));
}

function subF32(left: number, right: number): number {
  return f32(f32(left) - f32(right));
}

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

function emptyLabeledTouchCounts(): TouchStats["labeled_touch_counts"] {
  return {
    entries: TOUCH_DODGE_STATES.flatMap((dodgeState) =>
      TOUCH_HEIGHT_BANDS.flatMap((heightBand) =>
        TOUCH_KINDS.flatMap((kind) =>
          TOUCH_SURFACES.map((surface) => ({
            labels: [
              { key: "dodge_state", value: dodgeState },
              { key: "height_band", value: heightBand },
              { key: "kind", value: kind },
              { key: "surface", value: surface },
            ],
            count: 0,
          })),
        ),
      ),
    ).sort((left, right) =>
      JSON.stringify(left.labels).localeCompare(JSON.stringify(right.labels)),
    ),
  };
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
    first_touch_count: 0,
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
    labeled_touch_counts: emptyLabeledTouchCounts(),
  };
}

const DEFAULT_TOUCH_STATS = defaultTouchStats();

function createTouchAccumulator(): TouchAccumulator {
  return {
    stats: defaultTouchStats(),
    labeledCountsVersion: 0,
    labeledCountsSnapshot: undefined,
    labeledCountsSnapshotVersion: -1,
  };
}

function sortBySample<
  T extends { time: number; frame: number; sample_time?: number; sample_frame?: number },
>(events: readonly T[]): T[] {
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

function incrementLabeledTouchCount(stats: TouchStats, labels: StatLabel[]): void {
  labels.sort((left, right) =>
    left.key === right.key
      ? left.value.localeCompare(right.value)
      : left.key.localeCompare(right.key),
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
    entries.sort((left, right) =>
      JSON.stringify(left.labels).localeCompare(JSON.stringify(right.labels)),
    );
  }
}

function cloneLabeledTouchCounts(
  source: NonNullable<TouchStats["labeled_touch_counts"]>,
): NonNullable<TouchStats["labeled_touch_counts"]> {
  return {
    entries: source.entries.map((entry) => ({
      labels: entry.labels.map((label) => ({ ...label })),
      count: entry.count,
    })),
  };
}

function applyTouchClassificationEvent(
  accumulator: TouchAccumulator,
  event: TouchClassificationEvent,
  frame: StatsFrame,
): void {
  const stats = accumulator.stats;
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
  accumulator.labeledCountsVersion += 1;
  stats.last_touch_time = event.time;
  stats.last_touch_frame = event.frame;
  stats.time_since_last_touch = Math.max(0, subF32(frame.time, event.time));
  stats.frames_since_last_touch = Math.max(0, frame.frame_number - event.frame);
  stats.last_ball_speed_change = event.ball_speed_change;
  stats.max_ball_speed_change = Math.max(stats.max_ball_speed_change, event.ball_speed_change);
  stats.cumulative_ball_speed_change = addF32(
    stats.cumulative_ball_speed_change,
    event.ball_speed_change,
  );
}

function getLabeledTouchCountsSnapshot(
  accumulator: TouchAccumulator,
): TouchStats["labeled_touch_counts"] {
  if (accumulator.labeledCountsSnapshotVersion !== accumulator.labeledCountsVersion) {
    accumulator.labeledCountsSnapshot = accumulator.stats.labeled_touch_counts
      ? cloneLabeledTouchCounts(accumulator.stats.labeled_touch_counts)
      : undefined;
    accumulator.labeledCountsSnapshotVersion = accumulator.labeledCountsVersion;
  }
  return accumulator.labeledCountsSnapshot;
}

function assignTouchStats(target: TouchStats, source: TouchAccumulator | undefined): void {
  if (!source) {
    Object.assign(target, DEFAULT_TOUCH_STATS);
    return;
  }
  Object.assign(target, source.stats, {
    labeled_touch_counts: getLabeledTouchCountsSnapshot(source),
  });
}

export function applyTouchEventDerivedStats(
  timeline: MaterializedStatsTimeline,
): MaterializedStatsTimeline {
  const accumulator = createTouchEventDerivedStatsAccumulator(timeline);

  for (const frame of timeline.frames) {
    accumulator.applyFrame(frame);
  }

  return timeline;
}

export function createTouchEventDerivedStatsAccumulator(timeline: MaterializedStatsTimeline): {
  applyFrame(frame: StatsFrame): void;
} {
  const touchEvents = sortBySample(statsEventPayloads(timeline, "touch"));
  const movementEvents = touchEvents
    .flatMap((event): TouchMovementCredit[] =>
      event.ball_movement ? [{ player: event.player, movement: event.ball_movement }] : [],
    )
    .sort((left, right) => {
      if (left.movement.end_frame !== right.movement.end_frame) {
        return left.movement.end_frame - right.movement.end_frame;
      }
      return left.movement.end_time - right.movement.end_time;
    });

  let touchEventIndex = 0;
  let movementEventIndex = 0;
  let currentLastTouchPlayerKey: string | null = null;
  const players = new Map<string, TouchAccumulator>();

  return {
    applyFrame(frame: StatsFrame): void {
      if (!frame.is_live_play) {
        currentLastTouchPlayerKey = null;
      } else {
        for (const accumulator of players.values()) {
          const stats = accumulator.stats;
          stats.is_last_touch = false;
          if (stats.last_touch_time != null) {
            stats.time_since_last_touch = Math.max(0, subF32(frame.time, stats.last_touch_time));
          }
          if (stats.last_touch_frame != null) {
            stats.frames_since_last_touch = Math.max(
              0,
              frame.frame_number - stats.last_touch_frame,
            );
          }
        }

        while (
          touchEventIndex < touchEvents.length &&
          (touchEvents[touchEventIndex]!.sample_frame ?? touchEvents[touchEventIndex]!.frame) <=
            frame.frame_number
        ) {
          const event = touchEvents[touchEventIndex] as TouchClassificationEvent;
          const playerKey = remoteIdKey(event.player);
          const accumulator = players.get(playerKey) ?? createTouchAccumulator();
          players.set(playerKey, accumulator);
          applyTouchClassificationEvent(accumulator, event, frame);
          currentLastTouchPlayerKey = playerKey;
          touchEventIndex += 1;
        }

        if (currentLastTouchPlayerKey != null) {
          const accumulator = players.get(currentLastTouchPlayerKey);
          if (accumulator) {
            accumulator.stats.is_last_touch = true;
          }
        }
      }

      while (
        movementEventIndex < movementEvents.length &&
        movementEvents[movementEventIndex]!.movement.end_frame <= frame.frame_number
      ) {
        const event = movementEvents[movementEventIndex] as TouchMovementCredit;
        const playerKey = remoteIdKey(event.player);
        const accumulator = players.get(playerKey) ?? createTouchAccumulator();
        players.set(playerKey, accumulator);
        const stats = accumulator.stats;
        stats.total_ball_travel_distance = addF32(
          stats.total_ball_travel_distance,
          event.movement.travel_distance,
        );
        stats.total_ball_advance_distance = addF32(
          stats.total_ball_advance_distance,
          event.movement.advance_distance,
        );
        stats.total_ball_retreat_distance = addF32(
          stats.total_ball_retreat_distance,
          event.movement.retreat_distance,
        );
        movementEventIndex += 1;
      }

      for (const player of frame.players) {
        assignTouchStats(player.touch, players.get(remoteIdKey(player.player_id)));
      }
    },
  };
}
