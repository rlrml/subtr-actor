import type { ReplayModel, ReplayTimelineRange } from "@rlrml/player";
import type { StatsTimeline } from "./statsTimeline.ts";
import { formatMechanicKind, isVisibleMechanicKind } from "./timelineMechanics.ts";

const BLUE_TIMELINE_COLOR = "#3b82f6";
const ORANGE_TIMELINE_COLOR = "#f59e0b";
const MECHANIC_SHORT_LABELS: Record<string, string> = {
  air_dribble: "AD",
  ball_carry: "BC",
  ceiling_shot: "CS",
  double_tap: "DT",
  flick: "F",
  half_flip: "HF",
  half_volley: "HV",
  musty_flick: "M",
  one_timer: "OT",
  pass: "P",
  wavedash: "WD",
};

function getReplayFrameTime(
  replay: ReplayModel | undefined,
  frame: number | undefined,
  fallbackTime: number,
): number {
  return replay?.frames?.[frame ?? -1]?.time ?? fallbackTime;
}

function teamTimelineColor(isTeamZero: boolean | null | undefined): string | null {
  if (isTeamZero === true) {
    return BLUE_TIMELINE_COLOR;
  }
  if (isTeamZero === false) {
    return ORANGE_TIMELINE_COLOR;
  }

  return null;
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

function remoteIdToString(playerId: Record<string, unknown>): string {
  const [kind, value] = Object.entries(playerId)[0] ?? ["Unknown", "unknown"];
  const normalizedValue = typeof value === "string" ? value : JSON.stringify(value);
  return `${kind}:${normalizedValue}`;
}

export function buildMechanicTimelineRanges(
  statsTimeline: StatsTimeline,
  replay: ReplayModel,
  enabledKinds?: Iterable<string>,
): ReplayTimelineRange[] {
  const enabled = enabledKinds ? new Set(enabledKinds) : null;
  const playerNames = new Map(replay.players.map((player) => [player.id, player.name]));

  return (statsTimeline.events.mechanics ?? [])
    .filter(
      (event) =>
        isVisibleMechanicKind(event.kind) &&
        event.timing.type === "span" &&
        (!enabled || enabled.has(event.kind)),
    )
    .map((event): ReplayTimelineRange => {
      if (event.timing.type !== "span") {
        throw new Error("unreachable non-span mechanic event");
      }

      const playerId = remoteIdToString(event.player_id as Record<string, unknown>);
      const playerName = playerNames.get(playerId) ?? playerId;
      const mechanicLabel = formatMechanicKind(event.kind);
      const startTime = getReplayFrameTime(
        replay,
        event.timing.start_frame,
        event.timing.start_time,
      );
      const endTime = Math.max(
        startTime,
        getReplayFrameTime(replay, event.timing.end_frame, event.timing.end_time),
      );

      return {
        id: event.id,
        startTime,
        endTime,
        lane: `mechanic:${event.kind}`,
        laneLabel: mechanicLabel,
        label: `${playerName} ${mechanicLabel.toLowerCase()}`,
        shortLabel: mechanicShortLabel(event.kind),
        isTeamZero: event.is_team_0,
        color: teamTimelineColor(event.is_team_0) ?? undefined,
      };
    })
    .sort((left, right) => {
      if (left.startTime !== right.startTime) {
        return left.startTime - right.startTime;
      }
      return (left.id ?? "").localeCompare(right.id ?? "");
    });
}
