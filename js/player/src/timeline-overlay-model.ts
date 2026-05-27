import type {
  ReplayPlayerPluginContext,
  ReplayPlayerTimelineProjection,
  ReplayTimelineEvent,
  ReplayTimelineEventKind,
  ReplayTimelineEventSource,
  ReplayTimelineRange,
  ReplayTimelineRangeSource,
} from "./types";

export interface TimelineEventBucket {
  key: string;
  time: number;
  events: ReplayTimelineEvent[];
}

export interface TimelineEventLane {
  key: string;
  label: string;
  buckets: TimelineEventBucket[];
}

export interface TimelineEventSourceRecord {
  key: string;
  label: string;
  source: ReplayTimelineEventSource;
}

export interface TimelineRangeLane {
  key: string;
  label: string;
  ranges: ReplayTimelineRange[];
}

export const DEFAULT_REPLAY_EVENT_KINDS = new Set<ReplayTimelineEventKind>(["goal", "save"]);

const DEFAULT_EVENT_SEEK_LEAD_SECONDS = 2;
const GOAL_EVENT_SEEK_LEAD_SECONDS = 4;
const COLLAPSED_SKIPPED_RANGE_WIDTH_SECONDS = 0.01;

export function formatPlaybackTime(seconds: number): string {
  if (!Number.isFinite(seconds)) {
    return "--:--.--";
  }

  const safeSeconds = Math.max(0, seconds);
  const minutes = Math.floor(safeSeconds / 60);
  const wholeSeconds = Math.floor(safeSeconds % 60);
  const hundredths = Math.floor((safeSeconds - Math.floor(safeSeconds)) * 100);
  return `${minutes}:${String(wholeSeconds).padStart(2, "0")}.${String(hundredths).padStart(2, "0")}`;
}

export function timelineEventSeekTime(event: ReplayTimelineEvent): number {
  if (event.seekTime !== undefined && Number.isFinite(event.seekTime)) {
    return Math.max(0, event.seekTime);
  }
  if (!Number.isFinite(event.time)) {
    return 0;
  }
  return Math.max(0, event.time - eventSeekLeadSeconds(event));
}

export function eventAccent(event: ReplayTimelineEvent): string {
  if (event.color) {
    return event.color;
  }

  if (event.isTeamZero === true) {
    return "#3b82f6";
  }
  if (event.isTeamZero === false) {
    return "#f59e0b";
  }

  switch (event.kind) {
    case "goal":
      return "#f5f7fa";
    case "demo":
      return "#ef4444";
    case "save":
      return "#34d399";
    case "assist":
      return "#c084fc";
    case "shot":
      return "#60a5fa";
    default:
      return "#d1d9e0";
  }
}

export function eventBadgeText(bucket: TimelineEventBucket): string {
  if (bucket.events.length > 1) {
    return `${bucket.events.length}`;
  }

  const event = bucket.events[0];
  if (!event) {
    return "";
  }

  if (event.shortLabel && event.shortLabel.trim() !== "") {
    return event.shortLabel.slice(0, 3).toUpperCase();
  }

  return event.kind.slice(0, 1).toUpperCase();
}

export function bucketTitle(bucket: TimelineEventBucket): string {
  return bucket.events
    .map((event) => `${formatPlaybackTime(event.time)} ${event.label ?? event.kind}`)
    .join("\n");
}

export function groupEvents(events: ReplayTimelineEvent[]): TimelineEventBucket[] {
  const groups = new Map<string, TimelineEventBucket>();
  for (const event of events) {
    const key =
      event.frame !== undefined ? `frame:${event.frame}` : `time:${event.time.toFixed(2)}`;
    const existing = groups.get(key);
    if (existing) {
      existing.events.push(event);
      continue;
    }
    groups.set(key, {
      key,
      time: event.time,
      events: [event],
    });
  }

  return [...groups.values()]
    .map((bucket) => ({
      ...bucket,
      events: [...bucket.events].sort((left, right) => {
        const priorityDiff = eventPriority(right) - eventPriority(left);
        if (priorityDiff !== 0) {
          return priorityDiff;
        }
        return left.time - right.time;
      }),
    }))
    .sort((left, right) => left.time - right.time);
}

export function resolveCustomEvents(
  source: ReplayTimelineEventSource | undefined,
  context: ReplayPlayerPluginContext,
): ReplayTimelineEvent[] {
  if (!source) {
    return [];
  }

  return typeof source === "function" ? source(context) : source;
}

export function resolveEventSources(
  sources: Iterable<TimelineEventSourceRecord>,
  context: ReplayPlayerPluginContext,
): TimelineEventLane[] {
  const lanes: TimelineEventLane[] = [];
  for (const source of sources) {
    const events = resolveCustomEvents(source.source, context);
    if (events.length === 0) {
      continue;
    }
    lanes.push({
      key: source.key,
      label: source.label,
      buckets: groupEvents(events),
    });
  }
  return lanes;
}

export function resolveCustomRanges(
  source: ReplayTimelineRangeSource | undefined,
  context: ReplayPlayerPluginContext,
): ReplayTimelineRange[] {
  if (!source) {
    return [];
  }

  return typeof source === "function" ? source(context) : source;
}

export function resolveRangeSources(
  sources: Iterable<ReplayTimelineRangeSource>,
  context: ReplayPlayerPluginContext,
): ReplayTimelineRange[] {
  const rangesById = new Set<string>();
  const ranges: ReplayTimelineRange[] = [];
  for (const source of sources) {
    for (const range of resolveCustomRanges(source, context)) {
      const rangeId = range.id;
      if (rangeId !== undefined) {
        if (rangesById.has(rangeId)) {
          continue;
        }
        rangesById.add(rangeId);
      }
      ranges.push(range);
    }
  }
  return ranges;
}

export function groupRanges(ranges: ReplayTimelineRange[]): TimelineRangeLane[] {
  const lanes = new Map<string, TimelineRangeLane>();
  for (const range of ranges) {
    const laneKey = range.lane ?? "default";
    const laneLabel = range.laneLabel ?? range.lane ?? "";
    const existing = lanes.get(laneKey);
    if (existing) {
      existing.ranges.push(range);
      continue;
    }
    lanes.set(laneKey, {
      key: laneKey,
      label: laneLabel,
      ranges: [range],
    });
  }

  return [...lanes.values()].map((lane) => ({
    ...lane,
    ranges: [...lane.ranges].sort((left, right) => left.startTime - right.startTime),
  }));
}

export function rangeAccent(range: ReplayTimelineRange): string {
  if (range.color) {
    return range.color;
  }

  if (range.isTeamZero === true) {
    return "#3b82f6";
  }
  if (range.isTeamZero === false) {
    return "#f59e0b";
  }

  return "#d1d9e0";
}

export function markerLeftPercent(timelineTime: number, duration: number): string {
  return `${(timelineTime / Math.max(duration, 0.0001)) * 100}%`;
}

export function projectedRangeTimelineBounds(
  startProjection: ReplayPlayerTimelineProjection,
  endProjection: ReplayPlayerTimelineProjection,
  duration: number,
): { startTimelineTime: number; endTimelineTime: number } {
  let startTimelineTime = startProjection.timelineTime;
  let endTimelineTime = endProjection.timelineTime;

  if (
    endTimelineTime <= startTimelineTime &&
    (startProjection.hiddenBySkip || endProjection.hiddenBySkip)
  ) {
    if (startTimelineTime >= duration) {
      startTimelineTime = Math.max(0, duration - COLLAPSED_SKIPPED_RANGE_WIDTH_SECONDS);
      endTimelineTime = duration;
    } else {
      endTimelineTime = Math.min(
        duration,
        startTimelineTime + COLLAPSED_SKIPPED_RANGE_WIDTH_SECONDS,
      );
    }
  }

  return { startTimelineTime, endTimelineTime };
}

function eventPriority(event: ReplayTimelineEvent): number {
  switch (event.kind) {
    case "goal":
      return 5;
    case "demo":
      return 4;
    case "save":
      return 3;
    case "assist":
      return 2;
    case "shot":
      return 1;
    default:
      return 0;
  }
}

function eventSeekLeadSeconds(event: ReplayTimelineEvent): number {
  switch (event.kind) {
    case "goal":
    case "goal-context":
    case "goal-tag":
      return GOAL_EVENT_SEEK_LEAD_SECONDS;
    default:
      return DEFAULT_EVENT_SEEK_LEAD_SECONDS;
  }
}
