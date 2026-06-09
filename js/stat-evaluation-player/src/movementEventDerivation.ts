import type { LabeledFloatSums } from "./generated/LabeledFloatSums.ts";
import type { MovementEvent } from "./generated/MovementEvent.ts";
import type { MovementStats } from "./generated/MovementStats.ts";
import type { StatLabel } from "./generated/StatLabel.ts";
import type { StatsFrame, MaterializedStatsTimeline } from "./statsTimeline.ts";
import { statsEventPayloads } from "./statsTimeline.ts";

const MOVEMENT_SPEED_BANDS = ["boost", "slow", "supersonic"] as const;
const MOVEMENT_HEIGHT_BANDS = ["ground", "high_air", "low_air"] as const;

function f32(value: number): number {
  return Math.fround(value);
}

function addF32(left: number, right: number): number {
  return f32(f32(left) + f32(right));
}

function mulF32(left: number, right: number): number {
  return f32(f32(left) * f32(right));
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

function emptyLabeledTrackedTime(): LabeledFloatSums {
  return {
    entries: MOVEMENT_HEIGHT_BANDS.flatMap((heightBand) =>
      MOVEMENT_SPEED_BANDS.map((speedBand) => ({
        labels: [
          { key: "height_band", value: heightBand },
          { key: "speed_band", value: speedBand },
        ],
        value: 0,
      })),
    ).sort((left, right) =>
      JSON.stringify(left.labels).localeCompare(JSON.stringify(right.labels)),
    ),
  };
}

function defaultMovementStats(completeLabeledTrackedTime = false): MovementStats {
  return {
    tracked_time: 0,
    total_distance: 0,
    speed_integral: 0,
    time_slow_speed: 0,
    time_boost_speed: 0,
    time_supersonic_speed: 0,
    time_on_ground: 0,
    time_low_air: 0,
    time_high_air: 0,
    labeled_tracked_time: completeLabeledTrackedTime ? emptyLabeledTrackedTime() : { entries: [] },
  };
}

function sortMovementEvents(events: readonly MovementEvent[]): MovementEvent[] {
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

function sortLabels(labels: StatLabel[]): StatLabel[] {
  return labels.sort((left, right) =>
    left.key === right.key
      ? left.value.localeCompare(right.value)
      : left.key.localeCompare(right.key),
  );
}

function addLabeledTime(sums: LabeledFloatSums, labels: StatLabel[], value: number): void {
  const sortedLabels = sortLabels(labels);
  const entry = sums.entries.find(
    (candidate) =>
      candidate.labels.length === sortedLabels.length &&
      candidate.labels.every(
        (label, index) =>
          label.key === sortedLabels[index]?.key && label.value === sortedLabels[index]?.value,
      ),
  );
  if (entry) {
    entry.value = addF32(entry.value, value);
  } else {
    sums.entries.push({ labels: sortedLabels, value: f32(value) });
    sums.entries.sort((left, right) =>
      JSON.stringify(left.labels).localeCompare(JSON.stringify(right.labels)),
    );
  }
}

function cloneLabeledTrackedTime(source: LabeledFloatSums): LabeledFloatSums {
  return {
    entries: source.entries.map((entry) => ({
      labels: entry.labels.map((label) => ({ ...label })),
      value: entry.value,
    })),
  };
}

function applyMovementEvent(stats: MovementStats, event: MovementEvent): void {
  const dt = f32(event.dt);
  stats.tracked_time = addF32(stats.tracked_time, dt);
  stats.total_distance = addF32(stats.total_distance, event.distance);
  stats.speed_integral = addF32(stats.speed_integral, mulF32(event.speed, dt));

  if (event.speed_band === "slow") {
    stats.time_slow_speed = addF32(stats.time_slow_speed, dt);
  } else if (event.speed_band === "boost") {
    stats.time_boost_speed = addF32(stats.time_boost_speed, dt);
  } else if (event.speed_band === "supersonic") {
    stats.time_supersonic_speed = addF32(stats.time_supersonic_speed, dt);
  }

  if (event.height_band === "ground") {
    stats.time_on_ground = addF32(stats.time_on_ground, dt);
  } else if (event.height_band === "low_air") {
    stats.time_low_air = addF32(stats.time_low_air, dt);
  } else if (event.height_band === "high_air") {
    stats.time_high_air = addF32(stats.time_high_air, dt);
  }

  const labeledTrackedTime = stats.labeled_tracked_time ?? { entries: [] };
  stats.labeled_tracked_time = labeledTrackedTime;
  addLabeledTime(
    labeledTrackedTime,
    [
      { key: "speed_band", value: event.speed_band },
      { key: "height_band", value: event.height_band },
    ],
    dt,
  );
}

function assignMovementStats(target: MovementStats, source: MovementStats | undefined): void {
  const stats = source ?? defaultMovementStats(true);
  const labeledTrackedTime = stats.labeled_tracked_time;
  Object.assign(target, stats, {
    labeled_tracked_time: labeledTrackedTime
      ? cloneLabeledTrackedTime(labeledTrackedTime)
      : undefined,
  });
  if (!labeledTrackedTime?.entries.length) {
    delete target.labeled_tracked_time;
  }
}

export function applyMovementEventDerivedStats(
  timeline: MaterializedStatsTimeline,
): MaterializedStatsTimeline {
  const accumulator = createMovementEventDerivedStatsAccumulator(timeline);

  for (const frame of timeline.frames) {
    accumulator.applyFrame(frame);
  }

  return timeline;
}

export function createMovementEventDerivedStatsAccumulator(timeline: MaterializedStatsTimeline): {
  applyFrame(frame: StatsFrame): void;
} {
  const events = sortMovementEvents(statsEventPayloads(timeline, "movement"));

  let eventIndex = 0;
  const players = new Map<string, MovementStats>();
  const teamZero = defaultMovementStats();
  const teamOne = defaultMovementStats();

  return {
    applyFrame(frame: StatsFrame): void {
      while (eventIndex < events.length && events[eventIndex]!.frame <= frame.frame_number) {
        const event = events[eventIndex] as MovementEvent;
        const playerKey = remoteIdKey(event.player);
        const playerStats = players.get(playerKey) ?? defaultMovementStats(true);
        players.set(playerKey, playerStats);
        applyMovementEvent(playerStats, event);
        applyMovementEvent(event.is_team_0 ? teamZero : teamOne, event);
        eventIndex += 1;
      }

      assignMovementStats(frame.team_zero.movement, teamZero);
      assignMovementStats(frame.team_one.movement, teamOne);
      for (const player of frame.players) {
        assignMovementStats(player.movement, players.get(remoteIdKey(player.player_id)));
      }
    },
  };
}
