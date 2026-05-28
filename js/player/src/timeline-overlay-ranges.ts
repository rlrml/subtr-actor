import type {
  ReplayPlayerPluginContext,
  ReplayTimelineRange,
  ReplayTimelineRangeSource,
} from "./types";
import {
  groupRanges,
  markerLeftPercent,
  projectedRangeTimelineBounds,
  rangeAccent,
  resolveRangeSources,
} from "./timeline-overlay-model";

export interface TimelineRangeRecord {
  range: ReplayTimelineRange;
  element: HTMLDivElement;
  startTimelineTime: number;
  endTimelineTime: number;
}

export interface TimelineRangeLanePlayhead {
  element: HTMLDivElement;
}

interface TimelineRangeRenderResult {
  rangeElements: TimelineRangeRecord[];
  rangeLanePlayheads: TimelineRangeLanePlayhead[];
}

export function renderTimelineRangeLanes(
  rangesRoot: HTMLDivElement,
  rangeSources: readonly ReplayTimelineRangeSource[],
  context: ReplayPlayerPluginContext,
): TimelineRangeRenderResult {
  rangesRoot.replaceChildren();

  const rangeElements: TimelineRangeRecord[] = [];
  const rangeLanePlayheads: TimelineRangeLanePlayhead[] = [];
  const customRanges = resolveRangeSources(rangeSources, context).filter(
    (range) =>
      Number.isFinite(range.startTime) &&
      Number.isFinite(range.endTime) &&
      range.endTime > range.startTime,
  );
  const rangeLanes = groupRanges(customRanges);
  const duration = Math.max(context.player.getTimelineDuration(), 0.0001);

  if (rangeLanes.length === 0) {
    rangesRoot.hidden = true;
    return { rangeElements, rangeLanePlayheads };
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

  return { rangeElements, rangeLanePlayheads };
}
