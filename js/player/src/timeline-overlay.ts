import type {
  ReplayPlayerPlugin,
  ReplayPlayerPluginContext,
  ReplayPlayerPluginStateContext,
  ReplayTimelineEvent,
  ReplayTimelineEventKind,
  ReplayTimelineEventSource,
  ReplayTimelineRangeSource,
} from "./types";
import { createTimelineOverlayElements } from "./timeline-overlay-dom";
import {
  renderTimelineRangeLanes,
  type TimelineRangeLanePlayhead,
  type TimelineRangeRecord,
} from "./timeline-overlay-ranges";
import { ensureTimelineOverlayStyles } from "./timeline-overlay-styles";
import {
  DEFAULT_REPLAY_EVENT_KINDS,
  bucketTitle,
  eventAccent,
  eventBadgeText,
  formatPlaybackTime,
  groupEvents,
  markerLeftPercent,
  resolveCustomEvents,
  resolveEventSources,
  timelineEventSeekTime,
  type TimelineEventBucket,
  type TimelineEventLane,
  type TimelineEventSourceRecord,
} from "./timeline-overlay-model";

export { projectedRangeTimelineBounds, timelineEventSeekTime } from "./timeline-overlay-model";

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

interface TimelineEventLanePlayhead {
  element: HTMLDivElement;
}

interface TimelineMarkerRecord {
  element: HTMLButtonElement;
  timelineTime: number;
}

const ACTIVE_MARKER_WINDOW_SECONDS = 0.2;
const HIDDEN_EVENT_SEEK_EPSILON_SECONDS = 0.01;

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
  let projectionCacheKey: string | null = null;
  const markerElements = new Map<string, TimelineMarkerRecord>();
  let rangeElements: TimelineRangeRecord[] = [];
  let rangeLanePlayheads: TimelineRangeLanePlayhead[] = [];
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

    ({ rangeElements, rangeLanePlayheads } = renderTimelineRangeLanes(
      rangesRoot,
      extraRangeSources,
      context,
    ));
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

      const elementRefs = createTimelineOverlayElements(context, {
        beginScrub,
        endScrub,
      });
      root = elementRefs.root;
      shell = elementRefs.shell;
      rangesRoot = elementRefs.rangesRoot;
      range = elementRefs.range;
      toggleButton = elementRefs.toggleButton;
      toggleButtonIcon = elementRefs.toggleButtonIcon;
      toggleButtonLabel = elementRefs.toggleButtonLabel;
      currentTimeText = elementRefs.currentTimeText;
      remainingTimeText = elementRefs.remainingTimeText;
      eventLanesRoot = elementRefs.eventLanesRoot;
      markers = elementRefs.markers;
      removeWindowListeners = elementRefs.removeWindowListeners;
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
      projectionCacheKey = null;
      markerElements.clear();
      rangeElements = [];
      rangeLanePlayheads = [];
      eventLanePlayheads.splice(0, eventLanePlayheads.length);
      if (changedContainerPosition) {
        context.container.style.position = originalContainerPosition;
        changedContainerPosition = false;
      }
    },
  };
}
