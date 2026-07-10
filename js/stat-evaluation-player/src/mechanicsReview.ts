import type { PlaylistManifestPage, ReplayPlayerTrack } from "@rlrml/player";
import type { ReplayLoadBundle, ReplayLoadProgress } from "./replayLoader.ts";
import { formatMechanicKind } from "./timelineMarkers.ts";

export type MechanicsReviewDecision = "confirmed" | "rejected" | "uncertain" | "bad_candidate";

/**
 * Map a single keystroke to a review decision for rapid labeling:
 * `y`/`1` confirmed, `n`/`2` rejected, `u`/`3` uncertain, `b`/`4` bad candidate.
 */
export function getMechanicsReviewDecisionForKey(key: string): MechanicsReviewDecision | null {
  switch (key.toLowerCase()) {
    case "y":
    case "1":
      return "confirmed";
    case "n":
    case "2":
      return "rejected";
    case "u":
    case "3":
      return "uncertain";
    case "b":
    case "4":
      return "bad_candidate";
    default:
      return null;
  }
}

/**
 * True when a review decision endpoint targets the local flat-file label sink
 * (the dev-server `/review-labels/<dataset>` middleware), which accepts the
 * richer `{status, item_id, meta}` payload.
 */
export function isReviewLabelsEndpoint(endpoint: string): boolean {
  try {
    return new URL(endpoint, "http://localhost").pathname.startsWith("/review-labels/");
  } catch {
    return false;
  }
}

export type MechanicsReviewPlaybackBound =
  | { kind: "time"; value: number }
  | { kind: "frame"; value: number };

export type MechanicsReviewTimeBase = "playback" | "rawReplay";

export interface MechanicsReviewTimingReplay {
  duration: number;
  rawStartTime?: number;
  frames: readonly { time: number }[];
}

export interface MechanicsReviewPlaybackConfig {
  timeBase?: MechanicsReviewTimeBase;
  [key: string]: unknown;
}

export type MechanicsReviewBallCamMode = "off" | "on" | "player";

export interface MechanicsReviewClipPerspective {
  kind: "player";
  playerId?: string;
  playerName?: string;
  ballCam?: MechanicsReviewBallCamMode;
  usePlayerCameraSettings?: boolean;
}

export interface MechanicsReviewReplay {
  id: string;
  path?: string;
  label?: string;
  locator?: Record<string, unknown>;
  meta?: Record<string, unknown>;
}

export interface MechanicsReviewItemMeta {
  confidence?: number | null;
  eventId?: string;
  eventCategory?: string;
  eventType?: string;
  eventTypeLabel?: string;
  mechanic?: string;
  mechanicLabel?: string;
  playerId?: string;
  playerName?: string | null;
  reason?: string;
  reviewEndpoint?: string;
  reviewStatus?: string | null;
  target?: Record<string, unknown>;
  followupGoal?: unknown;
  [key: string]: unknown;
}

export interface MechanicsReviewItem {
  id?: string;
  replay: string;
  start: MechanicsReviewPlaybackBound;
  end: MechanicsReviewPlaybackBound;
  label?: string;
  perspective?: MechanicsReviewClipPerspective;
  meta?: MechanicsReviewItemMeta;
}

export interface MechanicsReviewPlaylist {
  label?: string;
  replays?: MechanicsReviewReplay[];
  items: MechanicsReviewItem[];
  page?: PlaylistManifestPage;
  playback?: MechanicsReviewPlaybackConfig;
  meta?: unknown;
}

export type MechanicsReviewReplayLoadStatus = "idle" | "loading" | "loaded" | "error";

export interface MechanicsReviewReplayLoadState {
  replayId: string;
  label: string;
  path: string;
  clipCount: number;
  status: MechanicsReviewReplayLoadStatus;
  progress: ReplayLoadProgress | null;
  error: string | null;
}

export interface ActiveMechanicsReview {
  manifest: MechanicsReviewPlaylist;
  sourceUrl: string | null;
  replaysById: Map<string, MechanicsReviewReplay>;
  replayLoadStates: Map<string, MechanicsReviewReplayLoadState>;
  replayLoadCache: Map<string, Promise<ReplayLoadBundle>>;
  currentIndex: number;
  loading: boolean;
  preloading: boolean;
  currentReplayId: string | null;
  currentClip: { startTime: number; endTime: number; targetTime: number | null } | null;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function parseMechanicsReviewBound(value: unknown): MechanicsReviewPlaybackBound | null {
  if (!isRecord(value)) {
    return null;
  }
  if (
    (value.kind === "time" || value.kind === "frame") &&
    typeof value.value === "number" &&
    Number.isFinite(value.value)
  ) {
    return {
      kind: value.kind,
      value: value.value,
    };
  }
  return null;
}

function parseOptionalMechanicsReviewPageInteger(
  value: unknown,
  field: string,
): number | undefined {
  if (value === undefined || value === null) {
    return undefined;
  }
  if (
    typeof value !== "number" ||
    !Number.isInteger(value) ||
    !Number.isFinite(value) ||
    value < 0
  ) {
    throw new Error(`Review playlist page ${field} must be a non-negative integer.`);
  }
  return value;
}

function parseOptionalMechanicsReviewPageString(value: unknown, field: string): string | undefined {
  if (value === undefined || value === null) {
    return undefined;
  }
  if (typeof value !== "string") {
    throw new Error(`Review playlist page ${field} must be a string.`);
  }
  return value;
}

function parseMechanicsReviewPage(value: unknown): PlaylistManifestPage | undefined {
  if (value === undefined || value === null) {
    return undefined;
  }
  if (!isRecord(value)) {
    throw new Error("Review playlist page must be an object.");
  }

  return {
    next: parseOptionalMechanicsReviewPageString(value.next, "next"),
    previous: parseOptionalMechanicsReviewPageString(value.previous, "previous"),
    total: parseOptionalMechanicsReviewPageInteger(value.total, "total"),
    count: parseOptionalMechanicsReviewPageInteger(value.count, "count"),
    limit: parseOptionalMechanicsReviewPageInteger(value.limit, "limit"),
    offset: parseOptionalMechanicsReviewPageInteger(value.offset, "offset"),
  };
}

function parseMechanicsReviewPlayback(value: unknown): MechanicsReviewPlaybackConfig | undefined {
  if (value === undefined || value === null) {
    return undefined;
  }
  if (!isRecord(value)) {
    throw new Error("Review playlist playback must be an object.");
  }
  if (
    value.timeBase !== undefined &&
    value.timeBase !== "playback" &&
    value.timeBase !== "rawReplay"
  ) {
    throw new Error('Review playlist playback timeBase must be "playback" or "rawReplay".');
  }
  return {
    ...value,
    timeBase: value.timeBase,
  };
}

function parseMechanicsReviewPerspective(
  value: unknown,
  itemNumber: number,
): MechanicsReviewClipPerspective | undefined {
  if (value === undefined || value === null) {
    return undefined;
  }
  if (!isRecord(value)) {
    throw new Error(`Review item ${itemNumber} perspective must be an object.`);
  }
  if (value.kind !== "player") {
    throw new Error(`Review item ${itemNumber} perspective kind must be "player".`);
  }
  const playerId =
    typeof value.playerId === "string" && value.playerId.trim() ? value.playerId.trim() : undefined;
  const playerName =
    typeof value.playerName === "string" && value.playerName.trim()
      ? value.playerName.trim()
      : undefined;
  if (!playerId && !playerName) {
    throw new Error(`Review item ${itemNumber} player perspective needs playerId or playerName.`);
  }
  if (
    value.ballCam !== undefined &&
    value.ballCam !== "off" &&
    value.ballCam !== "on" &&
    value.ballCam !== "player"
  ) {
    throw new Error(`Review item ${itemNumber} perspective ballCam must be off, on, or player.`);
  }
  if (
    value.usePlayerCameraSettings !== undefined &&
    typeof value.usePlayerCameraSettings !== "boolean"
  ) {
    throw new Error(
      `Review item ${itemNumber} perspective usePlayerCameraSettings must be boolean.`,
    );
  }

  return {
    kind: "player",
    playerId,
    playerName,
    ballCam: value.ballCam,
    usePlayerCameraSettings: value.usePlayerCameraSettings,
  };
}

export function parseMechanicsReviewPlaylist(value: unknown): MechanicsReviewPlaylist {
  if (!isRecord(value) || !Array.isArray(value.items)) {
    throw new Error("Review playlist must contain an items array.");
  }

  const items = value.items.map((rawItem, index): MechanicsReviewItem => {
    if (!isRecord(rawItem) || typeof rawItem.replay !== "string") {
      throw new Error(`Invalid review item at index ${index}.`);
    }
    const start = parseMechanicsReviewBound(rawItem.start);
    const end = parseMechanicsReviewBound(rawItem.end);
    if (!start || !end) {
      throw new Error(`Review item ${index + 1} has invalid start or end.`);
    }
    return {
      id: typeof rawItem.id === "string" ? rawItem.id : undefined,
      replay: rawItem.replay,
      start,
      end,
      label: typeof rawItem.label === "string" ? rawItem.label : undefined,
      perspective: parseMechanicsReviewPerspective(rawItem.perspective, index + 1),
      meta: isRecord(rawItem.meta) ? rawItem.meta : undefined,
    };
  });

  const replays = Array.isArray(value.replays)
    ? value.replays
        .map((rawReplay): MechanicsReviewReplay | null => {
          if (!isRecord(rawReplay) || typeof rawReplay.id !== "string") {
            return null;
          }
          return {
            id: rawReplay.id,
            path: typeof rawReplay.path === "string" ? rawReplay.path : undefined,
            label: typeof rawReplay.label === "string" ? rawReplay.label : undefined,
            locator: isRecord(rawReplay.locator) ? rawReplay.locator : undefined,
            meta: isRecord(rawReplay.meta) ? rawReplay.meta : undefined,
          };
        })
        .filter((replay): replay is MechanicsReviewReplay => replay !== null)
    : undefined;

  return {
    label: typeof value.label === "string" ? value.label : undefined,
    replays,
    items,
    page: parseMechanicsReviewPage(value.page),
    playback: parseMechanicsReviewPlayback(value.playback),
    meta: value.meta,
  };
}

export function parseMechanicsReviewPlaylistJson(text: string): MechanicsReviewPlaylist {
  let parsed: unknown;
  try {
    parsed = JSON.parse(text);
  } catch (error) {
    throw new Error(
      `Invalid review playlist JSON: ${error instanceof Error ? error.message : String(error)}`,
    );
  }
  return parseMechanicsReviewPlaylist(parsed);
}

export function getMechanicsReviewUrlFromLocation(): string | null {
  const params = new URLSearchParams(window.location.search);
  return (
    params.get("reviewPlaylist")?.trim() ||
    params.get("review")?.trim() ||
    params.get("playlist")?.trim() ||
    params.get("playlistUrl")?.trim() ||
    null
  );
}

function isLikelyLocalFilePath(path: string): boolean {
  return /^\/(?:home|Users|tmp|var\/tmp|mnt|media|run\/user|nix\/store)\//.test(path);
}

export function resolveMechanicsReviewUrl(value: string, sourceUrl: string | null): string {
  const path = value.startsWith("path:") ? value.slice("path:".length) : value;
  if (/^https?:\/\//i.test(path) || path.startsWith("/@fs/")) {
    return path;
  }
  if (path.startsWith("/")) {
    if (isLikelyLocalFilePath(path)) {
      return `/@fs${path}`;
    }
    if (sourceUrl) {
      const base = new URL(sourceUrl, window.location.href);
      if (base.origin !== window.location.origin) {
        return new URL(path, base.origin).href;
      }
    }
    return path;
  }
  return sourceUrl ? new URL(path, sourceUrl).href : path;
}

export function getMechanicsReviewReplayPath(
  item: MechanicsReviewItem,
  review: Pick<ActiveMechanicsReview, "replaysById">,
): string {
  const replay = review.replaysById.get(item.replay);
  if (replay?.path) {
    return replay.path;
  }
  if (
    isRecord(replay?.locator) &&
    replay.locator.kind === "path" &&
    typeof replay.locator.path === "string"
  ) {
    return replay.locator.path;
  }
  if (
    /^https?:\/\//i.test(item.replay) ||
    item.replay.startsWith("/") ||
    item.replay.startsWith("/@fs/") ||
    item.replay.startsWith("path:")
  ) {
    return item.replay;
  }
  throw new Error(`Review replay "${item.replay}" does not include a loadable path.`);
}

export function getMechanicsReviewReplayLabel(
  item: MechanicsReviewItem,
  review: Pick<ActiveMechanicsReview, "replaysById">,
): string {
  const replay = review.replaysById.get(item.replay);
  const rawPath = replay?.path ?? getMechanicsReviewReplayPath(item, review);
  const fileName = rawPath
    .replace(/^path:/, "")
    .split("/")
    .filter(Boolean)
    .pop();
  return replay?.label ?? fileName ?? "review replay";
}

export function formatMechanicsReviewTime(value: number | null | undefined): string {
  return typeof value === "number" && Number.isFinite(value) ? `${value.toFixed(2)}s` : "--";
}

export function formatMechanicsReviewBound(bound: MechanicsReviewPlaybackBound): string {
  return bound.kind === "time"
    ? formatMechanicsReviewTime(bound.value)
    : `frame ${Math.trunc(bound.value)}`;
}

export function getMechanicsReviewTargetNumber(
  item: MechanicsReviewItem,
  key: "startTime" | "endTime" | "eventTime",
): number | null {
  if (!isRecord(item.meta?.target)) {
    return null;
  }
  const value = item.meta.target[key];
  return typeof value === "number" && Number.isFinite(value) ? value : null;
}

export function getMechanicsReviewTargetFrame(
  item: MechanicsReviewItem,
  key: "startFrame" | "endFrame" | "eventFrame",
): number | null {
  if (!isRecord(item.meta?.target)) {
    return null;
  }
  const value = item.meta.target[key];
  return typeof value === "number" && Number.isFinite(value) ? Math.trunc(value) : null;
}

function getMechanicsReviewRawTimeOffset(
  item: MechanicsReviewItem,
  replay: MechanicsReviewTimingReplay,
): number {
  for (const [timeKey, frameKey] of [
    ["eventTime", "eventFrame"],
    ["startTime", "startFrame"],
    ["endTime", "endFrame"],
  ] as const) {
    const targetTime = getMechanicsReviewTargetNumber(item, timeKey);
    const targetFrame = getMechanicsReviewTargetFrame(item, frameKey);
    const replayFrameTime = targetFrame === null ? null : replay.frames[targetFrame]?.time;
    if (
      targetTime !== null &&
      typeof replayFrameTime === "number" &&
      Number.isFinite(replayFrameTime)
    ) {
      return targetTime - replayFrameTime;
    }
  }
  return 0;
}

function getMechanicsReviewTimeOffset(
  item: MechanicsReviewItem,
  replay: MechanicsReviewTimingReplay,
  timeBase: MechanicsReviewTimeBase | undefined,
): number {
  if (timeBase === "playback") {
    return 0;
  }
  if (
    timeBase === "rawReplay" &&
    typeof replay.rawStartTime === "number" &&
    Number.isFinite(replay.rawStartTime)
  ) {
    return replay.rawStartTime;
  }
  return getMechanicsReviewRawTimeOffset(item, replay);
}

function clampMechanicsReviewTime(value: number, duration: number): number {
  return Math.min(Math.max(0, value), Math.max(0, duration));
}

export function resolveMechanicsReviewBoundTime(
  item: MechanicsReviewItem,
  bound: MechanicsReviewPlaybackBound,
  replay: MechanicsReviewTimingReplay,
  timeBase?: MechanicsReviewTimeBase,
): number {
  if (bound.kind === "frame") {
    const frameIndex = Math.max(0, Math.trunc(bound.value));
    return clampMechanicsReviewTime(replay.frames[frameIndex]?.time ?? 0, replay.duration);
  }

  const timeOffset = getMechanicsReviewTimeOffset(item, replay, timeBase);
  return clampMechanicsReviewTime(bound.value - timeOffset, replay.duration);
}

export function resolveMechanicsReviewTargetTime(
  item: MechanicsReviewItem,
  replay: MechanicsReviewTimingReplay,
  timeBase?: MechanicsReviewTimeBase,
): number | null {
  const targetTime = getMechanicsReviewTargetTime(item);
  if (targetTime === null) {
    return null;
  }
  return clampMechanicsReviewTime(
    targetTime - getMechanicsReviewTimeOffset(item, replay, timeBase),
    replay.duration,
  );
}

export function formatMechanicsReviewClipDetails(item: MechanicsReviewItem): string {
  const clipStart = item.start.kind === "time" ? item.start.value : null;
  const clipEnd = item.end.kind === "time" ? item.end.value : null;
  const parts = [
    `${formatMechanicsReviewBound(item.start)} to ${formatMechanicsReviewBound(item.end)}`,
  ];
  if (clipStart !== null && clipEnd !== null) {
    parts.push(`${Math.max(0, clipEnd - clipStart).toFixed(1)}s clip`);
  }
  const targetStart =
    getMechanicsReviewTargetNumber(item, "startTime") ??
    getMechanicsReviewTargetNumber(item, "eventTime");
  const targetEnd =
    getMechanicsReviewTargetNumber(item, "endTime") ??
    getMechanicsReviewTargetNumber(item, "eventTime");
  if (clipStart !== null && targetStart !== null) {
    parts.push(`${Math.max(0, targetStart - clipStart).toFixed(1)}s preroll`);
  }
  if (clipEnd !== null && targetEnd !== null) {
    parts.push(`${Math.max(0, clipEnd - targetEnd).toFixed(1)}s postroll`);
  }
  return parts.join(" · ");
}

export function formatMechanicsReviewEventDetails(item: MechanicsReviewItem): string {
  const eventTime = getMechanicsReviewTargetNumber(item, "eventTime");
  const startTime = getMechanicsReviewTargetNumber(item, "startTime");
  const endTime = getMechanicsReviewTargetNumber(item, "endTime");
  const eventFrame = getMechanicsReviewTargetFrame(item, "eventFrame");
  const startFrame = getMechanicsReviewTargetFrame(item, "startFrame");
  const endFrame = getMechanicsReviewTargetFrame(item, "endFrame");
  const time =
    startTime !== null && endTime !== null && Math.abs(endTime - startTime) > 0.001
      ? `${formatMechanicsReviewTime(startTime)} to ${formatMechanicsReviewTime(endTime)}`
      : formatMechanicsReviewTime(eventTime ?? startTime ?? endTime);
  const frame =
    startFrame !== null && endFrame !== null && endFrame !== startFrame
      ? `frames ${startFrame}-${endFrame}`
      : eventFrame !== null
        ? `frame ${eventFrame}`
        : startFrame !== null
          ? `frame ${startFrame}`
          : null;
  return [time, frame].filter((part) => part && part !== "--").join(" · ") || "--";
}

export function getMechanicsReviewItemLabel(item: MechanicsReviewItem, index: number): string {
  return (
    item.label ??
    item.meta?.eventTypeLabel ??
    item.meta?.mechanicLabel ??
    `Review item ${index + 1}`
  );
}

export function getMechanicsReviewPlayerName(item: MechanicsReviewItem): string | null {
  if (typeof item.meta?.playerName === "string" && item.meta.playerName.trim()) {
    return item.meta.playerName.trim();
  }
  if (isRecord(item.meta?.target) && typeof item.meta.target.playerName === "string") {
    const playerName = item.meta.target.playerName.trim();
    return playerName ? playerName : null;
  }
  return null;
}

export function resolveMechanicsReviewPerspectivePlayerTrack(
  perspective: MechanicsReviewClipPerspective | undefined,
  players: readonly ReplayPlayerTrack[],
): ReplayPlayerTrack | null {
  if (!perspective) {
    return null;
  }
  if (perspective.playerId) {
    const exact = players.find((player) => player.id === perspective.playerId);
    if (exact) {
      return exact;
    }
  }

  const playerName = perspective.playerName?.toLowerCase();
  return playerName
    ? (players.find((player) => player.name.trim().toLowerCase() === playerName) ?? null)
    : null;
}

export function getMechanicsReviewMechanicLabel(item: MechanicsReviewItem): string {
  if (typeof item.meta?.eventTypeLabel === "string" && item.meta.eventTypeLabel.trim()) {
    return item.meta.eventTypeLabel;
  }
  if (typeof item.meta?.mechanicLabel === "string" && item.meta.mechanicLabel.trim()) {
    return item.meta.mechanicLabel;
  }
  const eventType = item.meta?.eventType ?? item.meta?.mechanic;
  return typeof eventType === "string" ? formatMechanicKind(eventType) : "--";
}

export function getMechanicsReviewCategoryLabel(item: MechanicsReviewItem): string {
  const category = item.meta?.eventCategory;
  return typeof category === "string" && category.trim()
    ? formatMechanicKind(category.trim())
    : "--";
}

export function getMechanicsReviewMechanicKind(item: MechanicsReviewItem): string | null {
  const mechanic = item.meta?.eventType ?? item.meta?.mechanic;
  return typeof mechanic === "string" && mechanic.trim()
    ? mechanic.trim().replaceAll("-", "_")
    : null;
}

export function getMechanicsReviewTargetTime(item: MechanicsReviewItem): number | null {
  return (
    getMechanicsReviewTargetNumber(item, "eventTime") ??
    getMechanicsReviewTargetNumber(item, "startTime") ??
    getMechanicsReviewTargetNumber(item, "endTime")
  );
}
