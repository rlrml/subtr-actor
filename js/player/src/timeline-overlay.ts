import type {
  ReplayPlayerPlugin,
  ReplayPlayerPluginContext,
  ReplayPlayerPluginStateContext,
  ReplayPlayerTimelineProjection,
  ReplayTimelineEvent,
  ReplayTimelineEventKind,
  ReplayTimelineEventSource,
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

interface TimelineEventLanePlayhead {
  element: HTMLDivElement;
}

interface TimelineMarkerRecord {
  element: HTMLButtonElement;
  timelineTime: number;
}

const DEFAULT_REPLAY_EVENT_KINDS = new Set<ReplayTimelineEventKind>(["goal", "save", "bookmark"]);
const ACTIVE_MARKER_WINDOW_SECONDS = 0.2;
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

  let root: HTMLDivElement | null = null;
  let shell: HTMLDivElement | null = null;
  let rangesRoot: HTMLDivElement | null = null;
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
  const rangeElements: TimelineRangeRecord[] = [];
  const rangeLanePlayheads: TimelineRangeLanePlayhead[] = [];
  const eventLanePlayheads: TimelineEventLanePlayhead[] = [];

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

    for (const markerRecord of markerElements.values()) {
      const timeSinceEvent = currentTime - markerRecord.timelineTime;
      const active = timeSinceEvent >= 0 && timeSinceEvent <= ACTIVE_MARKER_WINDOW_SECONDS;
      markerRecord.element.dataset.active = active ? "true" : "false";
      markerRecord.element.dataset.passed =
        markerRecord.timelineTime <= currentTime ? "true" : "false";
    }

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

    markerElements.set(bucket.key, {
      element: marker,
      timelineTime: projection.timelineTime,
    });

    return marker;
  }

  function buildMarkers(context: ReplayPlayerPluginContext): void {
    if (!markers || !eventLanesRoot) {
      return;
    }

    markers.replaceChildren();
    eventLanesRoot.replaceChildren();
    markerElements.clear();
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
      for (const bucket of replayLane.buckets) {
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

      for (const bucket of lane.buckets) {
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
      trackWrap.append(rangesRoot, eventLanesRoot, toggleButton, trackRail);
      shell.append(topLine, trackWrap);
      root.append(shell);
      context.container.append(root);
      buildMarkers(context);
      buildRanges(context);
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
      rangeElements.splice(0, rangeElements.length);
      rangeLanePlayheads.splice(0, rangeLanePlayheads.length);
      eventLanePlayheads.splice(0, eventLanePlayheads.length);
      if (changedContainerPosition) {
        context.container.style.position = originalContainerPosition;
        changedContainerPosition = false;
      }
    },
  };
}
