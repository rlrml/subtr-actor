import type {
  ReplayPlayerPlugin,
  ReplayPlayerPluginContext,
  ReplayPlayerPluginStateContext,
  ReplayPlayerTimelineProjection,
  ReplayTimelineEvent,
  ReplayTimelineEventKind,
  ReplayTimelineEventSource,
  ReplayTimelineGraph,
  ReplayTimelineGraphPoint,
  ReplayTimelineGraphSource,
  ReplayTimelineRange,
  ReplayTimelineRangeSource,
} from "./types";
import { ensureTimelineOverlayStyles } from "./timeline-overlay-style";

export interface TimelineOverlayPluginOptions {
  pauseWhileScrubbing?: boolean;
  includeReplayEvents?: boolean;
  replayEventKinds?: Iterable<ReplayTimelineEventKind>;
  replayEventsLabel?: string;
  replayEvents?: ReplayTimelineEventSource;
  eventsLabel?: string;
  events?: ReplayTimelineEventSource;
  ranges?: ReplayTimelineRangeSource;
  graphs?: ReplayTimelineGraphSource;
}

export interface TimelineOverlayEventSourceOptions {
  id?: string;
  label?: string;
}

export interface TimelineOverlayPlugin extends ReplayPlayerPlugin {
  addEventSource(
    source: ReplayTimelineEventSource,
    options?: TimelineOverlayEventSourceOptions,
  ): () => void;
  removeEventSource(source: ReplayTimelineEventSource): boolean;
  refreshEvents(): void;
  addRangeSource(source: ReplayTimelineRangeSource): () => void;
  removeRangeSource(source: ReplayTimelineRangeSource): boolean;
  refreshRanges(): void;
  addGraphSource(source: ReplayTimelineGraphSource): () => void;
  removeGraphSource(source: ReplayTimelineGraphSource): boolean;
  refreshGraphs(): void;
}

interface TimelineEventBucket {
  key: string;
  time: number;
  events: ReplayTimelineEvent[];
}

interface TimelineEventLane {
  key: string;
  label: string;
  buckets: TimelineEventBucket[];
}

interface TimelineEventSourceRecord {
  key: string;
  label: string;
  source: ReplayTimelineEventSource;
}

interface TimelineRangeLane {
  key: string;
  label: string;
  ranges: ReplayTimelineRange[];
}

interface TimelineRangeRecord {
  range: ReplayTimelineRange;
  element: HTMLDivElement;
  startTimelineTime: number;
  endTimelineTime: number;
}

interface TimelineRangeLanePlayhead {
  element: HTMLDivElement;
}

interface TimelineGraphPlayhead {
  element: SVGLineElement;
  track: HTMLDivElement;
}

interface TimelineEventLanePlayhead {
  element: HTMLDivElement;
}

interface TimelineMarkerRecord {
  element: HTMLButtonElement;
  timelineTime: number;
  active: boolean;
  passed: boolean;
}

const DEFAULT_REPLAY_EVENT_KINDS = new Set<ReplayTimelineEventKind>(["goal", "save", "bookmark"]);
const ACTIVE_MARKER_WINDOW_SECONDS = 0.2;
const MAX_MARKERS_PER_TIMELINE_LANE = 60;
const DEFAULT_EVENT_SEEK_LEAD_SECONDS = 2;
const GOAL_EVENT_SEEK_LEAD_SECONDS = 4;
const HIDDEN_EVENT_SEEK_EPSILON_SECONDS = 0.01;
const COLLAPSED_SKIPPED_RANGE_WIDTH_SECONDS = 0.01;

function formatPlaybackTime(seconds: number): string {
  if (!Number.isFinite(seconds)) {
    return "--:--.--";
  }

  const safeSeconds = Math.max(0, seconds);
  const minutes = Math.floor(safeSeconds / 60);
  const wholeSeconds = Math.floor(safeSeconds % 60);
  const hundredths = Math.floor((safeSeconds - Math.floor(safeSeconds)) * 100);
  return `${minutes}:${String(wholeSeconds).padStart(2, "0")}.${String(hundredths).padStart(2, "0")}`;
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
    case "bookmark":
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

export function timelineEventSeekTime(event: ReplayTimelineEvent): number {
  if (event.seekTime !== undefined && Number.isFinite(event.seekTime)) {
    return Math.max(0, event.seekTime);
  }
  if (!Number.isFinite(event.time)) {
    return 0;
  }
  return Math.max(0, event.time - eventSeekLeadSeconds(event));
}

function eventAccent(event: ReplayTimelineEvent): string {
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
    case "bookmark":
      return "#facc15";
    default:
      return "#d1d9e0";
  }
}

function eventBadgeText(bucket: TimelineEventBucket): string {
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

function sortBucketEvents(events: ReplayTimelineEvent[]): ReplayTimelineEvent[] {
  return [...events].sort((left, right) => {
    const priorityDiff = eventPriority(right) - eventPriority(left);
    if (priorityDiff !== 0) {
      return priorityDiff;
    }
    return left.time - right.time;
  });
}

function bucketTitle(bucket: TimelineEventBucket): string {
  return bucket.events
    .map((event) => `${formatPlaybackTime(event.time)} ${event.label ?? event.kind}`)
    .join("\n");
}

function groupEvents(events: ReplayTimelineEvent[]): TimelineEventBucket[] {
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
      events: sortBucketEvents(bucket.events),
    }))
    .sort((left, right) => left.time - right.time);
}

function compactDenseTimelineBuckets(buckets: TimelineEventBucket[]): TimelineEventBucket[] {
  if (buckets.length <= MAX_MARKERS_PER_TIMELINE_LANE) {
    return buckets;
  }

  const firstTime = buckets[0]?.time ?? 0;
  const lastTime = buckets[buckets.length - 1]?.time ?? firstTime;
  const span = lastTime - firstTime;
  if (span <= 0) {
    return [
      {
        key: "compact:0",
        time: firstTime,
        events: sortBucketEvents(buckets.flatMap((bucket) => bucket.events)),
      },
    ];
  }

  const bucketWidth = span / MAX_MARKERS_PER_TIMELINE_LANE;
  const compactBuckets = new Map<number, TimelineEventBucket>();
  for (const bucket of buckets) {
    const compactIndex = Math.min(
      MAX_MARKERS_PER_TIMELINE_LANE - 1,
      Math.max(0, Math.floor((bucket.time - firstTime) / bucketWidth)),
    );
    const existing = compactBuckets.get(compactIndex);
    if (existing) {
      existing.events.push(...bucket.events);
      continue;
    }
    compactBuckets.set(compactIndex, {
      key: `compact:${compactIndex}`,
      time: bucket.time,
      events: [...bucket.events],
    });
  }

  return [...compactBuckets.values()]
    .map((bucket) => ({
      ...bucket,
      events: sortBucketEvents(bucket.events),
    }))
    .sort((left, right) => left.time - right.time);
}

function resolveCustomEvents(
  source: ReplayTimelineEventSource | undefined,
  context: ReplayPlayerPluginContext,
): ReplayTimelineEvent[] {
  if (!source) {
    return [];
  }

  return typeof source === "function" ? source(context) : source;
}

function resolveEventSources(
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

function resolveCustomRanges(
  source: ReplayTimelineRangeSource | undefined,
  context: ReplayPlayerPluginContext,
): ReplayTimelineRange[] {
  if (!source) {
    return [];
  }

  return typeof source === "function" ? source(context) : source;
}

function resolveRangeSources(
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

function resolveCustomGraphs(
  source: ReplayTimelineGraphSource | undefined,
  context: ReplayPlayerPluginContext,
): ReplayTimelineGraph[] {
  if (!source) {
    return [];
  }
  return typeof source === "function" ? source(context) : source;
}

function resolveGraphSources(
  sources: Iterable<ReplayTimelineGraphSource>,
  context: ReplayPlayerPluginContext,
): ReplayTimelineGraph[] {
  const graphIds = new Set<string>();
  const graphs: ReplayTimelineGraph[] = [];
  for (const source of sources) {
    for (const graph of resolveCustomGraphs(source, context)) {
      if (graph.id !== undefined) {
        if (graphIds.has(graph.id)) {
          continue;
        }
        graphIds.add(graph.id);
      }
      graphs.push(graph);
    }
  }
  return graphs;
}

function groupRanges(ranges: ReplayTimelineRange[]): TimelineRangeLane[] {
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

function rangeAccent(range: ReplayTimelineRange): string {
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

function resolveReplayEvents(
  options: TimelineOverlayPluginOptions,
  context: ReplayPlayerPluginContext,
): ReplayTimelineEvent[] {
  if (options.replayEvents) {
    return resolveCustomEvents(options.replayEvents, context);
  }

  if (options.includeReplayEvents === false) {
    return [];
  }

  const allowedKinds = new Set(options.replayEventKinds ?? DEFAULT_REPLAY_EVENT_KINDS);
  return context.replay.timelineEvents.filter((event) => allowedKinds.has(event.kind));
}

function markerSeekTime(event: ReplayTimelineEvent, context: ReplayPlayerPluginContext): number {
  const projection = context.player.projectReplayTimeToTimeline(timelineEventSeekTime(event));
  if (!projection.hiddenBySkip) {
    return projection.seekTime;
  }

  const nextTimelineTime = Math.min(
    context.player.getTimelineDuration(),
    projection.timelineTime + HIDDEN_EVENT_SEEK_EPSILON_SECONDS,
  );
  return context.player.projectTimelineTimeToReplay(nextTimelineTime);
}

function markerLeftPercent(timelineTime: number, duration: number): string {
  return `${(timelineTime / Math.max(duration, 0.0001)) * 100}%`;
}

const TIMELINE_GRAPH_WIDTH = 1000;
const TIMELINE_GRAPH_HEIGHT = 100;

function timelineGraphY(value: number, minValue: number, maxValue: number): number {
  const span = Math.max(maxValue - minValue, Number.EPSILON);
  const normalized = Math.max(0, Math.min(1, (value - minValue) / span));
  return (1 - normalized) * TIMELINE_GRAPH_HEIGHT;
}

/** Build an SVG path for timeline-projected points, preserving null gaps. */
export function buildTimelineGraphPath(
  points: readonly ReplayTimelineGraphPoint[],
  duration: number,
  minValue: number,
  maxValue: number,
): string {
  const safeDuration = Math.max(duration, 0.0001);
  let needsMove = true;
  const commands: string[] = [];
  for (const point of points) {
    if (point.value === null || !Number.isFinite(point.time) || !Number.isFinite(point.value)) {
      needsMove = true;
      continue;
    }
    const x = Math.max(
      0,
      Math.min(TIMELINE_GRAPH_WIDTH, (point.time / safeDuration) * TIMELINE_GRAPH_WIDTH),
    );
    const y = timelineGraphY(point.value, minValue, maxValue);
    commands.push(`${needsMove ? "M" : "L"}${x.toFixed(2)},${y.toFixed(2)}`);
    needsMove = false;
  }
  return commands.join(" ");
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

export function createTimelineOverlayPlugin(
  options: TimelineOverlayPluginOptions = {},
): TimelineOverlayPlugin {
  const pauseWhileScrubbing = options.pauseWhileScrubbing ?? true;
  let nextEventSourceId = 0;
  const extraEventSources: TimelineEventSourceRecord[] = options.events
    ? [
        {
          key: "events:initial",
          label: options.eventsLabel ?? "Events",
          source: options.events,
        },
      ]
    : [];
  const extraRangeSources = options.ranges ? [options.ranges] : [];
  const extraGraphSources = options.graphs ? [options.graphs] : [];

  let root: HTMLDivElement | null = null;
  let shell: HTMLDivElement | null = null;
  let rangesRoot: HTMLDivElement | null = null;
  let graphsRoot: HTMLDivElement | null = null;
  let range: HTMLInputElement | null = null;
  let toggleButton: HTMLButtonElement | null = null;
  let toggleButtonIcon: HTMLSpanElement | null = null;
  let toggleButtonLabel: HTMLSpanElement | null = null;
  let currentTimeText: HTMLSpanElement | null = null;
  let remainingTimeText: HTMLSpanElement | null = null;
  let eventLanesRoot: HTMLDivElement | null = null;
  let markers: HTMLDivElement | null = null;
  let removeWindowListeners: (() => void) | null = null;
  let changedContainerPosition = false;
  let originalContainerPosition = "";
  let scrubbing = false;
  let resumePlaybackAfterScrub = false;
  let playerContext: ReplayPlayerPluginContext | null = null;
  let eventLanes: TimelineEventLane[] = [];
  let rangeLanes: TimelineRangeLane[] = [];
  let projectionCacheKey: string | null = null;
  const markerElements = new Map<string, TimelineMarkerRecord>();
  const timelineMarkers: TimelineMarkerRecord[] = [];
  const rangeElements: TimelineRangeRecord[] = [];
  const rangeLanePlayheads: TimelineRangeLanePlayhead[] = [];
  const graphPlayheads: TimelineGraphPlayhead[] = [];
  const eventLanePlayheads: TimelineEventLanePlayhead[] = [];
  let passedMarkerEndIndex = 0;
  let activeMarkers = new Set<TimelineMarkerRecord>();

  function refreshMarkers(): void {
    if (!playerContext) {
      return;
    }

    buildMarkers(playerContext);
    syncState({
      ...playerContext,
      state: playerContext.player.getState(),
    });
  }

  function refreshRangeLanes(): void {
    if (!playerContext) {
      return;
    }

    buildRanges(playerContext);
    syncState({
      ...playerContext,
      state: playerContext.player.getState(),
    });
  }

  function refreshGraphLanes(): void {
    if (!playerContext) {
      return;
    }
    buildGraphs(playerContext);
    syncState({
      ...playerContext,
      state: playerContext.player.getState(),
    });
  }

  function syncState(context: ReplayPlayerPluginStateContext): void {
    if (
      !range ||
      !toggleButton ||
      !toggleButtonIcon ||
      !toggleButtonLabel ||
      !currentTimeText ||
      !remainingTimeText ||
      !shell
    ) {
      return;
    }

    const currentTime = context.player.getTimelineCurrentTime();
    const duration = context.player.getTimelineDuration();
    const nextProjectionCacheKey = [
      duration.toFixed(4),
      context.state.skipKickoffsEnabled ? "1" : "0",
      context.state.skipPostGoalTransitionsEnabled ? "1" : "0",
    ].join(":");
    if (projectionCacheKey !== nextProjectionCacheKey) {
      buildMarkers(context);
      buildRanges(context);
      buildGraphs(context);
      projectionCacheKey = nextProjectionCacheKey;
    }
    range.min = "0";
    range.max = `${duration}`;
    range.step = "0.01";
    range.value = `${Math.min(currentTime, duration)}`;
    toggleButton.dataset.playing = context.state.playing ? "true" : "false";
    toggleButton.setAttribute("aria-label", context.state.playing ? "Pause replay" : "Play replay");
    toggleButton.title = context.state.playing ? "Pause replay" : "Play replay";
    toggleButtonIcon.textContent = context.state.playing ? "||" : ">";
    toggleButtonLabel.textContent = context.state.playing ? "Pause" : "Play";
    currentTimeText.textContent = formatPlaybackTime(currentTime);
    remainingTimeText.textContent = `-${formatPlaybackTime(duration - currentTime)}`;
    shell.dataset.scrubbing = scrubbing ? "true" : "false";

    syncMarkerStates(currentTime);

    for (const record of rangeElements) {
      const leftTime = Math.max(0, record.startTimelineTime);
      const rightTime = Math.min(duration, record.endTimelineTime);
      const widthTime = Math.max(0, rightTime - leftTime);

      if (widthTime <= 0.0001) {
        record.element.hidden = true;
        continue;
      }

      record.element.hidden = false;
      record.element.dataset.active =
        currentTime >= leftTime && currentTime <= rightTime ? "true" : "false";
    }

    const playheadLeft = markerLeftPercent(Math.min(currentTime, duration), duration);
    for (const playhead of eventLanePlayheads) {
      playhead.element.style.left = playheadLeft;
    }
    for (const playhead of rangeLanePlayheads) {
      playhead.element.style.left = playheadLeft;
    }
    const graphPlayheadX = `${(Math.min(currentTime, duration) / Math.max(duration, 0.0001)) * TIMELINE_GRAPH_WIDTH}`;
    for (const playhead of graphPlayheads) {
      playhead.element.setAttribute("x1", graphPlayheadX);
      playhead.element.setAttribute("x2", graphPlayheadX);
      playhead.track.setAttribute("aria-valuenow", `${Math.min(currentTime, duration)}`);
      playhead.track.setAttribute("aria-valuetext", formatPlaybackTime(currentTime));
    }
  }

  function upperBoundMarkerTime(time: number): number {
    let low = 0;
    let high = timelineMarkers.length;
    while (low < high) {
      const mid = Math.floor((low + high) / 2);
      if (timelineMarkers[mid]!.timelineTime <= time) {
        low = mid + 1;
      } else {
        high = mid;
      }
    }
    return low;
  }

  function lowerBoundMarkerTime(time: number): number {
    let low = 0;
    let high = timelineMarkers.length;
    while (low < high) {
      const mid = Math.floor((low + high) / 2);
      if (timelineMarkers[mid]!.timelineTime < time) {
        low = mid + 1;
      } else {
        high = mid;
      }
    }
    return low;
  }

  function setMarkerActive(record: TimelineMarkerRecord, active: boolean): void {
    if (record.active === active) {
      return;
    }
    record.active = active;
    record.element.dataset.active = active ? "true" : "false";
  }

  function setMarkerPassed(record: TimelineMarkerRecord, passed: boolean): void {
    if (record.passed === passed) {
      return;
    }
    record.passed = passed;
    record.element.dataset.passed = passed ? "true" : "false";
  }

  function syncMarkerStates(currentTime: number): void {
    if (timelineMarkers.length === 0) {
      return;
    }

    const nextPassedEndIndex = upperBoundMarkerTime(currentTime);
    if (nextPassedEndIndex > passedMarkerEndIndex) {
      for (let index = passedMarkerEndIndex; index < nextPassedEndIndex; index += 1) {
        setMarkerPassed(timelineMarkers[index]!, true);
      }
    } else if (nextPassedEndIndex < passedMarkerEndIndex) {
      for (let index = nextPassedEndIndex; index < passedMarkerEndIndex; index += 1) {
        setMarkerPassed(timelineMarkers[index]!, false);
      }
    }
    passedMarkerEndIndex = nextPassedEndIndex;

    const activeStartIndex = lowerBoundMarkerTime(currentTime - ACTIVE_MARKER_WINDOW_SECONDS);
    const activeEndIndex = nextPassedEndIndex;
    const nextActiveMarkers = new Set<TimelineMarkerRecord>();
    for (let index = activeStartIndex; index < activeEndIndex; index += 1) {
      const record = timelineMarkers[index]!;
      nextActiveMarkers.add(record);
      setMarkerActive(record, true);
    }
    for (const record of activeMarkers) {
      if (!nextActiveMarkers.has(record)) {
        setMarkerActive(record, false);
      }
    }
    activeMarkers = nextActiveMarkers;
  }

  function createMarker(
    bucket: TimelineEventBucket,
    context: ReplayPlayerPluginContext,
    duration: number,
  ): HTMLButtonElement | null {
    const primaryEvent = bucket.events[0];
    if (!primaryEvent) {
      return null;
    }
    const projection = context.player.projectReplayTimeToTimeline(bucket.time);

    const marker = document.createElement("button");
    marker.type = "button";
    marker.className = "sap-tl-marker";
    marker.style.left = markerLeftPercent(projection.timelineTime, duration);
    marker.style.color = eventAccent(primaryEvent);
    marker.title = bucketTitle(bucket);
    marker.textContent = eventBadgeText(bucket);
    marker.addEventListener("click", () => {
      context.player.seek(markerSeekTime(primaryEvent, context));
    });
    marker.dataset.active = "false";
    marker.dataset.passed = "false";

    const markerRecord: TimelineMarkerRecord = {
      element: marker,
      timelineTime: projection.timelineTime,
      active: false,
      passed: false,
    };
    markerElements.set(bucket.key, markerRecord);
    timelineMarkers.push(markerRecord);

    return marker;
  }

  function buildMarkers(context: ReplayPlayerPluginContext): void {
    if (!markers || !eventLanesRoot) {
      return;
    }

    markers.replaceChildren();
    eventLanesRoot.replaceChildren();
    markerElements.clear();
    timelineMarkers.splice(0, timelineMarkers.length);
    passedMarkerEndIndex = 0;
    activeMarkers = new Set<TimelineMarkerRecord>();
    eventLanePlayheads.splice(0, eventLanePlayheads.length);

    const replayEvents = resolveReplayEvents(options, context);
    eventLanes = [];
    if (replayEvents.length > 0) {
      eventLanes.push({
        key: "replay",
        label: options.replayEventsLabel ?? "Replay",
        buckets: groupEvents(replayEvents),
      });
    }
    eventLanes.push(...resolveEventSources(extraEventSources, context));
    const duration = Math.max(context.player.getTimelineDuration(), 0.0001);

    const replayLane = eventLanes[0];
    if (replayLane?.key === "replay") {
      for (const bucket of compactDenseTimelineBuckets(replayLane.buckets)) {
        const marker = createMarker(
          { ...bucket, key: `${replayLane.key}:${bucket.key}` },
          context,
          duration,
        );
        if (marker) {
          markers.append(marker);
        }
      }
    }

    const customLanes = eventLanes.filter((lane) => lane.key !== "replay");
    eventLanesRoot.hidden = customLanes.length === 0;

    for (const lane of customLanes) {
      const laneEl = document.createElement("div");
      laneEl.className = "sap-tl-event-lane";
      laneEl.dataset.label = lane.label;

      const label = document.createElement("span");
      label.className = "sap-tl-event-lane-label";
      label.textContent = lane.label;
      label.setAttribute("aria-label", lane.label);
      laneEl.append(label);

      const track = document.createElement("div");
      track.className = "sap-tl-event-lane-track";

      const laneMarkers = document.createElement("div");
      laneMarkers.className = "sap-tl-markers";

      for (const bucket of compactDenseTimelineBuckets(lane.buckets)) {
        const marker = createMarker(
          { ...bucket, key: `${lane.key}:${bucket.key}` },
          context,
          duration,
        );
        if (marker) {
          laneMarkers.append(marker);
        }
      }

      const playhead = document.createElement("div");
      playhead.className = "sap-tl-event-playhead";
      track.append(laneMarkers, playhead);
      eventLanePlayheads.push({ element: playhead });
      laneEl.append(track);
      eventLanesRoot.append(laneEl);
    }

    timelineMarkers.sort((left, right) => left.timelineTime - right.timelineTime);
  }

  function buildRanges(context: ReplayPlayerPluginContext): void {
    if (!rangesRoot) {
      return;
    }

    rangesRoot.replaceChildren();
    rangeElements.splice(0, rangeElements.length);
    rangeLanePlayheads.splice(0, rangeLanePlayheads.length);

    const customRanges = resolveRangeSources(extraRangeSources, context).filter(
      (range) =>
        Number.isFinite(range.startTime) &&
        Number.isFinite(range.endTime) &&
        range.endTime > range.startTime,
    );
    rangeLanes = groupRanges(customRanges);
    const duration = Math.max(context.player.getTimelineDuration(), 0.0001);

    if (rangeLanes.length === 0) {
      rangesRoot.hidden = true;
      return;
    }

    rangesRoot.hidden = false;

    for (const lane of rangeLanes) {
      const laneEl = document.createElement("div");
      laneEl.className = "sap-tl-range-lane";

      const track = document.createElement("div");
      track.className = "sap-tl-range-lane-track";

      if (lane.label) {
        laneEl.dataset.label = lane.label;
        const label = document.createElement("span");
        label.className = "sap-tl-range-lane-label";
        label.textContent = lane.label;
        label.setAttribute("aria-label", lane.label);
        laneEl.append(label);
      }

      for (const range of lane.ranges) {
        const startProjection = context.player.projectReplayTimeToTimeline(range.startTime);
        const endProjection = context.player.projectReplayTimeToTimeline(range.endTime);
        const { startTimelineTime, endTimelineTime } = projectedRangeTimelineBounds(
          startProjection,
          endProjection,
          duration,
        );
        const segment = document.createElement("div");
        segment.className = "sap-tl-range-segment";
        if (range.className) {
          segment.classList.add(range.className);
        }
        segment.style.background = rangeAccent(range);
        segment.title = range.label ?? lane.label;
        segment.dataset.active = "false";
        segment.style.left = markerLeftPercent(startTimelineTime, duration);
        segment.style.width = markerLeftPercent(
          Math.max(0, endTimelineTime - startTimelineTime),
          duration,
        );
        track.append(segment);
        rangeElements.push({
          range,
          element: segment,
          startTimelineTime,
          endTimelineTime,
        });
      }

      const playhead = document.createElement("div");
      playhead.className = "sap-tl-range-playhead";
      track.append(playhead);
      rangeLanePlayheads.push({ element: playhead });

      laneEl.append(track);
      rangesRoot.append(laneEl);
    }
  }

  function buildGraphs(context: ReplayPlayerPluginContext): void {
    if (!graphsRoot) {
      return;
    }

    graphsRoot.replaceChildren();
    graphPlayheads.splice(0, graphPlayheads.length);
    const graphs = resolveGraphSources(extraGraphSources, context);
    if (graphs.length === 0) {
      graphsRoot.hidden = true;
      return;
    }

    graphsRoot.hidden = false;
    const duration = Math.max(context.player.getTimelineDuration(), 0.0001);
    const svgNamespace = "http://www.w3.org/2000/svg";

    for (const graph of graphs) {
      const minValue = graph.minValue ?? 0;
      const maxValue = graph.maxValue ?? 1;
      if (!Number.isFinite(minValue) || !Number.isFinite(maxValue) || maxValue <= minValue) {
        continue;
      }

      const lane = document.createElement("div");
      lane.className = "sap-tl-graph-lane";
      lane.dataset.label = graph.label;

      const label = document.createElement("span");
      label.className = "sap-tl-graph-lane-label";
      label.textContent = graph.label;
      label.setAttribute("aria-label", graph.label);

      const track = document.createElement("div");
      track.className = "sap-tl-graph-lane-track";
      track.tabIndex = 0;
      track.setAttribute("role", "slider");
      track.setAttribute("aria-label", `${graph.label} timeline; click or drag to seek`);
      track.setAttribute("aria-valuemin", "0");
      track.setAttribute("aria-valuemax", `${duration}`);

      const svg = document.createElementNS(svgNamespace, "svg");
      svg.classList.add("sap-tl-graph-svg");
      svg.setAttribute("aria-hidden", "true");
      svg.setAttribute("viewBox", `0 0 ${TIMELINE_GRAPH_WIDTH} ${TIMELINE_GRAPH_HEIGHT}`);
      svg.setAttribute("preserveAspectRatio", "none");

      for (const highlight of graph.highlights ?? []) {
        if (
          !Number.isFinite(highlight.startTime) ||
          !Number.isFinite(highlight.endTime) ||
          highlight.endTime <= highlight.startTime
        ) {
          continue;
        }
        const bounds = projectedRangeTimelineBounds(
          context.player.projectReplayTimeToTimeline(highlight.startTime),
          context.player.projectReplayTimeToTimeline(highlight.endTime),
          duration,
        );
        const rect = document.createElementNS(svgNamespace, "rect");
        rect.classList.add("sap-tl-graph-highlight");
        if (highlight.className) rect.classList.add(highlight.className);
        rect.setAttribute("x", `${(bounds.startTimelineTime / duration) * TIMELINE_GRAPH_WIDTH}`);
        rect.setAttribute("y", "0");
        rect.setAttribute(
          "width",
          `${Math.max(0, ((bounds.endTimelineTime - bounds.startTimelineTime) / duration) * TIMELINE_GRAPH_WIDTH)}`,
        );
        rect.setAttribute("height", `${TIMELINE_GRAPH_HEIGHT}`);
        rect.setAttribute("fill", highlight.color ?? "rgba(255,255,255,0.08)");
        if (highlight.label) {
          const title = document.createElementNS(svgNamespace, "title");
          title.textContent = highlight.label;
          rect.append(title);
        }
        svg.append(rect);
      }

      for (const reference of graph.references ?? []) {
        if (!Number.isFinite(reference.value)) continue;
        const y = timelineGraphY(reference.value, minValue, maxValue);
        const line = document.createElementNS(svgNamespace, "line");
        line.classList.add("sap-tl-graph-reference");
        if (reference.className) line.classList.add(reference.className);
        line.setAttribute("x1", "0");
        line.setAttribute("x2", `${TIMELINE_GRAPH_WIDTH}`);
        line.setAttribute("y1", `${y}`);
        line.setAttribute("y2", `${y}`);
        line.setAttribute("stroke", reference.color ?? "rgba(255,255,255,0.32)");
        svg.append(line);
        if (reference.label) {
          const text = document.createElementNS(svgNamespace, "text");
          text.classList.add("sap-tl-graph-reference-label");
          text.setAttribute("x", "6");
          text.setAttribute("y", `${Math.max(8, y - 3)}`);
          text.setAttribute("fill", reference.color ?? "rgba(255,255,255,0.62)");
          text.textContent = reference.label;
          svg.append(text);
        }
      }

      for (const series of graph.series) {
        const projectedPoints = series.points.map((point) => ({
          time: context.player.projectReplayTimeToTimeline(point.time).timelineTime,
          value: point.value,
        }));
        const path = document.createElementNS(svgNamespace, "path");
        path.classList.add("sap-tl-graph-series");
        path.setAttribute(
          "d",
          buildTimelineGraphPath(projectedPoints, duration, minValue, maxValue),
        );
        path.setAttribute("stroke", series.color);
        if (series.label) {
          const title = document.createElementNS(svgNamespace, "title");
          title.textContent = series.label;
          path.append(title);
        }
        svg.append(path);
      }

      for (const marker of graph.markers ?? []) {
        if (!Number.isFinite(marker.time) || !Number.isFinite(marker.value)) continue;
        const projection = context.player.projectReplayTimeToTimeline(marker.time);
        const circle = document.createElementNS(svgNamespace, "circle");
        circle.classList.add("sap-tl-graph-marker");
        if (marker.className) circle.classList.add(marker.className);
        circle.setAttribute("cx", `${(projection.timelineTime / duration) * TIMELINE_GRAPH_WIDTH}`);
        circle.setAttribute("cy", `${timelineGraphY(marker.value, minValue, maxValue)}`);
        circle.setAttribute("r", "5");
        circle.setAttribute("fill", marker.color ?? "#ffffff");
        if (marker.label) {
          const title = document.createElementNS(svgNamespace, "title");
          title.textContent = marker.label;
          circle.append(title);
        }
        svg.append(circle);
      }

      const playhead = document.createElementNS(svgNamespace, "line");
      playhead.classList.add("sap-tl-graph-playhead");
      playhead.setAttribute("x1", "0");
      playhead.setAttribute("x2", "0");
      playhead.setAttribute("y1", "0");
      playhead.setAttribute("y2", `${TIMELINE_GRAPH_HEIGHT}`);
      svg.append(playhead);
      graphPlayheads.push({ element: playhead, track });

      const seekFromPointer = (event: PointerEvent): void => {
        const rect = track.getBoundingClientRect();
        const ratio = Math.max(
          0,
          Math.min(1, (event.clientX - rect.left) / Math.max(rect.width, 1)),
        );
        context.player.seek(context.player.projectTimelineTimeToReplay(ratio * duration));
      };
      track.addEventListener("pointerdown", (event) => {
        track.setPointerCapture(event.pointerId);
        seekFromPointer(event);
      });
      track.addEventListener("pointermove", (event) => {
        if (track.hasPointerCapture(event.pointerId)) seekFromPointer(event);
      });
      track.addEventListener("keydown", (event) => {
        const current = context.player.getTimelineCurrentTime();
        const increments: Partial<Record<string, number>> = {
          ArrowLeft: -1,
          ArrowRight: 1,
          PageDown: -10,
          PageUp: 10,
        };
        let next = increments[event.key] === undefined ? null : current + increments[event.key]!;
        if (event.key === "Home") next = 0;
        if (event.key === "End") next = duration;
        if (next === null) return;
        event.preventDefault();
        context.player.seek(
          context.player.projectTimelineTimeToReplay(Math.max(0, Math.min(duration, next))),
        );
      });

      track.append(svg);
      lane.append(label, track);
      graphsRoot.append(lane);
    }
  }

  function endScrub(): void {
    if (!scrubbing) {
      return;
    }

    scrubbing = false;
    shell?.setAttribute("data-scrubbing", "false");
    if (resumePlaybackAfterScrub) {
      playerContext?.player.play();
    }
    resumePlaybackAfterScrub = false;
  }

  function beginScrub(): void {
    if (scrubbing) {
      return;
    }

    scrubbing = true;
    shell?.setAttribute("data-scrubbing", "true");
    if (!pauseWhileScrubbing) {
      return;
    }

    const player = playerContext?.player;
    if (!player) {
      return;
    }

    resumePlaybackAfterScrub = player.getState().playing;
    if (resumePlaybackAfterScrub) {
      player.pause();
    }
  }

  return {
    id: "timeline-overlay",
    addEventSource(source, sourceOptions = {}): () => void {
      extraEventSources.push({
        key: sourceOptions.id ?? `events:${nextEventSourceId++}`,
        label: sourceOptions.label ?? "Events",
        source,
      });
      refreshMarkers();
      return () => {
        this.removeEventSource(source);
      };
    },
    removeEventSource(source): boolean {
      const index = extraEventSources.findIndex((record) => record.source === source);
      if (index < 0) {
        return false;
      }

      extraEventSources.splice(index, 1);
      refreshMarkers();
      return true;
    },
    refreshEvents(): void {
      refreshMarkers();
    },
    addRangeSource(source): () => void {
      extraRangeSources.push(source);
      refreshRangeLanes();
      return () => {
        this.removeRangeSource(source);
      };
    },
    removeRangeSource(source): boolean {
      const index = extraRangeSources.indexOf(source);
      if (index < 0) {
        return false;
      }

      extraRangeSources.splice(index, 1);
      refreshRangeLanes();
      return true;
    },
    refreshRanges(): void {
      refreshRangeLanes();
    },
    addGraphSource(source): () => void {
      extraGraphSources.push(source);
      refreshGraphLanes();
      return () => {
        this.removeGraphSource(source);
      };
    },
    removeGraphSource(source): boolean {
      const index = extraGraphSources.indexOf(source);
      if (index < 0) {
        return false;
      }
      extraGraphSources.splice(index, 1);
      refreshGraphLanes();
      return true;
    },
    refreshGraphs(): void {
      refreshGraphLanes();
    },
    setup(context): void {
      playerContext = context;
      ensureTimelineOverlayStyles();

      if (getComputedStyle(context.container).position === "static") {
        changedContainerPosition = true;
        originalContainerPosition = context.container.style.position;
        context.container.style.position = "relative";
      }

      root = document.createElement("div");
      root.className = "sap-tl-root";
      shell = document.createElement("div");
      shell.className = "sap-tl-shell";
      shell.dataset.scrubbing = "false";

      const topLine = document.createElement("div");
      topLine.className = "sap-tl-topline";

      const primary = document.createElement("div");
      primary.className = "sap-tl-primary";

      toggleButton = document.createElement("button");
      toggleButton.type = "button";
      toggleButton.className = "sap-tl-toggle sap-tl-track-toggle";
      toggleButtonIcon = document.createElement("span");
      toggleButtonIcon.className = "sap-tl-toggle-icon";
      toggleButtonIcon.setAttribute("aria-hidden", "true");
      toggleButtonIcon.textContent = ">";
      toggleButtonLabel = document.createElement("span");
      toggleButtonLabel.className = "sap-tl-toggle-label";
      toggleButtonLabel.textContent = "Play";
      toggleButton.append(toggleButtonIcon, toggleButtonLabel);
      toggleButton.addEventListener("click", () => {
        context.player.togglePlayback();
      });

      currentTimeText = document.createElement("span");
      currentTimeText.className = "sap-tl-current";
      currentTimeText.textContent = "0:00.00";

      remainingTimeText = document.createElement("span");
      remainingTimeText.className = "sap-tl-remaining";
      remainingTimeText.textContent = "-0:00.00";

      primary.append(currentTimeText);
      topLine.append(primary, remainingTimeText);

      const trackWrap = document.createElement("div");
      trackWrap.className = "sap-tl-track-wrap";

      rangesRoot = document.createElement("div");
      rangesRoot.className = "sap-tl-ranges";
      rangesRoot.hidden = true;

      graphsRoot = document.createElement("div");
      graphsRoot.className = "sap-tl-graphs";
      graphsRoot.hidden = true;

      eventLanesRoot = document.createElement("div");
      eventLanesRoot.className = "sap-tl-event-lanes";
      eventLanesRoot.hidden = true;

      const trackRail = document.createElement("div");
      trackRail.className = "sap-tl-track-rail";

      const mainRail = document.createElement("div");
      mainRail.className = "sap-tl-main-rail";

      markers = document.createElement("div");
      markers.className = "sap-tl-markers";

      range = document.createElement("input");
      range.className = "sap-tl-range";
      range.type = "range";
      range.min = "0";
      range.max = `${context.replay.duration}`;
      range.step = "0.01";
      range.value = "0";

      const handlePointerDown = (): void => {
        beginScrub();
      };
      const handleInput = (): void => {
        if (!range) {
          return;
        }

        context.player.seek(context.player.projectTimelineTimeToReplay(Number(range.value)));
      };
      const handleWindowPointerUp = (): void => {
        endScrub();
      };

      range.addEventListener("pointerdown", handlePointerDown);
      range.addEventListener("input", handleInput);
      range.addEventListener("change", handleWindowPointerUp);
      window.addEventListener("pointerup", handleWindowPointerUp);
      window.addEventListener("pointercancel", handleWindowPointerUp);
      removeWindowListeners = (): void => {
        range?.removeEventListener("pointerdown", handlePointerDown);
        range?.removeEventListener("input", handleInput);
        range?.removeEventListener("change", handleWindowPointerUp);
        window.removeEventListener("pointerup", handleWindowPointerUp);
        window.removeEventListener("pointercancel", handleWindowPointerUp);
      };

      trackRail.append(mainRail, markers, range);
      trackWrap.append(graphsRoot, rangesRoot, eventLanesRoot, toggleButton, trackRail);
      shell.append(topLine, trackWrap);
      root.append(shell);
      context.container.append(root);
      buildMarkers(context);
      buildRanges(context);
      buildGraphs(context);
      syncState({
        ...context,
        state: context.player.getState(),
      });
    },
    onStateChange(context): void {
      playerContext = context;
      syncState(context);
    },
    teardown(context): void {
      removeWindowListeners?.();
      removeWindowListeners = null;
      endScrub();
      root?.remove();
      root = null;
      shell = null;
      rangesRoot = null;
      graphsRoot = null;
      eventLanesRoot = null;
      range = null;
      toggleButton = null;
      toggleButtonIcon = null;
      toggleButtonLabel = null;
      currentTimeText = null;
      remainingTimeText = null;
      markers = null;
      playerContext = null;
      eventLanes = [];
      rangeLanes = [];
      projectionCacheKey = null;
      markerElements.clear();
      timelineMarkers.splice(0, timelineMarkers.length);
      rangeElements.splice(0, rangeElements.length);
      rangeLanePlayheads.splice(0, rangeLanePlayheads.length);
      graphPlayheads.splice(0, graphPlayheads.length);
      eventLanePlayheads.splice(0, eventLanePlayheads.length);
      passedMarkerEndIndex = 0;
      activeMarkers = new Set<TimelineMarkerRecord>();
      if (changedContainerPosition) {
        context.container.style.position = originalContainerPosition;
        changedContainerPosition = false;
      }
    },
  };
}
