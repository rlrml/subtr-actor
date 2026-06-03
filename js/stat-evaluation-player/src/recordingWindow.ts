import type { CanvasRecorderPlugin, CanvasRecorderStatus, ReplayPlayer } from "@rlrml/player";
import type { RecordingConfig } from "./playerConfig.ts";

export interface RecordingWindowElements {
  readonly fps: HTMLInputElement;
  readonly playbackRate: HTMLSelectElement;
  readonly start: HTMLButtonElement;
  readonly fullReplay: HTMLButtonElement;
  readonly stop: HTMLButtonElement;
  readonly download: HTMLButtonElement;
  readonly clear: HTMLButtonElement;
  readonly status: HTMLElement;
  readonly elapsed: HTMLElement;
  readonly size: HTMLElement;
  readonly type: HTMLElement;
}

export interface RecordingWindowOptions {
  readonly elements: RecordingWindowElements;
  getCanvasRecorder(): CanvasRecorderPlugin | null;
  getReplayPlayer(): ReplayPlayer | null;
  getLoadedReplayName(): string | null;
  setStatus(message: string): void;
  requestConfigSync(): void;
}

function formatBytes(bytes: number): string {
  if (bytes <= 0) {
    return "--";
  }
  const units = ["B", "KB", "MB", "GB"];
  let value = bytes;
  let unitIndex = 0;
  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024;
    unitIndex += 1;
  }
  const precision = unitIndex === 0 ? 0 : value >= 10 ? 1 : 2;
  return `${value.toFixed(precision)} ${units[unitIndex]}`;
}

function recordingLabel(status: CanvasRecorderStatus | null): string {
  if (!status) {
    return "No replay";
  }
  if (status.error) {
    return status.error;
  }
  switch (status.state) {
    case "idle":
      return "Idle";
    case "recording":
      return "Recording";
    case "stopping":
      return "Stopping";
    case "ready":
      return "Ready";
    case "error":
      return "Error";
  }
}

function getRecordingOptions({
  fpsValue,
  playbackRateValue,
}: {
  fpsValue: string;
  playbackRateValue: string;
}): { fps: number; playbackRate: number } {
  const fps = Number(fpsValue);
  const playbackRate = Number(playbackRateValue);
  return {
    fps: Number.isFinite(fps) ? Math.max(1, Math.min(120, Math.trunc(fps))) : 60,
    playbackRate: Number.isFinite(playbackRate) ? Math.max(0.1, playbackRate) : 1,
  };
}

function recordingFileName(sourceName: string | null, now = new Date()): string {
  const source = sourceName?.replace(/\.replay$/i, "") || "replay";
  const safeSource = source.replace(/[^a-zA-Z0-9._-]+/g, "-").replace(/^-+|-+$/g, "");
  const timestamp = now.toISOString().replace(/[:.]/g, "-");
  return `${safeSource || "replay"}-${timestamp}.webm`;
}

function downloadRecording(blob: Blob, fileName: string): void {
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = fileName;
  document.body.append(link);
  link.click();
  link.remove();
  window.setTimeout(() => URL.revokeObjectURL(url), 0);
}

export class RecordingWindowController {
  constructor(private readonly options: RecordingWindowOptions) {}

  getConfigSnapshot(): RecordingConfig {
    const { elements } = this.options;
    return {
      fps: Number(elements.fps.value),
      playbackRate: Number(elements.playbackRate.value),
    };
  }

  applyConfig(config: RecordingConfig): void {
    const { elements } = this.options;
    if (config.fps !== undefined) {
      elements.fps.value = `${config.fps}`;
    }
    if (config.playbackRate !== undefined) {
      elements.playbackRate.value = `${config.playbackRate}`;
    }
  }

  sync(status = this.options.getCanvasRecorder()?.getStatus() ?? null): void {
    const { elements } = this.options;
    const hasRecorder =
      this.options.getCanvasRecorder() !== null && this.options.getReplayPlayer() !== null;
    const state = status?.state ?? "idle";
    const isRecording = state === "recording" || state === "stopping";
    const hasRecording = (this.options.getCanvasRecorder()?.getRecording() ?? null) !== null;

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

  installEventListeners(signal: AbortSignal): void {
    const { elements } = this.options;
    elements.start.addEventListener(
      "click",
      () => {
        const canvasRecorder = this.options.getCanvasRecorder();
        if (!canvasRecorder) {
          return;
        }
        try {
          const { fps } = this.getRecordingOptions();
          canvasRecorder.start({ fps });
          this.sync();
        } catch (error) {
          console.error("Failed to start recording:", error);
          this.options.setStatus(
            error instanceof Error ? error.message : "Failed to start recording",
          );
          this.sync(canvasRecorder.getStatus());
        }
      },
      { signal },
    );

    elements.fullReplay.addEventListener(
      "click",
      () => {
        const canvasRecorder = this.options.getCanvasRecorder();
        if (!canvasRecorder) {
          return;
        }
        const { fps, playbackRate } = this.getRecordingOptions();
        void canvasRecorder
          .recordFullReplay({
            fps,
            playbackRate,
            restorePlaybackState: true,
          })
          .catch((error) => {
            console.error("Failed to record replay:", error);
            this.options.setStatus(
              error instanceof Error ? error.message : "Failed to record replay",
            );
            this.sync(this.options.getCanvasRecorder()?.getStatus() ?? null);
          });
        this.sync();
      },
      { signal },
    );

    elements.stop.addEventListener(
      "click",
      () => {
        void this.options
          .getCanvasRecorder()
          ?.stop()
          .catch((error) => {
            console.error("Failed to stop recording:", error);
            this.options.setStatus(
              error instanceof Error ? error.message : "Failed to stop recording",
            );
          });
        this.sync();
      },
      { signal },
    );

    elements.download.addEventListener(
      "click",
      () => {
        const blob = this.options.getCanvasRecorder()?.getRecording();
        if (blob) {
          downloadRecording(blob, recordingFileName(this.options.getLoadedReplayName()));
        }
      },
      { signal },
    );

    elements.clear.addEventListener(
      "click",
      () => {
        try {
          this.options.getCanvasRecorder()?.clear();
          this.sync();
        } catch (error) {
          console.error("Failed to clear recording:", error);
        }
      },
      { signal },
    );

    elements.fps.addEventListener("change", this.options.requestConfigSync, { signal });
    elements.playbackRate.addEventListener("change", this.options.requestConfigSync, { signal });
  }

  private getRecordingOptions(): { fps: number; playbackRate: number } {
    const { elements } = this.options;
    return getRecordingOptions({
      fpsValue: elements.fps.value,
      playbackRateValue: elements.playbackRate.value,
    });
  }
}

export function createRecordingWindowController(
  options: RecordingWindowOptions,
): RecordingWindowController {
  return new RecordingWindowController(options);
}
