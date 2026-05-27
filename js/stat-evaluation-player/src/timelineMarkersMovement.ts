import type { ReplayModel, ReplayTimelineEvent } from "@rlrml/player";
import { playerIdToString } from "./touchOverlay.ts";
import type { StatsTimeline } from "./statsTimeline.ts";
import {
  BLUE_TIMELINE_COLOR,
  ORANGE_TIMELINE_COLOR,
  getReplayFrameTime,
  getReplayPlayerName,
} from "./timelineMarkerHelpers.ts";

export function buildDodgeResetTimelineEvents(
  statsTimeline: StatsTimeline,
  replay: ReplayModel,
): ReplayTimelineEvent[] {
  return (statsTimeline.events?.dodge_reset ?? [])
    .filter((event) => !event.on_ball)
    .map((event) => {
      const playerId = playerIdToString(event.player);
      const playerName = getReplayPlayerName(replay, playerId);
      return {
        id: `dodge-reset:${event.frame}:${playerId}:${event.counter_value}:air`,
        time: getReplayFrameTime(replay, event.frame, event.time),
        frame: event.frame,
        kind: "dodge-reset",
        label: `${playerName} dodge refresh`,
        shortLabel: "DR",
        playerId,
        playerName,
        isTeamZero: event.is_team_0,
        color: event.is_team_0 ? BLUE_TIMELINE_COLOR : ORANGE_TIMELINE_COLOR,
      };
    });
}

export function buildBallCarryTimelineEvents(
  statsTimeline: StatsTimeline,
  replay: ReplayModel,
): ReplayTimelineEvent[] {
  return (statsTimeline.events?.ball_carry ?? [])
    .filter((event) => event.kind === "carry")
    .map((event, index) => {
      const playerId = playerIdToString(event.player_id);
      const playerName = getReplayPlayerName(replay, playerId);
      return {
        id: `ball-carry:${event.end_frame}:${playerId}:${index + 1}`,
        time: getReplayFrameTime(replay, event.end_frame, event.end_time),
        frame: event.end_frame,
        kind: "ball-carry",
        label: `${playerName} ball carry`,
        shortLabel: "BC",
        playerId,
        playerName,
        isTeamZero: event.is_team_0,
        color: event.is_team_0 ? BLUE_TIMELINE_COLOR : ORANGE_TIMELINE_COLOR,
      };
    });
}

export function buildPowerslideTimelineEvents(
  statsTimeline: StatsTimeline,
  replay: ReplayModel,
): ReplayTimelineEvent[] {
  return (statsTimeline.events?.powerslide ?? [])
    .filter((event) => event.active)
    .map((event, index) => {
      const playerId = playerIdToString(event.player);
      const playerName = getReplayPlayerName(replay, playerId);
      return {
        id: `powerslide:${event.frame}:${playerId}:${index + 1}`,
        time: getReplayFrameTime(replay, event.frame, event.time),
        frame: event.frame,
        kind: "powerslide",
        label: `${playerName} powerslide`,
        shortLabel: "PS",
        playerId,
        playerName,
        isTeamZero: event.is_team_0,
        color: event.is_team_0 ? BLUE_TIMELINE_COLOR : ORANGE_TIMELINE_COLOR,
      };
    });
}

export function buildSpeedFlipTimelineEvents(
  statsTimeline: StatsTimeline,
  replay: ReplayModel,
): ReplayTimelineEvent[] {
  return statsTimeline.events.speed_flip.map((event) => {
    const playerId = event.player ? playerIdToString(event.player) : null;
    const playerName = playerId ? getReplayPlayerName(replay, playerId) : "Unknown";
    const eventTime = getReplayFrameTime(replay, event.frame, event.time);
    const qualityPercent = Math.round(event.confidence * 100);

    return {
      id: `speed-flip:${event.frame}:${playerId}:${Math.round(event.confidence * 1000)}`,
      time: eventTime,
      frame: event.frame,
      kind: "speed-flip",
      label: `${playerName} speed flip ${qualityPercent}%`,
      shortLabel: "SF",
      playerId,
      playerName,
      isTeamZero: event.is_team_0,
      color: event.is_team_0 ? BLUE_TIMELINE_COLOR : ORANGE_TIMELINE_COLOR,
    };
  });
}

export function buildHalfFlipTimelineEvents(
  statsTimeline: StatsTimeline,
  replay: ReplayModel,
): ReplayTimelineEvent[] {
  return statsTimeline.events.half_flip.map((event, index) => {
    const playerId = playerIdToString(event.player);
    const playerName = getReplayPlayerName(replay, playerId);
    const eventTime = getReplayFrameTime(replay, event.frame, event.time);
    const qualityPercent = Math.round(event.confidence * 100);
    const speedGain = Math.round(event.end_speed - event.start_speed);

    return {
      id: `half-flip:${event.frame}:${playerId}:${index}`,
      time: eventTime,
      frame: event.frame,
      kind: "half-flip",
      label: `${playerName} half flip ${qualityPercent}% | +${speedGain}uu/s`,
      shortLabel: "HF",
      playerId,
      playerName,
      isTeamZero: event.is_team_0,
      color: event.is_team_0 ? BLUE_TIMELINE_COLOR : ORANGE_TIMELINE_COLOR,
    };
  });
}

export function buildWavedashTimelineEvents(
  statsTimeline: StatsTimeline,
  replay: ReplayModel,
): ReplayTimelineEvent[] {
  return statsTimeline.events.wavedash.map((event, index) => {
    const playerId = playerIdToString(event.player);
    const playerName = getReplayPlayerName(replay, playerId);
    const eventTime = getReplayFrameTime(replay, event.frame, event.time);
    const qualityPercent = Math.round(event.confidence * 100);
    const speedGain = Math.round(event.horizontal_speed_gain);

    return {
      id: `wavedash:${event.frame}:${playerId}:${index}`,
      time: eventTime,
      frame: event.frame,
      kind: "wavedash",
      label: `${playerName} wavedash ${qualityPercent}% | +${speedGain}uu/s`,
      shortLabel: "WD",
      playerId,
      playerName,
      isTeamZero: event.is_team_0,
      color: event.is_team_0 ? BLUE_TIMELINE_COLOR : ORANGE_TIMELINE_COLOR,
    };
  });
}
