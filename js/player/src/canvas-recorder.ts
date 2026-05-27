import type {
  ReplayPlayerPluginContext,
  ReplayPlayerRenderContext,
  ReplayPlayerState,
} from "./types";
import {
  chooseCanvasRecorderMimeType,
  DEFAULT_CANVAS_RECORDER_FPS,
  getRecorderErrorMessage,
} from "./canvas-recorder-helpers";
import type {
  CanvasRecorderPlugin,
  CanvasRecorderPluginOptions,
  CanvasRecorderRangeOptions,
  CanvasRecorderStartOptions,
  CanvasRecorderStatus,
  CanvasRecorderStatusListener,
} from "./canvas-recorder-types";

export type {
  CanvasRecorderPlugin,
  CanvasRecorderPluginOptions,
  CanvasRecorderRangeOptions,
  CanvasRecorderStartOptions,
  CanvasRecorderState,
  CanvasRecorderStatus,
  CanvasRecorderStatusListener,
} from "./canvas-recorder-types";

export function createCanvasRecorderPlugin(
  options: CanvasRecorderPluginOptions = {},
): CanvasRecorderPlugin {
  let context: ReplayPlayerPluginContext | null = null;
  let recorder: MediaRecorder | null = null;
  let chunks: Blob[] = [];
  let recording: Blob | null = null;
  let startedAt = 0;
  let elapsedSeconds = 0;
  let mimeType = "";
  let sizeBytes = 0;
  let error: string | null = null;
  let stopPromise: Promise<Blob | null> | null = null;
  let stopResolve: ((recording: Blob | null) => void) | null = null;
  let rangeEndTime: number | null = null;
  let autoStopOnPlaybackEnd = false;
  let restoreState: ReplayPlayerState | null = null;
  const listeners = new Set<CanvasRecorderStatusListener>();

  function status(): CanvasRecorderStatus {
    return {
      state: recorder
        ? recorder.state === "recording"
          ? "recording"
          : "stopping"
        : error
          ? "error"
          : recording
            ? "ready"
            : "idle",
      elapsedSeconds,
      mimeType,
      sizeBytes,
      error,
    };
  }

  function notify(): void {
    const current = status();
    options.onStatusChange?.(current);
    for (const listener of listeners) {
      listener(current);
    }
  }

  function requireContext(): ReplayPlayerPluginContext {
    if (!context) {
      throw new Error("Canvas recorder plugin is not installed");
    }
    return context;
  }

  function finish(nextRecording: Blob | null): void {
    recorder = null;
    rangeEndTime = null;
    autoStopOnPlaybackEnd = false;
    recording = nextRecording;
    sizeBytes = nextRecording?.size ?? 0;
    if (restoreState && context) {
      context.player.setState({
        currentTime: restoreState.currentTime,
        speed: restoreState.speed,
        playing: restoreState.playing,
      });
    }
    restoreState = null;
    if (nextRecording) {
      options.onComplete?.(nextRecording);
    }
    notify();
    stopResolve?.(nextRecording);
    stopResolve = null;
    stopPromise = null;
  }

  function fail(nextError: unknown): void {
    error = getRecorderErrorMessage(nextError);
    recorder = null;
    rangeEndTime = null;
    autoStopOnPlaybackEnd = false;
    restoreState = null;
    notify();
    stopResolve?.(null);
    stopResolve = null;
    stopPromise = null;
  }

  const plugin: CanvasRecorderPlugin = {
    id: "canvas-recorder",

    setup(nextContext): void {
      context = nextContext;
    },

    beforeRender(renderContext: ReplayPlayerRenderContext): void {
      if (recorder?.state === "recording") {
        elapsedSeconds = (performance.now() - startedAt) / 1000;
        notify();
      }

      if (
        recorder?.state === "recording" &&
        rangeEndTime !== null &&
        renderContext.currentTime >= rangeEndTime
      ) {
        void plugin.stop();
      }
    },

    onStateChange(stateContext): void {
      if (
        autoStopOnPlaybackEnd &&
        recorder?.state === "recording" &&
        !stateContext.state.playing &&
        elapsedSeconds > 0
      ) {
        void plugin.stop();
      }
    },

    teardown(): void {
      if (recorder?.state === "recording") {
        recorder.stop();
      }
      context = null;
      recorder = null;
      rangeEndTime = null;
      autoStopOnPlaybackEnd = false;
      restoreState = null;
      stopResolve?.(null);
      stopResolve = null;
      stopPromise = null;
      listeners.clear();
    },

    start(startOptions: CanvasRecorderStartOptions = {}): void {
      const pluginContext = requireContext();
      if (recorder?.state === "recording") {
        throw new Error("Canvas recording is already in progress");
      }
      if (typeof MediaRecorder === "undefined") {
        throw new Error("MediaRecorder is not available in this browser");
      }

      const canvas = pluginContext.scene.renderer.domElement;
      if (!canvas.captureStream) {
        throw new Error("Canvas captureStream is not available in this browser");
      }

      error = null;
      recording = null;
      chunks = [];
      sizeBytes = 0;
      elapsedSeconds = 0;
      startedAt = performance.now();
      mimeType = chooseCanvasRecorderMimeType(startOptions.mimeType ?? options.mimeType);
      const fps = Math.max(1, startOptions.fps ?? options.fps ?? DEFAULT_CANVAS_RECORDER_FPS);
      const stream = canvas.captureStream(fps);
      recorder = new MediaRecorder(stream, {
        mimeType,
        videoBitsPerSecond: startOptions.videoBitsPerSecond ?? options.videoBitsPerSecond,
      });

      stopPromise = new Promise((resolve) => {
        stopResolve = resolve;
      });

      recorder.addEventListener("dataavailable", (event) => {
        if (event.data.size > 0) {
          chunks.push(event.data);
          sizeBytes += event.data.size;
          notify();
        }
      });
      recorder.addEventListener(
        "stop",
        () => {
          stream.getTracks().forEach((track) => track.stop());
          finish(new Blob(chunks, { type: mimeType || "video/webm" }));
        },
        { once: true },
      );
      recorder.addEventListener(
        "error",
        (event) => {
          stream.getTracks().forEach((track) => track.stop());
          fail((event as ErrorEvent).error ?? event);
        },
        { once: true },
      );

      recorder.start(1000);
      notify();
    },

    stop(): Promise<Blob | null> {
      if (!recorder) {
        return Promise.resolve(recording);
      }
      if (recorder.state === "inactive") {
        return stopPromise ?? Promise.resolve(recording);
      }
      const promise =
        stopPromise ??
        new Promise<Blob | null>((resolve) => {
          stopResolve = resolve;
        });
      recorder.stop();
      notify();
      return promise;
    },

    clear(): void {
      if (recorder?.state === "recording") {
        throw new Error("Cannot clear a recording while recording is in progress");
      }
      recording = null;
      chunks = [];
      sizeBytes = 0;
      elapsedSeconds = 0;
      error = null;
      notify();
    },

    getRecording(): Blob | null {
      return recording;
    },

    getStatus(): CanvasRecorderStatus {
      return status();
    },

    subscribe(listener: CanvasRecorderStatusListener): () => void {
      listeners.add(listener);
      listener(status());
      return () => {
        listeners.delete(listener);
      };
    },

    recordRange(rangeOptions: CanvasRecorderRangeOptions = {}): Promise<Blob> {
      const pluginContext = requireContext();
      const state = pluginContext.player.getState();
      if (rangeOptions.restorePlaybackState ?? true) {
        restoreState = state;
      }
      const playbackRate = rangeOptions.playbackRate ?? state.speed;
      const startTime = rangeOptions.startTime ?? state.currentTime;
      rangeEndTime = rangeOptions.endTime ?? state.duration;
      autoStopOnPlaybackEnd = true;

      pluginContext.player.setState({
        currentTime: startTime,
        speed: playbackRate,
        playing: false,
      });
      plugin.start(rangeOptions);
      const completion = stopPromise;
      pluginContext.player.play();

      return (completion ?? Promise.resolve(null)).then((blob) => {
        if (!blob) {
          throw new Error("Recording stopped without producing a video");
        }
        return blob;
      });
    },

    recordFullReplay(rangeOptions: CanvasRecorderRangeOptions = {}): Promise<Blob> {
      return plugin.recordRange({
        ...rangeOptions,
        startTime: rangeOptions.startTime ?? 0,
        endTime: rangeOptions.endTime ?? requireContext().replay.duration,
      });
    },
  };

  return plugin;
}
