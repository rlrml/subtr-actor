import type { ReplayModel, ReplayTimelineEvent } from "@rlrml/player";
import { playerIdToString } from "./touchOverlay.ts";
import type { StatsTimeline } from "./statsTimeline.ts";
import {
  BLUE_TIMELINE_COLOR,
  ORANGE_TIMELINE_COLOR,
  getReplayFrameTime,
  getReplayPlayerName,
} from "./timelineMarkerHelpers.ts";
import { formatMechanicKind } from "./timelineMechanics.ts";

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
