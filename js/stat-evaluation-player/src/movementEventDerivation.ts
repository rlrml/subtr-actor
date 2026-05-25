import type { LabeledFloatSums } from "./generated/LabeledFloatSums.ts";
import type { MovementEvent } from "./generated/MovementEvent.ts";
import type { MovementStats } from "./generated/MovementStats.ts";
import type { StatLabel } from "./generated/StatLabel.ts";
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

function defaultMovementStats(): MovementStats {
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
    labeled_tracked_time: { entries: [] },
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
    left.key === right.key ? left.value.localeCompare(right.value) : left.key.localeCompare(right.key),
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
    entry.value += value;
  } else {
    sums.entries.push({ labels: sortedLabels, value });
    sums.entries.sort((left, right) =>
      JSON.stringify(left.labels).localeCompare(JSON.stringify(right.labels)),
    );
  }
}

function applyMovementEvent(stats: MovementStats, event: MovementEvent): void {
  stats.tracked_time += event.dt;
  stats.total_distance += event.distance;
  stats.speed_integral += event.speed * event.dt;

  if (event.speed_band === "slow") {
    stats.time_slow_speed += event.dt;
  } else if (event.speed_band === "boost") {
    stats.time_boost_speed += event.dt;
  } else if (event.speed_band === "supersonic") {
    stats.time_supersonic_speed += event.dt;
  }

  if (event.height_band === "ground") {
    stats.time_on_ground += event.dt;
  } else if (event.height_band === "low_air") {
    stats.time_low_air += event.dt;
  } else if (event.height_band === "high_air") {
    stats.time_high_air += event.dt;
  }

  const labeledTrackedTime = stats.labeled_tracked_time ?? { entries: [] };
  stats.labeled_tracked_time = labeledTrackedTime;
  addLabeledTime(
    labeledTrackedTime,
    [
      { key: "speed_band", value: event.speed_band },
      { key: "height_band", value: event.height_band },
    ],
    event.dt,
  );
}

function assignMovementStats(target: MovementStats, source: MovementStats | undefined): void {
  Object.assign(target, source ?? defaultMovementStats());
}

export function applyMovementEventDerivedStats(timeline: StatsTimeline): StatsTimeline {
  const events = sortMovementEvents(timeline.events.movement ?? []);

  let eventIndex = 0;
  const players = new Map<string, MovementStats>();
  const teamZero = defaultMovementStats();
  const teamOne = defaultMovementStats();

  for (const frame of timeline.frames) {
    while (eventIndex < events.length && events[eventIndex]!.frame <= frame.frame_number) {
      const event = events[eventIndex] as MovementEvent;
      const playerKey = remoteIdKey(event.player);
      const playerStats = players.get(playerKey) ?? defaultMovementStats();
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
  }

  return timeline;
}
