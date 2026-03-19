import type { ExportedStat, StatLabel } from "./statsTimeline.ts";

type StatLike = ExportedStat | Map<unknown, unknown> | Record<string, unknown>;

function getField(
  value: unknown,
  key: string,
): unknown {
  if (value instanceof Map) {
    return value.get(key);
  }

  if (value && typeof value === "object") {
    return (value as Record<string, unknown>)[key];
  }

  return undefined;
}

export function getExportedStatDomain(stat: unknown): string | undefined {
  const domain = getField(stat, "domain");
  return typeof domain === "string" ? domain : undefined;
}

export function getExportedStatName(stat: unknown): string | undefined {
  const name = getField(stat, "name");
  return typeof name === "string" ? name : undefined;
}

export function getExportedStatVariant(stat: unknown): string | undefined {
  const variant = getField(stat, "variant");
  return typeof variant === "string" ? variant : undefined;
}

export function getExportedStatValueType(stat: unknown): string | undefined {
  const directValueType = getField(stat, "value_type");
  if (typeof directValueType === "string") {
    return directValueType;
  }

  const nestedValueType = getField(getField(stat, "value"), "value_type");
  return typeof nestedValueType === "string" ? nestedValueType : undefined;
}

export function getExportedStatValue(stat: unknown): number | undefined {
  const directValue = getField(stat, "value");
  if (typeof directValue === "number" && Number.isFinite(directValue)) {
    return directValue;
  }

  const nestedValue = getField(directValue, "value");
  return typeof nestedValue === "number" && Number.isFinite(nestedValue)
    ? nestedValue
    : undefined;
}

export function getExportedStatLabels(stat: unknown): StatLabel[] {
  const labels = getField(stat, "labels");
  if (!Array.isArray(labels)) {
    return [];
  }

  return labels.flatMap((label) => {
    const key = getField(label, "key");
    const value = getField(label, "value");
    if (typeof key !== "string" || typeof value !== "string") {
      return [];
    }

    return [{ key, value }];
  });
}

export function isExportedStat(
  stat: unknown,
): stat is StatLike {
  return stat instanceof Map || !!stat;
}
