import type { RawPlayerId, ReplayPlayerTrack } from "./types";

export function playerIdToString(playerId: RawPlayerId): string {
  const [kind, value] = Object.entries(playerId)[0] ?? ["Unknown", "unknown"];
  if (typeof value === "string" || typeof value === "number") {
    return `${kind}:${value}`;
  }

  if (value && typeof value === "object") {
    return `${kind}:${JSON.stringify(value)}`;
  }

  return `${kind}:${JSON.stringify(value)}`;
}

export function normalizeReplayTime(rawTime: number, startTime: number): number {
  return Math.max(0, rawTime - startTime);
}

export function buildPlayerLookup(players: ReplayPlayerTrack[]): Map<string, ReplayPlayerTrack> {
  return new Map(players.map((player) => [player.id, player]));
}
