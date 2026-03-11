import { createReplayPlayer } from "./player-bridge.js";
import { adaptReplayDataForViewer, normalizeStatsTimeline } from "./replay-adapter.js";
import { StatsPanel } from "./stats-panel.js";
import { initializeWasm, loadReplayArtifacts } from "./wasm-api.js";

export class StatsTimelineViewerApp {
  constructor(root) {
    this.root = root;
    this.uploadArea = document.getElementById("uploadArea");
    this.fileInput = document.getElementById("fileInput");
    this.uploadCard = document.getElementById("uploadCard");
    this.workspace = document.getElementById("workspace");
    this.progressCard = document.getElementById("progressCard");
    this.progressFill = document.getElementById("progressFill");
    this.progressLabel = document.getElementById("progressLabel");
    this.errorBanner = document.getElementById("errorBanner");
    this.viewerMeta = document.getElementById("viewerMeta");
    this.statsPanel = new StatsPanel(document.getElementById("statsPanel"));
  }

  async initialize() {
    this.bindEvents();
    this.setProgress(5, "Loading WebAssembly runtime…");
    await initializeWasm();
    this.hideProgress();
  }

  bindEvents() {
    this.fileInput.addEventListener("change", (event) => {
      const [file] = event.target.files ?? [];
      if (file) {
        this.handleReplayFile(file);
      }
    });

    this.uploadArea.addEventListener("dragover", (event) => {
      event.preventDefault();
      this.uploadArea.classList.add("is-dragover");
    });

    this.uploadArea.addEventListener("dragleave", () => {
      this.uploadArea.classList.remove("is-dragover");
    });

    this.uploadArea.addEventListener("drop", (event) => {
      event.preventDefault();
      this.uploadArea.classList.remove("is-dragover");
      const [file] = event.dataTransfer?.files ?? [];
      if (file) {
        this.handleReplayFile(file);
      }
    });
  }

  async handleReplayFile(file) {
    this.clearError();

    if (!file.name.endsWith(".replay")) {
      this.showError("Expected a Rocket League .replay file.");
      return;
    }

    try {
      this.setProgress(20, "Reading replay bytes…");
      const replayBytes = new Uint8Array(await file.arrayBuffer());

      this.setProgress(45, "Extracting structured frame data…");
      const { info, frameData, statsTimeline } = await loadReplayArtifacts(replayBytes);
      const startTime = frameData.frame_data.metadata_frames?.[0]?.time
        ?? statsTimeline.frames?.[0]?.time
        ?? 0;
      const normalizedStatsTimeline = normalizeStatsTimeline(statsTimeline, startTime);

      this.setProgress(75, "Adapting replay for the viewer…");
      const replayData = adaptReplayDataForViewer(frameData, statsTimeline);

      this.setProgress(90, "Initializing 3D playback and stats panel…");
      this.uploadCard.hidden = true;
      this.workspace.hidden = false;
      this.populateViewerMeta(file, info, normalizedStatsTimeline);
      this.statsPanel.setTimeline(normalizedStatsTimeline);
      await createReplayPlayer(replayData, (time) => this.statsPanel.updateTime(time));

      this.setProgress(100, "Ready.");
      this.hideProgress();
    } catch (error) {
      console.error("Failed to handle replay file:", error);
      this.showError(error instanceof Error ? error.message : String(error));
      this.hideProgress();
    }
  }

  populateViewerMeta(file, info, statsTimeline) {
    const playerCount = statsTimeline.replay_meta?.player_order?.length
      ?? (statsTimeline.replay_meta?.team_zero?.length ?? 0) + (statsTimeline.replay_meta?.team_one?.length ?? 0);
    const duration = statsTimeline.frames?.at(-1)?.time ?? 0;

    this.viewerMeta.innerHTML = `
      <div class="meta-chip">
        <span class="meta-label">Replay</span>
        <strong>${escapeHtml(file.name)}</strong>
      </div>
      <div class="meta-chip">
        <span class="meta-label">Duration</span>
        <strong>${formatClock(duration)}</strong>
      </div>
      <div class="meta-chip">
        <span class="meta-label">Players</span>
        <strong>${playerCount}</strong>
      </div>
      <div class="meta-chip">
        <span class="meta-label">Version</span>
        <strong>${info.major_version}.${info.minor_version}</strong>
      </div>
    `;
  }

  setProgress(percent, label) {
    this.progressCard.hidden = false;
    this.progressFill.style.width = `${percent}%`;
    this.progressLabel.textContent = label;
  }

  hideProgress() {
    this.progressCard.hidden = true;
    this.progressFill.style.width = "0%";
  }

  showError(message) {
    this.errorBanner.hidden = false;
    this.errorBanner.textContent = message;
  }

  clearError() {
    this.errorBanner.hidden = true;
    this.errorBanner.textContent = "";
  }
}

function formatClock(time) {
  const minutes = Math.floor(time / 60);
  const seconds = Math.floor(time % 60);
  return `${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}`;
}

function escapeHtml(value) {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");
}
