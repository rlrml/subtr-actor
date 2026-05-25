import type { BackboardBounceEvent } from "./generated/BackboardBounceEvent.ts";
import type { BallCarryEvent } from "./generated/BallCarryEvent.ts";
import type { BoostLedgerEvent } from "./generated/BoostLedgerEvent.ts";
import type { BoostLedgerTransactionKind } from "./generated/BoostLedgerTransactionKind.ts";
import type { BoostPickupComparisonEvent } from "./generated/BoostPickupComparisonEvent.ts";
import type { BumpEvent } from "./generated/BumpEvent.ts";
import type { CeilingShotEvent } from "./generated/CeilingShotEvent.ts";
import type { CorePlayerStatsEvent } from "./generated/CorePlayerStatsEvent.ts";
import type { CoreTeamStatsEvent } from "./generated/CoreTeamStatsEvent.ts";
import type { DodgeResetEvent } from "./generated/DodgeResetEvent.ts";
import type { DoubleTapEvent } from "./generated/DoubleTapEvent.ts";
import type { FiftyFiftyEvent } from "./generated/FiftyFiftyEvent.ts";
import type { FlickEvent } from "./generated/FlickEvent.ts";
import type { LabeledCountEntry } from "./generated/LabeledCountEntry.ts";
import type { LabeledCounts } from "./generated/LabeledCounts.ts";
import type { LabeledFloatSumEntry } from "./generated/LabeledFloatSumEntry.ts";
import type { LabeledFloatSums } from "./generated/LabeledFloatSums.ts";
import type { MechanicEvent } from "./generated/MechanicEvent.ts";
import type { MechanicEventProperty } from "./generated/MechanicEventProperty.ts";
import type { MechanicEventPropertyValue } from "./generated/MechanicEventPropertyValue.ts";
import type { MechanicTiming } from "./generated/MechanicTiming.ts";
import type { MovementEvent } from "./generated/MovementEvent.ts";
import type { MustyFlickEvent } from "./generated/MustyFlickEvent.ts";
import type { OneTimerEvent } from "./generated/OneTimerEvent.ts";
import type { PassEvent } from "./generated/PassEvent.ts";
import type { PlayerStatsSnapshot as GeneratedPlayerStatsSnapshot } from "./generated/PlayerStatsSnapshot.ts";
import type { PossessionEvent } from "./generated/PossessionEvent.ts";
import type { PositioningEvent } from "./generated/PositioningEvent.ts";
import type { PressureEvent } from "./generated/PressureEvent.ts";
import type { PowerslideEvent } from "./generated/PowerslideEvent.ts";
import type { ReplayMeta } from "./generated/ReplayMeta.ts";
import type { ReplayStatsFrame } from "./generated/ReplayStatsFrame.ts";
import type { ReplayStatsTimeline } from "./generated/ReplayStatsTimeline.ts";
import type { ReplayStatsTimelineEvents } from "./generated/ReplayStatsTimelineEvents.ts";
import type { RotationPlayerEvent } from "./generated/RotationPlayerEvent.ts";
import type { RotationTeamEvent } from "./generated/RotationTeamEvent.ts";
import type { RushEvent } from "./generated/RushEvent.ts";
import type { SpeedFlipEvent } from "./generated/SpeedFlipEvent.ts";
import type { StatLabel } from "./generated/StatLabel.ts";
import type { StatsTimelineConfig } from "./generated/StatsTimelineConfig.ts";
import type { TeamStatsSnapshot as GeneratedTeamStatsSnapshot } from "./generated/TeamStatsSnapshot.ts";
import type { TimelineEvent } from "./generated/TimelineEvent.ts";
import type { TouchBallMovementEvent } from "./generated/TouchBallMovementEvent.ts";
import type { TouchLastTouchEvent } from "./generated/TouchLastTouchEvent.ts";
import type { TouchStatsEvent } from "./generated/TouchStatsEvent.ts";
import type { HalfFlipEvent } from "./generated/HalfFlipEvent.ts";
import type { HalfVolleyEvent } from "./generated/HalfVolleyEvent.ts";
import type { WavedashEvent } from "./generated/WavedashEvent.ts";
import type { WallAerialEvent } from "./generated/WallAerialEvent.ts";
import type { WallAerialShotEvent } from "./generated/WallAerialShotEvent.ts";
import type { WhiffEvent } from "./generated/WhiffEvent.ts";

export type StatsTimeline = ReplayStatsTimeline;
export type StatsFrame = ReplayStatsFrame;
export type StatsEvents = ReplayStatsTimelineEvents;
export type TeamStatsSnapshot = GeneratedTeamStatsSnapshot;
export type PlayerStatsSnapshot = GeneratedPlayerStatsSnapshot;
export type BackboardEvent = BackboardBounceEvent;
export type RushTimelineEvent = RushEvent;
export type {
  CeilingShotEvent,
  DoubleTapEvent,
  FiftyFiftyEvent,
  FlickEvent,
  LabeledCountEntry,
  LabeledCounts,
  LabeledFloatSumEntry,
  LabeledFloatSums,
  BallCarryEvent,
  MechanicEvent,
  MechanicEventProperty,
  MechanicEventPropertyValue,
  MechanicTiming,
  MovementEvent,
  MustyFlickEvent,
  OneTimerEvent,
  PassEvent,
  PossessionEvent,
  PositioningEvent,
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
  TouchBallMovementEvent,
  TouchLastTouchEvent,
  TouchStatsEvent,
  HalfFlipEvent,
  HalfVolleyEvent,
  WavedashEvent,
  WallAerialEvent,
  WallAerialShotEvent,
  WhiffEvent,
  BoostLedgerEvent,
  BoostLedgerTransactionKind,
  BoostPickupComparisonEvent,
  BumpEvent,
  CorePlayerStatsEvent,
  CoreTeamStatsEvent,
  DodgeResetEvent,
};

export function createStatsFrameLookup(statsTimeline: StatsTimeline): Map<number, StatsFrame> {
  return new Map(statsTimeline.frames.map((frame) => [frame.frame_number, frame]));
}

export function getStatsFrameForReplayFrame(
  statsFrameLookup: Map<number, StatsFrame>,
  replayFrameNumber: number,
): StatsFrame | null {
  return statsFrameLookup.get(replayFrameNumber) ?? null;
}
