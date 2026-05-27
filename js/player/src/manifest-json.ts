export type JsonObject = Record<string, unknown>;

export function isObject(value: unknown): value is JsonObject {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

export function isRecordOfUnknown(value: unknown): value is Record<string, unknown> {
  return isObject(value);
}
