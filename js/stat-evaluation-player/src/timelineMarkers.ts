import type { ReplayModel, ReplayTimelineEvent, ReplayTimelineEventKind } from "@rlrml/player";
import { buildFiftyFiftyMarkers } from "./fiftyFiftyOverlay.ts";
import { buildCeilingShotMarkers } from "./ceilingShotOverlay.ts";
import { buildTouchMarkers, playerIdToString } from "./touchOverlay.ts";
import type { GoalTag } from "./generated/GoalTag.ts";
import type { StatsTimeline } from "./statsTimeline.ts";
import {
  BLUE_TIMELINE_COLOR,
  ORANGE_TIMELINE_COLOR,
  formatMechanicKind,
  isVisibleMechanicKind,
  mechanicShortLabel,
} from "./timelinePresentation.ts";

const NEUTRAL_TIMELINE_COLOR = "#d1d9e0";
const PERFORMER_ATTRIBUTED_GOAL_TAG_KINDS = new Set([
  "flick_goal",
  "double_tap_goal",
  "one_timer_goal",
  "passing_goal",
  "air_dribble_goal",
  "flip_reset_goal",
  "bump_goal",
  "demo_goal",
  "half_volley_goal",
]);

export { formatMechanicKind, isVisibleMechanicKind };

export type GoalTagPerformerRole = "scorer" | "teammate" | "unknown";

export function goalTagPerformerRole(tag: GoalTag): GoalTagPerformerRole {
  if (tag.metadata.performer === "scorer" || tag.metadata.modifiers?.includes("by_scorer")) {
    return "scorer";
  }
  if (tag.metadata.performer === "teammate") {
    return "teammate";
  }
  return PERFORMER_ATTRIBUTED_GOAL_TAG_KINDS.has(tag.kind) ? "unknown" : "scorer";
}

export function isScorerGoalTag(tag: GoalTag): boolean {
  return goalTagPerformerRole(tag) === "scorer";
}

export function isTeammatePerformedGoalTag(tag: GoalTag): boolean {
  return goalTagPerformerRole(tag) === "teammate";
}

export function formatGoalTagPerformer(tag: GoalTag): string | null {
  const role = goalTagPerformerRole(tag);
  if (role === "unknown") return "performer unknown";
  return PERFORMER_ATTRIBUTED_GOAL_TAG_KINDS.has(tag.kind) ? `by ${role}` : null;
}

function getReplayPlayerName(replay: ReplayModel, playerId: string): string {
  return replay.players.find((player) => player.id === playerId)?.name ?? playerId;
}

function getReplayFrameTime(
  replay: ReplayModel,
  frame: number | undefined,
  fallbackTime: number,
): number {
  return replay.frames[frame ?? -1]?.time ?? fallbackTime;
}

function flickMarkerKindLabel(event: { kind?: string; setup_rotation_direction?: string }): string {
  if (event.kind !== "reverse") {
    return "flick";
  }
  if (event.setup_rotation_direction === "left" || event.setup_rotation_direction === "right") {
    return `${event.setup_rotation_direction} reverse flick`;
  }
  return "reverse flick";
}

export function getMechanicKinds(statsTimeline: StatsTimeline | null): string[] {
  return [
    ...new Set(
      (statsTimeline?.events.mechanics ?? [])
        .filter((event) => isVisibleMechanicKind(event.kind))
        .map((event) => event.kind),
    ),
  ].sort((left, right) => formatMechanicKind(left).localeCompare(formatMechanicKind(right)));
}

export function mechanicKindToModuleId(kind: string): string {
  return kind.replaceAll("_", "-");
}

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
    const kindLabel = flickMarkerKindLabel(event);
    return {
      id: `flick:${event.frame}:${playerId}:${index + 1}`,
      time: getReplayFrameTime(replay, event.frame, event.time),
      frame: event.frame,
      kind: "flick",
      label: `${playerName} ${kindLabel}`,
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
  return (statsTimeline.events.goal_context ?? []).flatMap((goal, goalIndex) => {
    return (goal.tags ?? []).map((tag, tagIndex) => {
      const scorerId = goal.scorer ? playerIdToString(goal.scorer) : null;
      const scorerName = scorerId ? getReplayPlayerName(replay, scorerId) : null;
      const eventTime = getReplayFrameTime(replay, goal.frame, goal.time);
      const tagLabel = formatGoalTagKind(tag.kind);
      const performerRole = goalTagPerformerRole(tag);
      const confidencePercent = Math.round(tag.metadata.confidence * 100);
      const goalLabel =
        performerRole === "teammate"
          ? `${tagLabel.toLowerCase()} assist goal`
          : `${tagLabel.toLowerCase()} goal`;

      return {
        id: `goal-tag:${goalIndex}:${tag.kind}:${tagIndex}`,
        time: eventTime,
        frame: goal.frame,
        kind: "goal-tag",
        label: `${scorerName ?? "Goal"} ${goalLabel} ${confidencePercent}%`,
        shortLabel: "GT",
        playerId: scorerId,
        playerName: scorerName,
        isTeamZero: goal.scoring_team_is_team_0,
        color: goal.scoring_team_is_team_0 ? BLUE_TIMELINE_COLOR : ORANGE_TIMELINE_COLOR,
      };
    });
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
    const playerName = playerId
      ? (replay.players.find((player) => player.id === playerId)?.name ?? playerId)
      : "Unknown";
    const eventTime = replay.frames[event.frame]?.time ?? event.time;
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

export function buildFlipImpulseTimelineEvents(
  statsTimeline: StatsTimeline,
  replay: ReplayModel,
): ReplayTimelineEvent[] {
  return (statsTimeline.events.flip_impulse ?? []).map((event, index) => {
    const playerId = playerIdToString(event.player);
    const playerName = replay.players.find((player) => player.id === playerId)?.name ?? playerId;
    const eventTime = getReplayFrameTime(replay, event.frame, event.time);
    const confidencePercent = Math.round(event.confidence * 100);
    const directionLabel = event.direction_label.replaceAll("_", " ");

    return {
      id: `flip-impulse:${event.frame}:${playerId}:${index}`,
      time: eventTime,
      frame: event.frame,
      kind: "flip-impulse",
      label: `${playerName} flip impulse ${directionLabel} ${confidencePercent}%`,
      shortLabel: "FI",
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
    const playerName = replay.players.find((player) => player.id === playerId)?.name ?? playerId;
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
    const playerName = replay.players.find((player) => player.id === playerId)?.name ?? playerId;
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

  if (active.has("flip-impulse")) {
    count += buildFlipImpulseTimelineEvents(statsTimeline, replay).length;
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
