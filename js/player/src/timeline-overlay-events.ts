import type {
  ReplayPlayerPluginContext,
  ReplayTimelineEvent,
  ReplayTimelineEventKind,
  ReplayTimelineEventSource,
} from "./types";
import {
  DEFAULT_REPLAY_EVENT_KINDS,
  bucketTitle,
  eventAccent,
  eventBadgeText,
  groupEvents,
  markerLeftPercent,
  resolveCustomEvents,
  resolveEventSources,
  timelineEventSeekTime,
  type TimelineEventBucket,
  type TimelineEventLane,
  type TimelineEventSourceRecord,
} from "./timeline-overlay-model";

const HIDDEN_EVENT_SEEK_EPSILON_SECONDS = 0.01;

export interface TimelineEventLanePlayhead {
  element: HTMLDivElement;
}

export interface TimelineMarkerRecord {
  element: HTMLButtonElement;
  timelineTime: number;
}

interface TimelineEventRenderOptions {
  includeReplayEvents?: boolean;
  replayEventKinds?: Iterable<ReplayTimelineEventKind>;
  replayEventsLabel?: string;
  replayEvents?: ReplayTimelineEventSource;
}

interface TimelineEventRenderResult {
  markerElements: Map<string, TimelineMarkerRecord>;
  eventLanePlayheads: TimelineEventLanePlayhead[];
}

function resolveReplayEvents(
  options: TimelineEventRenderOptions,
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

function createMarker(
  bucket: TimelineEventBucket,
  context: ReplayPlayerPluginContext,
  duration: number,
  markerElements: Map<string, TimelineMarkerRecord>,
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

export function renderTimelineEventLanes(
  markersRoot: HTMLDivElement,
  eventLanesRoot: HTMLDivElement,
  options: TimelineEventRenderOptions,
  eventSources: readonly TimelineEventSourceRecord[],
  context: ReplayPlayerPluginContext,
): TimelineEventRenderResult {
  markersRoot.replaceChildren();
  eventLanesRoot.replaceChildren();

  const markerElements = new Map<string, TimelineMarkerRecord>();
  const eventLanePlayheads: TimelineEventLanePlayhead[] = [];
  const eventLanes: TimelineEventLane[] = [];
  const replayEvents = resolveReplayEvents(options, context);
  if (replayEvents.length > 0) {
    eventLanes.push({
      key: "replay",
      label: options.replayEventsLabel ?? "Replay",
      buckets: groupEvents(replayEvents),
    });
  }
  eventLanes.push(...resolveEventSources(eventSources, context));
  const duration = Math.max(context.player.getTimelineDuration(), 0.0001);

  const replayLane = eventLanes[0];
  if (replayLane?.key === "replay") {
    for (const bucket of replayLane.buckets) {
      const marker = createMarker(
        { ...bucket, key: `${replayLane.key}:${bucket.key}` },
        context,
        duration,
        markerElements,
      );
      if (marker) {
        markersRoot.append(marker);
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
        markerElements,
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

  return { markerElements, eventLanePlayheads };
}
