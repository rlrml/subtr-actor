export type DeepPartial<T> = {
  [K in keyof T]?: T[K] extends Array<infer U>
    ? Array<DeepPartial<U>>
    : T[K] extends object
      ? DeepPartial<T[K]>
      : T[K];
};

export function merge<T>(base: T, overrides: DeepPartial<T> | undefined): T {
  if (!overrides) {
    return base;
  }

  const result: Record<string, unknown> = { ...(base as Record<string, unknown>) };
  for (const [key, value] of Object.entries(overrides)) {
    if (key === "player_id") {
      result[key] = value;
      continue;
    }

    if (Array.isArray(value)) {
      result[key] = value;
      continue;
    }

    const baseValue = result[key];
    if (
      value &&
      typeof value === "object" &&
      baseValue &&
      typeof baseValue === "object" &&
      !Array.isArray(baseValue)
    ) {
      result[key] = merge(baseValue as Record<string, unknown>, value as Record<string, unknown>);
      continue;
    }

    result[key] = value;
  }

  return result as T;
}
