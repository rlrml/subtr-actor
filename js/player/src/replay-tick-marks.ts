import type {
  RawReplayFramesData,
  RawReplayTickMark,
  ReplayTickMark,
  ReplayTimelineEvent,
} from "./types";
import { normalizeReplayTime } from "./replay-data-helpers";

interface ReplayTickMarkProgressTracker {
  advance(units?: number): unknown;
}

function tickMarkFrame(tickMark: RawReplayTickMark): number | null {
  return Number.isInteger(tickMark.frame) && tickMark.frame >= 0 ? tickMark.frame : null;
}

function tickMarkRawTime(tickMark: RawReplayTickMark, raw: RawReplayFramesData): number | null {
  if (typeof tickMark.time === "number" && Number.isFinite(tickMark.time)) {
    return tickMark.time;
  }

  const frame = tickMarkFrame(tickMark);
  if (frame === null) {
    return null;
  }

  const frameTime = raw.frame_data.metadata_frames[frame]?.time;
  return typeof frameTime === "number" && Number.isFinite(frameTime) ? frameTime : null;
}

function replayTickMarkId(tickMark: RawReplayTickMark, index: number): string {
  const frame = tickMarkFrame(tickMark);
  return `bookmark:${frame ?? "unknown"}:${tickMark.description || "tick-mark"}:${index}`;
}

export function buildReplayTickMarks(
  raw: RawReplayFramesData,
  startTime: number,
  progressTracker?: ReplayTickMarkProgressTracker,
): ReplayTickMark[] {
  return (raw.replay_tick_marks ?? []).flatMap((tickMark, index) => {
    progressTracker?.advance();
    const rawTime = tickMarkRawTime(tickMark, raw);
    if (rawTime === null) {
      return [];
    }

    return [
      {
        id: replayTickMarkId(tickMark, index),
        description: tickMark.description,
        frame: tickMarkFrame(tickMark),
        time: normalizeReplayTime(rawTime, startTime),
      },
    ];
  });
}

export function replayTickMarkTimelineEvent(tickMark: ReplayTickMark): ReplayTimelineEvent {
  const label = tickMark.description.trim() || "Replay bookmark";
  return {
    id: tickMark.id,
    time: tickMark.time,
    seekTime: tickMark.time,
    frame: tickMark.frame ?? undefined,
    kind: "bookmark",
    label,
    shortLabel: "BM",
    iconName: "bookmark",
  };
}
