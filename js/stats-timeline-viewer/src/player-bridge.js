import { ReplayPlayer } from "./local-player.js";

const CAMERA_MODES = ["overview", "attached", "third-person"];
const PLAYBACK_SPEEDS = [0.25, 0.5, 1, 1.5, 2, 3, 4];

let activePlayer = null;
let activeChangeListener = null;
let controlsBound = false;
let playbackRateIndex = PLAYBACK_SPEEDS.indexOf(1);

export async function createReplayPlayer(replay, onTimeUpdate) {
  detachActivePlayer();

  const container = document.getElementById("player");
  container.replaceChildren();
  document.getElementById("player-container").style.display = "block";
  document.getElementById("details-watch").style.display = "block";

  activePlayer = new ReplayPlayer(container, replay, {
    initialCameraMode: "overview",
    initialTrackedPlayerId: replay.players[0]?.id,
  });

  bindControlsOnce();
  populateTrackedPlayer(replay);

  playbackRateIndex = PLAYBACK_SPEEDS.indexOf(1);
  activePlayer.setPlaybackRate(PLAYBACK_SPEEDS[playbackRateIndex]);

  activeChangeListener = (event) => {
    const snapshot = event.detail;
    syncControls(snapshot);
    onTimeUpdate?.(snapshot.currentTime, snapshot);
  };

  activePlayer.addEventListener("change", activeChangeListener);
  const initialSnapshot = activePlayer.getSnapshot();
  syncControls(initialSnapshot);
  onTimeUpdate?.(initialSnapshot.currentTime, initialSnapshot);
  return activePlayer;
}

function detachActivePlayer() {
  if (activePlayer && activeChangeListener) {
    activePlayer.removeEventListener("change", activeChangeListener);
  }
  if (activePlayer) {
    activePlayer.dispose();
  }
  activePlayer = null;
  activeChangeListener = null;
}

function bindControlsOnce() {
  if (controlsBound) {
    return;
  }

  const playPause = document.getElementById("play-pause");
  const seekbar = document.getElementById("seekbar");
  const cameraSwitcher = document.getElementById("cameraSwitcher");
  const fullScreen = document.getElementById("full-screen");
  const slower = document.getElementById("playback-speed-dn");
  const faster = document.getElementById("playback-speed-up");

  playPause.addEventListener("click", () => {
    activePlayer?.togglePlayback();
  });

  seekbar.addEventListener("input", () => {
    const snapshot = activePlayer?.getSnapshot();
    if (!snapshot || snapshot.duration <= 0) {
      return;
    }
    const nextTime = (Number(seekbar.value) / 1000) * snapshot.duration;
    activePlayer.seek(nextTime);
  });

  cameraSwitcher.addEventListener("click", () => {
    if (!activePlayer) {
      return;
    }
    const { cameraMode } = activePlayer.getSnapshot();
    const index = CAMERA_MODES.indexOf(cameraMode);
    const nextMode = CAMERA_MODES[(index + 1) % CAMERA_MODES.length];
    activePlayer.setCameraMode(nextMode);
  });

  fullScreen.addEventListener("click", async () => {
    const element = document.getElementById("player-container");
    if (!document.fullscreenElement) {
      await element.requestFullscreen?.();
    } else {
      await document.exitFullscreen?.();
    }
  });

  slower.addEventListener("click", () => {
    if (!activePlayer) {
      return;
    }
    playbackRateIndex = Math.max(0, playbackRateIndex - 1);
    activePlayer.setPlaybackRate(PLAYBACK_SPEEDS[playbackRateIndex]);
    syncPlaybackRateLabel();
  });

  faster.addEventListener("click", () => {
    if (!activePlayer) {
      return;
    }
    playbackRateIndex = Math.min(PLAYBACK_SPEEDS.length - 1, playbackRateIndex + 1);
    activePlayer.setPlaybackRate(PLAYBACK_SPEEDS[playbackRateIndex]);
    syncPlaybackRateLabel();
  });

  controlsBound = true;
}

function populateTrackedPlayer(replay) {
  const trackedPlayerId = replay.players[0]?.id ?? null;
  const cameraSwitcher = document.getElementById("cameraSwitcher");

  if (trackedPlayerId) {
    activePlayer.setTrackedPlayer(trackedPlayerId);
    cameraSwitcher.disabled = false;
  } else {
    cameraSwitcher.disabled = true;
  }
}

function syncControls(snapshot) {
  const currentTime = document.getElementById("current-time");
  const totalTime = document.getElementById("total-time");
  const seekbar = document.getElementById("seekbar");
  const playPause = document.getElementById("play-pause");
  const cameraSwitcher = document.getElementById("cameraSwitcher");

  currentTime.textContent = formatClock(snapshot.currentTime);
  totalTime.textContent = formatClock(snapshot.duration);
  seekbar.value =
    snapshot.duration > 0
      ? String(Math.round((snapshot.currentTime / snapshot.duration) * 1000))
      : "0";
  playPause.textContent = snapshot.playing ? "Pause" : "Play";
  cameraSwitcher.textContent = `Camera: ${formatCameraMode(snapshot.cameraMode)}`;
  syncPlaybackRateLabel();
}

function syncPlaybackRateLabel() {
  document.getElementById("playback-speed-value").textContent =
    `${PLAYBACK_SPEEDS[playbackRateIndex].toFixed(2).replace(/\.?0+$/, "")}x`;
}

function formatCameraMode(mode) {
  if (mode === "third-person") {
    return "Third Person";
  }
  return mode.charAt(0).toUpperCase() + mode.slice(1);
}

function formatClock(time) {
  const minutes = Math.floor(time / 60);
  const seconds = Math.floor(time % 60);
  return `${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}`;
}
