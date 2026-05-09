export interface StatsPlayerConfigAdapter {
  id: string;
  aliases?: readonly string[];
  getConfig?(): unknown;
  applyConfig?(config: unknown): void;
}

export function getConfigAdapterSnapshot(
  adapters: readonly StatsPlayerConfigAdapter[],
): Record<string, unknown> {
  const snapshot: Record<string, unknown> = {};
  for (const adapter of adapters) {
    if (!adapter.getConfig) {
      continue;
    }
    if (Object.hasOwn(snapshot, adapter.id)) {
      throw new Error(`Duplicate stats player config adapter id: ${adapter.id}`);
    }
    snapshot[adapter.id] = adapter.getConfig();
  }
  return snapshot;
}

export function applyConfigAdapterSnapshot(
  adapters: readonly StatsPlayerConfigAdapter[],
  snapshot: Record<string, unknown>,
): void {
  for (const adapter of adapters) {
    if (!adapter.applyConfig) {
      continue;
    }

    if (Object.hasOwn(snapshot, adapter.id)) {
      adapter.applyConfig(snapshot[adapter.id]);
      continue;
    }

    for (const alias of adapter.aliases ?? []) {
      if (Object.hasOwn(snapshot, alias)) {
        adapter.applyConfig(snapshot[alias]);
        break;
      }
    }
  }
}
