import type { BackboardBounceEvent } from "./generated/BackboardBounceEvent.ts";
import type { BallCarryEvent } from "./generated/BallCarryEvent.ts";
import type { BoostPickupEvent } from "./generated/BoostPickupEvent.ts";
import type { RespawnEvent } from "./generated/RespawnEvent.ts";
import type { BumpEvent } from "./generated/BumpEvent.ts";
import type { CeilingShotEvent } from "./generated/CeilingShotEvent.ts";
import type { CorePlayerGoalContextEvent } from "./generated/CorePlayerGoalContextEvent.ts";
import type { CorePlayerScoreboardEvent } from "./generated/CorePlayerScoreboardEvent.ts";
import type { DodgeResetEvent } from "./generated/DodgeResetEvent.ts";
import type { DoubleTapEvent } from "./generated/DoubleTapEvent.ts";
import type { FiftyFiftyEvent } from "./generated/FiftyFiftyEvent.ts";
import type { FlickEvent } from "./generated/FlickEvent.ts";
import type { DodgeEvent } from "./generated/DodgeEvent.ts";
import type { LabeledCountEntry } from "./generated/LabeledCountEntry.ts";
import type { LabeledCounts } from "./generated/LabeledCounts.ts";
import type { LabeledFloatSumEntry } from "./generated/LabeledFloatSumEntry.ts";
import type { LabeledFloatSums } from "./generated/LabeledFloatSums.ts";
import type { Event } from "./generated/Event.ts";
import type { EventMeta } from "./generated/EventMeta.ts";
import type { EventPayload } from "./generated/EventPayload.ts";
import type { EventProperty } from "./generated/EventProperty.ts";
import type { EventPropertyValue } from "./generated/EventPropertyValue.ts";
import type { EventTiming } from "./generated/EventTiming.ts";
import type { MovementEvent } from "./generated/MovementEvent.ts";
import type { MustyFlickEvent } from "./generated/MustyFlickEvent.ts";
import type { OneTimerEvent } from "./generated/OneTimerEvent.ts";
import type { PassEvent } from "./generated/PassEvent.ts";
import type { PlayerStatsSnapshot as GeneratedPlayerStatsSnapshot } from "./generated/PlayerStatsSnapshot.ts";
import type { PossessionEvent } from "./generated/PossessionEvent.ts";
import type { PositioningActivityEvent } from "./generated/PositioningActivityEvent.ts";
import type { PositioningBallRelativeDepthEvent } from "./generated/PositioningBallRelativeDepthEvent.ts";
import type { PositioningBallProximityEvent } from "./generated/PositioningBallProximityEvent.ts";
import type { PositioningFieldZoneEvent } from "./generated/PositioningFieldZoneEvent.ts";
import type { PositioningTeammateRoleEvent } from "./generated/PositioningTeammateRoleEvent.ts";
import type { PressureEvent } from "./generated/PressureEvent.ts";
import type { PowerslideEvent } from "./generated/PowerslideEvent.ts";
import type { ReplayMeta } from "./generated/ReplayMeta.ts";
import type { ReplayStatsFrame } from "./generated/ReplayStatsFrame.ts";
import type { ReplayStatsFrameScaffold } from "./generated/ReplayStatsFrameScaffold.ts";
import type { ReplayStatsTimeline } from "./generated/ReplayStatsTimeline.ts";
import type { ReplayStatsTimelineScaffold } from "./generated/ReplayStatsTimelineScaffold.ts";
import type { ReplayStatsTimelineEvents } from "./generated/ReplayStatsTimelineEvents.ts";
import type { RotationPlayerEvent } from "./generated/RotationPlayerEvent.ts";
import type { RotationTeamEvent } from "./generated/RotationTeamEvent.ts";
import type { RushEvent } from "./generated/RushEvent.ts";
import type { SpeedFlipEvent } from "./generated/SpeedFlipEvent.ts";
import type { StatLabel } from "./generated/StatLabel.ts";
import type { StatsTimelineConfig } from "./generated/StatsTimelineConfig.ts";
import type { TeamStatsSnapshot as GeneratedTeamStatsSnapshot } from "./generated/TeamStatsSnapshot.ts";
import type { TimelineEvent } from "./generated/TimelineEvent.ts";
import type { TouchBallMovement } from "./generated/TouchBallMovement.ts";
import type { TouchClassificationEvent } from "./generated/TouchClassificationEvent.ts";
import type { HalfFlipEvent } from "./generated/HalfFlipEvent.ts";
import type { HalfVolleyEvent } from "./generated/HalfVolleyEvent.ts";
import type { WavedashEvent } from "./generated/WavedashEvent.ts";
import type { WallAerialEvent } from "./generated/WallAerialEvent.ts";
import type { WallAerialShotEvent } from "./generated/WallAerialShotEvent.ts";
import type { WhiffEvent } from "./generated/WhiffEvent.ts";
import { applyEventCountDerivedStats, type EventCountStats } from "./eventCountDerivation.ts";
import type { ReplayLoadProgress } from "./replayLoadProgress.ts";
import { createEventDerivedStatsFrameLookup } from "./statsTimelineDerivation.ts";

export type MaterializedStatsTimeline = ReplayStatsTimeline;
export type CompactStatsTimeline = ReplayStatsTimelineScaffold;
export type StatsTimeline = MaterializedStatsTimeline | CompactStatsTimeline;
export type StatsFrame = ReplayStatsFrame;
export type StatsFrameScaffold = ReplayStatsFrameScaffold;
export type StatsEvents = ReplayStatsTimelineEvents;
export type StatsEventPayloadKind = EventPayload["kind"];
export type StatsEventPayload<K extends StatsEventPayloadKind> = Extract<
  EventPayload,
  { kind: K }
>["payload"];
export type TeamStatsSnapshot = GeneratedTeamStatsSnapshot & { event_counts?: EventCountStats };
export type PlayerStatsSnapshot = GeneratedPlayerStatsSnapshot & { event_counts?: EventCountStats };
export type BackboardEvent = BackboardBounceEvent;
export type RushTimelineEvent = RushEvent;
export interface StatsFrameLookup {
  get(frameNumber: number): StatsFrame | undefined;
}
export type {
  CeilingShotEvent,
  DoubleTapEvent,
  FiftyFiftyEvent,
  FlickEvent,
  DodgeEvent,
  LabeledCountEntry,
  LabeledCounts,
  LabeledFloatSumEntry,
  LabeledFloatSums,
  BallCarryEvent,
  Event,
  EventMeta,
  EventPayload,
  EventProperty,
  EventPropertyValue,
  EventTiming,
  MovementEvent,
  MustyFlickEvent,
  OneTimerEvent,
  PassEvent,
  PossessionEvent,
  PositioningActivityEvent,
  PositioningBallRelativeDepthEvent,
  PositioningBallProximityEvent,
  PositioningFieldZoneEvent,
  PositioningTeammateRoleEvent,
  PressureEvent,
  PowerslideEvent,
  ReplayMeta,
  RotationPlayerEvent,
  RotationTeamEvent,
  RushEvent,
  SpeedFlipEvent,
  StatLabel,
  StatsTimelineConfig,
  TimelineEvent,
  TouchBallMovement,
  TouchClassificationEvent,
  HalfFlipEvent,
  HalfVolleyEvent,
  WavedashEvent,
  WallAerialEvent,
  WallAerialShotEvent,
  WhiffEvent,
  BoostPickupEvent,
  RespawnEvent,
  BumpEvent,
  CorePlayerGoalContextEvent,
  CorePlayerScoreboardEvent,
  DodgeResetEvent,
};

export function statsEventEnvelopes(statsTimeline: StatsTimeline): Event[] {
  return statsTimeline.events?.events ?? [];
}

export function statsEventsByStream(statsTimeline: StatsTimeline, stream: string): Event[] {
  return statsEventEnvelopes(statsTimeline).filter((event) => event.meta.stream === stream);
}

export function statsEventPayloads<K extends StatsEventPayloadKind>(
  statsTimeline: StatsTimeline,
  kind: K,
): Array<StatsEventPayload<K>> {
  return statsEventEnvelopes(statsTimeline)
    .filter((event): event is Event & { payload: Extract<EventPayload, { kind: K }> } => {
      return event.payload.kind === kind;
    })
    .map((event) => event.payload.payload) as Array<StatsEventPayload<K>>;
}

export function statsEventPayloadsByStream<K extends StatsEventPayloadKind>(
  statsTimeline: StatsTimeline,
  stream: string,
  kind: K,
): Array<StatsEventPayload<K>> {
  return statsEventEnvelopes(statsTimeline)
    .filter((event): event is Event & { payload: Extract<EventPayload, { kind: K }> } => {
      return event.meta.stream === stream && event.payload.kind === kind;
    })
    .map((event) => event.payload.payload) as Array<StatsEventPayload<K>>;
}

const PLAYER_IDENTITY_FIELDS = new Set(["is_team_0", "name", "player_id"]);

function isEmptyRecord(value: unknown): boolean {
  return (
    !!value && typeof value === "object" && !Array.isArray(value) && Object.keys(value).length === 0
  );
}

function isCompactPlayerIdentity(player: unknown): boolean {
  if (!player || typeof player !== "object" || Array.isArray(player)) {
    return false;
  }
  return Object.keys(player).every((key) => PLAYER_IDENTITY_FIELDS.has(key));
}

function isCompactStatsFrame(frame: StatsFrame | StatsFrameScaffold): boolean {
  return (
    isEmptyRecord(frame.team_zero) &&
    isEmptyRecord(frame.team_one) &&
    frame.players.every((player) => isCompactPlayerIdentity(player))
  );
}

export function isCompactStatsTimeline(statsTimeline: StatsTimeline): boolean {
  return statsTimeline.frames.every((frame) => isCompactStatsFrame(frame));
}

export function createMaterializedStatsFrameLookup(
  statsTimeline: MaterializedStatsTimeline,
): Map<number, StatsFrame> {
  return new Map(
    applyEventCountDerivedStats(statsTimeline).frames.map((frame) => [frame.frame_number, frame]),
  );
}

/**
 * Build a frame lookup for either full legacy timelines or compact event-backed
 * timelines. Compact timelines are materialized lazily from event streams so
 * callers do not need transferred partial-sum snapshots.
 */
export function createStatsFrameLookup(
  statsTimeline: StatsTimeline,
  onProgress?: (progress: ReplayLoadProgress) => void,
  options?: { materializationChunkSize?: number; maxMaterializationChunkSize?: number },
): StatsFrameLookup {
  const compactFrameCount = statsTimeline.frames.filter((frame) =>
    isCompactStatsFrame(frame),
  ).length;
  if (compactFrameCount === statsTimeline.frames.length) {
    return createEventDerivedStatsFrameLookup(statsTimeline, onProgress, options);
  }
  if (compactFrameCount > 0) {
    throw new Error(
      "stats timeline frames must be either all compact scaffolds or all materialized snapshots",
    );
  }
  return createMaterializedStatsFrameLookup(statsTimeline as MaterializedStatsTimeline);
}

export function getStatsFrameForReplayFrame(
  statsFrameLookup: StatsFrameLookup,
  replayFrameNumber: number,
): StatsFrame | null {
  return statsFrameLookup.get(replayFrameNumber) ?? null;
}
