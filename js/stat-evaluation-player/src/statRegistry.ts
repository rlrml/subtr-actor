import type { PlayerStatsSnapshot, StatsFrame, TeamStatsSnapshot } from "./statsTimeline.ts";
import { createPlayerStatsSnapshot, createTeamStatsSnapshot } from "./statsSnapshotFactories.ts";

export type StatScopeKind = "player" | "team";

export interface StatDefinition {
  readonly id: string;
  readonly label: string;
  readonly category: string;
  readonly scope: StatScopeKind;
  readonly path: readonly string[];
  read(target: PlayerStatsSnapshot | TeamStatsSnapshot): unknown;
  format(value: unknown): string;
}

const PLAYER_METADATA_KEYS = new Set(["player_id", "name", "is_team_0"]);
const LIVE_PLAYBACK_PREFIXES = ["is_last_", "time_since_last_", "frames_since_last_"] as const;

function isLeafStatValue(value: unknown): boolean {
  return (
    value === null ||
    typeof value === "number" ||
    typeof value === "string" ||
    typeof value === "boolean" ||
    Array.isArray(value)
  );
}

function getPathValue(value: unknown, path: readonly string[]): unknown {
  let current = value;
  for (const segment of path) {
    if (!current || typeof current !== "object" || Array.isArray(current)) {
      return undefined;
    }
    current = (current as Record<string, unknown>)[segment];
  }
  return current;
}

function formatStatValue(value: unknown): string {
  if (value === undefined || value === null) {
    return "--";
  }
  if (typeof value === "number") {
    if (!Number.isFinite(value)) {
      return "--";
    }
    if (Number.isInteger(value)) {
      return `${value}`;
    }
    return `${Number(value.toFixed(3))}`;
  }
  if (typeof value === "boolean") {
    return value ? "true" : "false";
  }
  if (Array.isArray(value)) {
    return value.length === 0 ? "[]" : JSON.stringify(value);
  }
  return `${value}`;
}

function isLivePlaybackStatKey(key: string, source: Record<string, unknown>): boolean {
  if (LIVE_PLAYBACK_PREFIXES.some((prefix) => key.startsWith(prefix))) {
    return true;
  }

  const lastTimeMatch = key.match(/^last_(.+)_time$/);
  const lastFrameMatch = key.match(/^last_(.+)_frame$/);
  const statName = lastTimeMatch?.[1] ?? lastFrameMatch?.[1];
  if (!statName) {
    return false;
  }

  return (
    `is_last_${statName}` in source ||
    `time_since_last_${statName}` in source ||
    `frames_since_last_${statName}` in source
  );
}

function collectStatDefinitions(
  target: unknown,
  scope: StatScopeKind,
  path: string[],
  out: StatDefinition[],
): void {
  if (!target || typeof target !== "object" || Array.isArray(target)) {
    return;
  }

  const targetRecord = target as Record<string, unknown>;
  for (const [key, value] of Object.entries(targetRecord)) {
    if (scope === "player" && path.length === 0 && PLAYER_METADATA_KEYS.has(key)) {
      continue;
    }
    if (isLivePlaybackStatKey(key, targetRecord)) {
      continue;
    }

    const nextPath = [...path, key];
    if (isLeafStatValue(value)) {
      const id = `${scope}:${nextPath.join(".")}`;
      out.push({
        id,
        label: nextPath.join("."),
        category: nextPath[0] ?? scope,
        scope,
        path: nextPath,
        read(source) {
          return getPathValue(source, nextPath);
        },
        format: formatStatValue,
      });
      continue;
    }

    collectStatDefinitions(value, scope, nextPath, out);
  }
}

function uniqueDefinitions(definitions: StatDefinition[]): StatDefinition[] {
  const seen = new Set<string>();
  return definitions.filter((definition) => {
    if (seen.has(definition.id)) {
      return false;
    }
    seen.add(definition.id);
    return true;
  });
}

function createStatRegistryForTargets(
  player: PlayerStatsSnapshot | null,
  team: TeamStatsSnapshot | null,
): StatDefinition[] {
  const definitions: StatDefinition[] = [];
  if (player) {
    collectStatDefinitions(player, "player", [], definitions);
  }
  if (team) {
    collectStatDefinitions(team, "team", [], definitions);
  }

  return uniqueDefinitions(definitions).sort((left, right) =>
    left.label.localeCompare(right.label),
  );
}

export function createDefaultStatRegistry(): StatDefinition[] {
  return createStatRegistryForTargets(createPlayerStatsSnapshot(), createTeamStatsSnapshot());
}

export function createStatRegistry(frame: StatsFrame | null): StatDefinition[] {
  if (!frame) {
    return createDefaultStatRegistry();
  }

  return createStatRegistryForTargets(
    frame.players[0] ?? createPlayerStatsSnapshot(),
    frame.team_zero ?? frame.team_one ?? createTeamStatsSnapshot(),
  );
}
