import type { PlaylistManifestPage } from "./types";
import { isObject } from "./manifest-json";

export function parseManifestPage(value: unknown): PlaylistManifestPage {
  if (!isObject(value)) {
    throw new Error("manifest.page must be an object when provided");
  }

  return {
    next: parseOptionalString(value.next, "manifest.page.next"),
    previous: parseOptionalString(value.previous, "manifest.page.previous"),
    total: parseOptionalFiniteNonnegativeInteger(value.total, "manifest.page.total"),
    count: parseOptionalFiniteNonnegativeInteger(value.count, "manifest.page.count"),
    limit: parseOptionalFiniteNonnegativeInteger(value.limit, "manifest.page.limit"),
    offset: parseOptionalFiniteNonnegativeInteger(value.offset, "manifest.page.offset"),
  };
}

function parseOptionalFiniteNonnegativeInteger(value: unknown, path: string): number | undefined {
  if (value === undefined || value === null) {
    return undefined;
  }
  if (
    typeof value !== "number" ||
    !Number.isInteger(value) ||
    !Number.isFinite(value) ||
    value < 0
  ) {
    throw new Error(`${path} must be a non-negative integer when provided`);
  }
  return value;
}

function parseOptionalString(value: unknown, path: string): string | undefined {
  if (value === undefined || value === null) {
    return undefined;
  }
  if (typeof value !== "string") {
    throw new Error(`${path} must be a string when provided`);
  }
  return value;
}
