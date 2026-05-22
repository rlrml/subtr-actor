import { deflateSync, inflateSync, strFromU8, strToU8 } from "fflate";
import type {
  CameraSettings,
  ReplayCameraViewMode,
  ReplayFreeCameraPreset,
} from "subtr-actor-player";

export const STATS_PLAYER_CONFIG_VERSION = 1;

export type StatsWindowKind =
  | "player"
  | "team"
  | "all-players"
  | "all-teams"
  | "ad-hoc";

export type TeamScope = "blue" | "orange";
export type ModuleCapabilityKind = "events" | "ranges" | "effects";
export type SingletonWindowId = "camera" | "playback" | "recording" | "boost-pickups";
export type ConfigWindowKind = SingletonWindowId | "stats";

export interface ConfigViewportSize {
  readonly width: number;
  readonly height: number;
}

export interface WindowPlacementConfig {
  readonly x: number;
  readonly y: number;
  readonly viewport: ConfigViewportSize;
  readonly zIndex?: number;
  readonly visible: boolean;
}

export interface SelectedStatConfig {
  readonly statId: string;
  readonly targetId?: string;
}

export interface StatsWindowConfig {
  readonly id: string;
  readonly kind: StatsWindowKind;
  readonly placement: WindowPlacementConfig;
  readonly playerId: string | null;
  readonly team: TeamScope | null;
  readonly entries: SelectedStatConfig[];
}

export interface SingletonWindowConfig {
  readonly id: SingletonWindowId;
  readonly placement: WindowPlacementConfig;
}

export interface PlayerPlaybackConfig {
  readonly currentTime?: number;
  readonly rate?: number;
  readonly skipPostGoalTransitions?: boolean;
  readonly skipKickoffs?: boolean;
}

export interface PlayerCameraConfig {
  readonly mode?: ReplayCameraViewMode;
  readonly freePreset?: ReplayFreeCameraPreset | null;
  readonly attachedPlayerId?: string | null;
  readonly distanceScale?: number;
  readonly ballCam?: boolean;
  readonly customSettings?: CameraSettings | null;
}

export interface PlayerOverlayConfig {
  readonly timelineEvents: string[];
  readonly timelineRanges: string[];
  readonly renderEffects: string[];
  readonly followedPlayerHud: boolean;
  readonly boostPads: boolean;
  readonly boostPickupAnimation: boolean;
}

export interface RecordingConfig {
  readonly fps?: number;
  readonly playbackRate?: number;
}

export interface StatsPlayerConfigV1 {
  readonly version: typeof STATS_PLAYER_CONFIG_VERSION;
  readonly playback: PlayerPlaybackConfig;
  readonly camera: PlayerCameraConfig;
  readonly overlays: PlayerOverlayConfig;
  readonly recording: RecordingConfig;
  readonly singletonWindows: SingletonWindowConfig[];
  readonly statsWindows: StatsWindowConfig[];
  readonly moduleConfigs: Record<string, unknown>;
}

export type StatsPlayerConfig = StatsPlayerConfigV1;

export const STATS_PLAYER_CONFIG_PARAM = "cfg";
export const STATS_PLAYER_CONFIG_DEBUG_PARAM = "cfgDebug";

export type StatsPlayerConfigParamSource = "hash" | "search";

export interface StatsPlayerConfigParamSnapshot {
  readonly search: string;
  readonly hash: string;
  readonly searchParams: readonly [string, string][];
  readonly hashParams: readonly [string, string][];
  readonly searchValues: readonly string[];
  readonly hashValues: readonly string[];
  readonly selectedSource: StatsPlayerConfigParamSource | null;
  readonly selectedValue: string | null;
}

function bytesToBase64Url(bytes: Uint8Array): string {
  let binary = "";
  for (const byte of bytes) {
    binary += String.fromCharCode(byte);
  }

  return btoa(binary)
    .replaceAll("+", "-")
    .replaceAll("/", "_")
    .replace(/=+$/, "");
}

function base64UrlToBytes(value: string): Uint8Array {
  const normalized = value.replaceAll("-", "+").replaceAll("_", "/");
  const padded = normalized.padEnd(Math.ceil(normalized.length / 4) * 4, "=");
  const binary = atob(padded);
  const bytes = new Uint8Array(binary.length);
  for (let index = 0; index < binary.length; index += 1) {
    bytes[index] = binary.charCodeAt(index);
  }
  return bytes;
}

export function encodeStatsPlayerConfig(config: StatsPlayerConfig): string {
  return bytesToBase64Url(
    deflateSync(strToU8(JSON.stringify(config)), { level: 9 }),
  );
}

export function decodeStatsPlayerConfig(value: string): StatsPlayerConfig {
  let parsed: unknown;
  try {
    parsed = JSON.parse(strFromU8(inflateSync(base64UrlToBytes(value))));
  } catch (error) {
    throw new Error(
      `Invalid stats player config: ${
        error instanceof Error ? error.message : String(error)
      }`,
    );
  }

  return normalizeStatsPlayerConfig(parsed);
}

export function getStatsPlayerConfigFromLocation(
  location: Pick<Location, "search" | "hash">,
): StatsPlayerConfig | null {
  const snapshot = getStatsPlayerConfigParamSnapshot(location);
  return snapshot.selectedValue
    ? decodeStatsPlayerConfig(snapshot.selectedValue)
    : null;
}

export function getStatsPlayerConfigParamSnapshot(
  location: Pick<Location, "search" | "hash">,
): StatsPlayerConfigParamSnapshot {
  const hashParams = new URLSearchParams(getHashParamText(location.hash));
  const searchParams = new URLSearchParams(location.search);
  const hashValues = hashParams.getAll(STATS_PLAYER_CONFIG_PARAM);
  const searchValues = searchParams.getAll(STATS_PLAYER_CONFIG_PARAM);
  const selectedSource = hashValues[0]
    ? "hash"
    : searchValues[0]
      ? "search"
      : null;
  const selectedValue = selectedSource === "hash"
    ? hashValues[0]
    : selectedSource === "search"
      ? searchValues[0]
      : null;

  return {
    search: location.search,
    hash: location.hash,
    searchParams: [...searchParams.entries()],
    hashParams: [...hashParams.entries()],
    searchValues,
    hashValues,
    selectedSource,
    selectedValue,
  };
}

export function isStatsPlayerConfigDebugEnabled(
  location: Pick<Location, "search" | "hash">,
): boolean {
  const searchParams = new URLSearchParams(location.search);
  const hashParams = new URLSearchParams(getHashParamText(location.hash));
  const value = searchParams.get(STATS_PLAYER_CONFIG_DEBUG_PARAM) ??
    hashParams.get(STATS_PLAYER_CONFIG_DEBUG_PARAM);
  return value === "" || value === "1" || value === "true";
}

export function setStatsPlayerConfigOnUrl(
  url: URL,
  config: StatsPlayerConfig,
): URL {
  const next = new URL(url.href);
  const hashParams = new URLSearchParams(getHashParamText(next.hash));
  hashParams.set(STATS_PLAYER_CONFIG_PARAM, encodeStatsPlayerConfig(config));
  next.hash = hashParams.toString();
  return next;
}

function getHashParamText(hash: string): string {
  return hash.startsWith("#") ? hash.slice(1) : hash;
}

export function mapWindowPlacementToViewport(
  placement: WindowPlacementConfig,
  viewport: ConfigViewportSize,
  minimumVisibleWidth = 120,
  minimumVisibleHeight = 100,
): { x: number; y: number } {
  const sourceWidth = finitePositive(placement.viewport.width) ?? viewport.width;
  const sourceHeight = finitePositive(placement.viewport.height) ?? viewport.height;
  const scaleX = viewport.width / Math.max(1, sourceWidth);
  const scaleY = viewport.height / Math.max(1, sourceHeight);
  const maxX = Math.max(8, viewport.width - minimumVisibleWidth);
  const maxY = Math.max(8, viewport.height - minimumVisibleHeight);

  return {
    x: clamp(placement.x * scaleX, 8, maxX),
    y: clamp(placement.y * scaleY, 8, maxY),
  };
}

export function normalizeStatsPlayerConfig(value: unknown): StatsPlayerConfig {
  if (!isRecord(value) || value.version !== STATS_PLAYER_CONFIG_VERSION) {
    throw new Error("Unsupported stats player config version");
  }

  return {
    version: STATS_PLAYER_CONFIG_VERSION,
    playback: normalizePlaybackConfig(value.playback),
    camera: normalizeCameraConfig(value.camera),
    overlays: normalizeOverlayConfig(value.overlays),
    recording: normalizeRecordingConfig(value.recording),
    singletonWindows: normalizeSingletonWindows(value.singletonWindows),
    statsWindows: normalizeStatsWindows(value.statsWindows),
    moduleConfigs: isRecord(value.moduleConfigs) ? value.moduleConfigs : {},
  };
}

function normalizeRecordingConfig(value: unknown): RecordingConfig {
  if (!isRecord(value)) {
    return {};
  }
  return {
    fps: finiteNumber(value.fps),
    playbackRate: finiteNumber(value.playbackRate),
  };
}

function normalizePlaybackConfig(value: unknown): PlayerPlaybackConfig {
  if (!isRecord(value)) {
    return {};
  }
  return {
    currentTime: finiteNumber(value.currentTime),
    rate: finiteNumber(value.rate),
    skipPostGoalTransitions: booleanValue(value.skipPostGoalTransitions),
    skipKickoffs: booleanValue(value.skipKickoffs),
  };
}

function normalizeCameraConfig(value: unknown): PlayerCameraConfig {
  if (!isRecord(value)) {
    return {};
  }
  const config: {
    -readonly [K in keyof PlayerCameraConfig]?: PlayerCameraConfig[K];
  } = {};
  const mode = value.mode === "follow"
    ? "follow"
    : value.mode === "free"
      ? "free"
      : undefined;
  const freePreset = value.freePreset === "overhead"
    ? "overhead"
    : value.freePreset === "side"
      ? "side"
      : value.freePreset === null
        ? null
        : undefined;
  const attachedPlayerId = stringOrNull(value.attachedPlayerId);
  const distanceScale = finiteNumber(value.distanceScale);
  const ballCam = booleanValue(value.ballCam);
  const customSettings = normalizeCameraSettings(value.customSettings);
  if (mode !== undefined) config.mode = mode;
  if (freePreset !== undefined) config.freePreset = freePreset;
  if (attachedPlayerId !== undefined) config.attachedPlayerId = attachedPlayerId;
  if (distanceScale !== undefined) config.distanceScale = distanceScale;
  if (ballCam !== undefined) config.ballCam = ballCam;
  if (customSettings !== undefined) config.customSettings = customSettings;
  return config;
}

function normalizeCameraSettings(value: unknown): CameraSettings | null | undefined {
  if (value === null) {
    return null;
  }
  if (!isRecord(value)) {
    return undefined;
  }
  const settings: CameraSettings = {};
  const fov = finiteNumber(value.fov);
  const height = finiteNumber(value.height);
  const pitch = finiteNumber(value.pitch);
  const distance = finiteNumber(value.distance);
  const stiffness = finiteNumber(value.stiffness);
  const swivelSpeed = finiteNumber(value.swivelSpeed);
  const transitionSpeed = finiteNumber(value.transitionSpeed);
  if (fov !== undefined) settings.fov = fov;
  if (height !== undefined) settings.height = height;
  if (pitch !== undefined) settings.pitch = pitch;
  if (distance !== undefined) settings.distance = distance;
  if (stiffness !== undefined) settings.stiffness = stiffness;
  if (swivelSpeed !== undefined) settings.swivelSpeed = swivelSpeed;
  if (transitionSpeed !== undefined) settings.transitionSpeed = transitionSpeed;
  return settings;
}

function normalizeOverlayConfig(value: unknown): PlayerOverlayConfig {
  const record = isRecord(value) ? value : {};
  return {
    timelineEvents: stringArray(record.timelineEvents),
    timelineRanges: stringArray(record.timelineRanges),
    renderEffects: stringArray(record.renderEffects),
    followedPlayerHud: booleanValue(record.followedPlayerHud) ?? false,
    boostPads: booleanValue(record.boostPads) ?? true,
    boostPickupAnimation: booleanValue(record.boostPickupAnimation) ?? false,
  };
}

function normalizeSingletonWindows(value: unknown): SingletonWindowConfig[] {
  if (!Array.isArray(value)) {
    return [];
  }
  return value
    .map((item): SingletonWindowConfig | null => {
      if (!isRecord(item) || !isSingletonWindowId(item.id)) {
        return null;
      }
      return {
        id: item.id,
        placement: normalizePlacement(item.placement),
      };
    })
    .filter((item): item is SingletonWindowConfig => item !== null);
}

function normalizeStatsWindows(value: unknown): StatsWindowConfig[] {
  if (!Array.isArray(value)) {
    return [];
  }
  return value
    .map((item): StatsWindowConfig | null => {
      if (!isRecord(item) || typeof item.id !== "string" ||
        !isStatsWindowKind(item.kind)) {
        return null;
      }
      return {
        id: item.id,
        kind: item.kind,
        placement: normalizePlacement(item.placement),
        playerId: stringOrNull(item.playerId) ?? null,
        team: item.team === "orange" ? "orange" : item.team === "blue" ? "blue" : null,
        entries: normalizeSelectedStats(item.entries),
      };
    })
    .filter((item): item is StatsWindowConfig => item !== null);
}

function normalizeSelectedStats(value: unknown): SelectedStatConfig[] {
  if (!Array.isArray(value)) {
    return [];
  }
  return value
    .map((item): SelectedStatConfig | null => {
      if (!isRecord(item) || typeof item.statId !== "string") {
        return null;
      }
      return {
        statId: item.statId,
        targetId: typeof item.targetId === "string" ? item.targetId : undefined,
      };
    })
    .filter((item): item is SelectedStatConfig => item !== null);
}

function normalizePlacement(value: unknown): WindowPlacementConfig {
  const record = isRecord(value) ? value : {};
  const viewport = isRecord(record.viewport) ? record.viewport : {};
  return {
    x: finiteNumber(record.x) ?? 8,
    y: finiteNumber(record.y) ?? 8,
    viewport: {
      width: finitePositive(viewport.width) ?? 1,
      height: finitePositive(viewport.height) ?? 1,
    },
    zIndex: finiteNumber(record.zIndex),
    visible: booleanValue(record.visible) ?? true,
  };
}

function isSingletonWindowId(value: unknown): value is SingletonWindowId {
  return value === "camera" || value === "playback" || value === "recording" ||
    value === "boost-pickups";
}

function isStatsWindowKind(value: unknown): value is StatsWindowKind {
  return value === "player" || value === "team" || value === "all-players" ||
    value === "all-teams" || value === "ad-hoc";
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function finiteNumber(value: unknown): number | undefined {
  return typeof value === "number" && Number.isFinite(value) ? value : undefined;
}

function finitePositive(value: unknown): number | undefined {
  const number = finiteNumber(value);
  return number !== undefined && number > 0 ? number : undefined;
}

function booleanValue(value: unknown): boolean | undefined {
  return typeof value === "boolean" ? value : undefined;
}

function stringOrNull(value: unknown): string | null | undefined {
  if (value === null) {
    return null;
  }
  return typeof value === "string" ? value : undefined;
}

function stringArray(value: unknown): string[] {
  return Array.isArray(value)
    ? value.filter((item): item is string => typeof item === "string")
    : [];
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}
