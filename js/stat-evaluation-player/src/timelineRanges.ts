import type { ReplayBoostPadSize, ReplayModel, ReplayTimelineRange } from "@rlrml/player";
import type { PlayerStatsSnapshot, StatsFrame, StatsTimeline } from "./statsTimeline.ts";
import { statsEventPayloads } from "./statsTimeline.ts";
import { teamTimelineColor } from "./timelinePresentation.ts";
import type { BoostPickupActivity } from "./generated/BoostPickupActivity.ts";
import type { BoostPickupDetection } from "./generated/BoostPickupDetection.ts";
import type { BoostPickupFieldHalf } from "./generated/BoostPickupFieldHalf.ts";
import type { BoostPickupPadType } from "./generated/BoostPickupPadType.ts";
import type { FiftyFiftyEvent } from "./generated/FiftyFiftyEvent.ts";
import type { PossessionEvent } from "./generated/PossessionEvent.ts";
import type { FieldThirdEvent } from "./statsTimeline.ts";
import type { PowerslideEvent } from "./generated/PowerslideEvent.ts";
import type { BallHalfEvent } from "./generated/BallHalfEvent.ts";
import type { BallThirdEvent } from "./generated/BallThirdEvent.ts";

const RANGE_MERGE_EPSILON_SECONDS = 0.02;
const DELTA_EPSILON = 0.0001;
const DEFAULT_PRESSURE_NEUTRAL_ZONE_HALF_WIDTH_Y = 200;
const BOOST_PICKUP_TICK_SECONDS = 0.08;
const BOOST_PICKUP_COLORS: Record<ReplayBoostPadSize, string> = {
  big: "rgba(245, 158, 11, 0.92)",
  small: "rgba(52, 211, 153, 0.86)",
};
const BOOST_PICKUP_DETECTION_COLORS: Record<BoostPickupDetection, string> = {
  both: "rgba(52, 211, 153, 0.86)",
  inferred_only: "rgba(239, 68, 68, 0.9)",
  reported_only: "rgba(59, 130, 246, 0.9)",
};
type BallHalfControlState = "team_zero_side" | "team_one_side" | "neutral";
type BallThirdControlState = "team_zero_third" | "team_one_third" | "neutral_third";

export interface BoostPickupTimelineRangeOptions {
  sizes?: Iterable<ReplayBoostPadSize>;
  padTypes?: Iterable<BoostPickupPadType>;
  detections?: Iterable<BoostPickupDetection>;
  activities?: Iterable<BoostPickupActivity>;
  fieldHalves?: Iterable<BoostPickupFieldHalf>;
  playerIds?: Iterable<string>;
}

function getBallHalfNeutralZoneHalfWidthY(timeline: StatsTimeline): number {
  const configured = timeline.config?.ball_half_neutral_zone_half_width_y;
  if (typeof configured === "number" && Number.isFinite(configured)) {
    return Math.max(0, configured);
  }

  return DEFAULT_PRESSURE_NEUTRAL_ZONE_HALF_WIDTH_Y;
}

function getReplayFrameTime(
  replay: ReplayModel | undefined,
  frame: number | undefined,
  fallbackTime: number,
): number {
  return replay?.frames?.[frame ?? -1]?.time ?? fallbackTime;
}

function resolveBallHalfControlState(
  frameNumber: number,
  replay: ReplayModel | undefined,
  neutralZoneHalfWidthY: number,
  deltaTeamZero: number,
  deltaTeamOne: number,
  deltaNeutral: number,
): BallHalfControlState | null {
  const ballY = replay?.ballFrames[frameNumber]?.position?.y;
  if (
    typeof ballY === "number" &&
    Number.isFinite(ballY) &&
    Math.abs(ballY) <= neutralZoneHalfWidthY + DELTA_EPSILON
  ) {
    return "neutral";
  }

  if (deltaNeutral > DELTA_EPSILON) {
    return "neutral";
  }
  if (deltaTeamZero > deltaTeamOne + DELTA_EPSILON) {
    return "team_zero_side";
  }
  if (deltaTeamOne > deltaTeamZero + DELTA_EPSILON) {
    return "team_one_side";
  }

  return null;
}

function createBallHalfRange(
  halfControlState: BallHalfControlState,
  startTime: number,
  endTime: number,
): ReplayTimelineRange {
  if (halfControlState === "neutral") {
    return {
      id: `half-control:neutral:${startTime.toFixed(3)}`,
      startTime,
      endTime,
      lane: "half-control",
      laneLabel: "Half Control",
      label: "Neutral half control",
      color: "rgba(209, 217, 224, 0.7)",
      isTeamZero: null,
    };
  }

  const isTeamZero = halfControlState === "team_zero_side";
  return {
    id: `half-control:${halfControlState}:${startTime.toFixed(3)}`,
    startTime,
    endTime,
    lane: "half-control",
    laneLabel: "Half Control",
    label: isTeamZero ? "Blue half control" : "Orange half control",
    color: teamTimelineColor(isTeamZero) ?? undefined,
    isTeamZero,
  };
}

function sortTimelineEvents<T extends { frame: number; time: number }>(events: readonly T[]): T[] {
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

function buildPossessionTimelineRangesFromEvents(
  timeline: StatsTimeline,
  replay?: ReplayModel,
): ReplayTimelineRange[] {
  const events = sortTimelineEvents(statsEventPayloads(timeline, "possession"));
  const ranges: ReplayTimelineRange[] = [];
  let eventIndex = 0;
  let active = false;
  let possessionState = "neutral";

  let previousFrame: StatsTimeline["frames"][number] | null = null;
  for (const frame of timeline.frames) {
    while (eventIndex < events.length && events[eventIndex]!.frame <= frame.frame_number) {
      const event = events[eventIndex] as PossessionEvent;
      active = event.active;
      possessionState = event.possession_state;
      eventIndex += 1;
    }

    if (!Number.isFinite(frame.time) || !Number.isFinite(frame.dt) || frame.dt <= 0) {
      previousFrame = frame;
      continue;
    }

    const { startTime, endTime } = resolveRangeBounds(frame, previousFrame, replay);
    let nextRange: ReplayTimelineRange | null = null;
    if (active && possessionState === "team_zero") {
      nextRange = {
        id: `possession:team_zero:${startTime.toFixed(3)}`,
        startTime,
        endTime,
        lane: "possession",
        laneLabel: "Possession",
        label: "Blue possession",
        color: "rgba(59, 130, 246, 0.88)",
        isTeamZero: true,
      };
    } else if (active && possessionState === "team_one") {
      nextRange = {
        id: `possession:team_one:${startTime.toFixed(3)}`,
        startTime,
        endTime,
        lane: "possession",
        laneLabel: "Possession",
        label: "Orange possession",
        color: "rgba(245, 158, 11, 0.88)",
        isTeamZero: false,
      };
    }
    // Neutral (and non-live / inactive) stretches draw nothing: the lane only
    // shows spans where a team actively has possession, leaving gaps elsewhere.

    mergeRange(ranges, nextRange);
    previousFrame = frame;
  }

  return ranges;
}

export function buildPossessionTimelineRanges(
  timeline: StatsTimeline,
  replay?: ReplayModel,
): ReplayTimelineRange[] {
  if (statsEventPayloads(timeline, "possession").length > 0) {
    return buildPossessionTimelineRangesFromEvents(timeline, replay);
  }

  const ranges: ReplayTimelineRange[] = [];

  let previousTeamZero = 0;
  let previousTeamOne = 0;
  let previousNeutral = 0;

  let previousFrame: StatsTimeline["frames"][number] | null = null;
  for (const frame of timeline.frames) {
    if (!Number.isFinite(frame.time) || !Number.isFinite(frame.dt) || frame.dt <= 0) {
      previousFrame = frame;
      continue;
    }

    const statsFrame = frame as StatsFrame;
    const currentTeamZero = statsFrame.team_zero?.possession?.possession_time ?? 0;
    const currentTeamOne = statsFrame.team_one?.possession?.possession_time ?? 0;
    const currentNeutral = statsFrame.team_zero?.possession?.neutral_time ?? 0;

    const deltaTeamZero = currentTeamZero - previousTeamZero;
    const deltaTeamOne = currentTeamOne - previousTeamOne;
    const deltaNeutral = currentNeutral - previousNeutral;

    previousTeamZero = currentTeamZero;
    previousTeamOne = currentTeamOne;
    previousNeutral = currentNeutral;

    let nextRange: ReplayTimelineRange | null = null;
    const { startTime, endTime } = resolveRangeBounds(frame, previousFrame, replay);

    if (
      deltaTeamZero > deltaTeamOne + DELTA_EPSILON &&
      deltaTeamZero > deltaNeutral + DELTA_EPSILON
    ) {
      nextRange = {
        id: `possession:team_zero:${startTime.toFixed(3)}`,
        startTime,
        endTime,
        lane: "possession",
        laneLabel: "Possession",
        label: "Blue possession",
        color: "rgba(59, 130, 246, 0.88)",
        isTeamZero: true,
      };
    } else if (
      deltaTeamOne > deltaTeamZero + DELTA_EPSILON &&
      deltaTeamOne > deltaNeutral + DELTA_EPSILON
    ) {
      nextRange = {
        id: `possession:team_one:${startTime.toFixed(3)}`,
        startTime,
        endTime,
        lane: "possession",
        laneLabel: "Possession",
        label: "Orange possession",
        color: "rgba(245, 158, 11, 0.88)",
        isTeamZero: false,
      };
    }
    // Neutral stretches draw nothing: only spans where a team actively has
    // possession render, leaving gaps elsewhere.

    mergeRange(ranges, nextRange);
    previousFrame = frame;
  }

  return ranges;
}

function buildBallHalfTimelineRangesFromEvents(
  timeline: StatsTimeline,
  replay?: ReplayModel,
): ReplayTimelineRange[] {
  const events = sortTimelineEvents(statsEventPayloads(timeline, "ball_half"));
  const ranges: ReplayTimelineRange[] = [];
  let eventIndex = 0;
  let active = false;
  let fieldHalf: BallHalfControlState = "neutral";

  let previousFrame: StatsTimeline["frames"][number] | null = null;
  for (const frame of timeline.frames) {
    while (eventIndex < events.length && events[eventIndex]!.frame <= frame.frame_number) {
      const event = events[eventIndex] as BallHalfEvent;
      active = event.active;
      fieldHalf =
        event.field_half === "team_zero_side" || event.field_half === "team_one_side"
          ? event.field_half
          : "neutral";
      eventIndex += 1;
    }

    if (!Number.isFinite(frame.time) || !Number.isFinite(frame.dt) || frame.dt <= 0) {
      previousFrame = frame;
      continue;
    }

    const { startTime, endTime } = resolveRangeBounds(frame, previousFrame, replay);
    mergeRange(ranges, active ? createBallHalfRange(fieldHalf, startTime, endTime) : null);
    previousFrame = frame;
  }

  return ranges;
}

export function buildBallHalfTimelineRanges(
  timeline: StatsTimeline,
  replay?: ReplayModel,
): ReplayTimelineRange[] {
  if (statsEventPayloads(timeline, "ball_half").length > 0) {
    return buildBallHalfTimelineRangesFromEvents(timeline, replay);
  }

  const ranges: ReplayTimelineRange[] = [];

  let previousTeamZero = 0;
  let previousTeamOne = 0;
  let previousNeutral = 0;
  const neutralZoneHalfWidthY = getBallHalfNeutralZoneHalfWidthY(timeline);

  let previousFrame: StatsTimeline["frames"][number] | null = null;
  for (const frame of timeline.frames) {
    if (!Number.isFinite(frame.time) || !Number.isFinite(frame.dt) || frame.dt <= 0) {
      previousFrame = frame;
      continue;
    }

    const statsFrame = frame as StatsFrame;
    const currentTeamZero = statsFrame.team_zero?.ball_half?.defensive_half_time ?? 0;
    const currentTeamOne = statsFrame.team_one?.ball_half?.defensive_half_time ?? 0;
    const currentNeutral = statsFrame.team_zero?.ball_half?.neutral_time ?? 0;
    const deltaTeamZero = currentTeamZero - previousTeamZero;
    const deltaTeamOne = currentTeamOne - previousTeamOne;
    const deltaNeutral = currentNeutral - previousNeutral;

    previousTeamZero = currentTeamZero;
    previousTeamOne = currentTeamOne;
    previousNeutral = currentNeutral;

    const { startTime, endTime } = resolveRangeBounds(frame, previousFrame, replay);
    const halfControlState = resolveBallHalfControlState(
      frame.frame_number,
      replay,
      neutralZoneHalfWidthY,
      deltaTeamZero,
      deltaTeamOne,
      deltaNeutral,
    );
    const nextRange = halfControlState
      ? createBallHalfRange(halfControlState, startTime, endTime)
      : null;

    mergeRange(ranges, nextRange);
    previousFrame = frame;
  }

  return ranges;
}

function resolveBallThirdControlState(
  deltaTeamZero: number,
  deltaTeamOne: number,
  deltaNeutral: number,
): BallThirdControlState | null {
  if (deltaNeutral > DELTA_EPSILON) {
    return "neutral_third";
  }
  if (deltaTeamZero > deltaTeamOne + DELTA_EPSILON) {
    return "team_zero_third";
  }
  if (deltaTeamOne > deltaTeamZero + DELTA_EPSILON) {
    return "team_one_third";
  }

  return null;
}

function createBallThirdRange(
  thirdControlState: BallThirdControlState,
  startTime: number,
  endTime: number,
): ReplayTimelineRange {
  if (thirdControlState === "neutral_third") {
    return {
      id: `third-control:neutral_third:${startTime.toFixed(3)}`,
      startTime,
      endTime,
      lane: "third-control",
      laneLabel: "Third Control",
      label: "Neutral third control",
      color: "rgba(209, 217, 224, 0.7)",
      isTeamZero: null,
    };
  }

  const isTeamZero = thirdControlState === "team_zero_third";
  return {
    id: `third-control:${thirdControlState}:${startTime.toFixed(3)}`,
    startTime,
    endTime,
    lane: "third-control",
    laneLabel: "Third Control",
    label: isTeamZero ? "Blue third control" : "Orange third control",
    color: teamTimelineColor(isTeamZero) ?? undefined,
    isTeamZero,
  };
}

function buildBallThirdTimelineRangesFromEvents(
  timeline: StatsTimeline,
  replay?: ReplayModel,
): ReplayTimelineRange[] {
  const events = sortTimelineEvents(statsEventPayloads(timeline, "ball_third"));
  const ranges: ReplayTimelineRange[] = [];
  let eventIndex = 0;
  let active = false;
  let fieldThird: BallThirdControlState = "neutral_third";

  let previousFrame: StatsTimeline["frames"][number] | null = null;
  for (const frame of timeline.frames) {
    while (eventIndex < events.length && events[eventIndex]!.frame <= frame.frame_number) {
      const event = events[eventIndex] as BallThirdEvent;
      active = event.active;
      fieldThird =
        event.field_third === "team_zero_third" || event.field_third === "team_one_third"
          ? event.field_third
          : "neutral_third";
      eventIndex += 1;
    }

    if (!Number.isFinite(frame.time) || !Number.isFinite(frame.dt) || frame.dt <= 0) {
      previousFrame = frame;
      continue;
    }

    const { startTime, endTime } = resolveRangeBounds(frame, previousFrame, replay);
    mergeRange(ranges, active ? createBallThirdRange(fieldThird, startTime, endTime) : null);
    previousFrame = frame;
  }

  return ranges;
}

export function buildBallThirdTimelineRanges(
  timeline: StatsTimeline,
  replay?: ReplayModel,
): ReplayTimelineRange[] {
  if (statsEventPayloads(timeline, "ball_third").length > 0) {
    return buildBallThirdTimelineRangesFromEvents(timeline, replay);
  }

  const ranges: ReplayTimelineRange[] = [];

  let previousTeamZero = 0;
  let previousTeamOne = 0;
  let previousNeutral = 0;

  let previousFrame: StatsTimeline["frames"][number] | null = null;
  for (const frame of timeline.frames) {
    if (!Number.isFinite(frame.time) || !Number.isFinite(frame.dt) || frame.dt <= 0) {
      previousFrame = frame;
      continue;
    }

    const statsFrame = frame as StatsFrame;
    const currentTeamZero = statsFrame.team_zero?.ball_third?.defensive_third_time ?? 0;
    const currentTeamOne = statsFrame.team_one?.ball_third?.defensive_third_time ?? 0;
    const currentNeutral = statsFrame.team_zero?.ball_third?.neutral_third_time ?? 0;
    const deltaTeamZero = currentTeamZero - previousTeamZero;
    const deltaTeamOne = currentTeamOne - previousTeamOne;
    const deltaNeutral = currentNeutral - previousNeutral;

    previousTeamZero = currentTeamZero;
    previousTeamOne = currentTeamOne;
    previousNeutral = currentNeutral;

    const { startTime, endTime } = resolveRangeBounds(frame, previousFrame, replay);
    const thirdControlState = resolveBallThirdControlState(
      deltaTeamZero,
      deltaTeamOne,
      deltaNeutral,
    );
    const nextRange = thirdControlState
      ? createBallThirdRange(thirdControlState, startTime, endTime)
      : null;

    mergeRange(ranges, nextRange);
    previousFrame = frame;
  }

  return ranges;
}

export function buildFiftyFiftyTimelineRanges(
  timeline: StatsTimeline,
  replay?: ReplayModel,
): ReplayTimelineRange[] {
  return statsEventPayloads(timeline, "fifty_fifty")
    .map((event: FiftyFiftyEvent, index): ReplayTimelineRange => {
      const startTime = getReplayFrameTime(replay, event.start_frame, event.start_time);
      const endTime = Math.max(
        startTime,
        getReplayFrameTime(replay, event.resolve_frame, event.resolve_time),
      );
      const outcome =
        event.winning_team_is_team_0 == null
          ? "Neutral"
          : event.winning_team_is_team_0
            ? "Blue win"
            : "Orange win";
      const phase = event.is_kickoff ? "kickoff " : "";

      return {
        id: `fifty-fifty:${event.start_frame}:${event.resolve_frame}:${index}`,
        startTime,
        endTime,
        lane: "fifty-fifty",
        laneLabel: "50/50",
        label: `${outcome} ${phase}50/50`,
        shortLabel: event.is_kickoff ? "KO" : "50",
        color:
          event.winning_team_is_team_0 == null
            ? "rgba(209, 217, 224, 0.7)"
            : event.winning_team_is_team_0
              ? "rgba(59, 130, 246, 0.48)"
              : "rgba(245, 158, 11, 0.48)",
        isTeamZero: event.winning_team_is_team_0,
      };
    })
    .sort((left, right) => {
      if (left.startTime !== right.startTime) {
        return left.startTime - right.startTime;
      }
      return (left.id ?? "").localeCompare(right.id ?? "");
    });
}

export function buildRushTimelineRanges(
  timeline: StatsTimeline,
  replay?: ReplayModel,
): ReplayTimelineRange[] {
  return statsEventPayloads(timeline, "rush").map((event, index) => {
    const startTime = replay?.frames[event.start_frame]?.time ?? event.start_time;
    const endTime = replay?.frames[event.end_frame]?.time ?? event.end_time;
    const matchupLabel = `${event.attackers}v${event.defenders}`;
    const isTeamZero = event.is_team_0;

    return {
      id: `rush-range:${event.start_frame}:${event.end_frame}:${index}`,
      startTime,
      endTime: Math.max(startTime, endTime),
      lane: "rush",
      laneLabel: "Rush",
      label: `${isTeamZero ? "Blue" : "Orange"} rush ${matchupLabel}`,
      color: isTeamZero ? "rgba(59, 130, 246, 0.4)" : "rgba(245, 158, 11, 0.4)",
      isTeamZero,
    };
  });
}

export function buildPowerslideTimelineRanges(
  timeline: StatsTimeline,
  replay?: ReplayModel,
): ReplayTimelineRange[] {
  const events = sortTimelineEvents(statsEventPayloads(timeline, "powerslide"));
  const activeByPlayer = new Map<string, PowerslideEvent>();
  const ranges: ReplayTimelineRange[] = [];
  const playerNames = new Map((replay?.players ?? []).map((player) => [player.id, player.name]));

  for (const event of events) {
    const playerId = remoteIdToString(event.player as Record<string, unknown>);
    if (event.active) {
      activeByPlayer.set(playerId, event);
      continue;
    }

    const active = activeByPlayer.get(playerId);
    if (!active) {
      continue;
    }
    activeByPlayer.delete(playerId);

    const startTime = getReplayFrameTime(replay, active.frame, active.time);
    const endTime = Math.max(startTime, getReplayFrameTime(replay, event.frame, event.time));
    const playerName = playerNames.get(playerId) ?? playerId;
    ranges.push({
      id: `powerslide:${active.frame}:${event.frame}:${playerId}`,
      startTime,
      endTime,
      lane: `powerslide:${playerId}`,
      laneLabel: playerName,
      label: `${playerName} powerslide`,
      shortLabel: "PS",
      color: teamTimelineColor(active.is_team_0) ?? undefined,
      isTeamZero: active.is_team_0,
    });
  }

  const replayEndTime =
    replay?.duration ??
    replay?.frames.at(-1)?.time ??
    timeline.frames.at(-1)?.time ??
    Number.POSITIVE_INFINITY;
  for (const [playerId, active] of activeByPlayer) {
    const startTime = getReplayFrameTime(replay, active.frame, active.time);
    if (!Number.isFinite(replayEndTime) || replayEndTime <= startTime) {
      continue;
    }
    const playerName = playerNames.get(playerId) ?? playerId;
    ranges.push({
      id: `powerslide:${active.frame}:open:${playerId}`,
      startTime,
      endTime: replayEndTime,
      lane: `powerslide:${playerId}`,
      laneLabel: playerName,
      label: `${playerName} powerslide`,
      shortLabel: "PS",
      color: teamTimelineColor(active.is_team_0) ?? undefined,
      isTeamZero: active.is_team_0,
    });
  }

  return ranges.sort((left, right) => {
    if (left.startTime !== right.startTime) {
      return left.startTime - right.startTime;
    }
    return (left.id ?? "").localeCompare(right.id ?? "");
  });
}

function buildReplayBoostPickupTimelineRanges(
  replay: ReplayModel,
  options: BoostPickupTimelineRangeOptions = {},
): ReplayTimelineRange[] {
  const enabledPadTypes = padTypesFromOptions(options);
  const enabledDetections = new Set<BoostPickupDetection>(
    options.detections ?? ["both", "inferred_only", "reported_only"],
  );
  const enabledActivities = new Set<BoostPickupActivity>(
    options.activities ?? ["active", "inactive", "unknown"],
  );
  const enabledFieldHalves = new Set<BoostPickupFieldHalf>(
    options.fieldHalves ?? ["own", "opponent", "unknown"],
  );
  const enabledPlayerIds = options.playerIds ? new Set(options.playerIds) : null;
  if (
    enabledPadTypes.size === 0 ||
    enabledDetections.size === 0 ||
    !enabledActivities.has("unknown") ||
    !enabledFieldHalves.has("unknown") ||
    enabledPlayerIds?.size === 0
  ) {
    return [];
  }

  const playerTeams = new Map(replay.players.map((player) => [player.id, player.isTeamZero]));
  const ranges: ReplayTimelineRange[] = [];

  for (const pad of replay.boostPads) {
    if (!enabledPadTypes.has(pad.size)) {
      continue;
    }

    for (let eventIndex = 0; eventIndex < pad.events.length; eventIndex += 1) {
      const event = pad.events[eventIndex]!;
      if (event.available || !Number.isFinite(event.time)) {
        continue;
      }
      if (enabledPlayerIds && !event.playerId) {
        continue;
      }
      if (event.playerId && enabledPlayerIds && !enabledPlayerIds.has(event.playerId)) {
        continue;
      }

      const startTime = Math.max(0, getReplayFrameTime(replay, event.frame, event.time));
      const sizeLabel = pad.size === "big" ? "Big" : "Small";
      const playerLabel = event.playerName ? `${event.playerName} ` : "";
      const isTeamZero = event.playerId ? (playerTeams.get(event.playerId) ?? null) : null;
      ranges.push({
        id: `boost-pickup:${pad.index}:${event.frame}:${eventIndex}`,
        startTime,
        endTime: Math.max(startTime + BOOST_PICKUP_TICK_SECONDS, startTime),
        lane: "boost-pickups",
        laneLabel: "Boost Pickups",
        label: `${playerLabel}picked up ${sizeLabel.toLowerCase()} boost pad ${pad.index}`,
        shortLabel: pad.size === "big" ? "100" : "12",
        color: teamTimelineColor(isTeamZero) ?? BOOST_PICKUP_COLORS[pad.size],
        isTeamZero,
      });
    }
  }

  return ranges.sort((left, right) => {
    if (left.startTime !== right.startTime) {
      return left.startTime - right.startTime;
    }
    return (left.id ?? "").localeCompare(right.id ?? "");
  });
}

function padTypesFromOptions(options: BoostPickupTimelineRangeOptions): Set<BoostPickupPadType> {
  if (options.padTypes) {
    return new Set(options.padTypes);
  }

  if (options.sizes) {
    const sizes = new Set(options.sizes);
    const padTypes = new Set<BoostPickupPadType>();
    if (sizes.has("big")) {
      padTypes.add("big");
    }
    if (sizes.has("small")) {
      padTypes.add("small");
    }
    if (sizes.has("big") && sizes.has("small")) {
      padTypes.add("ambiguous");
    }
    return padTypes;
  }

  return new Set(["big", "small", "ambiguous"]);
}

function remoteIdToString(playerId: Record<string, unknown>): string {
  const [kind, value] = Object.entries(playerId)[0] ?? ["Unknown", "unknown"];
  const normalizedValue = typeof value === "string" ? value : JSON.stringify(value);
  return `${kind}:${normalizedValue}`;
}

function formatBoostPickupPadType(padType: BoostPickupPadType): string {
  return {
    big: "big",
    small: "small",
    ambiguous: "ambiguous",
  }[padType];
}

function formatBoostPickupDetection(detection: BoostPickupDetection): string {
  return {
    both: "counted",
    inferred_only: "inferred",
    reported_only: "reported",
  }[detection];
}

function boostPickupShortLabel(
  detection: BoostPickupDetection,
  padType: BoostPickupPadType,
): string {
  if (detection === "inferred_only") {
    return "I";
  }
  if (detection === "reported_only") {
    return "R";
  }
  return {
    big: "100",
    small: "12",
    ambiguous: "?",
  }[padType];
}

export function buildBoostPickupTimelineRanges(
  timeline: StatsTimeline,
  replay?: ReplayModel,
  options: BoostPickupTimelineRangeOptions = {},
): ReplayTimelineRange[] {
  const events = statsEventPayloads(timeline, "boost_pickup");
  if (events.length === 0 && replay) {
    return buildReplayBoostPickupTimelineRanges(replay, options);
  }

  const enabledPadTypes = padTypesFromOptions(options);
  const enabledDetections = new Set<BoostPickupDetection>(
    options.detections ?? ["both", "inferred_only", "reported_only"],
  );
  const enabledActivities = new Set<BoostPickupActivity>(
    options.activities ?? ["active", "inactive", "unknown"],
  );
  const enabledFieldHalves = new Set<BoostPickupFieldHalf>(
    options.fieldHalves ?? ["own", "opponent", "unknown"],
  );
  const enabledPlayerIds = options.playerIds ? new Set(options.playerIds) : null;
  if (
    enabledPadTypes.size === 0 ||
    enabledDetections.size === 0 ||
    enabledActivities.size === 0 ||
    enabledFieldHalves.size === 0 ||
    enabledPlayerIds?.size === 0
  ) {
    return [];
  }

  const playerNames = new Map((replay?.players ?? []).map((player) => [player.id, player.name]));
  return events
    .filter((event) => {
      const playerId = remoteIdToString(event.player_id as Record<string, unknown>);
      return (
        enabledPadTypes.has(event.pad_type) &&
        enabledDetections.has(event.detection) &&
        enabledActivities.has(event.activity) &&
        enabledFieldHalves.has(event.field_half) &&
        (!enabledPlayerIds || enabledPlayerIds.has(playerId))
      );
    })
    .map((event, index): ReplayTimelineRange => {
      const playerId = remoteIdToString(event.player_id as Record<string, unknown>);
      const playerName = playerNames.get(playerId) ?? playerId;
      const startTime = Math.max(0, getReplayFrameTime(replay, event.frame, event.time));
      const detectionLabel = formatBoostPickupDetection(event.detection);
      const padLabel = formatBoostPickupPadType(event.pad_type);
      return {
        id: `boost-pickup:${event.detection}:${event.frame}:${playerId}:${index}`,
        startTime,
        endTime: Math.max(startTime + BOOST_PICKUP_TICK_SECONDS, startTime),
        lane: "boost-pickups",
        laneLabel: "Boost Pickups",
        label: `${playerName} ${detectionLabel} ${padLabel} boost pickup`,
        shortLabel: boostPickupShortLabel(event.detection, event.pad_type),
        color:
          teamTimelineColor(event.is_team_0) ??
          (event.detection === "both"
            ? event.pad_type === "big"
              ? BOOST_PICKUP_COLORS.big
              : event.pad_type === "small"
                ? BOOST_PICKUP_COLORS.small
                : BOOST_PICKUP_DETECTION_COLORS.both
            : BOOST_PICKUP_DETECTION_COLORS[event.detection]),
        isTeamZero: event.is_team_0,
      };
    })
    .sort((left, right) => {
      if (left.startTime !== right.startTime) {
        return left.startTime - right.startTime;
      }
      return (left.id ?? "").localeCompare(right.id ?? "");
    });
}

interface PlayerZoneSpec {
  fieldName: string;
  aliases?: string[];
  label: string;
  relativeColor: "own" | "neutral" | "opp";
}

const PLAYER_ZONE_SPECS: PlayerZoneSpec[] = [
  {
    fieldName: "time_defensive_third",
    aliases: ["time_defensive_zone"],
    label: "Def third",
    relativeColor: "own",
  },
  {
    fieldName: "time_neutral_third",
    aliases: ["time_neutral_zone"],
    label: "Neutral third",
    relativeColor: "neutral",
  },
  {
    fieldName: "time_offensive_third",
    aliases: ["time_offensive_zone"],
    label: "Off third",
    relativeColor: "opp",
  },
];

function getPlayerZoneColor(spec: PlayerZoneSpec, isTeamZero: boolean): string {
  if (spec.relativeColor === "neutral") {
    return "rgba(209, 217, 224, 0.68)";
  }

  const isOwnTeamColor = spec.relativeColor === "own";
  const shouldUseBlue = isOwnTeamColor ? isTeamZero : !isTeamZero;
  return shouldUseBlue ? "rgba(89, 195, 255, 0.74)" : "rgba(255, 193, 92, 0.78)";
}

function playerIdToString(playerId: Record<string, unknown>): string {
  const [kind, value] = Object.entries(playerId)[0] ?? ["Unknown", "unknown"];
  const normalizedValue = typeof value === "string" ? value : JSON.stringify(value);
  return `${kind}:${normalizedValue}`;
}

function extractPlayerStatValue(player: PlayerStatsSnapshot, spec: PlayerZoneSpec): number {
  const positioning = player.positioning as Record<string, unknown> | undefined;
  if (!positioning) {
    return 0;
  }

  for (const fieldName of [spec.fieldName, ...(spec.aliases ?? [])]) {
    const value = positioning[fieldName];
    if (typeof value === "number" && Number.isFinite(value)) {
      return value;
    }
  }

  return 0;
}

function fieldThirdZoneSpec(state: FieldThirdEvent["state"]): PlayerZoneSpec {
  switch (state) {
    case "defensive":
      return PLAYER_ZONE_SPECS[0]!;
    case "neutral":
      return PLAYER_ZONE_SPECS[1]!;
    case "offensive":
      return PLAYER_ZONE_SPECS[2]!;
  }
}

function buildPlayerNameLookup(timeline: StatsTimeline): Map<string, string> {
  const names = new Map<string, string>();
  for (const frame of timeline.frames) {
    for (const player of frame.players) {
      const playerId = playerIdToString(player.player_id as Record<string, unknown>);
      if (!names.has(playerId)) {
        names.set(playerId, player.name);
      }
    }
  }
  return names;
}

function buildTimeInZoneTimelineRangesFromEvents(timeline: StatsTimeline): ReplayTimelineRange[] {
  const events = sortTimelineEvents(statsEventPayloads(timeline, "field_third"));
  const ranges: ReplayTimelineRange[] = [];
  const lastRangeByLane = new Map<string, ReplayTimelineRange>();
  const playerNames = buildPlayerNameLookup(timeline);

  for (const event of events) {
    if (!Number.isFinite(event.time) || !Number.isFinite(event.end_time)) {
      continue;
    }
    if (event.end_time - event.time <= DELTA_EPSILON) {
      continue;
    }
    const playerId = playerIdToString(event.player as Record<string, unknown>);
    const spec = fieldThirdZoneSpec(event.state);
    mergeRangeForLane(ranges, lastRangeByLane, {
      id: `time-in-zone:${playerId}:${spec.fieldName}:${event.time.toFixed(3)}`,
      startTime: event.time,
      endTime: event.end_time,
      lane: `time-in-zone:${playerId}`,
      laneLabel: playerNames.get(playerId) ?? playerId,
      label: spec.label,
      color: getPlayerZoneColor(spec, event.is_team_0),
      isTeamZero: event.is_team_0,
    });
  }

  return ranges;
}

export function buildTimeInZoneTimelineRanges(
  timeline: StatsTimeline,
  replay?: ReplayModel,
): ReplayTimelineRange[] {
  if (statsEventPayloads(timeline, "field_third").length > 0) {
    return buildTimeInZoneTimelineRangesFromEvents(timeline);
  }

  const previousValues = new Map<string, Map<string, number>>();
  const ranges: ReplayTimelineRange[] = [];
  const lastRangeByLane = new Map<string, ReplayTimelineRange>();

  let previousFrame: StatsTimeline["frames"][number] | null = null;
  for (const frame of timeline.frames) {
    if (!Number.isFinite(frame.time) || !Number.isFinite(frame.dt) || frame.dt <= 0) {
      previousFrame = frame;
      continue;
    }

    const { startTime, endTime } = resolveRangeBounds(frame, previousFrame, replay);
    if (endTime - startTime <= DELTA_EPSILON) {
      previousFrame = frame;
      continue;
    }

    for (const player of (frame as StatsFrame).players) {
      const playerId = playerIdToString(player.player_id);
      const previous = previousValues.get(playerId) ?? new Map<string, number>();

      let winningSpec: PlayerZoneSpec | null = null;
      let winningDelta = 0;

      for (const spec of PLAYER_ZONE_SPECS) {
        const value = extractPlayerStatValue(player, spec);
        const delta = value - (previous.get(spec.fieldName) ?? 0);
        if (delta > winningDelta + DELTA_EPSILON) {
          winningDelta = delta;
          winningSpec = spec;
        }
        previous.set(spec.fieldName, value);
      }

      previousValues.set(playerId, previous);

      if (!winningSpec) {
        continue;
      }

      mergeRangeForLane(ranges, lastRangeByLane, {
        id: `time-in-zone:${playerId}:${winningSpec.fieldName}:${startTime.toFixed(3)}`,
        startTime,
        endTime,
        lane: `time-in-zone:${playerId}`,
        laneLabel: player.name,
        label: winningSpec.label,
        color: getPlayerZoneColor(winningSpec, player.is_team_0),
        isTeamZero: player.is_team_0,
      });
    }

    previousFrame = frame;
  }

  return ranges;
}

function resolveRangeBounds(
  frame: { frame_number: number; time: number; dt: number },
  previousFrame: { frame_number: number; time: number } | null,
  replay?: ReplayModel,
): { startTime: number; endTime: number } {
  const endTime = replay?.frames[frame.frame_number]?.time ?? frame.time;
  const startTime = previousFrame
    ? (replay?.frames[previousFrame.frame_number]?.time ?? previousFrame.time)
    : Math.max(0, endTime - frame.dt);

  return {
    startTime: Math.max(0, startTime),
    endTime: Math.max(startTime, endTime),
  };
}

function mergeRange(ranges: ReplayTimelineRange[], nextRange: ReplayTimelineRange | null): void {
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

function mergeRangeForLane(
  ranges: ReplayTimelineRange[],
  lastRangeByLane: Map<string, ReplayTimelineRange>,
  nextRange: ReplayTimelineRange | null,
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
