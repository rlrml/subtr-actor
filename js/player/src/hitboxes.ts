import type { RawPlayerInfo } from "./raw-types";

export type ReplayHitboxKind = "breakout" | "dominus" | "hybrid" | "merc" | "octane" | "plank";

export interface ReplayHitboxSpec {
  kind: ReplayHitboxKind;
  label: string;
  length: number;
  width: number;
  height: number;
  slopeDegrees: number;
  groundHeightFront: number;
  groundHeightBack: number;
}

export const DEFAULT_REPLAY_HITBOX_KIND: ReplayHitboxKind = "octane";

export const REPLAY_HITBOX_SPECS: Readonly<Record<ReplayHitboxKind, ReplayHitboxSpec>> = {
  breakout: {
    kind: "breakout",
    label: "Breakout",
    length: 131.4924,
    width: 80.521,
    height: 30.3,
    slopeDegrees: -0.9795,
    groundHeightFront: 43.8976,
    groundHeightBack: 46.1454,
  },
  dominus: {
    kind: "dominus",
    label: "Dominus",
    length: 127.9268,
    width: 83.27995,
    height: 31.3,
    slopeDegrees: -0.9635,
    groundHeightFront: 47.2238,
    groundHeightBack: 49.3749,
  },
  hybrid: {
    kind: "hybrid",
    label: "Hybrid",
    length: 127.0192,
    width: 82.18787,
    height: 34.15907,
    slopeDegrees: -0.5499,
    groundHeightFront: 54.0982,
    groundHeightBack: 55.3173,
  },
  merc: {
    kind: "merc",
    label: "Merc",
    length: 120.72,
    width: 76.71,
    height: 41.66,
    slopeDegrees: 0.28,
    groundHeightFront: 60.76,
    groundHeightBack: 61.35,
  },
  octane: {
    kind: "octane",
    label: "Octane",
    length: 118.0074,
    width: 84.19941,
    height: 36.15907,
    slopeDegrees: -0.5518,
    groundHeightFront: 55.1449,
    groundHeightBack: 56.2814,
  },
  plank: {
    kind: "plank",
    label: "Plank",
    length: 128.8198,
    width: 84.67036,
    height: 29.3944,
    slopeDegrees: -0.3447,
    groundHeightFront: 44.998,
    groundHeightBack: 45.773,
  },
};

const BODY_HITBOX_BY_NORMALIZED_NAME: Readonly<Record<string, ReplayHitboxKind>> = {
  "16batmobile": "plank",
  "70dodgechargerrt": "dominus",
  "89batmobile": "dominus",
  "99nissanskylinegtrr34": "hybrid",
  aftershock: "dominus",
  animusgp: "breakout",
  artemis: "plank",
  artemisg1: "plank",
  artemisgxt: "plank",
  backfire: "octane",
  battlebus: "merc",
  breakout: "breakout",
  breakouttypes: "breakout",
  centio: "plank",
  centiov17: "plank",
  cyclone: "breakout",
  deloreantimemachine: "octane",
  diestro: "dominus",
  dominus: "dominus",
  dominusgt: "dominus",
  endo: "hybrid",
  esper: "hybrid",
  fast4wd: "octane",
  fennec: "octane",
  gazellagt: "dominus",
  gizmo: "octane",
  grog: "octane",
  guardian: "dominus",
  guardiang1: "dominus",
  guardiangxt: "dominus",
  hotshot: "dominus",
  icecharger: "dominus",
  imperatordt5: "dominus",
  jager619rs: "hybrid",
  jurassicjeepwrangler: "octane",
  mantis: "plank",
  marauder: "octane",
  masamune: "dominus",
  maverick: "dominus",
  maverickg1: "dominus",
  maverickgxt: "dominus",
  mclaren570s: "dominus",
  merc: "merc",
  mr11: "dominus",
  nimbus: "hybrid",
  octane: "octane",
  octanezsr: "octane",
  paladin: "plank",
  proteus: "octane",
  ripper: "dominus",
  roadhog: "octane",
  roadhogxl: "octane",
  samurai: "breakout",
  scarab: "octane",
  takumi: "octane",
  takumirxt: "octane",
  thedarkknightrisestumbler: "octane",
  triton: "octane",
  twinmilliii: "plank",
  twinzer: "octane",
  venom: "hybrid",
  vulcan: "octane",
  werewolf: "dominus",
  xdevil: "hybrid",
  xdevilmk2: "hybrid",
  zippy: "octane",
};

function normalizedText(value: string): string {
  return value.toLowerCase().replace(/[^a-z0-9]+/g, "");
}

export function normalizeReplayHitboxKind(
  value: string | null | undefined,
): ReplayHitboxKind | null {
  if (!value) {
    return null;
  }
  const normalized = normalizedText(value);
  switch (normalized) {
    case "breakout":
      return "breakout";
    case "dominus":
      return "dominus";
    case "hybrid":
      return "hybrid";
    case "merc":
      return "merc";
    case "octane":
      return "octane";
    case "batmobile":
    case "plank":
      return "plank";
    default:
      return BODY_HITBOX_BY_NORMALIZED_NAME[normalized] ?? null;
  }
}

export function getReplayHitboxSpec(kind: ReplayHitboxKind): ReplayHitboxSpec {
  return REPLAY_HITBOX_SPECS[kind];
}

function collectHeaderPropText(value: unknown, out: string[]): void {
  if (!value || typeof value !== "object") {
    return;
  }

  if ("Str" in value && typeof value.Str === "string") {
    out.push(value.Str);
    return;
  }
  if ("Name" in value && typeof value.Name === "string") {
    out.push(value.Name);
    return;
  }
  if ("Byte" in value && value.Byte && typeof value.Byte === "object") {
    const byte = value.Byte as { kind?: unknown; value?: unknown };
    if (typeof byte.kind === "string") out.push(byte.kind);
    if (typeof byte.value === "string") out.push(byte.value);
    return;
  }
  if ("Struct" in value && value.Struct && typeof value.Struct === "object") {
    const struct = value.Struct as { name?: unknown; fields?: unknown };
    if (typeof struct.name === "string") out.push(struct.name);
    if (Array.isArray(struct.fields)) {
      for (const field of struct.fields) {
        if (Array.isArray(field)) {
          if (typeof field[0] === "string") out.push(field[0]);
          collectHeaderPropText(field[1], out);
        }
      }
    }
    return;
  }
  if ("Array" in value && Array.isArray(value.Array)) {
    for (const entry of value.Array) {
      if (!Array.isArray(entry)) continue;
      for (const field of entry) {
        if (Array.isArray(field)) {
          if (typeof field[0] === "string") out.push(field[0]);
          collectHeaderPropText(field[1], out);
        }
      }
    }
  }
}

export function inferReplayHitboxKind(
  playerInfo: RawPlayerInfo | null | undefined,
): ReplayHitboxKind {
  const stats = playerInfo?.stats;
  if (!stats) {
    return DEFAULT_REPLAY_HITBOX_KIND;
  }

  const textValues: string[] = [];
  for (const [key, prop] of Object.entries(stats)) {
    textValues.push(key);
    collectHeaderPropText(prop, textValues);
  }

  for (const value of textValues) {
    const hitbox = normalizeReplayHitboxKind(value);
    if (hitbox) {
      return hitbox;
    }
  }

  return DEFAULT_REPLAY_HITBOX_KIND;
}
