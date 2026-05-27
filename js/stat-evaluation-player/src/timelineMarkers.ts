import type { ReplayModel, ReplayTimelineEvent, ReplayTimelineEventKind } from "@rlrml/player";
import { buildFiftyFiftyMarkers } from "./fiftyFiftyOverlay.ts";
import { buildCeilingShotMarkers } from "./ceilingShotOverlay.ts";
import { buildTouchMarkers, playerIdToString } from "./touchOverlay.ts";
import type { StatsTimeline } from "./statsTimeline.ts";
import {
  BLUE_TIMELINE_COLOR,
  NEUTRAL_TIMELINE_COLOR,
  ORANGE_TIMELINE_COLOR,
  getReplayFrameTime,
  getReplayPlayerName,
} from "./timelineMarkerHelpers.ts";
import {
  buildBallCarryTimelineEvents,
  buildDodgeResetTimelineEvents,
  buildHalfFlipTimelineEvents,
  buildPowerslideTimelineEvents,
  buildSpeedFlipTimelineEvents,
  buildWavedashTimelineEvents,
} from "./timelineMarkersMovement.ts";
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
  buildBallCarryTimelineEvents,
  buildDodgeResetTimelineEvents,
  buildHalfFlipTimelineEvents,
  buildPowerslideTimelineEvents,
  buildSpeedFlipTimelineEvents,
  buildWavedashTimelineEvents,
} from "./timelineMarkersMovement.ts";

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

export function buildFiftyFiftyTimelineEvents(
  statsTimeline: StatsTimeline,
  replay: ReplayModel,
): ReplayTimelineEvent[] {
  return buildFiftyFiftyMarkers(statsTimeline, replay).map((marker) => ({
    id: marker.id,
    time: marker.time,
    frame: marker.frame,
    kind: "fifty-fifty",
    label: marker.label,
    shortLabel: marker.label.startsWith("Kickoff 50/50") ? "KO" : "50",
    isTeamZero: marker.winnerIsTeamZero,
    color:
      marker.winnerIsTeamZero === null
        ? NEUTRAL_TIMELINE_COLOR
        : marker.winnerIsTeamZero
          ? BLUE_TIMELINE_COLOR
          : ORANGE_TIMELINE_COLOR,
  }));
}

export function buildMustyFlickTimelineEvents(
  statsTimeline: StatsTimeline,
  replay: ReplayModel,
): ReplayTimelineEvent[] {
  return (statsTimeline.events?.musty_flick ?? []).map((event, index) => {
    const playerId = playerIdToString(event.player);
    const playerName = getReplayPlayerName(replay, playerId);
    return {
      id: `musty-flick:${event.frame}:${playerId}:${index + 1}`,
      time: getReplayFrameTime(replay, event.frame, event.time),
      frame: event.frame,
      kind: "musty-flick",
      label: `${playerName} musty flick`,
      shortLabel: "M",
      playerId,
      playerName,
      isTeamZero: event.is_team_0,
      color: event.is_team_0 ? BLUE_TIMELINE_COLOR : ORANGE_TIMELINE_COLOR,
    };
  });
}

export function buildFlickTimelineEvents(
  statsTimeline: StatsTimeline,
  replay: ReplayModel,
): ReplayTimelineEvent[] {
  return (statsTimeline.events?.flick ?? []).map((event, index) => {
    const playerId = playerIdToString(event.player);
    const playerName = getReplayPlayerName(replay, playerId);
    return {
      id: `flick:${event.frame}:${playerId}:${index + 1}`,
      time: getReplayFrameTime(replay, event.frame, event.time),
      frame: event.frame,
      kind: "flick",
      label: `${playerName} flick`,
      shortLabel: "F",
      playerId,
      playerName,
      isTeamZero: event.is_team_0,
      color: event.is_team_0 ? BLUE_TIMELINE_COLOR : ORANGE_TIMELINE_COLOR,
    };
  });
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

export function buildBackboardTimelineEvents(
  statsTimeline: StatsTimeline,
  replay: ReplayModel,
): ReplayTimelineEvent[] {
  return statsTimeline.events.backboard.map((event, index) => {
    const playerId = playerIdToString(event.player);
    const playerName = replay.players.find((player) => player.id === playerId)?.name ?? playerId;
    return {
      id: `backboard:${event.frame}:${playerId}:${index}`,
      time: getReplayFrameTime(replay, event.frame, event.time),
      frame: event.frame,
      kind: "backboard",
      label: `${playerName} backboard`,
      shortLabel: "BB",
      playerId,
      playerName,
      isTeamZero: event.is_team_0,
      color: event.is_team_0 ? BLUE_TIMELINE_COLOR : ORANGE_TIMELINE_COLOR,
    };
  });
}

export function buildCeilingShotTimelineEvents(
  statsTimeline: StatsTimeline,
  replay: ReplayModel,
): ReplayTimelineEvent[] {
  return buildCeilingShotMarkers(statsTimeline, replay).map((marker) => ({
    id: marker.id,
    time: marker.time,
    frame: marker.frame,
    kind: "ceiling-shot",
    label: `${marker.playerName} ceiling shot ${marker.qualityLabel}`,
    shortLabel: "CS",
    playerId: marker.playerId,
    playerName: marker.playerName,
    isTeamZero: marker.isTeamZero,
    color: marker.isTeamZero ? BLUE_TIMELINE_COLOR : ORANGE_TIMELINE_COLOR,
  }));
}

export function buildWallAerialTimelineEvents(
  statsTimeline: StatsTimeline,
  replay: ReplayModel,
): ReplayTimelineEvent[] {
  return statsTimeline.events.wall_aerial.map((event, index) => {
    const playerId = playerIdToString(event.player);
    const playerName = getReplayPlayerName(replay, playerId);
    const eventTime = getReplayFrameTime(replay, event.frame, event.time);
    const qualityPercent = Math.round(event.confidence * 100);
    const wallLabel = formatMechanicKind(event.wall).toLowerCase();

    return {
      id: `wall-aerial:${event.frame}:${playerId}:${index}`,
      time: eventTime,
      frame: event.frame,
      kind: "wall-aerial",
      label: `${playerName} wall-to-air setup ${qualityPercent}% | ${wallLabel} wall`,
      shortLabel: "W2A",
      playerId,
      playerName,
      isTeamZero: event.is_team_0,
      color: event.is_team_0 ? BLUE_TIMELINE_COLOR : ORANGE_TIMELINE_COLOR,
    };
  });
}

export function buildWallAerialShotTimelineEvents(
  statsTimeline: StatsTimeline,
  replay: ReplayModel,
): ReplayTimelineEvent[] {
  return statsTimeline.events.wall_aerial_shot.map((event, index) => {
    const playerId = playerIdToString(event.player);
    const playerName = getReplayPlayerName(replay, playerId);
    const eventTime = getReplayFrameTime(replay, event.frame, event.time);
    const qualityPercent = Math.round(event.confidence * 100);
    const wallLabel = formatMechanicKind(event.wall).toLowerCase();

    return {
      id: `wall-aerial-shot:${event.frame}:${playerId}:${index}`,
      time: eventTime,
      frame: event.frame,
      kind: "wall-aerial-shot",
      label: `${playerName} wall aerial shot ${qualityPercent}% | ${wallLabel} wall`,
      shortLabel: "WS",
      playerId,
      playerName,
      isTeamZero: event.is_team_0,
      color: event.is_team_0 ? BLUE_TIMELINE_COLOR : ORANGE_TIMELINE_COLOR,
    };
  });
}

export function buildDoubleTapTimelineEvents(
  statsTimeline: StatsTimeline,
  replay: ReplayModel,
): ReplayTimelineEvent[] {
  return statsTimeline.events.double_tap.map((event, index) => {
    const playerId = playerIdToString(event.player);
    const playerName = getReplayPlayerName(replay, playerId);
    return {
      id: `double-tap:${event.frame}:${playerId}:${index}`,
      time: getReplayFrameTime(replay, event.frame, event.time),
      frame: event.frame,
      kind: "double-tap",
      label: `${playerName} double tap`,
      shortLabel: "DT",
      playerId,
      playerName,
      isTeamZero: event.is_team_0,
      color: event.is_team_0 ? BLUE_TIMELINE_COLOR : ORANGE_TIMELINE_COLOR,
    };
  });
}

export function buildCenterTimelineEvents(
  statsTimeline: StatsTimeline,
  replay: ReplayModel,
): ReplayTimelineEvent[] {
  return statsTimeline.events.center.map((event, index) => {
    const playerId = playerIdToString(event.player);
    const playerName = getReplayPlayerName(replay, playerId);
    const eventTime = getReplayFrameTime(replay, event.frame, event.time);
    const distance = Math.round(event.lateral_centering_distance);

    return {
      id: `center:${event.frame}:${playerId}:${index}`,
      time: eventTime,
      frame: event.frame,
      kind: "center",
      label: `${playerName} center | ${distance}uu lateral`,
      shortLabel: "C",
      playerId,
      playerName,
      isTeamZero: event.is_team_0,
      color: event.is_team_0 ? BLUE_TIMELINE_COLOR : ORANGE_TIMELINE_COLOR,
    };
  });
}

export function buildOneTimerTimelineEvents(
  statsTimeline: StatsTimeline,
  replay: ReplayModel,
): ReplayTimelineEvent[] {
  return statsTimeline.events.one_timer.map((event, index) => {
    const playerId = playerIdToString(event.player);
    const passerId = playerIdToString(event.passer);
    const playerName = getReplayPlayerName(replay, playerId);
    const passerName = getReplayPlayerName(replay, passerId);
    const eventTime = getReplayFrameTime(replay, event.frame, event.time);
    const ballSpeed = Math.round(event.ball_speed);

    return {
      id: `one-timer:${event.frame}:${passerId}:${playerId}:${index}`,
      time: eventTime,
      frame: event.frame,
      kind: "one-timer",
      label: `${playerName} one-timer from ${passerName} | ${ballSpeed}uu/s`,
      shortLabel: "OT",
      playerId,
      playerName,
      secondaryPlayerId: passerId,
      secondaryPlayerName: passerName,
      isTeamZero: event.is_team_0,
      color: event.is_team_0 ? BLUE_TIMELINE_COLOR : ORANGE_TIMELINE_COLOR,
    };
  });
}

function formatPassKind(kind: string): string {
  return formatMechanicKind(kind.replace(/_pass$/, ""));
}

export function buildPassTimelineEvents(
  statsTimeline: StatsTimeline,
  replay: ReplayModel,
): ReplayTimelineEvent[] {
  return statsTimeline.events.pass.map((event, index) => {
    const passerId = playerIdToString(event.passer);
    const receiverId = playerIdToString(event.receiver);
    const passerName = getReplayPlayerName(replay, passerId);
    const receiverName = getReplayPlayerName(replay, receiverId);
    const eventTime = getReplayFrameTime(replay, event.frame, event.time);
    const distance = Math.round(event.ball_travel_distance);
    const kindLabel = formatPassKind(event.pass_kind);

    return {
      id: `pass:${event.frame}:${passerId}:${receiverId}:${index}`,
      time: eventTime,
      frame: event.frame,
      kind: "pass",
      label: `${passerName} to ${receiverName} ${kindLabel.toLowerCase()} pass | ${distance}uu`,
      shortLabel: "P",
      playerId: passerId,
      playerName: passerName,
      secondaryPlayerId: receiverId,
      secondaryPlayerName: receiverName,
      isTeamZero: event.is_team_0,
      color: event.is_team_0 ? BLUE_TIMELINE_COLOR : ORANGE_TIMELINE_COLOR,
    };
  });
}

export function buildHalfVolleyTimelineEvents(
  statsTimeline: StatsTimeline,
  replay: ReplayModel,
): ReplayTimelineEvent[] {
  return statsTimeline.events.half_volley.map((event, index) => {
    const playerId = playerIdToString(event.player);
    const playerName = getReplayPlayerName(replay, playerId);
    const eventTime = getReplayFrameTime(replay, event.frame, event.time);
    const ballSpeed = Math.round(event.ball_speed);

    return {
      id: `half-volley:${event.frame}:${playerId}:${index}`,
      time: eventTime,
      frame: event.frame,
      kind: "half-volley",
      label: `${playerName} half volley | ${ballSpeed}uu/s`,
      shortLabel: "HV",
      playerId,
      playerName,
      isTeamZero: event.is_team_0,
      color: event.is_team_0 ? BLUE_TIMELINE_COLOR : ORANGE_TIMELINE_COLOR,
    };
  });
}

export function buildRushTimelineEvents(
  statsTimeline: StatsTimeline,
  replay: ReplayModel,
): ReplayTimelineEvent[] {
  return statsTimeline.events.rush.map((event, index) => {
    const eventTime = getReplayFrameTime(replay, event.end_frame, event.end_time);
    const matchupLabel = `${event.attackers}v${event.defenders}`;
    const teamName = event.is_team_0 ? "Blue" : "Orange";

    return {
      id: `rush:${event.start_frame}:${event.end_frame}:${index}`,
      time: eventTime,
      frame: event.end_frame,
      kind: "rush",
      label: `${teamName} rush ${matchupLabel}`,
      shortLabel: "R",
      playerId: null,
      playerName: null,
      isTeamZero: event.is_team_0,
      color: event.is_team_0 ? BLUE_TIMELINE_COLOR : ORANGE_TIMELINE_COLOR,
    };
  });
}

function formatGoalTagKind(kind: string): string {
  return formatMechanicKind(kind.replace(/_goal$/, ""));
}

export function buildGoalTagTimelineEvents(
  statsTimeline: StatsTimeline,
  replay: ReplayModel,
): ReplayTimelineEvent[] {
  return statsTimeline.events.goal_tags.map((event, index) => {
    const scorerId = event.scorer ? playerIdToString(event.scorer) : null;
    const scorerName = scorerId ? getReplayPlayerName(replay, scorerId) : null;
    const eventTime = getReplayFrameTime(replay, event.frame, event.time);
    const tagLabel = formatGoalTagKind(event.kind);
    const confidencePercent = Math.round(event.confidence * 100);

    return {
      id: `goal-tag:${event.goal_index}:${event.kind}:${index}`,
      time: eventTime,
      frame: event.frame,
      kind: "goal-tag",
      label: `${scorerName ?? "Goal"} ${tagLabel.toLowerCase()} goal ${confidencePercent}%`,
      shortLabel: "GT",
      playerId: scorerId,
      playerName: scorerName,
      isTeamZero: event.scoring_team_is_team_0,
      color: event.scoring_team_is_team_0 ? BLUE_TIMELINE_COLOR : ORANGE_TIMELINE_COLOR,
    };
  });
}

export function buildGoalContextTimelineEvents(
  statsTimeline: StatsTimeline,
  replay: ReplayModel,
): ReplayTimelineEvent[] {
  return statsTimeline.events.goal_context.map((event, index) => {
    const scorerId = event.scorer ? playerIdToString(event.scorer) : null;
    const scorerName = scorerId ? getReplayPlayerName(replay, scorerId) : null;
    const eventTime = getReplayFrameTime(replay, event.frame, event.time);

    return {
      id: `goal-context:${event.frame}:${scorerId ?? "team"}:${index}`,
      time: eventTime,
      frame: event.frame,
      kind: "goal-context",
      label: scorerName ? `${scorerName} goal context` : "Goal context",
      shortLabel: "GC",
      playerId: scorerId,
      playerName: scorerName,
      isTeamZero: event.scoring_team_is_team_0,
      color: event.scoring_team_is_team_0 ? BLUE_TIMELINE_COLOR : ORANGE_TIMELINE_COLOR,
    };
  });
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
