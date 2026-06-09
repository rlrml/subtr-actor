import type { AirDribbleOrigin } from "./generated/AirDribbleOrigin.ts";
import type { AirDribbleStats } from "./generated/AirDribbleStats.ts";
import type { BallCarryEvent } from "./generated/BallCarryEvent.ts";
import type { BallCarryStats } from "./generated/BallCarryStats.ts";
import type { LabeledCounts } from "./generated/LabeledCounts.ts";
import type { StatLabel } from "./generated/StatLabel.ts";
import type { StatsFrame, MaterializedStatsTimeline } from "./statsTimeline.ts";
import { statsEventPayloads } from "./statsTimeline.ts";

type BallCarryStatsWithLabels = BallCarryStats & {
  labeled_event_counts?: LabeledCounts;
};

type AirDribbleStatsWithLabels = AirDribbleStats & {
  labeled_event_counts?: LabeledCounts;
};

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

function defaultBallCarryStats(): BallCarryStatsWithLabels {
  return {
    carry_count: 0,
    total_carry_time: 0,
    total_straight_line_distance: 0,
    total_path_distance: 0,
    longest_carry_time: 0,
    furthest_carry_distance: 0,
    fastest_carry_speed: 0,
    carry_speed_sum: 0,
    average_horizontal_gap_sum: 0,
    average_vertical_gap_sum: 0,
  };
}

function defaultAirDribbleStats(): AirDribbleStatsWithLabels {
  return {
    count: 0,
    ground_to_air_count: 0,
    wall_to_air_count: 0,
    total_touch_count: 0,
    max_touch_count: 0,
    total_time: 0,
    total_straight_line_distance: 0,
    total_path_distance: 0,
    longest_time: 0,
    furthest_distance: 0,
    fastest_speed: 0,
    speed_sum: 0,
    average_horizontal_gap_sum: 0,
    average_vertical_gap_sum: 0,
  };
}

function sortBallCarryEvents(events: readonly BallCarryEvent[]): BallCarryEvent[] {
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

function labelSortKey(label: StatLabel): string {
  return `${label.key}\u0000${label.value}`;
}

function labelsSortKey(labels: readonly StatLabel[]): string {
  return labels.map(labelSortKey).join("\u0001");
}

function incrementLabels(
  stats: { labeled_event_counts?: LabeledCounts },
  labels: StatLabel[],
): void {
  labels.sort((left, right) => labelSortKey(left).localeCompare(labelSortKey(right)));
  const labeledCounts = (stats.labeled_event_counts ??= { entries: [] });
  const existing = labeledCounts.entries.find(
    (entry) => labelsSortKey(entry.labels) === labelsSortKey(labels),
  );
  if (existing) {
    existing.count += 1;
  } else {
    labeledCounts.entries.push({ labels: [...labels], count: 1 });
    labeledCounts.entries.sort((left, right) =>
      labelsSortKey(left.labels).localeCompare(labelsSortKey(right.labels)),
    );
  }
}

function countWithLabel(stats: AirDribbleStatsWithLabels, value: AirDribbleOrigin): number {
  return (
    stats.labeled_event_counts?.entries
      .filter((entry) =>
        entry.labels.some((label) => label.key === "origin" && label.value === value),
      )
      .reduce((total, entry) => total + entry.count, 0) ?? 0
  );
}

function totalLabeledCount(stats: { labeled_event_counts?: LabeledCounts }): number {
  return stats.labeled_event_counts?.entries.reduce((total, entry) => total + entry.count, 0) ?? 0;
}

function cloneLabeledCounts(counts: LabeledCounts): LabeledCounts {
  return {
    entries: counts.entries.map((entry) => ({
      labels: entry.labels.map((label) => ({ ...label })),
      count: entry.count,
    })),
  };
}

function applyBallCarryEvent(stats: BallCarryStatsWithLabels, event: BallCarryEvent): void {
  incrementLabels(stats, [{ key: "kind", value: "carry" }]);
  stats.carry_count = totalLabeledCount(stats);
  stats.total_carry_time += event.duration;
  stats.total_straight_line_distance += event.straight_line_distance;
  stats.total_path_distance += event.path_distance;
  stats.longest_carry_time = Math.max(stats.longest_carry_time, event.duration);
  stats.furthest_carry_distance = Math.max(
    stats.furthest_carry_distance,
    event.straight_line_distance,
  );
  stats.fastest_carry_speed = Math.max(stats.fastest_carry_speed, event.average_speed);
  stats.carry_speed_sum += event.average_speed;
  stats.average_horizontal_gap_sum += event.average_horizontal_gap;
  stats.average_vertical_gap_sum += event.average_vertical_gap;
}

function applyAirDribbleEvent(stats: AirDribbleStatsWithLabels, event: BallCarryEvent): void {
  if (event.air_dribble_origin != null) {
    incrementLabels(stats, [{ key: "origin", value: event.air_dribble_origin }]);
  }
  stats.count = totalLabeledCount(stats);
  stats.ground_to_air_count = countWithLabel(stats, "ground_to_air");
  stats.wall_to_air_count = countWithLabel(stats, "wall_to_air");
  stats.total_time += event.duration;
  stats.total_straight_line_distance += event.straight_line_distance;
  stats.total_path_distance += event.path_distance;
  stats.longest_time = Math.max(stats.longest_time, event.duration);
  stats.furthest_distance = Math.max(stats.furthest_distance, event.straight_line_distance);
  stats.fastest_speed = Math.max(stats.fastest_speed, event.average_speed);
  stats.speed_sum += event.average_speed;
  stats.average_horizontal_gap_sum += event.average_horizontal_gap;
  stats.average_vertical_gap_sum += event.average_vertical_gap;
  stats.total_touch_count += event.touch_count;
  stats.max_touch_count = Math.max(stats.max_touch_count, event.touch_count);
}

function assignBallCarryStats(
  target: BallCarryStats,
  source: BallCarryStatsWithLabels | undefined,
): void {
  Object.assign(target, source ?? defaultBallCarryStats());
  if (source?.labeled_event_counts) {
    (target as BallCarryStatsWithLabels).labeled_event_counts = cloneLabeledCounts(
      source.labeled_event_counts,
    );
  } else {
    delete (target as BallCarryStatsWithLabels).labeled_event_counts;
  }
}

function assignAirDribbleStats(
  target: AirDribbleStats,
  source: AirDribbleStatsWithLabels | undefined,
): void {
  Object.assign(target, source ?? defaultAirDribbleStats());
  if (source?.labeled_event_counts) {
    (target as AirDribbleStatsWithLabels).labeled_event_counts = cloneLabeledCounts(
      source.labeled_event_counts,
    );
  } else {
    delete (target as AirDribbleStatsWithLabels).labeled_event_counts;
  }
}

export function applyBallCarryEventDerivedStats(
  timeline: MaterializedStatsTimeline,
): MaterializedStatsTimeline {
  const accumulator = createBallCarryEventDerivedStatsAccumulator(timeline);

  for (const frame of timeline.frames) {
    accumulator.applyFrame(frame);
  }

  return timeline;
}

export function createBallCarryEventDerivedStatsAccumulator(timeline: MaterializedStatsTimeline): {
  applyFrame(frame: StatsFrame): void;
} {
  const events = sortBallCarryEvents(statsEventPayloads(timeline, "ball_carry"));

  let eventIndex = 0;
  const ballCarryPlayers = new Map<string, BallCarryStatsWithLabels>();
  const airDribblePlayers = new Map<string, AirDribbleStatsWithLabels>();
  const teamZeroBallCarry = defaultBallCarryStats();
  const teamOneBallCarry = defaultBallCarryStats();
  const teamZeroAirDribble = defaultAirDribbleStats();
  const teamOneAirDribble = defaultAirDribbleStats();

  return {
    applyFrame(frame: StatsFrame): void {
      while (eventIndex < events.length && events[eventIndex]!.end_frame < frame.frame_number) {
        const event = events[eventIndex] as BallCarryEvent;
        const playerKey = remoteIdKey(event.player_id);
        if (event.kind === "carry") {
          const playerStats = ballCarryPlayers.get(playerKey) ?? defaultBallCarryStats();
          ballCarryPlayers.set(playerKey, playerStats);
          applyBallCarryEvent(playerStats, event);
          applyBallCarryEvent(event.is_team_0 ? teamZeroBallCarry : teamOneBallCarry, event);
        } else {
          const playerStats = airDribblePlayers.get(playerKey) ?? defaultAirDribbleStats();
          airDribblePlayers.set(playerKey, playerStats);
          applyAirDribbleEvent(playerStats, event);
          applyAirDribbleEvent(event.is_team_0 ? teamZeroAirDribble : teamOneAirDribble, event);
        }
        eventIndex += 1;
      }

      assignBallCarryStats(frame.team_zero.ball_carry, teamZeroBallCarry);
      assignBallCarryStats(frame.team_one.ball_carry, teamOneBallCarry);
      assignAirDribbleStats(frame.team_zero.air_dribble, teamZeroAirDribble);
      assignAirDribbleStats(frame.team_one.air_dribble, teamOneAirDribble);
      for (const player of frame.players) {
        const playerKey = remoteIdKey(player.player_id);
        assignBallCarryStats(player.ball_carry, ballCarryPlayers.get(playerKey));
        assignAirDribbleStats(player.air_dribble, airDribblePlayers.get(playerKey));
      }
    },
  };
}
