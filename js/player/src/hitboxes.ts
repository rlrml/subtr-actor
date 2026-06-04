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
  offset: number;
  elevation: number;
}

export interface ReplayHitboxOverlayTransform {
  position: readonly [x: number, y: number, z: number];
  rotationYDegrees: number;
  dimensions: readonly [x: number, y: number, z: number];
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
    offset: 13.88,
    elevation: 17.05,
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
    offset: 13.88,
    elevation: 17.05,
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
    offset: 13.88,
    elevation: 17.05,
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
    offset: 13.88,
    elevation: 17.05,
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
    offset: 13.88,
    elevation: 17.05,
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
    offset: 13.88,
    elevation: 17.05,
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
  astonmartinvalhalla: "breakout",
  backfire: "octane",
  backtothefuturetimemachine: "dominus",
  batmobile1989: "dominus",
  battlebus: "merc",
  breakout: "breakout",
  breakouttypes: "breakout",
  centio: "plank",
  centiov17: "plank",
  cyclone: "breakout",
  deloreantimemachine: "dominus",
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
  thedarkknightstumbler: "octane",
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
  "1966cadillacdeville": "breakout",
  ace: "breakout",
  admiral: "dominus",
  azura: "breakout",
  behemoth: "merc",
  beskar: "hybrid",
  bmwm3e30: "dominus",
  bmwm2racing: "dominus",
  bmwm4gt3evo: "dominus",
  bmw1series: "octane",
  bmw1seriesrle: "octane",
  bmwm240i: "dominus",
  bugatticentodieci: "plank",
  bumblebee: "dominus",
  bumblebeecar: "dominus",
  chevroletastro: "merc",
  chevroletcorvettestingray: "breakout",
  chevroletcorvettezr1: "breakout",
  chryslerpacifica: "hybrid",
  corlay: "octane",
  cyberpunkquadra: "breakout",
  defenderd7xr: "merc",
  diesel: "breakout",
  dodgechargerdaytonascatpack: "dominus",
  dodgerchargerdaytonascatpack: "dominus",
  dominusneon: "dominus",
  emperor: "breakout",
  emperorii: "breakout",
  emperoriifrozen: "breakout",
  emperoriiscorched: "breakout",
  fastfuriousdodgecharger: "dominus",
  fastandfuriousdodgecharger: "dominus",
  fastandfuriousdodgechargersrthellcat: "dominus",
  fastfuriousmazdarx7: "breakout",
  fastandfuriousmazdarx7: "breakout",
  fastfuriousnissanskyline: "hybrid",
  fastandfuriousnissanskyline: "hybrid",
  fastfuriouspontiacfiero: "hybrid",
  fastandfuriouspontiacfiero: "hybrid",
  fenneczrf: "octane",
  ferrari296gtb: "dominus",
  ferrarif40: "breakout",
  fordbroncoraptorrle: "merc",
  fordf150rle: "octane",
  fordmustanggtd: "dominus",
  fordmustangshelbygt500: "dominus",
  fordmustangmacherle: "octane",
  fordmustangshelbygt350rrle: "dominus",
  formula12021: "plank",
  formula12022: "plank",
  fuse: "breakout",
  havoc: "breakout",
  hearse: "hybrid",
  homerscar: "dominus",
  hondacivictyper: "octane",
  hondacivictyperle: "octane",
  jackal: "octane",
  jeepwranglerrubicon: "octane",
  kitt: "dominus",
  knightindustries2000: "dominus",
  komodo: "breakout",
  lamborghinicountachlpi8004: "dominus",
  lamborghinihuracansto: "dominus",
  lamborghiniurus: "hybrid",
  lamborghiniurusse: "hybrid",
  lightningmcqueen: "dominus",
  lightningmcqueencar: "dominus",
  lockjaw: "dominus",
  luiginsr: "octane",
  maestro: "dominus",
  magnifique: "dominus",
  magnifiquegxt: "dominus",
  mako: "breakout",
  mamba: "dominus",
  mario: "octane",
  marionsr: "octane",
  maven: "dominus",
  mclaren765lt: "dominus",
  mclarenp1: "dominus",
  mclarensenna: "breakout",
  megastar: "breakout",
  mercedesamggt63s: "dominus",
  mercedesbenzcla: "dominus",
  mudcat: "octane",
  mudcatg1: "octane",
  mudcatgxt: "octane",
  nissan350z: "dominus",
  nissanfairladyz: "dominus",
  nissanfairladyzrle: "dominus",
  nissansilvia: "hybrid",
  nissansilviarle: "hybrid",
  nissanskylinegtr: "hybrid",
  nissanskylinegtrr32: "hybrid",
  nissanzperformance: "dominus",
  nissanzperformancecar: "dominus",
  outlaw: "octane",
  outlawgxt: "octane",
  pattywagon: "octane",
  pizzaplanetdeliverytruck: "merc",
  pontiacfirebird: "breakout",
  porsche918spyder: "breakout",
  porsche911gt3rs: "dominus",
  porsche911turbo: "dominus",
  porsche911turborle: "dominus",
  primo: "hybrid",
  psyclops: "octane",
  quadraturbor: "breakout",
  ram1500rho: "hybrid",
  recoilav: "merc",
  redline: "breakout",
  revolver: "breakout",
  rivianr1s: "hybrid",
  scorpion: "dominus",
  shokunin: "octane",
  shokuningxt: "octane",
  stampede: "merc",
  teslacybertruck: "hybrid",
  themysterymachine: "merc",
  theincredibile: "breakout",
  turtlevan: "merc",
  voidburn: "hybrid",
  volkswagengolfgti: "octane",
  volkswagengolfgtirle: "octane",
  xentari: "octane",
  zefira: "dominus",
  breakoutx: "breakout",
  nexus: "breakout",
  nexussc: "breakout",
  whiplash: "breakout",
  "007sastonmartindbs": "dominus",
  "007sastonmartinvalhalla": "dominus",
  batmobile2022: "dominus",
  chikara: "dominus",
  chikarag1: "dominus",
  chikaragxt: "dominus",
  ecto1: "dominus",
  ecto1ghostbusters: "dominus",
  fastfuriousdodgechargersrthellcat: "dominus",
  gazellagthotwheels: "dominus",
  kittknightrider: "dominus",
  lamborghinihuracnsto: "dominus",
  mr11hotwheels: "dominus",
  nascarchevroletcamaro: "dominus",
  nascarfordmustang: "dominus",
  nascartoyotacamry: "dominus",
  nascarnextgenchevroletcamaro: "dominus",
  nascarnextgenchevroletcamaro2022: "dominus",
  nascarnextgenfordmustang: "dominus",
  nascarnextgenfordmustang2022: "dominus",
  nascarnextgentoyotacamry: "dominus",
  nascarnextgentoyotacamry2022: "dominus",
  nemesis: "dominus",
  peregrinett: "dominus",
  perigrinett: "dominus",
  ronin: "dominus",
  roning1: "dominus",
  roningxt: "dominus",
  samusgunship: "dominus",
  samusgunshipnintendoexclusive: "dominus",
  tyranno: "dominus",
  tyrannogxt: "dominus",
  insidio: "hybrid",
  jager619: "hybrid",
  jger619: "hybrid",
  jger619rs: "hybrid",
  r3mx: "hybrid",
  r3mxgxt: "hybrid",
  tygris: "hybrid",
  nomad: "merc",
  nomadgxt: "merc",
  "007sastonmartindb5": "octane",
  armadillo: "octane",
  armadilloxboxexclusive: "octane",
  boneshaker: "octane",
  dingo: "octane",
  fast4wdhotwheels: "octane",
  harbinger: "octane",
  harbingergxt: "octane",
  hogsticker: "octane",
  hogstickerxboxexclusive: "octane",
  sweettooth: "octane",
  sweettoothplaystationexclusive: "octane",
  thedarkknighttumbler: "octane",
  batmobile2016: "plank",
  sentinel: "plank",
};

function idsToHitboxMap(
  entries: ReadonlyArray<readonly [readonly number[], ReplayHitboxKind]>,
): Readonly<Record<number, ReplayHitboxKind>> {
  const out: Record<number, ReplayHitboxKind> = {};
  for (const [bodyIds, hitbox] of entries) {
    for (const bodyId of bodyIds) {
      out[bodyId] = hitbox;
    }
  }
  return out;
}

const BODY_HITBOX_BY_ID: Readonly<Record<number, ReplayHitboxKind>> = idsToHitboxMap([
  [
    [
      22, 1416, 1894, 1932, 3031, 3311, 6243, 6489, 7651, 7696, 7890, 7901,
      8006, 8360, 8361, 8565, 8566, 8669, 9357, 10697, 10698, 10817, 10822,
      11038, 11394, 11505, 11677, 11800, 11933, 11949, 12173, 12315, 12361,
      12484,
    ],
    "breakout",
  ],
  [
    [
      29, 403, 597, 600, 1018, 1171, 1286, 1675, 1689, 1883, 2070, 2268, 2666,
      2950, 2951, 3155, 3156, 3157, 3265, 3426, 3875, 3879, 3880, 4014, 4155,
      4367, 4472, 4473, 4745, 4770, 4781, 4861, 4864, 5709, 5773, 5823, 5858,
      5964, 5979, 6122, 6244, 6247, 6260, 6836, 7211, 7337, 7338, 7341, 7343,
      7415, 7512, 7532, 7593, 7772, 8454, 9053, 9088, 9089, 9140, 9388, 9894,
      10094, 10440, 10441, 10694, 10695, 11016, 11095, 11315, 11336, 11534,
      11941, 11996, 12106, 12142, 12262, 12286, 12325, 12382, 12563, 12669,
    ],
    "dominus",
  ],
  [
    [
      28, 31, 1159, 1317, 1624, 1856, 2269, 3451, 3582, 3702, 5470, 5488,
      5879, 7012, 9084, 9085, 9427, 10044, 10805, 11138, 11141, 11379, 11932,
      12569, 12652,
    ],
    "hybrid",
  ],
  [
    [
      30, 4780, 7336, 7477, 7815, 7979, 10689, 11098, 11736, 11905, 11950, 12318,
      12335,
    ],
    "merc",
  ],
  [
    [
      21, 23, 25, 26, 27, 402, 404, 523, 607, 625, 723, 1172, 1295, 1300, 1475,
      1478, 1533, 1568, 1623, 2665, 2853, 2919, 2949, 4284, 4318, 4319, 4320,
      4782, 4906, 5020, 5039, 5188, 5361, 5547, 5713, 5837, 5951, 6939, 7947,
      7948, 8383, 8806, 8807, 10896, 10897, 10900, 10901, 11314, 11603, 12104,
      12105,
    ],
    "octane",
  ],
  [
    [24, 803, 1603, 1691, 1919, 3594, 3614, 3622, 4268, 5265, 7052, 8524],
    "plank",
  ],
]);

function normalizedText(value: string): string {
  return value.toLowerCase().replace(/[^a-z0-9]+/g, "");
}

function normalizeReplayHitboxFamily(
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
      return null;
  }
}

export function inferReplayHitboxKindFromBodyName(
  bodyName: string | null | undefined,
): ReplayHitboxKind | null {
  if (!bodyName) {
    return null;
  }
  return BODY_HITBOX_BY_NORMALIZED_NAME[normalizedText(bodyName)] ?? null;
}

export function normalizeReplayHitboxKind(
  value: string | null | undefined,
): ReplayHitboxKind | null {
  return normalizeReplayHitboxFamily(value) ?? inferReplayHitboxKindFromBodyName(value);
}

export function getReplayHitboxSpec(kind: ReplayHitboxKind): ReplayHitboxSpec {
  return REPLAY_HITBOX_SPECS[kind];
}

export function getReplayHitboxOverlayTransform(
  hitbox: ReplayHitboxSpec,
): ReplayHitboxOverlayTransform {
  return {
    position: [hitbox.offset, 0, hitbox.elevation],
    rotationYDegrees: hitbox.slopeDegrees,
    dimensions: [hitbox.length, hitbox.width, hitbox.height],
  };
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
  const explicitFamily = normalizeReplayHitboxFamily(playerInfo?.car_hitbox_family);
  if (explicitFamily) {
    return explicitFamily;
  }

  const bodyId = playerInfo?.car_body_id;
  if (typeof bodyId === "number") {
    const hitbox = BODY_HITBOX_BY_ID[bodyId];
    if (hitbox) {
      return hitbox;
    }
  }

  const explicitBodyName = inferReplayHitboxKindFromBodyName(playerInfo?.car_body_name);
  if (explicitBodyName) {
    return explicitBodyName;
  }

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
