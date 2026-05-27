export function toPlainData<T>(value: T): T {
  if (value instanceof Map) {
    return Object.fromEntries(
      Array.from(value.entries()).map(([key, entry]) => [key, toPlainData(entry)]),
    ) as T;
  }

  if (Array.isArray(value)) {
    return value.map((entry) => toPlainData(entry)) as T;
  }

  if (value && typeof value === "object") {
    const result: Record<string, unknown> = {};
    for (const [key, entry] of Object.entries(value as Record<string, unknown>)) {
      result[key] = toPlainData(entry);
    }
    return result as T;
  }

  return value;
}
