import "./styles.css";
import {
  createBallchasingOverlayPlugin,
  ReplayPlaylistPlayer,
  createReplayFileSource,
  loadPlaylistManifestFromFile,
  resolvePlaylistItemsFromManifest,
} from "../../player/src/lib.ts";
import type {
  PlaylistItem,
  PlaylistManifest,
  PlaylistManifestReplay,
  ReplayPlayer,
  ReplayPlaylistPlayerState,
  ReplaySource,
} from "../../player/src/lib.ts";
import { FlipResetOverlay, parseFlipResetClipMeta } from "./overlays.ts";

const app = document.getElementById("app");

if (!app) {
  throw new Error("Missing #app root");
}

app.innerHTML = `
  <main class="shell">
    <section class="hero panel">
      <div class="hero-copy">
        <p class="eyebrow">subtr-actor / mechanics evaluation</p>
        <h1>Manifest-driven replay playlist viewer</h1>
        <p class="lede">
          Load a playlist manifest plus the referenced replay files from disk, then
          step through event windows back to back with eager replay preloading.
        </p>
      </div>
      <div class="ingest-grid">
        <label class="file-picker">
          <span>Playlist Manifest</span>
          <input id="manifest-file" type="file" accept=".json,application/json" />
        </label>
        <label class="file-picker">
          <span>Replay Files</span>
          <input id="replay-files" type="file" accept=".replay" multiple />
        </label>
        <button id="load-playlist" disabled>Load Playlist</button>
      </div>
    </section>
    <section class="workspace">
      <div class="viewport-panel panel">
        <div id="viewport" class="viewport"></div>
        <div id="empty-state" class="empty-state">
          Choose a manifest and the replay files it references.
        </div>
      </div>
      <aside class="sidebar">
        <div class="panel">
          <h2>Playlist</h2>
          <div class="transport-row">
            <button id="previous-item" disabled>Previous</button>
            <button id="next-item" disabled>Next</button>
          </div>
          <div class="stat-grid compact-grid">
            <div>
              <span class="label">Manifest</span>
              <strong id="manifest-readout">--</strong>
            </div>
            <div>
              <span class="label">Clip</span>
              <strong id="clip-readout">0 / 0</strong>
            </div>
          </div>
          <div class="current-item">
            <span class="label">Current Label</span>
            <strong id="item-label">--</strong>
          </div>
          <ol id="playlist-items" class="playlist-list"></ol>
        </div>
        <div class="panel">
          <h2>Transport</h2>
          <div class="transport-row">
            <button id="toggle-playback" disabled>Play</button>
            <select id="playback-rate" disabled>
              <option value="0.25">0.25x</option>
              <option value="0.5">0.5x</option>
              <option value="1" selected>1.0x</option>
              <option value="1.5">1.5x</option>
              <option value="2">2.0x</option>
            </select>
          </div>
          <input id="timeline" type="range" min="0" max="0" step="0.01" value="0" disabled />
          <div class="stat-grid">
            <div>
              <span class="label">Clip Time</span>
              <strong id="time-readout">0.00s</strong>
            </div>
            <div>
              <span class="label">Clip Duration</span>
              <strong id="duration-readout">0.00s</strong>
            </div>
            <div>
              <span class="label">Replay Time</span>
              <strong id="replay-time-readout">0.00s</strong>
            </div>
            <div>
              <span class="label">Frame</span>
              <strong id="frame-readout">0</strong>
            </div>
          </div>
        </div>
        <div class="panel">
          <h2>Camera</h2>
          <div class="transport-row">
            <select id="attached-player" disabled>
              <option value="">Free camera</option>
            </select>
          </div>
          <label>
            <span class="label">Follow Distance</span>
            <input
              id="camera-distance"
              type="range"
              min="0.75"
              max="4"
              step="0.05"
              value="2.25"
              disabled
            />
          </label>
          <strong id="camera-distance-readout" class="control-readout">2.25x</strong>
          <label class="toggle">
            <input id="ball-cam" type="checkbox" disabled />
            <span>Ball cam</span>
          </label>
          <label class="toggle">
            <input id="skip-kickoffs" type="checkbox" checked />
            <span>Skip kickoffs</span>
          </label>
        </div>
        <div class="panel">
          <h2>Status</h2>
          <div class="stat-row">
            <span class="label">State</span>
            <span class="value" id="status-readout">Waiting for files</span>
          </div>
          <div class="stat-row">
            <span class="label">Replay Source</span>
            <span class="value truncate" id="replay-readout">--</span>
          </div>
          <div class="stat-row">
            <span class="label">Players</span>
            <span class="value truncate" id="players-readout">--</span>
          </div>
        </div>
        <div class="panel">
          <h2>Event</h2>
          <div class="stat-row">
            <span class="label">Player</span>
            <span class="value truncate" id="event-player-readout">--</span>
          </div>
          <div class="stat-row">
            <span class="label">Event Time</span>
            <span class="value" id="event-time-readout">--</span>
          </div>
          <div class="stat-row">
            <span class="label">Indicator</span>
            <span class="value" id="event-indicator-readout">--</span>
          </div>
        </div>
      </aside>
    </section>
  </main>
`;

type ReplayFileIndex = {
  exact: Map<string, File[]>;
  byName: Map<string, File[]>;
};

const DEFAULT_CLIP_DURATION_SECONDS = 6;

const manifestFileInput = mustElement<HTMLInputElement>("#manifest-file");
const replayFilesInput = mustElement<HTMLInputElement>("#replay-files");
const loadPlaylistButton = mustElement<HTMLButtonElement>("#load-playlist");
const viewport = mustElement<HTMLDivElement>("#viewport");
const emptyState = mustElement<HTMLDivElement>("#empty-state");
const previousItemButton = mustElement<HTMLButtonElement>("#previous-item");
const nextItemButton = mustElement<HTMLButtonElement>("#next-item");
const togglePlaybackButton = mustElement<HTMLButtonElement>("#toggle-playback");
const playbackRateSelect = mustElement<HTMLSelectElement>("#playback-rate");
const timeline = mustElement<HTMLInputElement>("#timeline");
const manifestReadout = mustElement<HTMLElement>("#manifest-readout");
const clipReadout = mustElement<HTMLElement>("#clip-readout");
const itemLabel = mustElement<HTMLElement>("#item-label");
const playlistItemsList = mustElement<HTMLOListElement>("#playlist-items");
const timeReadout = mustElement<HTMLElement>("#time-readout");
const durationReadout = mustElement<HTMLElement>("#duration-readout");
const replayTimeReadout = mustElement<HTMLElement>("#replay-time-readout");
const frameReadout = mustElement<HTMLElement>("#frame-readout");
const attachedPlayerSelect = mustElement<HTMLSelectElement>("#attached-player");
const cameraDistanceInput = mustElement<HTMLInputElement>("#camera-distance");
const cameraDistanceReadout = mustElement<HTMLElement>("#camera-distance-readout");
const ballCamInput = mustElement<HTMLInputElement>("#ball-cam");
const skipKickoffsInput = mustElement<HTMLInputElement>("#skip-kickoffs");
const statusReadout = mustElement<HTMLElement>("#status-readout");
const replayReadout = mustElement<HTMLElement>("#replay-readout");
const playersReadout = mustElement<HTMLElement>("#players-readout");
const eventPlayerReadout = mustElement<HTMLElement>("#event-player-readout");
const eventTimeReadout = mustElement<HTMLElement>("#event-time-readout");
const eventIndicatorReadout = mustElement<HTMLElement>("#event-indicator-readout");

let currentManifest: PlaylistManifest | null = null;
let currentPlaylistItems: PlaylistItem[] = [];
let replayPlaylistPlayer: ReplayPlaylistPlayer | null = null;
let unsubscribe: (() => void) | null = null;
let flipResetOverlay: FlipResetOverlay | null = null;
let flipResetOverlayPlayer: ReplayPlayer | null = null;
let flipResetOverlayItemIndex: number | null = null;

function mustElement<T extends Element>(selector: string): T {
  const element = document.querySelector(selector);
  if (!element) {
    throw new Error(`Missing element for selector: ${selector}`);
  }

  return element as T;
}

function setInteractiveState(enabled: boolean): void {
  previousItemButton.disabled = !enabled;
  nextItemButton.disabled = !enabled;
  togglePlaybackButton.disabled = !enabled;
  playbackRateSelect.disabled = !enabled;
  timeline.disabled = !enabled;
  attachedPlayerSelect.disabled = !enabled;
  cameraDistanceInput.disabled = !enabled;
  ballCamInput.disabled = !enabled;
}

function updateLoadButtonState(): void {
  loadPlaylistButton.disabled =
    !manifestFileInput.files?.[0] || !replayFilesInput.files?.length;
}

function basename(path: string): string {
  const normalized = path.replaceAll("\\", "/");
  const segments = normalized.split("/");
  return segments[segments.length - 1] ?? normalized;
}

function buildReplayFileIndex(files: Iterable<File>): ReplayFileIndex {
  const exact = new Map<string, File[]>();
  const byName = new Map<string, File[]>();

  for (const file of files) {
    const exactKeys = new Set<string>([file.name]);
    if (file.webkitRelativePath) {
      exactKeys.add(file.webkitRelativePath);
    }
    for (const key of exactKeys) {
      const existingExactMatches = exact.get(key) ?? [];
      existingExactMatches.push(file);
      exact.set(key, existingExactMatches);
    }

    const existing = byName.get(file.name) ?? [];
    existing.push(file);
    byName.set(file.name, existing);
  }

  return { exact, byName };
}

function resolveReplayFile(
  replayId: string,
  replay: PlaylistManifestReplay | undefined,
  replayFiles: ReplayFileIndex
): File {
  const candidates = [replay?.path, replayId].filter(
    (value): value is string => typeof value === "string" && value.trim() !== ""
  );

  for (const candidate of candidates) {
    const exactMatches = replayFiles.exact.get(candidate);
    if (exactMatches?.length === 1) {
      return exactMatches[0];
    }
    if (exactMatches && exactMatches.length > 1) {
      throw new Error(
        `Multiple replay files match "${candidate}". Use unique file names or manifest paths.`
      );
    }
  }

  for (const candidate of candidates) {
    const fileNameMatches = replayFiles.byName.get(basename(candidate));
    if (fileNameMatches?.length === 1) {
      return fileNameMatches[0];
    }
    if (fileNameMatches && fileNameMatches.length > 1) {
      throw new Error(
        `Multiple replay files match "${candidate}". Use unique file names or manifest paths.`
      );
    }
  }

  throw new Error(`No replay file selected for manifest replay "${replayId}"`);
}

function buildPlaylistItems(
  manifest: PlaylistManifest,
  replayFiles: FileList
): PlaylistItem[] {
  const replayFileIndex = buildReplayFileIndex(replayFiles);
  const sourceCache = new Map<string, ReplaySource>();

  return resolvePlaylistItemsFromManifest(
    manifest,
    ({ replayId, replay }) => {
      const sourceId = replay?.id ?? replayId;
      const cached = sourceCache.get(sourceId);
      if (cached) {
        return cached;
      }

      const file = resolveReplayFile(replayId, replay, replayFileIndex);
      const source = createReplayFileSource(file, sourceId);
      sourceCache.set(sourceId, source);
      return source;
    }
  );
}

function destroyPlaylistPlayer(): void {
  flipResetOverlay?.dispose();
  flipResetOverlay = null;
  flipResetOverlayPlayer = null;
  flipResetOverlayItemIndex = null;
  unsubscribe?.();
  unsubscribe = null;
  replayPlaylistPlayer?.destroy();
  replayPlaylistPlayer = null;
}

function syncFlipResetOverlay(state: ReplayPlaylistPlayerState): void {
  const currentPlayer = replayPlaylistPlayer?.getCurrentPlayer() ?? null;
  const meta = parseFlipResetClipMeta(state.item?.meta);

  if (!currentPlayer || !state.item || !meta) {
    flipResetOverlay?.dispose();
    flipResetOverlay = null;
    flipResetOverlayPlayer = null;
    flipResetOverlayItemIndex = null;
    return;
  }

  if (
    flipResetOverlay &&
    flipResetOverlayPlayer === currentPlayer &&
    flipResetOverlayItemIndex === state.itemIndex
  ) {
    return;
  }

  flipResetOverlay?.dispose();
  flipResetOverlay = new FlipResetOverlay(currentPlayer, meta);
  flipResetOverlayPlayer = currentPlayer;
  flipResetOverlayItemIndex = state.itemIndex;
}

function populateAttachedPlayerOptions(): void {
  const replay = replayPlaylistPlayer?.getCurrentReplay()?.replay;
  const selectedValue =
    replayPlaylistPlayer?.getSnapshot().attachedPlayerId ?? "";

  attachedPlayerSelect.replaceChildren();
  attachedPlayerSelect.append(new Option("Free camera", ""));

  for (const player of replay?.players ?? []) {
    attachedPlayerSelect.append(
      new Option(
        `${player.name} (${player.isTeamZero ? "Blue" : "Orange"})`,
        player.id
      )
    );
  }

  attachedPlayerSelect.value = selectedValue;
}

function renderPlaylistList(state: ReplayPlaylistPlayerState): void {
  playlistItemsList.replaceChildren();

  for (const [index, item] of currentPlaylistItems.entries()) {
    const listItem = document.createElement("li");
    listItem.className = "playlist-entry";
    if (index === state.itemIndex) {
      listItem.dataset.active = "true";
    }

    const button = document.createElement("button");
    button.type = "button";
    button.className = "playlist-entry-button";
    button.disabled = replayPlaylistPlayer === null;
    button.innerHTML = `
      <strong>${item.label ?? `Clip ${index + 1}`}</strong>
      <span>${item.replay.id}</span>
    `;
    button.addEventListener("click", async () => {
      try {
        await replayPlaylistPlayer?.setCurrentItemIndex(index);
      } catch (error) {
        statusReadout.textContent =
          error instanceof Error ? error.message : "Failed to switch clip";
      }
    });

    listItem.append(button);
    playlistItemsList.append(listItem);
  }
}

function renderState(state: ReplayPlaylistPlayerState): void {
  const loadedReplay = replayPlaylistPlayer?.getCurrentReplay()?.replay ?? null;
  const eventMeta = parseFlipResetClipMeta(state.item?.meta);

  manifestReadout.textContent =
    currentManifest?.label ?? manifestFileInput.files?.[0]?.name ?? "--";
  clipReadout.textContent = `${
    state.itemCount === 0 ? 0 : state.itemIndex + 1
  } / ${state.itemCount}`;
  itemLabel.textContent = state.item?.label ?? "--";
  timeReadout.textContent = `${state.currentTime.toFixed(2)}s`;
  durationReadout.textContent = `${state.duration.toFixed(2)}s`;
  replayTimeReadout.textContent = `${state.replayCurrentTime.toFixed(2)}s`;
  frameReadout.textContent = `${state.frameIndex}`;
  timeline.max = `${state.duration}`;
  timeline.value = `${Math.min(state.currentTime, state.duration)}`;
  togglePlaybackButton.textContent = state.playing ? "Pause" : "Play";
  previousItemButton.disabled = !state.ready || state.itemIndex <= 0 || state.loading;
  nextItemButton.disabled =
    !state.ready || state.itemIndex >= state.itemCount - 1 || state.loading;
  playbackRateSelect.disabled = !state.ready;
  timeline.disabled = !state.ready;
  attachedPlayerSelect.disabled = !state.ready;
  cameraDistanceInput.disabled = !state.ready || state.attachedPlayerId === null;
  ballCamInput.disabled = !state.ready || state.attachedPlayerId === null;
  cameraDistanceInput.value = `${state.cameraDistanceScale}`;
  cameraDistanceReadout.textContent = `${state.cameraDistanceScale.toFixed(2)}x`;
  ballCamInput.checked = state.ballCamEnabled;
  skipKickoffsInput.checked = state.skipKickoffsEnabled;
  replayReadout.textContent = state.item?.replay.id ?? "--";
  playersReadout.textContent =
    loadedReplay?.players.map((player) => player.name).join(", ") ?? "--";
  eventPlayerReadout.textContent = eventMeta?.playerName ?? eventMeta?.playerId ?? "--";
  eventTimeReadout.textContent =
    eventMeta?.eventTime !== undefined ? `${eventMeta.eventTime.toFixed(2)}s` : "--";
  eventIndicatorReadout.textContent = eventMeta?.markerPosition
    ? "Marker + player ring"
    : eventMeta?.playerId
      ? "Player ring"
      : "--";

  if (state.loading) {
    statusReadout.textContent = "Loading replay...";
  } else if (state.error) {
    statusReadout.textContent = state.error;
  } else if (state.ready) {
    statusReadout.textContent = "Ready";
  } else {
    statusReadout.textContent = "Waiting for playlist";
  }

  if (loadedReplay) {
    populateAttachedPlayerOptions();
  }
  attachedPlayerSelect.value = state.attachedPlayerId ?? "";

  syncFlipResetOverlay(state);
  renderPlaylistList(state);
}

async function loadPlaylistFromDisk(): Promise<void> {
  const manifestFile = manifestFileInput.files?.[0];
  const replayFiles = replayFilesInput.files;

  if (!manifestFile || !replayFiles?.length) {
    return;
  }

  setInteractiveState(false);
  statusReadout.textContent = "Parsing manifest...";
  emptyState.hidden = false;

  destroyPlaylistPlayer();

  currentManifest = await loadPlaylistManifestFromFile(manifestFile);
  currentPlaylistItems = buildPlaylistItems(currentManifest, replayFiles);

  if (currentPlaylistItems.length === 0) {
    throw new Error("Manifest contains no playlist items");
  }

  replayPlaylistPlayer = new ReplayPlaylistPlayer(viewport, currentPlaylistItems, {
    autoplay: false,
    preloadPolicy: { kind: "all" },
    initialSkipKickoffsEnabled: skipKickoffsInput.checked,
    initialCameraDistanceScale: Number(cameraDistanceInput.value),
    plugins: [createBallchasingOverlayPlugin()],
  });
  unsubscribe = replayPlaylistPlayer.subscribe(renderState);
  await replayPlaylistPlayer.waitForCurrentItem();

  emptyState.hidden = true;
  setInteractiveState(true);
}

manifestFileInput.addEventListener("change", updateLoadButtonState);
replayFilesInput.addEventListener("change", updateLoadButtonState);

loadPlaylistButton.addEventListener("click", async () => {
  try {
    await loadPlaylistFromDisk();
  } catch (error) {
    statusReadout.textContent =
      error instanceof Error ? error.message : "Failed to load playlist";
    emptyState.hidden = false;
  }
});

previousItemButton.addEventListener("click", async () => {
  try {
    await replayPlaylistPlayer?.previous();
  } catch (error) {
    statusReadout.textContent =
      error instanceof Error ? error.message : "Failed to load previous clip";
  }
});

nextItemButton.addEventListener("click", async () => {
  try {
    await replayPlaylistPlayer?.next();
  } catch (error) {
    statusReadout.textContent =
      error instanceof Error ? error.message : "Failed to load next clip";
  }
});

togglePlaybackButton.addEventListener("click", () => {
  replayPlaylistPlayer?.togglePlayback();
});

playbackRateSelect.addEventListener("change", () => {
  replayPlaylistPlayer?.setPlaybackRate(Number(playbackRateSelect.value));
});

timeline.addEventListener("input", () => {
  replayPlaylistPlayer?.seek(Number(timeline.value));
});

attachedPlayerSelect.addEventListener("change", () => {
  replayPlaylistPlayer?.setAttachedPlayer(attachedPlayerSelect.value || null);
});

cameraDistanceInput.addEventListener("input", () => {
  replayPlaylistPlayer?.setCameraDistanceScale(Number(cameraDistanceInput.value));
});

ballCamInput.addEventListener("change", () => {
  replayPlaylistPlayer?.setBallCamEnabled(ballCamInput.checked);
});

skipKickoffsInput.addEventListener("change", () => {
  replayPlaylistPlayer?.setSkipKickoffsEnabled(skipKickoffsInput.checked);
});

renderState({
  ready: false,
  loading: false,
  error: null,
  itemIndex: 0,
  itemCount: 0,
  item: null,
  currentTime: 0,
  duration: DEFAULT_CLIP_DURATION_SECONDS,
  replayCurrentTime: 0,
  replayDuration: 0,
  frameIndex: 0,
  activeMetadata: null,
  playing: false,
  speed: Number(playbackRateSelect.value),
  cameraDistanceScale: Number(cameraDistanceInput.value),
  attachedPlayerId: null,
  ballCamEnabled: false,
  skipPostGoalTransitionsEnabled: true,
  skipKickoffsEnabled: skipKickoffsInput.checked,
});
