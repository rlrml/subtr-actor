import { formatMechanicKind } from "./timelineMarkers.ts";
import { formatReplayLoadProgress, type ReplayLoadProgress } from "./replayLoader.ts";
import { isRecord } from "./mechanicsReviewPlaylist.ts";
import type {
  ActiveMechanicsReview,
  MechanicsReviewItem,
  MechanicsReviewPlaybackBound,
  MechanicsReviewReplayLoadState,
} from "./mechanicsReviewTypes.ts";

export type {
  ActiveMechanicsReview,
  MechanicsReviewItem,
  MechanicsReviewItemMeta,
  MechanicsReviewPlaybackBound,
  MechanicsReviewPlaylist,
  MechanicsReviewReplay,
  MechanicsReviewReplayLoadState,
  MechanicsReviewReplayLoadStatus,
} from "./mechanicsReviewTypes.ts";
export {
  parseMechanicsReviewPlaylist,
  parseMechanicsReviewPlaylistJson,
} from "./mechanicsReviewPlaylist.ts";

interface MechanicsReviewPlayer {
  id: string;
  name: string;
}

interface ReplayFrameTime {
  time: number;
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
  review: ActiveMechanicsReview,
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
  review: ActiveMechanicsReview,
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

export function createMechanicsReviewReplaySource(
  item: MechanicsReviewItem,
  review: ActiveMechanicsReview,
  signal?: AbortSignal,
) {
  const replayPath = getMechanicsReviewReplayPath(item, review);
  const url = resolveMechanicsReviewUrl(replayPath, review.sourceUrl);
  return {
    name: getMechanicsReviewReplayLabel(item, review),
    preparingStatus: "Loading review replay...",
    async readBytes() {
      const response = await fetch(url, { signal });
      if (!response.ok) {
        const statusText = response.statusText ? ` ${response.statusText}` : "";
        throw new Error(
          `Failed to fetch review replay from ${url} (${response.status}${statusText})`,
        );
      }
      return new Uint8Array(await response.arrayBuffer());
    },
  };
}

export function getMechanicsReviewBoundTime(
  bound: MechanicsReviewPlaybackBound,
  frames?: readonly ReplayFrameTime[],
): number {
  if (bound.kind === "time") {
    return bound.value;
  }
  const frameIndex = Math.max(0, Math.trunc(bound.value));
  return frames?.[frameIndex]?.time ?? frames?.at(-1)?.time ?? 0;
}

function formatMechanicsReviewTime(value: number | null | undefined): string {
  return typeof value === "number" && Number.isFinite(value) ? `${value.toFixed(2)}s` : "--";
}

function formatMechanicsReviewBound(bound: MechanicsReviewPlaybackBound): string {
  return bound.kind === "time"
    ? formatMechanicsReviewTime(bound.value)
    : `frame ${Math.trunc(bound.value)}`;
}

function getMechanicsReviewTargetNumber(
  item: MechanicsReviewItem,
  key: "startTime" | "endTime" | "eventTime",
): number | null {
  if (!isRecord(item.meta?.target)) {
    return null;
  }
  const value = item.meta.target[key];
  return typeof value === "number" && Number.isFinite(value) ? value : null;
}

function getMechanicsReviewTargetFrame(
  item: MechanicsReviewItem,
  key: "startFrame" | "endFrame" | "eventFrame",
): number | null {
  if (!isRecord(item.meta?.target)) {
    return null;
  }
  const value = item.meta.target[key];
  return typeof value === "number" && Number.isFinite(value) ? Math.trunc(value) : null;
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
  return item.label ?? item.meta?.mechanicLabel ?? `Review item ${index + 1}`;
}

export function getMechanicsReviewPlayerId(item: MechanicsReviewItem): string | null {
  if (typeof item.meta?.playerId === "string") {
    return item.meta.playerId;
  }
  if (isRecord(item.meta?.target) && typeof item.meta.target.playerId === "string") {
    return item.meta.target.playerId;
  }
  return null;
}

export function getMechanicsReviewPlayerName(
  item: MechanicsReviewItem,
  players?: readonly MechanicsReviewPlayer[],
): string {
  if (typeof item.meta?.playerName === "string" && item.meta.playerName.trim()) {
    return item.meta.playerName;
  }
  const playerId = getMechanicsReviewPlayerId(item);
  return playerId ? (players?.find((player) => player.id === playerId)?.name ?? playerId) : "--";
}

export function getMechanicsReviewMechanicLabel(item: MechanicsReviewItem): string {
  if (typeof item.meta?.mechanicLabel === "string" && item.meta.mechanicLabel.trim()) {
    return item.meta.mechanicLabel;
  }
  return typeof item.meta?.mechanic === "string" ? formatMechanicKind(item.meta.mechanic) : "--";
}

export function formatMechanicsReviewStatus(value: unknown): string {
  return typeof value === "string" && value.trim() ? value.replaceAll("_", " ") : "unreviewed";
}

export function getMechanicsReviewDecisionEndpoint(
  item: MechanicsReviewItem | null,
): string | null {
  if (!item) {
    return null;
  }
  if (typeof item.meta?.reviewEndpoint === "string" && item.meta.reviewEndpoint) {
    return item.meta.reviewEndpoint;
  }
  const eventId =
    typeof item.meta?.eventId === "string" && item.meta.eventId ? item.meta.eventId : item.id;
  return eventId ? `/api/v1/mechanics/events/${encodeURIComponent(eventId)}/reviews` : null;
}

export function mechanicsReviewAuthHeaders(): Record<string, string> {
  const params = new URLSearchParams(window.location.search);
  const token =
    params.get("reviewToken") ??
    params.get("token") ??
    window.localStorage.getItem("rocket_sense_access_token");
  return token ? { Authorization: `Bearer ${token}` } : {};
}

export function getMechanicsReviewReplayItems(
  review: ActiveMechanicsReview,
): Map<string, MechanicsReviewItem> {
  const itemsByReplayId = new Map<string, MechanicsReviewItem>();
  for (const item of review.manifest.items) {
    if (!itemsByReplayId.has(item.replay)) {
      itemsByReplayId.set(item.replay, item);
    }
  }
  return itemsByReplayId;
}

function getMechanicsReviewReplayClipCounts(review: ActiveMechanicsReview): Map<string, number> {
  const counts = new Map<string, number>();
  for (const item of review.manifest.items) {
    counts.set(item.replay, (counts.get(item.replay) ?? 0) + 1);
  }
  return counts;
}

export function initializeMechanicsReviewReplayLoadStates(review: ActiveMechanicsReview): void {
  const clipCounts = getMechanicsReviewReplayClipCounts(review);
  for (const [replayId, item] of getMechanicsReviewReplayItems(review)) {
    let path = "";
    let label = replayId;
    try {
      path = getMechanicsReviewReplayPath(item, review);
      label = getMechanicsReviewReplayLabel(item, review);
    } catch {
      const replay = review.replaysById.get(replayId);
      label = replay?.label ?? replayId;
    }
    review.replayLoadStates.set(replayId, {
      replayId,
      label,
      path,
      clipCount: clipCounts.get(replayId) ?? 0,
      status: "idle",
      progress: null,
      error: null,
    });
  }
}

function formatReplayLoadStateProgress(progress: ReplayLoadProgress | null): string {
  if (!progress) {
    return "";
  }
  const label = formatReplayLoadProgress(progress);
  if (progress.processedFrames !== undefined) {
    const total = progress.totalFrames !== undefined ? ` / ${progress.totalFrames}` : "";
    return `${label} (${progress.processedFrames}${total} frames)`;
  }
  if (progress.processedChunks !== undefined) {
    const total = progress.totalChunks !== undefined ? ` / ${progress.totalChunks}` : "";
    return `${label} (${progress.processedChunks}${total} chunks)`;
  }
  return label;
}

export function mechanicsReviewReplayLoadStatusText(state: MechanicsReviewReplayLoadState): string {
  if (state.status === "idle") {
    return "Pending";
  }
  if (state.status === "loading") {
    return formatReplayLoadStateProgress(state.progress) || "Loading";
  }
  if (state.status === "loaded") {
    return "Loaded";
  }
  return state.error ? `Failed: ${state.error}` : "Failed";
}

export function mechanicsReviewReplayLoadProgressValue(
  state: MechanicsReviewReplayLoadState,
): number {
  if (state.status === "loaded") {
    return 1;
  }
  const value = state.progress?.progress;
  return typeof value === "number" && Number.isFinite(value) ? Math.max(0, Math.min(1, value)) : 0;
}
