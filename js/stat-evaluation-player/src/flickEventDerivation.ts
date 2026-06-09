import type { FlickEvent } from "./generated/FlickEvent.ts";
import type { FlickStats } from "./generated/FlickStats.ts";
import type { LabeledCounts } from "./generated/LabeledCounts.ts";
import type { StatLabel } from "./generated/StatLabel.ts";
import type { StatsFrame, MaterializedStatsTimeline } from "./statsTimeline.ts";
import { statsEventPayloads } from "./statsTimeline.ts";

const FLICK_HIGH_CONFIDENCE = 0.8;

type FlickStatsWithLabels = FlickStats & {
  labeled_event_counts?: LabeledCounts;
};

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

function defaultFlickStats(): FlickStatsWithLabels {
  return {
    count: 0,
    high_confidence_count: 0,
    is_last_flick: false,
    last_flick_time: null,
    last_flick_frame: null,
    time_since_last_flick: null,
    frames_since_last_flick: null,
    last_confidence: null,
    best_confidence: 0,
    cumulative_confidence: 0,
    cumulative_setup_duration: 0,
    cumulative_ball_speed_change: 0,
  };
}

function sortFlickEvents(events: readonly FlickEvent[]): FlickEvent[] {
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
      if (left.event.time !== right.event.time) {
        return left.event.time - right.event.time;
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

function incrementLabels(stats: FlickStatsWithLabels, labels: StatLabel[]): void {
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

function countWithConfidenceLabel(stats: FlickStatsWithLabels, value: "standard" | "high"): number {
  return (
    stats.labeled_event_counts?.entries
      .filter((entry) =>
        entry.labels.some((label) => label.key === "confidence_band" && label.value === value),
      )
      .reduce((total, entry) => total + entry.count, 0) ?? 0
  );
}

function flickKindLabelValue(value: unknown): "other" | "reverse" {
  return value === "reverse" ? value : "other";
}

function setupRotationDirectionLabelValue(value: unknown): "unknown" | "left" | "right" {
  return value === "left" || value === "right" ? value : "unknown";
}

function totalLabeledCount(stats: FlickStatsWithLabels): number {
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

function advanceFlickStats(
  stats: FlickStatsWithLabels,
  frameNumber: number,
  frameTime: number,
  isLastFlickPlayer: boolean,
): void {
  stats.is_last_flick = isLastFlickPlayer;
  stats.time_since_last_flick =
    stats.last_flick_time == null ? null : Math.max(0, subF32(frameTime, stats.last_flick_time));
  stats.frames_since_last_flick =
    stats.last_flick_frame == null ? null : Math.max(0, frameNumber - stats.last_flick_frame);
}

function applyFlickEvent(
  stats: FlickStatsWithLabels,
  event: FlickEvent,
  frameNumber: number,
  frameTime: number,
): void {
  incrementLabels(stats, [
    {
      key: "confidence_band",
      value: event.confidence >= FLICK_HIGH_CONFIDENCE ? "high" : "standard",
    },
    {
      key: "kind",
      value: flickKindLabelValue(event.kind),
    },
    {
      key: "setup_rotation_direction",
      value: setupRotationDirectionLabelValue(event.setup_rotation_direction),
    },
  ]);
  stats.count = totalLabeledCount(stats);
  stats.high_confidence_count = countWithConfidenceLabel(stats, "high");
  stats.is_last_flick = true;
  stats.last_flick_time = event.time;
  stats.last_flick_frame = event.frame;
  stats.time_since_last_flick = Math.max(0, subF32(frameTime, event.time));
  stats.frames_since_last_flick = Math.max(0, frameNumber - event.frame);
  stats.last_confidence = event.confidence;
  stats.best_confidence = Math.max(stats.best_confidence, event.confidence);
  stats.cumulative_confidence = addF32(stats.cumulative_confidence, event.confidence);
  stats.cumulative_setup_duration = addF32(stats.cumulative_setup_duration, event.setup_duration);
  stats.cumulative_ball_speed_change = addF32(
    stats.cumulative_ball_speed_change,
    event.ball_speed_change,
  );
}

function assignFlickStats(target: FlickStats, source: FlickStatsWithLabels | undefined): void {
  Object.assign(target, source ?? defaultFlickStats());
  if (source?.labeled_event_counts) {
    (target as FlickStatsWithLabels).labeled_event_counts = cloneLabeledCounts(
      source.labeled_event_counts,
    );
  } else {
    delete (target as FlickStatsWithLabels).labeled_event_counts;
  }
}

export function applyFlickEventDerivedStats(
  timeline: MaterializedStatsTimeline,
): MaterializedStatsTimeline {
  const accumulator = createFlickEventDerivedStatsAccumulator(timeline);

  for (const frame of timeline.frames) {
    accumulator.applyFrame(frame);
  }

  return timeline;
}

export function createFlickEventDerivedStatsAccumulator(timeline: MaterializedStatsTimeline): {
  applyFrame(frame: StatsFrame): void;
} {
  const events = sortFlickEvents(statsEventPayloads(timeline, "flick"));

  let eventIndex = 0;
  let lastFlickPlayer: string | null = null;
  const players = new Map<string, FlickStatsWithLabels>();

  return {
    applyFrame(frame: StatsFrame): void {
      if (frame.is_live_play) {
        for (const [playerKey, stats] of players) {
          advanceFlickStats(stats, frame.frame_number, frame.time, playerKey === lastFlickPlayer);
        }

        while (
          eventIndex < events.length &&
          (events[eventIndex]!.sample_frame ?? events[eventIndex]!.frame) <= frame.frame_number
        ) {
          const event = events[eventIndex] as FlickEvent;
          const playerKey = remoteIdKey(event.player);
          const stats = players.get(playerKey) ?? defaultFlickStats();
          players.set(playerKey, stats);
          applyFlickEvent(stats, event, frame.frame_number, frame.time);
          lastFlickPlayer = playerKey;
          eventIndex += 1;
        }
        if (lastFlickPlayer != null) {
          const stats = players.get(lastFlickPlayer);
          if (stats) {
            stats.is_last_flick = true;
          }
        }
      } else {
        lastFlickPlayer = null;
      }

      for (const player of frame.players) {
        assignFlickStats(player.flick, players.get(remoteIdKey(player.player_id)));
      }
    },
  };
}
