import "./styles.css";
import {
  ReplayPlaylistPlayer,
  createBallchasingReplaySource,
  createReplaySource,
  loadPlaylistManifestFromFile,
  loadReplayFromBytes,
  parsePlaylistManifest,
  resolvePlaylistItemsFromManifest,
  type PlaylistAdvanceMode,
  type PlaylistEndMode,
  type PlaylistItem,
  type PlaylistManifest,
  type PlaylistManifestReplay,
  type PlaylistSourceLoadProgress,
  type PlaylistSourceLoadState,
  type ReplayPlaylistPlayerState,
  type ReplaySource,
} from "subtr-actor-player";

type CandidateMeta = {
  itemId?: string;
  eventId?: string;
  mechanic?: string;
  mechanicLabel?: string;
  detector?: string;
  confidence?: number;
  reason?: string;
  playerId?: string;
  playerName?: string | null;
  team?: string | null;
  target?: {
    kind?: string;
    playerId?: string;
    startTime?: number;
    endTime?: number;
    setupStartTime?: number;
    eventTime?: number;
    setupStartFrame?: number;
    eventFrame?: number;
  };
  reviewEndpoint?: string;
  reviewStatus?: string | null;
  event?: unknown;
};

const viewport = requireElement<HTMLDivElement>("viewport");
const statusLine = requireElement<HTMLDivElement>("status");
const playlistSummary = requireElement<HTMLDivElement>("playlist-summary");
const playlistFile = requireElement<HTMLInputElement>("playlist-file");
const playlistUrl = requireElement<HTMLInputElement>("playlist-url");
const loadUrlButton = requireElement<HTMLButtonElement>("load-url");
const candidateIndex = requireElement<HTMLDivElement>("candidate-index");
const candidateTitle = requireElement<HTMLHeadingElement>("candidate-title");
const candidateMechanic = requireElement<HTMLElement>("candidate-mechanic");
const candidatePlayer = requireElement<HTMLElement>("candidate-player");
const candidateConfidence = requireElement<HTMLElement>("candidate-confidence");
const candidateReplay = requireElement<HTMLElement>("candidate-replay");
const candidateReason = requireElement<HTMLElement>("candidate-reason");
const scrubber = requireElement<HTMLInputElement>("scrubber");
const timeReadout = requireElement<HTMLDivElement>("time-readout");
const previousButton = requireElement<HTMLButtonElement>("previous");
const playButton = requireElement<HTMLButtonElement>("play");
const nextButton = requireElement<HTMLButtonElement>("next");
const confirmButton = requireElement<HTMLButtonElement>("confirm");
const rejectButton = requireElement<HTMLButtonElement>("reject");
const uncertainButton = requireElement<HTMLButtonElement>("uncertain");
const reviewStatus = requireElement<HTMLDivElement>("review-status");
const advanceMode = requireElement<HTMLSelectElement>("advance-mode");
const endMode = requireElement<HTMLSelectElement>("end-mode");
const speedSelect = requireElement<HTMLSelectElement>("speed");
const followPlayer = requireElement<HTMLInputElement>("follow-player");
const replayLoadSummary = requireElement<HTMLElement>("replay-load-summary");
const replayLoads = requireElement<HTMLDivElement>("replay-loads");
const playlistCount = requireElement<HTMLElement>("playlist-count");
const playlistItems = requireElement<HTMLDivElement>("playlist-items");
const eventJson = requireElement<HTMLPreElement>("event-json");

let reviewPlayer: ReplayPlaylistPlayer | null = null;
let activeManifest: PlaylistManifest | null = null;
let activeManifestUrl: string | null = null;
let scrubbing = false;

function requireElement<T extends HTMLElement>(id: string): T {
  const element = document.getElementById(id);
  if (!element) {
    throw new Error(`Missing element #${id}`);
  }
  return element as T;
}

function candidateMeta(item: PlaylistItem | null): CandidateMeta {
  return (item?.meta ?? {}) as CandidateMeta;
}

function formatSeconds(value: number): string {
  return `${value.toFixed(1)}s`;
}

function formatConfidence(value: unknown): string {
  return typeof value === "number" && Number.isFinite(value) ? `${Math.round(value * 100)}%` : "-";
}

function formatReviewStatus(value: unknown): string {
  return typeof value === "string" && value.trim() ? value.replaceAll("_", " ") : "unreviewed";
}

function mechanicLabel(meta: CandidateMeta): string {
  return meta.mechanicLabel ?? meta.mechanic?.replaceAll("_", " ") ?? "-";
}

function isLikelyLocalFilePath(path: string): boolean {
  return /^\/(?:home|Users|tmp|var\/tmp|mnt|media|run\/user|nix\/store)\//.test(path);
}

function resolveReplayFetchUrl(path: string): string {
  if (/^https?:\/\//i.test(path)) {
    return path;
  }
  if (path.startsWith("/@fs/")) {
    return path;
  }
  if (path.startsWith("/")) {
    return isLikelyLocalFilePath(path) ? `/@fs${path}` : path;
  }
  if (activeManifestUrl) {
    return new URL(path, activeManifestUrl).href;
  }
  return path;
}

async function fetchReplayBytes(
  path: string,
  updateProgress?: (progress: PlaylistSourceLoadProgress) => void,
): Promise<Uint8Array> {
  const response = await fetch(resolveReplayFetchUrl(path));
  if (!response.ok) {
    throw new Error(`Failed to load replay ${path}: ${response.status}`);
  }
  const totalBytesHeader = response.headers.get("content-length");
  const parsedTotalBytes = totalBytesHeader === null ? NaN : Number(totalBytesHeader);
  const totalBytes = Number.isFinite(parsedTotalBytes) ? parsedTotalBytes : undefined;
  if (!response.body) {
    const bytes = new Uint8Array(await response.arrayBuffer());
    updateProgress?.({
      stage: "fetching",
      processedBytes: bytes.byteLength,
      totalBytes: totalBytes ?? bytes.byteLength,
      progress: totalBytes && totalBytes > 0 ? bytes.byteLength / totalBytes : undefined,
    });
    return bytes;
  }

  const reader = response.body.getReader();
  const chunks: Uint8Array[] = [];
  let processedBytes = 0;
  while (true) {
    const { done, value } = await reader.read();
    if (done) {
      break;
    }
    chunks.push(value);
    processedBytes += value.byteLength;
    updateProgress?.({
      stage: "fetching",
      processedBytes,
      totalBytes,
      progress: totalBytes && totalBytes > 0 ? processedBytes / totalBytes : undefined,
    });
  }

  const bytes = new Uint8Array(processedBytes);
  let offset = 0;
  for (const chunk of chunks) {
    bytes.set(chunk, offset);
    offset += chunk.byteLength;
  }
  return bytes;
}

function ballchasingIdFromReplayId(replayId: string): string | null {
  return replayId.startsWith("ballchasing:") ? replayId.slice("ballchasing:".length) : null;
}

function resolveReplaySource(context: {
  replayId: string;
  replay?: PlaylistManifestReplay;
}): ReplaySource {
  const path = context.replay?.path;
  if (path) {
    return createReplaySource(context.replayId, async (loadContext) =>
      loadReplayFromBytes(await fetchReplayBytes(path, loadContext?.updateProgress), {
        useWorker: true,
        reportEveryNFrames: 100,
        onProgress(progress) {
          loadContext?.updateProgress({
            stage: progress.stage,
            progress: progress.progress,
            processedFrames: progress.processedFrames,
            totalFrames: progress.totalFrames,
          });
        },
      }),
    );
  }

  const ballchasingId = ballchasingIdFromReplayId(context.replayId);
  if (ballchasingId) {
    return createBallchasingReplaySource(ballchasingId);
  }

  throw new Error(`No loadable replay locator for ${context.replayId}`);
}

async function loadPlaylist(manifest: PlaylistManifest, sourceUrl: string | null) {
  activeManifest = manifest;
  activeManifestUrl = sourceUrl;
  reviewPlayer?.destroy();
  viewport.replaceChildren();

  const items = resolvePlaylistItemsFromManifest(manifest, resolveReplaySource);
  reviewPlayer = new ReplayPlaylistPlayer(viewport, items, {
    autoplay: false,
    advanceMode: manifest.playback?.advanceMode ?? "manual",
    endMode: manifest.playback?.endMode ?? "stop",
    initialPlaybackRate: Number(speedSelect.value),
    initialCameraViewMode: "follow",
    initialBallCamEnabled: true,
    initialSkipPostGoalTransitionsEnabled: false,
    preloadPolicy: { kind: "all" },
  });
  reviewPlayer.subscribe(renderState);
  advanceMode.value = manifest.playback?.advanceMode ?? "manual";
  endMode.value = manifest.playback?.endMode ?? "stop";
  playlistSummary.textContent = `${manifest.label ?? "Playlist"} · ${items.length} candidates`;
  renderPlaylistItems(items);

  try {
    await reviewPlayer.waitForCurrentItem();
    attachCandidatePlayer();
  } catch (error) {
    setStatus(error instanceof Error ? error.message : String(error), true);
  }
}

function renderPlaylistItems(items: PlaylistItem[]) {
  playlistCount.textContent = `${items.length} ${items.length === 1 ? "item" : "items"}`;
  playlistItems.replaceChildren();
  for (const [index, item] of items.entries()) {
    const meta = candidateMeta(item);
    const button = document.createElement("button");
    button.type = "button";
    button.className = "playlist-item";
    button.dataset.index = String(index);
    button.innerHTML = `
      <span class="playlist-item-index">${index + 1}</span>
      <span class="playlist-item-main">
        <span class="playlist-item-title"></span>
        <span class="playlist-item-meta"></span>
      </span>
    `;
    button.querySelector(".playlist-item-title")!.textContent =
      item.label ?? `${mechanicLabel(meta)} candidate`;
    button.querySelector(".playlist-item-meta")!.textContent = [
      mechanicLabel(meta),
      formatConfidence(meta.confidence),
      meta.reason,
    ]
      .filter((value) => value && value !== "-")
      .join(" · ");
    button.addEventListener("click", async () => {
      await reviewPlayer?.setCurrentItemIndex(index);
      attachCandidatePlayer();
    });
    playlistItems.append(button);
  }
}

function updatePlaylistSelection(index: number) {
  for (const button of playlistItems.querySelectorAll<HTMLButtonElement>(".playlist-item")) {
    button.classList.toggle("active", Number(button.dataset.index) === index);
  }
  const active = playlistItems.querySelector<HTMLButtonElement>(".playlist-item.active");
  active?.scrollIntoView({ block: "nearest" });
}

function replayClipCounts(): Map<string, number> {
  const counts = new Map<string, number>();
  for (const item of reviewPlayer?.items ?? []) {
    counts.set(item.replay.id, (counts.get(item.replay.id) ?? 0) + 1);
  }
  return counts;
}

function manifestReplaysById(): Map<string, PlaylistManifestReplay> {
  return new Map((activeManifest?.replays ?? []).map((replay) => [replay.id, replay]));
}

function formatBytes(value: number): string {
  const units = ["B", "KB", "MB", "GB"];
  let scaled = value;
  let unitIndex = 0;
  while (scaled >= 1024 && unitIndex < units.length - 1) {
    scaled /= 1024;
    unitIndex += 1;
  }
  const digits = unitIndex === 0 ? 0 : scaled >= 10 ? 1 : 2;
  return `${scaled.toFixed(digits)} ${units[unitIndex]}`;
}

function progressFraction(progress: PlaylistSourceLoadProgress | null): number | null {
  if (!progress) {
    return null;
  }
  if (typeof progress.progress === "number" && Number.isFinite(progress.progress)) {
    return Math.max(0, Math.min(1, progress.progress));
  }
  if (
    typeof progress.processedBytes === "number" &&
    typeof progress.totalBytes === "number" &&
    progress.totalBytes > 0
  ) {
    return Math.max(0, Math.min(1, progress.processedBytes / progress.totalBytes));
  }
  if (
    typeof progress.processedFrames === "number" &&
    typeof progress.totalFrames === "number" &&
    progress.totalFrames > 0
  ) {
    return Math.max(0, Math.min(1, progress.processedFrames / progress.totalFrames));
  }
  return null;
}

function formatProgress(progress: PlaylistSourceLoadProgress | null): string {
  if (!progress) {
    return "";
  }
  const fraction = progressFraction(progress);
  if (progress.stage === "fetching" && typeof progress.processedBytes === "number") {
    const total =
      typeof progress.totalBytes === "number" ? ` / ${formatBytes(progress.totalBytes)}` : "";
    return `Fetching ${formatBytes(progress.processedBytes)}${total}`;
  }
  if (progress.stage && typeof progress.processedFrames === "number") {
    const total = typeof progress.totalFrames === "number" ? ` / ${progress.totalFrames}` : "";
    return `${progress.stage} ${progress.processedFrames}${total} frames`;
  }
  if (progress.stage && fraction !== null) {
    return `${progress.stage} ${Math.round(fraction * 100)}%`;
  }
  return progress.message ?? progress.stage ?? "";
}

function replayLoadLabel(state: PlaylistSourceLoadState): string {
  if (state.status === "idle") {
    return "Pending";
  }
  if (state.status === "loading") {
    return formatProgress(state.progress) || "Loading";
  }
  if (state.status === "loaded") {
    return "Loaded";
  }
  return state.error ? `Failed: ${state.error}` : "Failed";
}

function renderReplayLoads(states: PlaylistSourceLoadState[]) {
  const counts = replayClipCounts();
  const manifestReplays = manifestReplaysById();
  const loaded = states.filter((state) => state.status === "loaded").length;
  const loading = states.filter((state) => state.status === "loading").length;
  const failed = states.filter((state) => state.status === "error").length;
  replayLoadSummary.textContent =
    states.length === 0
      ? "0 replays"
      : `${loaded}/${states.length} loaded${loading > 0 ? `, ${loading} loading` : ""}${
          failed > 0 ? `, ${failed} failed` : ""
        }`;
  replayLoads.replaceChildren();

  for (const state of states) {
    const replay = manifestReplays.get(state.sourceId);
    const row = document.createElement("div");
    row.className = `replay-load-item ${state.status}`;

    const main = document.createElement("div");
    main.className = "replay-load-main";
    const title = document.createElement("div");
    title.className = "replay-load-title";
    title.textContent = replay?.label || state.sourceId;
    const meta = document.createElement("div");
    meta.className = "replay-load-meta";
    const clipCount = counts.get(state.sourceId) ?? 0;
    meta.textContent = [
      state.sourceId,
      `${clipCount} ${clipCount === 1 ? "clip" : "clips"}`,
      replay?.path,
    ]
      .filter(Boolean)
      .join(" · ");
    main.append(title, meta);

    const status = document.createElement("div");
    status.className = "replay-load-status";
    status.textContent = replayLoadLabel(state);

    const progress = document.createElement("div");
    progress.className = "replay-load-progress";
    const bar = document.createElement("span");
    const fraction = state.status === "loaded" ? 1 : progressFraction(state.progress);
    bar.style.width = `${Math.round((fraction ?? 0) * 100)}%`;
    progress.append(bar);

    row.append(main, status, progress);
    replayLoads.append(row);
  }
}

function reviewAuthHeaders(): Record<string, string> {
  const params = new URLSearchParams(window.location.search);
  const token =
    params.get("reviewToken") ??
    params.get("token") ??
    window.localStorage.getItem("rocket_sense_access_token");
  return token ? { Authorization: `Bearer ${token}` } : {};
}

function activeReviewEndpoint(): string | null {
  const item = reviewPlayer?.getState().item ?? null;
  const meta = candidateMeta(item);
  if (typeof meta.reviewEndpoint === "string" && meta.reviewEndpoint) {
    return meta.reviewEndpoint;
  }
  if (typeof meta.eventId === "string" && meta.eventId) {
    return `/api/v1/mechanics/events/${encodeURIComponent(meta.eventId)}/reviews`;
  }
  return null;
}

async function submitReview(status: "confirmed" | "rejected" | "uncertain") {
  const player = reviewPlayer;
  const item = player?.getState().item ?? null;
  const endpoint = activeReviewEndpoint();
  if (!player || !item || !endpoint) {
    reviewStatus.textContent = "Current item has no review endpoint.";
    reviewStatus.classList.add("error");
    return;
  }

  reviewStatus.textContent = `Submitting ${formatReviewStatus(status)}...`;
  reviewStatus.classList.remove("error");
  const response = await fetch(endpoint, {
    method: "POST",
    headers: {
      "content-type": "application/json",
      ...reviewAuthHeaders(),
    },
    credentials: "same-origin",
    body: JSON.stringify({ status }),
  });
  if (!response.ok) {
    let message = `${response.status} ${response.statusText}`;
    try {
      const body = (await response.json()) as { error?: unknown };
      if (typeof body.error === "string") {
        message = body.error;
      }
    } catch {
      // Keep the HTTP status fallback.
    }
    reviewStatus.textContent = `Review failed: ${message}`;
    reviewStatus.classList.add("error");
    return;
  }

  const meta = candidateMeta(item);
  meta.reviewStatus = status;
  reviewStatus.textContent = `Marked ${formatReviewStatus(status)}.`;
  reviewStatus.classList.remove("error");
  renderState(player.getState());
}

async function loadPlaylistUrl(url: string) {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`Failed to load playlist: ${response.status}`);
  }
  const manifest = parsePlaylistManifest(await response.json());
  await loadPlaylist(manifest, new URL(url, window.location.href).href);
}

function setStatus(message: string, isError = false) {
  statusLine.textContent = message;
  statusLine.classList.toggle("error", isError);
}

function attachCandidatePlayer() {
  if (!reviewPlayer || !followPlayer.checked) {
    return;
  }
  const meta = candidateMeta(reviewPlayer.getState().item);
  const playerId = meta.target?.playerId ?? meta.playerId ?? null;
  if (playerId) {
    reviewPlayer.setAttachedPlayer(playerId);
  }
}

function renderState(state: ReplayPlaylistPlayerState) {
  const item = state.item;
  const meta = candidateMeta(item);
  candidateIndex.textContent =
    state.itemCount > 0 ? `${state.itemIndex + 1} / ${state.itemCount}` : "0 / 0";
  candidateTitle.textContent = item?.label ?? "No candidate selected";
  candidateMechanic.textContent = mechanicLabel(meta);
  candidatePlayer.textContent = meta.playerName
    ? `${meta.playerName}${meta.team ? ` (${meta.team})` : ""}`
    : (meta.playerId ?? "-");
  candidateConfidence.textContent = formatConfidence(meta.confidence);
  candidateReplay.textContent = item?.replay.id ?? "-";
  candidateReason.textContent = meta.reason ?? "-";
  eventJson.textContent = JSON.stringify(meta.event ?? meta.target ?? {}, null, 2);
  reviewStatus.textContent = `Review: ${formatReviewStatus(meta.reviewStatus)}`;
  reviewStatus.classList.remove("error");
  updatePlaylistSelection(state.itemIndex);
  renderReplayLoads(state.replayLoadStates);

  if (!scrubbing) {
    scrubber.value =
      state.duration > 0 ? String(Math.round((state.currentTime / state.duration) * 1000)) : "0";
  }
  timeReadout.textContent = `${formatSeconds(state.currentTime)} / ${formatSeconds(state.duration)}`;
  playButton.textContent = state.playing ? "Pause" : "Play";
  previousButton.disabled =
    state.itemCount === 0 || (state.itemIndex === 0 && state.endMode !== "loop");
  nextButton.disabled =
    state.itemCount === 0 || (state.itemIndex >= state.itemCount - 1 && state.endMode !== "loop");
  confirmButton.disabled = state.itemCount === 0 || activeReviewEndpoint() === null;
  rejectButton.disabled = confirmButton.disabled;
  uncertainButton.disabled = confirmButton.disabled;
  scrubber.disabled = !state.ready || state.duration <= 0;

  if (state.error) {
    setStatus(state.error, true);
  } else if (state.loading) {
    setStatus("Loading replay...");
  } else if (state.ready) {
    setStatus(state.itemEnded ? "Candidate ended." : "Ready.");
  } else {
    setStatus(activeManifest ? "No candidates in playlist." : "Load a playlist to begin.");
  }
}

playlistFile.addEventListener("change", async () => {
  const file = playlistFile.files?.[0];
  if (!file) {
    return;
  }
  try {
    setStatus("Loading playlist...");
    await loadPlaylist(await loadPlaylistManifestFromFile(file), null);
  } catch (error) {
    setStatus(error instanceof Error ? error.message : String(error), true);
  }
});

loadUrlButton.addEventListener("click", async () => {
  const url = playlistUrl.value.trim();
  if (!url) {
    return;
  }
  try {
    setStatus("Loading playlist...");
    await loadPlaylistUrl(url);
  } catch (error) {
    setStatus(error instanceof Error ? error.message : String(error), true);
  }
});

previousButton.addEventListener("click", async () => {
  if (await reviewPlayer?.previous()) {
    attachCandidatePlayer();
  }
});

nextButton.addEventListener("click", async () => {
  if (await reviewPlayer?.next()) {
    attachCandidatePlayer();
  }
});

playButton.addEventListener("click", () => {
  reviewPlayer?.togglePlayback();
});

confirmButton.addEventListener("click", () => {
  void submitReview("confirmed");
});

rejectButton.addEventListener("click", () => {
  void submitReview("rejected");
});

uncertainButton.addEventListener("click", () => {
  void submitReview("uncertain");
});

advanceMode.addEventListener("change", () => {
  reviewPlayer?.setAdvanceMode(advanceMode.value as PlaylistAdvanceMode);
});

endMode.addEventListener("change", () => {
  reviewPlayer?.setEndMode(endMode.value as PlaylistEndMode);
});

speedSelect.addEventListener("change", () => {
  reviewPlayer?.setPlaybackRate(Number(speedSelect.value));
});

followPlayer.addEventListener("change", () => {
  if (followPlayer.checked) {
    attachCandidatePlayer();
  } else {
    reviewPlayer?.setAttachedPlayer(null);
    reviewPlayer?.setCameraViewMode("free");
  }
});

scrubber.addEventListener("input", () => {
  const state = reviewPlayer?.getState();
  if (!reviewPlayer || !state || state.duration <= 0) {
    return;
  }
  scrubbing = true;
  reviewPlayer.seek((Number(scrubber.value) / 1000) * state.duration);
  scrubbing = false;
});

const searchParams = new URLSearchParams(window.location.search);
const urlParam = searchParams.get("playlist") ?? searchParams.get("playlistUrl");
if (urlParam) {
  playlistUrl.value = urlParam;
  void loadPlaylistUrl(urlParam).catch((error) => {
    setStatus(error instanceof Error ? error.message : String(error), true);
  });
}
