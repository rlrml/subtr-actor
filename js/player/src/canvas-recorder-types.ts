import type { ReplayPlayerPlugin } from "./types";

export type CanvasRecorderState = "idle" | "recording" | "stopping" | "ready" | "error";

export interface CanvasRecorderStatus {
  state: CanvasRecorderState;
  elapsedSeconds: number;
  mimeType: string;
  sizeBytes: number;
  error: string | null;
}

export interface CanvasRecorderStartOptions {
  mimeType?: string;
  fps?: number;
  videoBitsPerSecond?: number;
}

export interface CanvasRecorderRangeOptions extends CanvasRecorderStartOptions {
  startTime?: number;
  endTime?: number;
  playbackRate?: number;
  restorePlaybackState?: boolean;
}

export type CanvasRecorderStatusListener = (status: CanvasRecorderStatus) => void;

export interface CanvasRecorderPluginOptions extends CanvasRecorderStartOptions {
  onStatusChange?: CanvasRecorderStatusListener;
  onComplete?: (recording: Blob) => void;
}

export interface CanvasRecorderPlugin extends ReplayPlayerPlugin {
  start(options?: CanvasRecorderStartOptions): void;
  stop(): Promise<Blob | null>;
  clear(): void;
  getRecording(): Blob | null;
  getStatus(): CanvasRecorderStatus;
  subscribe(listener: CanvasRecorderStatusListener): () => void;
  recordRange(options?: CanvasRecorderRangeOptions): Promise<Blob>;
  recordFullReplay(options?: CanvasRecorderRangeOptions): Promise<Blob>;
}
