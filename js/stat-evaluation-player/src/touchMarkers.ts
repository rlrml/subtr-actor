import type { ReplayModel } from "@rlrml/player";
import type { PlayerStatsSnapshot, StatsFrame, StatsTimeline } from "./statsTimeline.ts";

export type TouchOverlayMode = "markers" | "advancement";

export interface TouchMarker {
  id: string;
  time: number;
  frame: number;
  isTeamZero: boolean;
  playerId: string | null;
  playerName: string;
  position: {
    x: number;
    y: number;
    z: number;
  };
  endPosition: {
    x: number;
    y: number;
    z: number;
  };
  totalBallTravelDistance: number;
  totalBallAdvanceDistance: number;
  totalBallRetreatDistance: number;
}

export function getLastTouchPlayer(statsFrame: StatsFrame): PlayerStatsSnapshot | null {
  return statsFrame.players.find((player) => player.touch?.is_last_touch) ?? null;
}

export function playerIdToString(playerId: Record<string, unknown>): string {
  const [kind, value] = Object.entries(playerId)[0] ?? ["Unknown", "unknown"];
  const normalizedValue = typeof value === "string" ? value : JSON.stringify(value);
  return `${kind}:${normalizedValue}`;
}

function positiveDelta(current: number, previous: number): number {
  return Math.max(0, current - previous);
}

export function buildTouchMarkers(
  statsTimeline: StatsTimeline,
  replay: ReplayModel,
): TouchMarker[] {
  const activeMarkerByPlayer = new Map<string, number>();
  const markers: TouchMarker[] = [];
  const events = [
    ...(statsTimeline.events?.touch ?? []).map((event, index) => ({
      kind: "touch" as const,
      frame: event.frame,
      time: event.time,
      index,
      event,
    })),
    ...(statsTimeline.events?.touch_ball_movement ?? []).map((event, index) => ({
      kind: "movement" as const,
      frame: event.frame,
      time: event.time,
      index,
      event,
    })),
  ].sort((left, right) => {
    if (left.frame !== right.frame) {
      return left.frame - right.frame;
    }
    if (left.time !== right.time) {
      return left.time - right.time;
    }
    if (left.kind !== right.kind) {
      return left.kind === "touch" ? -1 : 1;
    }
    return left.index - right.index;
  });

  for (const item of events) {
    if (item.kind === "touch") {
      const event = item.event;
      const playerId = playerIdToString(event.player);
      const ballPosition = replay.ballFrames[event.frame]?.position;
      if (!ballPosition) {
        continue;
      }
      const markerIndex = markers.length;
      markers.push({
        id: `touch-stat:${event.frame}:${playerId}:${markerIndex + 1}`,
        time: replay.frames[event.frame]?.time ?? event.time,
        frame: event.frame,
        isTeamZero: event.is_team_0,
        playerId,
        playerName: replay.players.find((player) => player.id === playerId)?.name ?? playerId,
        position: {
          x: ballPosition.x,
          y: ballPosition.y,
          z: ballPosition.z,
        },
        endPosition: {
          x: ballPosition.x,
          y: ballPosition.y,
          z: ballPosition.z,
        },
        totalBallTravelDistance: 0,
        totalBallAdvanceDistance: 0,
        totalBallRetreatDistance: 0,
      });
      activeMarkerByPlayer.set(playerId, markerIndex);
      continue;
    }

    const event = item.event;
    const playerId = playerIdToString(event.player);
    const activeMarkerIndex = activeMarkerByPlayer.get(playerId);
    const frameBallPosition = replay.ballFrames[event.frame]?.position;
    if (activeMarkerIndex === undefined || !frameBallPosition) {
      continue;
    }
    const activeMarker = markers[activeMarkerIndex];
    if (!activeMarker) {
      continue;
    }
    activeMarker.totalBallTravelDistance += positiveDelta(event.travel_distance, 0);
    activeMarker.totalBallAdvanceDistance += positiveDelta(event.advance_distance, 0);
    activeMarker.totalBallRetreatDistance += positiveDelta(event.retreat_distance, 0);
    activeMarker.endPosition = {
      x: frameBallPosition.x,
      y: frameBallPosition.y,
      z: frameBallPosition.z,
    };
  }

  return markers;
}

export function getVisibleTouchMarkers(
  markers: TouchMarker[],
  currentTime: number,
  decaySeconds: number,
): TouchMarker[] {
  const effectiveDecay = Math.max(0.1, decaySeconds);
  return markers.filter((marker) => {
    const age = currentTime - marker.time;
    return age >= 0 && age <= effectiveDecay;
  });
}
