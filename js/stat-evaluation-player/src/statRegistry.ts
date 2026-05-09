import type {
  PlayerStatsSnapshot,
  StatsFrame,
  TeamStatsSnapshot,
} from "./statsTimeline.ts";

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

const PLAYER_METADATA_KEYS = new Set([
  "player_id",
  "name",
  "is_team_0",
]);

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

function collectStatDefinitions(
  target: unknown,
  scope: StatScopeKind,
  path: string[],
  out: StatDefinition[],
): void {
  if (!target || typeof target !== "object" || Array.isArray(target)) {
    return;
  }

  for (const [key, value] of Object.entries(target)) {
    if (scope === "player" && path.length === 0 && PLAYER_METADATA_KEYS.has(key)) {
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

export function createStatRegistry(frame: StatsFrame | null): StatDefinition[] {
  if (!frame) {
    return [];
  }

  const definitions: StatDefinition[] = [];
  const player = frame.players[0];
  if (player) {
    collectStatDefinitions(player, "player", [], definitions);
  }
  if (frame.team_zero) {
    collectStatDefinitions(frame.team_zero, "team", [], definitions);
  } else if (frame.team_one) {
    collectStatDefinitions(frame.team_one, "team", [], definitions);
  }

  return uniqueDefinitions(definitions).sort((left, right) =>
    left.label.localeCompare(right.label)
  );
}
