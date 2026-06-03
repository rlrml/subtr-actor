import type { CanvasRecorderPlugin, CanvasRecorderStatus, ReplayPlayer } from "@rlrml/player";
import type { RecordingConfig } from "./playerConfig.ts";
import {
  downloadRecording,
  formatBytes,
  getRecordingOptions as readRecordingOptions,
  recordingFileName,
  recordingLabel,
} from "./recordingControls.ts";

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
    const hasRecorder = this.options.getCanvasRecorder() !== null && this.options.getReplayPlayer() !== null;
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
        void this.options.getCanvasRecorder()?.stop().catch((error) => {
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
    return readRecordingOptions({
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
