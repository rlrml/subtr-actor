import type { ReplayModel, ReplayTimelineEvent } from "@rlrml/player";
import { buildCeilingShotMarkers } from "./ceilingShotOverlay.ts";
import { buildFiftyFiftyMarkers } from "./fiftyFiftyOverlay.ts";
import { playerIdToString } from "./touchOverlay.ts";
import type { StatsTimeline } from "./statsTimeline.ts";
import {
  BLUE_TIMELINE_COLOR,
  NEUTRAL_TIMELINE_COLOR,
  ORANGE_TIMELINE_COLOR,
  getReplayFrameTime,
  getReplayPlayerName,
} from "./timelineMarkerHelpers.ts";
import { formatMechanicKind } from "./timelineMechanics.ts";

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
