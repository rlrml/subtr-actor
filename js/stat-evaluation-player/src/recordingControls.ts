import type { CanvasRecorderPlugin, CanvasRecorderStatus } from "@rlrml/player";
import { mustElement } from "./floatingWindows.ts";
import type { RecordingConfig } from "./playerConfig.ts";
import {
  downloadRecording,
  formatBytes,
  getRecordingOptions,
  recordingFileName,
  recordingLabel,
} from "./recordingControlHelpers.ts";

export interface RecordingControlElements {
  fps: HTMLInputElement;
  playbackRate: HTMLSelectElement;
  start: HTMLButtonElement;
  fullReplay: HTMLButtonElement;
  stop: HTMLButtonElement;
  download: HTMLButtonElement;
  clear: HTMLButtonElement;
  status: HTMLElement;
  elapsed: HTMLElement;
  size: HTMLElement;
  type: HTMLElement;
}

export interface RecordingControlsOptions {
  elements: RecordingControlElements;
  getRecorder: () => CanvasRecorderPlugin | null;
  getLoadedReplayName: () => string | null;
  hasReplayPlayer: () => boolean;
  scheduleConfigUrlUpdate: () => void;
  setStatus: (message: string) => void;
}

export interface RecordingControls {
  applyConfig(config: RecordingConfig): void;
  getConfigSnapshot(): RecordingConfig;
  getOptions(): { fps: number; playbackRate: number };
  installListeners(signal: AbortSignal): void;
  sync(status?: CanvasRecorderStatus | null): void;
}

export function getRecordingControlElements(root: ParentNode): RecordingControlElements {
  return {
    fps: mustElement<HTMLInputElement>(root, "#recording-fps"),
    playbackRate: mustElement<HTMLSelectElement>(root, "#recording-playback-rate"),
    start: mustElement<HTMLButtonElement>(root, "#recording-start"),
    fullReplay: mustElement<HTMLButtonElement>(root, "#recording-full-replay"),
    stop: mustElement<HTMLButtonElement>(root, "#recording-stop"),
    download: mustElement<HTMLButtonElement>(root, "#recording-download"),
    clear: mustElement<HTMLButtonElement>(root, "#recording-clear"),
    status: mustElement<HTMLElement>(root, "#recording-status"),
    elapsed: mustElement<HTMLElement>(root, "#recording-elapsed"),
    size: mustElement<HTMLElement>(root, "#recording-size"),
    type: mustElement<HTMLElement>(root, "#recording-type"),
  };
}

export function createRecordingControls(options: RecordingControlsOptions): RecordingControls {
  const { elements } = options;

  function readOptions(): { fps: number; playbackRate: number } {
    return getRecordingOptions({
      fps: elements.fps,
      playbackRate: elements.playbackRate,
    });
  }

  function sync(status = options.getRecorder()?.getStatus() ?? null): void {
    const recorder = options.getRecorder();
    const hasRecorder = recorder !== null && options.hasReplayPlayer();
    const state = status?.state ?? "idle";
    const isRecording = state === "recording" || state === "stopping";
    const hasRecording = (recorder?.getRecording() ?? null) !== null;

    elements.status.textContent = recordingLabel(status);
    elements.elapsed.textContent = `${(status?.elapsedSeconds ?? 0).toFixed(1)}s`;
    elements.size.textContent = formatBytes(status?.sizeBytes ?? 0);
    elements.type.textContent = status?.mimeType || "WebM";
    elements.start.disabled = !hasRecorder || isRecording;
    elements.fullReplay.disabled = !hasRecorder || isRecording;
    elements.stop.disabled = !hasRecorder || !isRecording;
    elements.download.disabled = !hasRecording || isRecording;
    elements.clear.disabled = !hasRecording || isRecording;
    elements.fps.disabled = isRecording;
    elements.playbackRate.disabled = isRecording;
  }

  return {
    applyConfig(config) {
      if (config.fps !== undefined) {
        elements.fps.value = `${config.fps}`;
      }
      if (config.playbackRate !== undefined) {
        elements.playbackRate.value = `${config.playbackRate}`;
      }
    },
    getConfigSnapshot() {
      return {
        fps: Number(elements.fps.value),
        playbackRate: Number(elements.playbackRate.value),
      };
    },
    getOptions: readOptions,
    installListeners(signal) {
      elements.start.addEventListener(
        "click",
        () => {
          const recorder = options.getRecorder();
          if (!recorder) {
            return;
          }
          try {
            const { fps } = readOptions();
            recorder.start({ fps });
            sync();
          } catch (error) {
            console.error("Failed to start recording:", error);
            options.setStatus(error instanceof Error ? error.message : "Failed to start recording");
            sync(recorder.getStatus());
          }
        },
        { signal },
      );

      elements.fullReplay.addEventListener(
        "click",
        () => {
          const recorder = options.getRecorder();
          if (!recorder) {
            return;
          }
          const { fps, playbackRate } = readOptions();
          void recorder
            .recordFullReplay({
              fps,
              playbackRate,
              restorePlaybackState: true,
            })
            .catch((error) => {
              console.error("Failed to record replay:", error);
              options.setStatus(
                error instanceof Error ? error.message : "Failed to record replay",
              );
              sync(options.getRecorder()?.getStatus() ?? null);
            });
          sync();
        },
        { signal },
      );

      elements.stop.addEventListener(
        "click",
        () => {
          void options
            .getRecorder()
            ?.stop()
            .catch((error) => {
              console.error("Failed to stop recording:", error);
              options.setStatus(
                error instanceof Error ? error.message : "Failed to stop recording",
              );
            });
          sync();
        },
        { signal },
      );

      elements.download.addEventListener(
        "click",
        () => {
          const blob = options.getRecorder()?.getRecording();
          if (blob) {
            downloadRecording(blob, recordingFileName(options.getLoadedReplayName()));
          }
        },
        { signal },
      );

      elements.clear.addEventListener(
        "click",
        () => {
          try {
            options.getRecorder()?.clear();
            sync();
          } catch (error) {
            console.error("Failed to clear recording:", error);
          }
        },
        { signal },
      );

      elements.fps.addEventListener("change", options.scheduleConfigUrlUpdate, { signal });
      elements.playbackRate.addEventListener("change", options.scheduleConfigUrlUpdate, {
        signal,
      });
    },
    sync,
  };
}
