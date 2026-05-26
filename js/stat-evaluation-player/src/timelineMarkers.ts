import type { ReplayModel, ReplayTimelineEvent, ReplayTimelineEventKind } from "subtr-actor-player";
import { buildFiftyFiftyMarkers } from "./fiftyFiftyOverlay.ts";
import { buildCeilingShotMarkers } from "./ceilingShotOverlay.ts";
import { buildTouchMarkers, playerIdToString } from "./touchOverlay.ts";
import type { StatsTimeline } from "./statsTimeline.ts";

const BLUE_TIMELINE_COLOR = "#3b82f6";
const ORANGE_TIMELINE_COLOR = "#f59e0b";
const NEUTRAL_TIMELINE_COLOR = "#d1d9e0";
const MECHANIC_SHORT_LABELS: Record<string, string> = {
  air_dribble: "AD",
  ball_carry: "BC",
  ceiling_shot: "CS",
  double_tap: "DT",
  flick: "F",
  flip_reset: "FR",
  half_flip: "HF",
  half_volley: "HV",
  musty_flick: "M",
  one_timer: "OT",
  pass: "P",
  speed_flip: "SF",
  wall_aerial: "WA",
  wall_aerial_shot: "WS",
  wavedash: "WD",
};
const HIDDEN_MECHANIC_KINDS = new Set(["wavedash"]);
const RANGE_ONLY_MECHANIC_KINDS = new Set(["air_dribble", "ball_carry"]);
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

export function formatMechanicKind(kind: string): string {
  return kind
    .split(/[_-]+/)
    .filter((part) => part.length > 0)
    .map((part) => `${part.slice(0, 1).toUpperCase()}${part.slice(1)}`)
    .join(" ");
}

function mechanicShortLabel(kind: string): string {
  return (
    MECHANIC_SHORT_LABELS[kind] ??
    (kind
      .split(/[_-]+/)
      .filter((part) => part.length > 0)
      .map((part) => part.slice(0, 1).toUpperCase())
      .join("")
      .slice(0, 3) ||
      "M")
  );
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

export function isVisibleMechanicKind(kind: string): boolean {
  return !HIDDEN_MECHANIC_KINDS.has(kind);
}

export function isRangeOnlyMechanicKind(kind: string): boolean {
  return RANGE_ONLY_MECHANIC_KINDS.has(kind);
}

export function getMechanicTimelineEventKinds(statsTimeline: StatsTimeline | null): string[] {
  return getMechanicKinds(statsTimeline).filter((kind) => !isRangeOnlyMechanicKind(kind));
}

export function mechanicKindToModuleId(kind: string): string {
  return kind.replaceAll("_", "-");
}

export function getMechanicTimelineModuleIds(statsTimeline: StatsTimeline | null): Set<string> {
  return new Set(getMechanicKinds(statsTimeline).map(mechanicKindToModuleId));
}

export function getMechanicTimelineEventModuleIds(
  statsTimeline: StatsTimeline | null,
): Set<string> {
  return new Set(getMechanicTimelineEventKinds(statsTimeline).map(mechanicKindToModuleId));
}

export function getNonMechanicTimelineEventModuleIds(
  moduleIds: Iterable<string>,
  statsTimeline: StatsTimeline | null,
): Set<string> {
  const mechanicModuleIds = getMechanicTimelineModuleIds(statsTimeline);
  return new Set([...moduleIds].filter((moduleId) => !mechanicModuleIds.has(moduleId)));
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
        !isRangeOnlyMechanicKind(event.kind) &&
        (!enabled || enabled.has(event.kind)),
    )
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
        id: event.id,
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

function buildPlayerCountTimelineEvents(
  statsTimeline: StatsTimeline,
  replay: ReplayModel,
  options: {
    kind: ReplayTimelineEventKind;
    idPrefix: string;
    shortLabel: string;
    getCount: (player: StatsTimeline["frames"][number]["players"][number]) => number;
    buildLabel: (player: StatsTimeline["frames"][number]["players"][number]) => string;
  },
): ReplayTimelineEvent[] {
  const events: ReplayTimelineEvent[] = [];
  const previousCounts = new Map<string, number>();

  for (const frame of statsTimeline.frames) {
    for (const player of frame.players) {
      const playerId = playerIdToString(player.player_id);
      const currentCount = options.getCount(player);
      const previousCount = previousCounts.get(playerId) ?? 0;
      previousCounts.set(playerId, currentCount);

      const delta = Math.max(0, currentCount - previousCount);
      if (delta === 0) {
        continue;
      }

      const eventTime = getReplayFrameTime(replay, frame.frame_number, frame.time);
      for (let index = 0; index < delta; index += 1) {
        const sequence = currentCount - delta + index + 1;
        events.push({
          id: `${options.idPrefix}:${frame.frame_number}:${playerId}:${sequence}`,
          time: eventTime,
          frame: frame.frame_number,
          kind: options.kind,
          label: options.buildLabel(player),
          shortLabel: options.shortLabel,
          playerId,
          playerName: player.name,
          isTeamZero: player.is_team_0,
          color: player.is_team_0 ? BLUE_TIMELINE_COLOR : ORANGE_TIMELINE_COLOR,
        });
      }
    }
  }

  return events;
}

export function getReplayTimelineEventKinds(
  activeModuleIds: Iterable<string>,
): ReplayTimelineEventKind[] {
  const active = new Set(activeModuleIds);
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
  activeModuleIds: Iterable<string>,
): ReplayTimelineEvent[] {
  const allowedKinds = new Set(getReplayTimelineEventKinds(activeModuleIds));
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
  const events: ReplayTimelineEvent[] = [];
  const previousCounts = new Map<string, number>();

  for (const frame of statsTimeline.frames) {
    for (const player of frame.players) {
      const playerId = playerIdToString(player.player_id);
      const currentCount = player.musty_flick?.count ?? 0;
      const previousCount = previousCounts.get(playerId) ?? 0;
      previousCounts.set(playerId, currentCount);

      const delta = Math.max(0, currentCount - previousCount);
      if (delta === 0) {
        continue;
      }

      const eventFrame = player.musty_flick?.last_musty_frame ?? frame.frame_number;
      const eventTime =
        replay.frames[eventFrame]?.time ?? player.musty_flick?.last_musty_time ?? frame.time;

      for (let index = 0; index < delta; index += 1) {
        events.push({
          id: `musty-flick:${eventFrame}:${playerId}:${currentCount - delta + index + 1}`,
          time: eventTime,
          frame: eventFrame,
          kind: "musty-flick",
          label: `${player.name} musty flick`,
          shortLabel: "M",
          playerId,
          playerName: player.name,
          isTeamZero: player.is_team_0,
          color: player.is_team_0 ? BLUE_TIMELINE_COLOR : ORANGE_TIMELINE_COLOR,
        });
      }
    }
  }

  return events;
}

export function buildFlickTimelineEvents(
  statsTimeline: StatsTimeline,
  replay: ReplayModel,
): ReplayTimelineEvent[] {
  const events: ReplayTimelineEvent[] = [];
  const previousCounts = new Map<string, number>();

  for (const frame of statsTimeline.frames) {
    for (const player of frame.players) {
      const playerId = playerIdToString(player.player_id);
      const currentCount = player.flick?.count ?? 0;
      const previousCount = previousCounts.get(playerId) ?? 0;
      previousCounts.set(playerId, currentCount);

      const delta = Math.max(0, currentCount - previousCount);
      if (delta === 0) {
        continue;
      }

      const eventFrame = player.flick?.last_flick_frame ?? frame.frame_number;
      const eventTime =
        replay.frames[eventFrame]?.time ?? player.flick?.last_flick_time ?? frame.time;

      for (let index = 0; index < delta; index += 1) {
        events.push({
          id: `flick:${eventFrame}:${playerId}:${currentCount - delta + index + 1}`,
          time: eventTime,
          frame: eventFrame,
          kind: "flick",
          label: `${player.name} flick`,
          shortLabel: "F",
          playerId,
          playerName: player.name,
          isTeamZero: player.is_team_0,
          color: player.is_team_0 ? BLUE_TIMELINE_COLOR : ORANGE_TIMELINE_COLOR,
        });
      }
    }
  }

  return events;
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

export function buildDodgeResetTimelineEvents(
  statsTimeline: StatsTimeline,
  replay: ReplayModel,
): ReplayTimelineEvent[] {
  const events: ReplayTimelineEvent[] = [];
  const previousCounts = new Map<string, number>();
  const previousOnBallCounts = new Map<string, number>();

  for (const frame of statsTimeline.frames) {
    const eventTime = getReplayFrameTime(replay, frame.frame_number, frame.time);

    for (const player of frame.players) {
      const playerId = playerIdToString(player.player_id);
      const currentCount = player.dodge_reset?.count ?? 0;
      const previousCount = previousCounts.get(playerId) ?? 0;
      previousCounts.set(playerId, currentCount);

      const currentOnBallCount = player.dodge_reset?.on_ball_count ?? 0;
      const previousOnBallCount = previousOnBallCounts.get(playerId) ?? 0;
      previousOnBallCounts.set(playerId, currentOnBallCount);

      const delta = Math.max(0, currentCount - previousCount);
      const onBallDelta = Math.min(delta, Math.max(0, currentOnBallCount - previousOnBallCount));

      for (let index = 0; index < delta; index += 1) {
        const sequence = currentCount - delta + index + 1;
        const onBall = index < onBallDelta;
        if (onBall) {
          continue;
        }
        events.push({
          id: `dodge-reset:${frame.frame_number}:${playerId}:${sequence}:air`,
          time: eventTime,
          frame: frame.frame_number,
          kind: "dodge-reset",
          label: `${player.name} dodge refresh`,
          shortLabel: "DR",
          playerId,
          playerName: player.name,
          isTeamZero: player.is_team_0,
          color: player.is_team_0 ? BLUE_TIMELINE_COLOR : ORANGE_TIMELINE_COLOR,
        });
      }
    }
  }

  return events;
}

export function buildBallCarryTimelineEvents(
  statsTimeline: StatsTimeline,
  replay: ReplayModel,
): ReplayTimelineEvent[] {
  return buildPlayerCountTimelineEvents(statsTimeline, replay, {
    kind: "ball-carry",
    idPrefix: "ball-carry",
    shortLabel: "BC",
    getCount: (player) => player.ball_carry?.carry_count ?? 0,
    buildLabel: (player) => `${player.name} ball carry`,
  });
}

export function buildPowerslideTimelineEvents(
  statsTimeline: StatsTimeline,
  replay: ReplayModel,
): ReplayTimelineEvent[] {
  return buildPlayerCountTimelineEvents(statsTimeline, replay, {
    kind: "powerslide",
    idPrefix: "powerslide",
    shortLabel: "PS",
    getCount: (player) => player.powerslide?.press_count ?? 0,
    buildLabel: (player) => `${player.name} powerslide`,
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
  activeModuleIds: Iterable<string>,
  replay: ReplayModel,
  statsTimeline: StatsTimeline,
): number {
  const active = getNonMechanicTimelineEventModuleIds(activeModuleIds, statsTimeline);
  let count = filterReplayTimelineEvents(replay, active).length;

  if (active.has("fifty-fifty")) {
    count += buildFiftyFiftyTimelineEvents(statsTimeline, replay).length;
  }

  if (active.has("goal-tags")) {
    count += buildGoalTagTimelineEvents(statsTimeline, replay).length;
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
