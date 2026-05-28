import type {
  ReplayModel,
  ReplayTimelineEvent,
  ReplayTimelineEventKind,
} from "@rlrml/player";
import { buildTouchMarkers, playerIdToString } from "./touchOverlay.ts";
import type { StatsTimeline } from "./statsTimeline.ts";
import {
  BLUE_TIMELINE_COLOR,
  ORANGE_TIMELINE_COLOR,
  getReplayFrameTime,
} from "./timelineMarkerHelpers.ts";
import {
  buildBackboardTimelineEvents,
  buildCeilingShotTimelineEvents,
  buildDoubleTapTimelineEvents,
  buildFiftyFiftyTimelineEvents,
  buildFlickTimelineEvents,
  buildMustyFlickTimelineEvents,
  buildWallAerialShotTimelineEvents,
  buildWallAerialTimelineEvents,
} from "./timelineMarkersMechanicShots.ts";
import {
  buildBallCarryTimelineEvents,
  buildDodgeResetTimelineEvents,
  buildHalfFlipTimelineEvents,
  buildPowerslideTimelineEvents,
  buildSpeedFlipTimelineEvents,
  buildWavedashTimelineEvents,
} from "./timelineMarkersMovement.ts";
import {
  buildCenterTimelineEvents,
  buildGoalContextTimelineEvents,
  buildGoalTagTimelineEvents,
  buildHalfVolleyTimelineEvents,
  buildOneTimerTimelineEvents,
  buildPassTimelineEvents,
  buildRushTimelineEvents,
} from "./timelineMarkersTeamplay.ts";
import {
  formatMechanicKind,
  getMechanicKinds,
  isVisibleMechanicKind,
  mechanicKindToModuleId,
  mechanicShortLabel,
} from "./timelineMechanics.ts";
export {
  formatMechanicKind,
  getMechanicKinds,
  isVisibleMechanicKind,
  mechanicKindToModuleId,
} from "./timelineMechanics.ts";
export {
  buildBackboardTimelineEvents,
  buildCeilingShotTimelineEvents,
  buildDoubleTapTimelineEvents,
  buildFiftyFiftyTimelineEvents,
  buildFlickTimelineEvents,
  buildMustyFlickTimelineEvents,
  buildWallAerialShotTimelineEvents,
  buildWallAerialTimelineEvents,
} from "./timelineMarkersMechanicShots.ts";
export {
  buildBallCarryTimelineEvents,
  buildDodgeResetTimelineEvents,
  buildHalfFlipTimelineEvents,
  buildPowerslideTimelineEvents,
  buildSpeedFlipTimelineEvents,
  buildWavedashTimelineEvents,
} from "./timelineMarkersMovement.ts";
export {
  buildCenterTimelineEvents,
  buildGoalContextTimelineEvents,
  buildGoalTagTimelineEvents,
  buildHalfVolleyTimelineEvents,
  buildOneTimerTimelineEvents,
  buildPassTimelineEvents,
  buildRushTimelineEvents,
} from "./timelineMarkersTeamplay.ts";

export function buildMechanicTimelineEvents(
  statsTimeline: StatsTimeline,
  replay: ReplayModel,
  enabledKinds?: Iterable<string>,
): ReplayTimelineEvent[] {
  const enabled = enabledKinds ? new Set(enabledKinds) : null;
  const playerNames = new Map(replay.players.map((player) => [player.id, player.name]));

  return (statsTimeline.events.mechanics ?? [])
    .filter(
      (event) =>
        isVisibleMechanicKind(event.kind) &&
        event.timing.type === "moment" &&
        (!enabled || enabled.has(event.kind)),
    )
    .map((event) => {
      const playerId = playerIdToString(event.player_id);
      const playerName = playerNames.get(playerId) ?? playerId;
      const mechanicLabel = formatMechanicKind(event.kind);
      if (event.timing.type !== "moment") {
        throw new Error("unreachable non-moment mechanic event");
      }

      return {
        id: event.id,
        time: getReplayFrameTime(replay, event.timing.frame, event.timing.time),
        frame: event.timing.frame,
        kind: event.kind,
        label: `${playerName} ${mechanicLabel.toLowerCase()}`,
        shortLabel: mechanicShortLabel(event.kind),
        playerId,
        playerName,
        isTeamZero: event.is_team_0,
        color: event.is_team_0 ? BLUE_TIMELINE_COLOR : ORANGE_TIMELINE_COLOR,
      };
    });
}

export function buildMechanicPlaylistEvents(
  statsTimeline: StatsTimeline,
  replay: ReplayModel,
  enabledKinds?: Iterable<string>,
): ReplayTimelineEvent[] {
  const enabled = enabledKinds ? new Set(enabledKinds) : null;
  const playerNames = new Map(replay.players.map((player) => [player.id, player.name]));

  return (statsTimeline.events.mechanics ?? [])
    .filter((event) => isVisibleMechanicKind(event.kind) && (!enabled || enabled.has(event.kind)))
    .map((event) => {
      const playerId = playerIdToString(event.player_id);
      const playerName = playerNames.get(playerId) ?? playerId;
      const mechanicLabel = formatMechanicKind(event.kind);
      const timing =
        event.timing.type === "moment"
          ? {
              frame: event.timing.frame,
              time: event.timing.time,
            }
          : {
              frame: event.timing.end_frame,
              time: event.timing.end_time,
            };

      return {
        id: `${event.id}:playlist`,
        time: getReplayFrameTime(replay, timing.frame, timing.time),
        frame: timing.frame,
        kind: event.kind,
        label: `${playerName} ${mechanicLabel.toLowerCase()}`,
        shortLabel: mechanicShortLabel(event.kind),
        playerId,
        playerName,
        isTeamZero: event.is_team_0,
        color: event.is_team_0 ? BLUE_TIMELINE_COLOR : ORANGE_TIMELINE_COLOR,
      };
    });
}

export function getReplayTimelineEventKinds(
  activeSourceIds: Iterable<string>,
): ReplayTimelineEventKind[] {
  const active = new Set(activeSourceIds);
  const allowedKinds = new Set<ReplayTimelineEventKind>(["goal"]);

  if (active.has("core")) {
    allowedKinds.add("save");
    allowedKinds.add("shot");
    allowedKinds.add("assist");
  }

  if (active.has("demo")) {
    allowedKinds.add("demo");
  }

  return [...allowedKinds];
}

export function filterReplayTimelineEvents(
  replay: ReplayModel,
  activeSourceIds: Iterable<string>,
): ReplayTimelineEvent[] {
  const allowedKinds = new Set(getReplayTimelineEventKinds(activeSourceIds));
  return replay.timelineEvents.filter((event) => allowedKinds.has(event.kind));
}

export function buildTouchTimelineEvents(
  statsTimeline: StatsTimeline,
  replay: ReplayModel,
): ReplayTimelineEvent[] {
  return buildTouchMarkers(statsTimeline, replay).map((marker) => ({
    id: marker.id,
    time: marker.time,
    frame: marker.frame,
    kind: "touch",
    label: `${marker.playerName} touch`,
    shortLabel: "T",
    playerId: marker.playerId,
    playerName: marker.playerName,
    isTeamZero: marker.isTeamZero,
    color: marker.isTeamZero ? BLUE_TIMELINE_COLOR : ORANGE_TIMELINE_COLOR,
  }));
}

export function buildBumpTimelineEvents(
  statsTimeline: StatsTimeline,
  replay: ReplayModel,
): ReplayTimelineEvent[] {
  return statsTimeline.events.bump.map((event, index) => {
    const initiatorId = playerIdToString(event.initiator);
    const victimId = playerIdToString(event.victim);
    const initiatorName =
      replay.players.find((player) => player.id === initiatorId)?.name ?? initiatorId;
    const victimName = replay.players.find((player) => player.id === victimId)?.name ?? victimId;
    const eventTime = getReplayFrameTime(replay, event.frame, event.time);
    const confidencePercent = Math.round(event.confidence * 100);

    return {
      id: `bump:${event.frame}:${initiatorId}:${victimId}:${index}`,
      time: eventTime,
      frame: event.frame,
      kind: "bump",
      label: `${initiatorName} bumped ${victimName} ${confidencePercent}%`,
      shortLabel: "B",
      playerId: initiatorId,
      playerName: initiatorName,
      isTeamZero: event.initiator_is_team_0,
      color: event.initiator_is_team_0 ? BLUE_TIMELINE_COLOR : ORANGE_TIMELINE_COLOR,
    };
  });
}

function getWhiffShortLabel(event: StatsTimeline["events"]["whiff"][number]): string {
  if (event.kind === "beaten_to_ball") {
    return "BT";
  }
  if (event.dodge_active) {
    return "DW";
  }
  if (event.aerial) {
    return "AW";
  }
  return "W";
}

function getWhiffKindLabel(event: StatsTimeline["events"]["whiff"][number]): string {
  const labels = [event.aerial ? "aerial" : "grounded"];
  if (event.dodge_active) {
    labels.push("dodge");
  }
  return labels.join(" ");
}

function getWhiffOutcomeLabel(event: StatsTimeline["events"]["whiff"][number]): string {
  return event.kind === "beaten_to_ball" ? "beaten to ball" : "whiff";
}

export function buildWhiffTimelineEvents(
  statsTimeline: StatsTimeline,
  replay: ReplayModel,
): ReplayTimelineEvent[] {
  return statsTimeline.events.whiff.map((event, index) => {
    const playerId = playerIdToString(event.player);
    const playerName = replay.players.find((player) => player.id === playerId)?.name ?? playerId;
    const eventTime = getReplayFrameTime(replay, event.frame, event.time);
    const closestApproach = Math.round(event.closest_approach_distance);
    const approachSpeed = Math.round(event.approach_speed);

    return {
      id: `whiff:${event.frame}:${playerId}:${index}`,
      time: eventTime,
      frame: event.frame,
      kind: "whiff",
      label: `${playerName} ${getWhiffKindLabel(event)} ${getWhiffOutcomeLabel(event)} | ${closestApproach}uu closest, ${approachSpeed}uu/s`,
      shortLabel: getWhiffShortLabel(event),
      playerId,
      playerName,
      isTeamZero: event.is_team_0,
      color: event.is_team_0 ? BLUE_TIMELINE_COLOR : ORANGE_TIMELINE_COLOR,
    };
  });
}

export function countEnabledTimelineEvents(
  activeSourceIds: Iterable<string>,
  replay: ReplayModel,
  statsTimeline: StatsTimeline,
): number {
  const mechanicModuleIds = new Set(getMechanicKinds(statsTimeline).map(mechanicKindToModuleId));
  const active = new Set(
    [...activeSourceIds].filter((sourceId) => !mechanicModuleIds.has(sourceId)),
  );
  let count = filterReplayTimelineEvents(replay, active).length;

  if (active.has("fifty-fifty")) {
    count += buildFiftyFiftyTimelineEvents(statsTimeline, replay).length;
  }

  if (active.has("musty-flick")) {
    count += buildMustyFlickTimelineEvents(statsTimeline, replay).length;
  }

  if (active.has("flick")) {
    count += buildFlickTimelineEvents(statsTimeline, replay).length;
  }

  if (active.has("backboard")) {
    count += buildBackboardTimelineEvents(statsTimeline, replay).length;
  }

  if (active.has("ceiling-shot")) {
    count += buildCeilingShotTimelineEvents(statsTimeline, replay).length;
  }

  if (active.has("wall-aerial")) {
    count += buildWallAerialTimelineEvents(statsTimeline, replay).length;
  }

  if (active.has("wall-aerial-shot")) {
    count += buildWallAerialShotTimelineEvents(statsTimeline, replay).length;
  }

  if (active.has("double-tap")) {
    count += buildDoubleTapTimelineEvents(statsTimeline, replay).length;
  }

  if (active.has("center")) {
    count += buildCenterTimelineEvents(statsTimeline, replay).length;
  }

  if (active.has("one-timer")) {
    count += buildOneTimerTimelineEvents(statsTimeline, replay).length;
  }

  if (active.has("pass")) {
    count += buildPassTimelineEvents(statsTimeline, replay).length;
  }

  if (active.has("touch")) {
    count += buildTouchTimelineEvents(statsTimeline, replay).length;
  }

  if (active.has("dodge-reset")) {
    count += buildDodgeResetTimelineEvents(statsTimeline, replay).length;
  }

  if (active.has("ball-carry")) {
    count += buildBallCarryTimelineEvents(statsTimeline, replay).length;
  }

  if (active.has("powerslide")) {
    count += buildPowerslideTimelineEvents(statsTimeline, replay).length;
  }

  if (active.has("speed-flip")) {
    count += buildSpeedFlipTimelineEvents(statsTimeline, replay).length;
  }

  if (active.has("half-flip")) {
    count += buildHalfFlipTimelineEvents(statsTimeline, replay).length;
  }

  if (active.has("half-volley")) {
    count += buildHalfVolleyTimelineEvents(statsTimeline, replay).length;
  }

  if (active.has("rush")) {
    count += buildRushTimelineEvents(statsTimeline, replay).length;
  }

  if (active.has("wavedash")) {
    count += buildWavedashTimelineEvents(statsTimeline, replay).length;
  }

  if (active.has("whiff")) {
    count += buildWhiffTimelineEvents(statsTimeline, replay).length;
  }

  if (active.has("bump")) {
    count += buildBumpTimelineEvents(statsTimeline, replay).length;
  }

  return count;
}
