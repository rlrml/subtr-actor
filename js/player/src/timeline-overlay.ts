import type {
  ReplayPlayerPlugin,
  ReplayPlayerPluginContext,
  ReplayPlayerPluginStateContext,
  ReplayTimelineEvent,
  ReplayTimelineEventKind,
  ReplayTimelineEventSource,
  ReplayTimelineRange,
  ReplayTimelineRangeSource,
} from "./types";

export interface TimelineOverlayPluginOptions {
  pauseWhileScrubbing?: boolean;
  includeReplayEvents?: boolean;
  replayEventKinds?: Iterable<ReplayTimelineEventKind>;
  replayEvents?: ReplayTimelineEventSource;
  events?: ReplayTimelineEventSource;
  ranges?: ReplayTimelineRangeSource;
}

export interface TimelineOverlayPlugin extends ReplayPlayerPlugin {
  addEventSource(source: ReplayTimelineEventSource): () => void;
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

interface TimelineRangeLane {
  key: string;
  label: string;
  ranges: ReplayTimelineRange[];
}

interface TimelineRangeRecord {
  range: ReplayTimelineRange;
  element: HTMLDivElement;
}

const STYLE_ID = "subtr-actor-timeline-overlay-styles";
const DEFAULT_REPLAY_EVENT_KINDS = new Set<ReplayTimelineEventKind>([
  "goal",
  "save",
]);
const ACTIVE_MARKER_WINDOW_SECONDS = 0.2;
const HIDDEN_EVENT_SEEK_EPSILON_SECONDS = 0.01;

function ensureStyles(): void {
  if (document.getElementById(STYLE_ID)) {
    return;
  }

  const style = document.createElement("style");
  style.id = STYLE_ID;
  style.textContent = `
    .sap-tl-root {
      position: absolute;
      inset: 0;
      z-index: 4;
      pointer-events: none;
      overflow: hidden;
      font-family: "IBM Plex Sans", "Segoe UI", Roboto, sans-serif;
    }

    .sap-tl-shell {
      --sap-tl-thumb-size: 1.35rem;
      position: absolute;
      left: 0.8rem;
      right: 0.8rem;
      bottom: 0.9rem;
      padding: 0.75rem 0.9rem 0.9rem;
      border: 1px solid rgba(180, 205, 226, 0.18);
      border-radius: 1.05rem;
      background:
        linear-gradient(180deg, rgba(13, 20, 28, 0.92), rgba(7, 12, 18, 0.96));
      box-shadow: 0 18px 42px rgba(0, 0, 0, 0.28);
      backdrop-filter: blur(12px);
      pointer-events: auto;
    }

    .sap-tl-shell::before {
      content: "";
      position: absolute;
      inset: 0;
      border-radius: inherit;
      background:
        linear-gradient(90deg, rgba(60, 134, 255, 0.18), transparent 28%, transparent 72%, rgba(242, 138, 37, 0.16));
      pointer-events: none;
    }

    .sap-tl-topline {
      position: relative;
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 0.55rem;
      color: #f5fbff;
      font-size: 0.82rem;
      font-weight: 600;
      font-variant-numeric: tabular-nums;
      gap: 0.85rem;
    }

    .sap-tl-primary {
      display: flex;
      align-items: center;
      gap: 0.65rem;
      min-width: 0;
    }

    .sap-tl-toggle {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      gap: 0.4rem;
      min-width: 4.9rem;
      padding: 0.42rem 0.72rem;
      border: 1px solid rgba(184, 214, 236, 0.24);
      border-radius: 999px;
      background: rgba(18, 30, 42, 0.92);
      color: #f5fbff;
      font: inherit;
      font-size: 0.76rem;
      font-weight: 700;
      letter-spacing: 0.02em;
      cursor: pointer;
      transition:
        transform 140ms ease,
        border-color 140ms ease,
        background 140ms ease;
    }

    .sap-tl-toggle:hover {
      border-color: rgba(184, 214, 236, 0.4);
      background: rgba(28, 45, 61, 0.96);
      transform: translateY(-1px);
    }

    .sap-tl-toggle:focus-visible {
      outline: 2px solid rgba(123, 180, 255, 0.9);
      outline-offset: 2px;
    }

    .sap-tl-toggle-icon {
      width: 0.85rem;
      text-align: center;
      font-size: 0.7rem;
      line-height: 1;
    }

    .sap-tl-current {
      color: #f5fbff;
    }

    .sap-tl-remaining {
      color: #b8c9d9;
    }

    .sap-tl-track-wrap {
      position: relative;
    }

    .sap-tl-ranges {
      display: flex;
      flex-direction: column;
      gap: 0.34rem;
      margin-bottom: 0.58rem;
      pointer-events: none;
    }

    .sap-tl-range-lane {
      position: relative;
      padding-left: 0.15rem;
    }

    .sap-tl-range-lane-track {
      position: relative;
      height: 0.55rem;
      margin: 0 calc(var(--sap-tl-thumb-size) / 2);
      border-radius: 999px;
      background: rgba(255, 255, 255, 0.06);
      box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.08);
      overflow: hidden;
    }

    .sap-tl-range-lane-label {
      position: absolute;
      left: 0.4rem;
      top: 50%;
      z-index: 2;
      padding: 0.08rem 0.38rem;
      border: 1px solid rgba(184, 214, 236, 0.18);
      border-radius: 999px;
      background: rgba(10, 16, 23, 0.82);
      color: #c8d7e4;
      font-size: 0.54rem;
      font-weight: 800;
      letter-spacing: 0.04em;
      text-transform: uppercase;
      transform: translateY(-50%);
      backdrop-filter: blur(6px);
    }

    .sap-tl-range-segment {
      position: absolute;
      top: 0;
      bottom: 0;
      min-width: 2px;
      border-radius: 999px;
      opacity: 0.62;
      transition:
        opacity 120ms ease,
        filter 120ms ease,
        transform 120ms ease;
    }

    .sap-tl-range-segment[data-active="true"] {
      opacity: 0.92;
      filter: brightness(1.12);
      transform: scaleY(1.06);
    }

    .sap-tl-track-rail {
      position: relative;
      padding-top: 1.05rem;
    }

    .sap-tl-range {
      position: relative;
      z-index: 2;
      width: 100%;
      margin: 0;
      appearance: none;
      background: transparent;
      cursor: pointer;
    }

    .sap-tl-range:focus {
      outline: none;
    }

    .sap-tl-range::-webkit-slider-runnable-track {
      height: 0.6rem;
      border-radius: 999px;
      border: 1px solid rgba(255, 255, 255, 0.12);
      background:
        linear-gradient(90deg, rgba(60, 134, 255, 0.42), rgba(103, 179, 255, 0.58) 45%, rgba(242, 138, 37, 0.58));
      box-shadow: inset 0 0 0 999px rgba(5, 10, 15, 0.4);
    }

    .sap-tl-range::-moz-range-track {
      height: 0.6rem;
      border-radius: 999px;
      border: 1px solid rgba(255, 255, 255, 0.12);
      background:
        linear-gradient(90deg, rgba(60, 134, 255, 0.42), rgba(103, 179, 255, 0.58) 45%, rgba(242, 138, 37, 0.58));
      box-shadow: inset 0 0 0 999px rgba(5, 10, 15, 0.4);
    }

    .sap-tl-range::-webkit-slider-thumb {
      appearance: none;
      margin-top: -0.38rem;
      width: var(--sap-tl-thumb-size);
      height: var(--sap-tl-thumb-size);
      border: 0;
      border-radius: 50%;
      background:
        radial-gradient(circle at 35% 35%, #ffffff 0%, #d8ebff 28%, #7bb4ff 55%, #27456d 100%);
      box-shadow: 0 8px 22px rgba(0, 0, 0, 0.34);
    }

    .sap-tl-range::-moz-range-thumb {
      width: var(--sap-tl-thumb-size);
      height: var(--sap-tl-thumb-size);
      border: 0;
      border-radius: 50%;
      background:
        radial-gradient(circle at 35% 35%, #ffffff 0%, #d8ebff 28%, #7bb4ff 55%, #27456d 100%);
      box-shadow: 0 8px 22px rgba(0, 0, 0, 0.34);
    }

    .sap-tl-shell[data-scrubbing="true"] .sap-tl-range::-webkit-slider-thumb,
    .sap-tl-shell[data-scrubbing="true"] .sap-tl-range::-moz-range-thumb {
      background:
        radial-gradient(circle at 35% 35%, #ffffff 0%, #ffe5c5 32%, #ffad47 58%, #7b3d00 100%);
      transform: scale(1.05);
    }

    .sap-tl-markers {
      position: absolute;
      inset: 0 calc(var(--sap-tl-thumb-size) / 2) auto;
      height: 1rem;
      pointer-events: none;
      z-index: 1;
    }

    .sap-tl-marker {
      position: absolute;
      top: 0;
      transform: translateX(-50%);
      width: 0.95rem;
      height: 0.95rem;
      padding: 0;
      border: 0;
      border-radius: 999px;
      background: rgba(12, 18, 24, 0.96);
      color: #f5fbff;
      font-size: 0.52rem;
      font-weight: 800;
      line-height: 1;
      box-shadow: 0 4px 14px rgba(0, 0, 0, 0.3);
      pointer-events: auto;
      cursor: pointer;
    }

    .sap-tl-marker::before {
      content: "";
      position: absolute;
      left: 50%;
      top: 0.85rem;
      width: 2px;
      height: 0.55rem;
      transform: translateX(-50%);
      background: currentColor;
      opacity: 0.7;
    }

    .sap-tl-marker:hover {
      filter: brightness(1.08);
    }

    .sap-tl-marker[data-passed="true"] {
      opacity: 0.9;
    }

    .sap-tl-marker[data-active="true"] {
      transform: translateX(-50%) scale(1.16);
      opacity: 1;
      box-shadow: 0 6px 18px rgba(0, 0, 0, 0.38);
    }

    @media (max-width: 720px) {
      .sap-tl-shell {
        bottom: 0.6rem;
        left: 0.5rem;
        right: 0.5rem;
        padding: 0.65rem 0.7rem 0.75rem;
      }

      .sap-tl-topline {
        font-size: 0.72rem;
      }
    }
  `;
  document.head.append(style);
}

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
      return 1;
    default:
      return 0;
  }
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
      event.frame !== undefined
        ? `frame:${event.frame}`
        : `time:${event.time.toFixed(2)}`;
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
  context: ReplayPlayerPluginContext
): ReplayTimelineEvent[] {
  if (!source) {
    return [];
  }

  return typeof source === "function" ? source(context) : source;
}

function resolveEventSources(
  sources: Iterable<ReplayTimelineEventSource>,
  context: ReplayPlayerPluginContext
): ReplayTimelineEvent[] {
  const events: ReplayTimelineEvent[] = [];
  for (const source of sources) {
    events.push(...resolveCustomEvents(source, context));
  }
  return events;
}

function resolveCustomRanges(
  source: ReplayTimelineRangeSource | undefined,
  context: ReplayPlayerPluginContext
): ReplayTimelineRange[] {
  if (!source) {
    return [];
  }

  return typeof source === "function" ? source(context) : source;
}

function resolveRangeSources(
  sources: Iterable<ReplayTimelineRangeSource>,
  context: ReplayPlayerPluginContext
): ReplayTimelineRange[] {
  const ranges: ReplayTimelineRange[] = [];
  for (const source of sources) {
    ranges.push(...resolveCustomRanges(source, context));
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
  context: ReplayPlayerPluginContext
): ReplayTimelineEvent[] {
  if (options.replayEvents) {
    return resolveCustomEvents(options.replayEvents, context);
  }

  if (options.includeReplayEvents === false) {
    return [];
  }

  const allowedKinds = new Set(
    options.replayEventKinds ?? DEFAULT_REPLAY_EVENT_KINDS
  );
  return context.replay.timelineEvents.filter((event) =>
    allowedKinds.has(event.kind)
  );
}

function markerSeekTime(
  eventTime: number,
  context: ReplayPlayerPluginContext
): number {
  const projection = context.player.projectReplayTimeToTimeline(eventTime);
  if (!projection.hiddenBySkip) {
    return projection.seekTime;
  }

  const nextTimelineTime = Math.min(
    context.player.getTimelineDuration(),
    projection.timelineTime + HIDDEN_EVENT_SEEK_EPSILON_SECONDS
  );
  return context.player.projectTimelineTimeToReplay(nextTimelineTime);
}

function markerLeftPercent(
  timelineTime: number,
  duration: number
): string {
  return `${(timelineTime / Math.max(duration, 0.0001)) * 100}%`;
}

export function createTimelineOverlayPlugin(
  options: TimelineOverlayPluginOptions = {}
): TimelineOverlayPlugin {
  const pauseWhileScrubbing = options.pauseWhileScrubbing ?? true;
  const extraEventSources = options.events ? [options.events] : [];
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
  let markers: HTMLDivElement | null = null;
  let removeWindowListeners: (() => void) | null = null;
  let changedContainerPosition = false;
  let originalContainerPosition = "";
  let scrubbing = false;
  let resumePlaybackAfterScrub = false;
  let playerContext: ReplayPlayerPluginContext | null = null;
  let eventBuckets: TimelineEventBucket[] = [];
  let rangeLanes: TimelineRangeLane[] = [];
  const markerElements = new Map<string, HTMLButtonElement>();
  const rangeElements: TimelineRangeRecord[] = [];

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
    range.min = "0";
    range.max = `${duration}`;
    range.step = "0.01";
    range.value = `${Math.min(currentTime, duration)}`;
    toggleButton.dataset.playing = context.state.playing ? "true" : "false";
    toggleButton.setAttribute(
      "aria-label",
      context.state.playing ? "Pause replay" : "Play replay"
    );
    toggleButton.title = context.state.playing ? "Pause replay" : "Play replay";
    toggleButtonIcon.textContent = context.state.playing ? "||" : ">";
    toggleButtonLabel.textContent = context.state.playing ? "Pause" : "Play";
    currentTimeText.textContent = formatPlaybackTime(currentTime);
    remainingTimeText.textContent = `-${formatPlaybackTime(duration - currentTime)}`;
    shell.dataset.scrubbing = scrubbing ? "true" : "false";

    for (const bucket of eventBuckets) {
      const marker = markerElements.get(bucket.key);
      if (!marker) {
        continue;
      }
      const projection = context.player.projectReplayTimeToTimeline(bucket.time);
      marker.style.left = markerLeftPercent(projection.timelineTime, duration);
      const timeSinceEvent = currentTime - projection.timelineTime;
      const active =
        timeSinceEvent >= 0 && timeSinceEvent <= ACTIVE_MARKER_WINDOW_SECONDS;
      marker.dataset.active = active ? "true" : "false";
      marker.dataset.passed =
        projection.timelineTime <= currentTime ? "true" : "false";
    }

    for (const record of rangeElements) {
      const startProjection = context.player.projectReplayTimeToTimeline(
        record.range.startTime
      );
      const endProjection = context.player.projectReplayTimeToTimeline(
        record.range.endTime
      );
      const leftTime = Math.max(0, startProjection.timelineTime);
      const rightTime = Math.min(duration, endProjection.timelineTime);
      const widthTime = Math.max(0, rightTime - leftTime);

      if (widthTime <= 0.0001) {
        record.element.hidden = true;
        continue;
      }

      record.element.hidden = false;
      record.element.style.left = markerLeftPercent(leftTime, duration);
      record.element.style.width = markerLeftPercent(widthTime, duration);
      record.element.dataset.active =
        currentTime >= leftTime && currentTime <= rightTime ? "true" : "false";
    }
  }

  function buildMarkers(
    context: ReplayPlayerPluginContext
  ): void {
    if (!markers) {
      return;
    }

    markers.replaceChildren();
    markerElements.clear();

    const replayEvents = resolveReplayEvents(options, context);
    const customEvents = resolveEventSources(extraEventSources, context);
    eventBuckets = groupEvents([...replayEvents, ...customEvents]);
    const duration = Math.max(context.player.getTimelineDuration(), 0.0001);

    for (const bucket of eventBuckets) {
      const primaryEvent = bucket.events[0];
      if (!primaryEvent) {
        continue;
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
        context.player.seek(markerSeekTime(bucket.time, context));
      });
      marker.dataset.active = "false";
      marker.dataset.passed = "false";
      markers.append(marker);
      markerElements.set(bucket.key, marker);
    }
  }

  function buildRanges(context: ReplayPlayerPluginContext): void {
    if (!rangesRoot) {
      return;
    }

    rangesRoot.replaceChildren();
    rangeElements.splice(0, rangeElements.length);

    const customRanges = resolveRangeSources(extraRangeSources, context).filter((range) =>
      Number.isFinite(range.startTime) &&
      Number.isFinite(range.endTime) &&
      range.endTime > range.startTime
    );
    rangeLanes = groupRanges(customRanges);

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
        const label = document.createElement("span");
        label.className = "sap-tl-range-lane-label";
        label.textContent = lane.label;
        laneEl.append(label);
      }

      for (const range of lane.ranges) {
        const segment = document.createElement("div");
        segment.className = "sap-tl-range-segment";
        if (range.className) {
          segment.classList.add(range.className);
        }
        segment.style.background = rangeAccent(range);
        segment.title = range.label ?? lane.label;
        segment.dataset.active = "false";
        track.append(segment);
        rangeElements.push({ range, element: segment });
      }

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
    addEventSource(source): () => void {
      extraEventSources.push(source);
      refreshMarkers();
      return () => {
        this.removeEventSource(source);
      };
    },
    removeEventSource(source): boolean {
      const index = extraEventSources.indexOf(source);
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
      ensureStyles();

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
      toggleButton.className = "sap-tl-toggle";
      toggleButtonIcon = document.createElement("span");
      toggleButtonIcon.className = "sap-tl-toggle-icon";
      toggleButtonIcon.setAttribute("aria-hidden", "true");
      toggleButtonIcon.textContent = ">";
      toggleButtonLabel = document.createElement("span");
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

      primary.append(toggleButton, currentTimeText);
      topLine.append(primary, remainingTimeText);

      const trackWrap = document.createElement("div");
      trackWrap.className = "sap-tl-track-wrap";

      rangesRoot = document.createElement("div");
      rangesRoot.className = "sap-tl-ranges";
      rangesRoot.hidden = true;

      const trackRail = document.createElement("div");
      trackRail.className = "sap-tl-track-rail";

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

        context.player.seek(
          context.player.projectTimelineTimeToReplay(Number(range.value))
        );
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

      trackRail.append(markers, range);
      trackWrap.append(rangesRoot, trackRail);
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
      range = null;
      toggleButton = null;
      toggleButtonIcon = null;
      toggleButtonLabel = null;
      currentTimeText = null;
      remainingTimeText = null;
      markers = null;
      playerContext = null;
      eventBuckets = [];
      rangeLanes = [];
      markerElements.clear();
      rangeElements.splice(0, rangeElements.length);
      if (changedContainerPosition) {
        context.container.style.position = originalContainerPosition;
        changedContainerPosition = false;
      }
    },
  };
}
