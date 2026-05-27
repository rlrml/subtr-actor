export const DELTA_EPSILON = 0.0001;

const RANGE_MERGE_EPSILON_SECONDS = 0.02;

interface ReplayFrameTimes {
  frames?: Array<{ time: number } | undefined>;
}

interface TimelineRangeLike {
  startTime: number;
  endTime: number;
  lane?: string;
  label?: string;
}

export function sortTimelineEvents<T extends { frame: number; time: number }>(
  events: readonly T[],
): T[] {
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

export function resolveRangeBounds(
  frame: { frame_number: number; time: number; dt: number },
  previousFrame: { frame_number: number; time: number } | null,
  replay?: ReplayFrameTimes,
): { startTime: number; endTime: number } {
  const endTime = replay?.frames?.[frame.frame_number]?.time ?? frame.time;
  const startTime = previousFrame
    ? (replay?.frames?.[previousFrame.frame_number]?.time ?? previousFrame.time)
    : Math.max(0, endTime - frame.dt);

  return {
    startTime: Math.max(0, startTime),
    endTime: Math.max(startTime, endTime),
  };
}

export function mergeRange<T extends TimelineRangeLike>(
  ranges: T[],
  nextRange: T | null,
): void {
  if (!nextRange) {
    return;
  }

  const previousRange = ranges[ranges.length - 1];
  if (
    previousRange &&
    previousRange.lane === nextRange.lane &&
    previousRange.label === nextRange.label &&
    Math.abs(previousRange.endTime - nextRange.startTime) <= RANGE_MERGE_EPSILON_SECONDS
  ) {
    previousRange.endTime = nextRange.endTime;
    return;
  }

  ranges.push(nextRange);
}

export function mergeRangeForLane<T extends TimelineRangeLike>(
  ranges: T[],
  lastRangeByLane: Map<string, T>,
  nextRange: T | null,
): void {
  if (!nextRange) {
    return;
  }

  const laneKey = nextRange.lane ?? "";
  const previousRange = lastRangeByLane.get(laneKey);
  if (
    previousRange &&
    previousRange.label === nextRange.label &&
    Math.abs(previousRange.endTime - nextRange.startTime) <= RANGE_MERGE_EPSILON_SECONDS
  ) {
    previousRange.endTime = nextRange.endTime;
    return;
  }

  ranges.push(nextRange);
  lastRangeByLane.set(laneKey, nextRange);
}
