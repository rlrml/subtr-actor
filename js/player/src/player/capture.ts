import { loadReplay } from "./adapter/wasm.js";
import { createPlayerFromParsed } from "./lib.js";
import type { ReplayLoadResult } from "../types";
import type { CameraSettings, PlayerFreeCameraPreset, PlayerOptions } from "./types.js";
import type { ReplayPlayerInfo } from "./adapter/SubtrActorPlayer.js";
import type { ReplayPlayer } from "./ReplayPlayer.js";

const DEFAULT_WIDTH = 1280;
const DEFAULT_HEIGHT = 720;
const DEFAULT_MIME_TYPE = "image/png";
const DEFAULT_SETTLE_FRAMES = 2;
const DEFAULT_READY_TIMEOUT_MS = 30_000;

export type PlayerImageBallCamMode = boolean | "replay";

export type PlayerImageCamera =
  | {
      mode: "free";
      preset?: PlayerFreeCameraPreset;
    }
  | {
      mode: "attached";
      playerId?: string | null;
      playerName?: string | null;
      ballCam?: PlayerImageBallCamMode;
      cameraDistanceScale?: number;
      cameraSettings?: CameraSettings | null;
    }
  | {
      mode: "custom";
      setup: (player: ReplayPlayer) => void | Promise<void>;
    };

export interface PlayerImageCaptureOptions {
  width?: number;
  height?: number;
  pixelRatio?: number;
  time?: number;
  frameIndex?: number | null;
  camera?: PlayerImageCamera;
  playerOptions?: PlayerOptions;
  mimeType?: string;
  quality?: number;
  settleFrames?: number;
  readyTimeoutMs?: number | false;
}

export type PlayerImageCaptureRequest = Omit<
  PlayerImageCaptureOptions,
  "playerOptions" | "readyTimeoutMs"
>;

export interface PlayerImageCaptureResult {
  blob: Blob;
  dataUrl: string;
  width: number;
  height: number;
  pixelRatio: number;
  mimeType: string;
  time: number;
  frameIndex: number;
}

export async function capturePlayerImage(
  replayBytes: Uint8Array,
  options: PlayerImageCaptureOptions = {},
): Promise<PlayerImageCaptureResult> {
  return capturePlayerImageFromParsed(await loadReplay(replayBytes), options);
}

export async function capturePlayerImages(
  replayBytes: Uint8Array,
  captures: PlayerImageCaptureRequest[],
  options: PlayerImageCaptureOptions = {},
): Promise<PlayerImageCaptureResult[]> {
  return capturePlayerImagesFromParsed(await loadReplay(replayBytes), captures, options);
}

export async function capturePlayerImageFromParsed(
  parsed: ReplayLoadResult,
  options: PlayerImageCaptureOptions = {},
): Promise<PlayerImageCaptureResult> {
  const [result] = await capturePlayerImagesFromParsed(parsed, [options], options);
  if (!result) {
    throw new Error("Static image capture did not produce a result");
  }
  return result;
}

export async function capturePlayerImagesFromParsed(
  parsed: ReplayLoadResult,
  captures: PlayerImageCaptureRequest[],
  options: PlayerImageCaptureOptions = {},
): Promise<PlayerImageCaptureResult[]> {
  if (captures.length === 0) {
    return [];
  }
  const initial = mergeCaptureOptions(options, captures[0]);
  const width = positiveInteger(initial.width, DEFAULT_WIDTH);
  const height = positiveInteger(initial.height, DEFAULT_HEIGHT);
  const container = createHiddenCaptureContainer(width, height);
  let player: ReplayPlayer | null = null;

  try {
    player = createPlayerFromParsed(container, parsed, {
      ...options.playerOptions,
      autoplay: false,
      loop: false,
      preserveDrawingBuffer: true,
    });
    await waitForPlayerReady(player, options.readyTimeoutMs);
    player.pause();
    const results: PlayerImageCaptureResult[] = [];
    for (const capture of captures) {
      results.push(await capturePlayerImageWithExistingPlayer(player, options, capture));
    }
    return results;
  } finally {
    player?.destroy();
    container.remove();
  }
}

async function capturePlayerImageWithExistingPlayer(
  player: ReplayPlayer,
  options: PlayerImageCaptureOptions,
  capture: PlayerImageCaptureRequest,
): Promise<PlayerImageCaptureResult> {
  const merged = mergeCaptureOptions(options, capture);
  const width = positiveInteger(merged.width, DEFAULT_WIDTH);
  const height = positiveInteger(merged.height, DEFAULT_HEIGHT);
  const pixelRatio = positiveNumber(
    merged.pixelRatio,
    typeof window === "undefined" ? 1 : window.devicePixelRatio || 1,
  );
  const mimeType = merged.mimeType ?? DEFAULT_MIME_TYPE;

  player.renderer.setPixelRatio(pixelRatio);
  player.renderer.setSize(width, height, false);
  player.camera.aspect = width / height;
  player.camera.updateProjectionMatrix();
  seekCaptureTime(player, merged);
  await applyCaptureCamera(player, merged.camera);
  renderSettledFrames(player, positiveInteger(merged.settleFrames, DEFAULT_SETTLE_FRAMES));

  const canvas = player.renderer.domElement;
  const dataUrl = canvas.toDataURL(mimeType, merged.quality);
  const blob = await canvasToBlob(canvas, mimeType, merged.quality, dataUrl);
  const state = player.getState();
  return {
    blob,
    dataUrl,
    width,
    height,
    pixelRatio,
    mimeType: blob.type || mimeType,
    time: state.currentTime,
    frameIndex: state.frameIndex,
  };
}

function mergeCaptureOptions(
  options: PlayerImageCaptureOptions,
  capture: PlayerImageCaptureRequest,
): PlayerImageCaptureOptions {
  return {
    ...options,
    ...capture,
    playerOptions: options.playerOptions,
    readyTimeoutMs: options.readyTimeoutMs,
  };
}

async function waitForPlayerReady(
  player: ReplayPlayer,
  timeoutMs: number | false | undefined,
): Promise<void> {
  if (timeoutMs === false) {
    await player.ready;
    return;
  }
  const timeout = positiveNumber(timeoutMs, DEFAULT_READY_TIMEOUT_MS);
  let timeoutHandle: ReturnType<typeof setTimeout> | null = null;
  try {
    const result = await Promise.race<"ready" | "timeout">([
      player.ready.then(() => "ready"),
      new Promise<"timeout">((resolve) => {
        timeoutHandle = setTimeout(() => resolve("timeout"), timeout);
      }),
    ]);
    if (result === "timeout") {
      console.warn(
        `[player] static image capture proceeding before assets settled after ${timeout}ms`,
      );
    }
  } finally {
    if (timeoutHandle != null) {
      clearTimeout(timeoutHandle);
    }
  }
}

function createHiddenCaptureContainer(width: number, height: number): HTMLDivElement {
  const container = document.createElement("div");
  container.style.position = "fixed";
  container.style.left = "-10000px";
  container.style.top = "0";
  container.style.width = `${width}px`;
  container.style.height = `${height}px`;
  container.style.overflow = "hidden";
  container.style.pointerEvents = "none";
  container.setAttribute("aria-hidden", "true");
  document.body.appendChild(container);
  return container;
}

function seekCaptureTime(player: ReplayPlayer, options: PlayerImageCaptureOptions): void {
  if (options.frameIndex != null) {
    player.setFrameIndex(options.frameIndex);
    return;
  }
  player.seek(options.time ?? 0);
}

async function applyCaptureCamera(
  player: ReplayPlayer,
  camera: PlayerImageCamera | undefined,
): Promise<void> {
  if (!camera || camera.mode === "free") {
    player.setFreeCameraPreset(camera?.preset ?? "side", { instant: true });
    return;
  }
  if (camera.mode === "custom") {
    await camera.setup(player);
    return;
  }

  const targetPlayer = resolveCapturePlayer(player, camera);
  if (!targetPlayer) {
    throw new Error("Unable to resolve static image capture player target");
  }
  if (camera.cameraSettings !== undefined) {
    player.setCustomCameraSettings(camera.cameraSettings);
  }
  if (camera.cameraDistanceScale !== undefined) {
    player.setCameraDistanceScale(camera.cameraDistanceScale);
  }
  player.setAttachedPlayer(targetPlayer.id);
  player.setBallCamEnabled(camera.ballCam === "replay" ? null : (camera.ballCam ?? true));
}

function resolveCapturePlayer(
  player: ReplayPlayer,
  camera: Extract<PlayerImageCamera, { mode: "attached" }>,
): ReplayPlayerInfo | null {
  if (camera.playerId) {
    const byId = player.adapter.playerList.find((candidate) => candidate.id === camera.playerId);
    if (byId) {
      return byId;
    }
  }
  const name = camera.playerName?.trim().toLowerCase();
  if (!name) {
    return player.adapter.playerList[0] ?? null;
  }
  return (
    player.adapter.playerList.find((candidate) => candidate.name.trim().toLowerCase() === name) ??
    null
  );
}

function renderSettledFrames(player: ReplayPlayer, count: number): void {
  for (let frame = 0; frame < count; frame += 1) {
    player.renderFrame(1 / 60);
  }
}

function canvasToBlob(
  canvas: HTMLCanvasElement,
  mimeType: string,
  quality: number | undefined,
  fallbackDataUrl: string,
): Promise<Blob> {
  return new Promise((resolve) => {
    canvas.toBlob(
      (blob) => {
        resolve(blob ?? dataUrlToBlob(fallbackDataUrl));
      },
      mimeType,
      quality,
    );
  });
}

function dataUrlToBlob(dataUrl: string): Blob {
  const [header, payload] = dataUrl.split(",", 2);
  const mimeType = header?.match(/^data:([^;]+)/)?.[1] ?? DEFAULT_MIME_TYPE;
  const binary = atob(payload ?? "");
  const bytes = new Uint8Array(binary.length);
  for (let index = 0; index < binary.length; index += 1) {
    bytes[index] = binary.charCodeAt(index);
  }
  return new Blob([bytes], { type: mimeType });
}

function positiveInteger(value: number | undefined, fallback: number): number {
  return typeof value === "number" && Number.isFinite(value) && value > 0
    ? Math.round(value)
    : fallback;
}

function positiveNumber(value: number | undefined, fallback: number): number {
  return typeof value === "number" && Number.isFinite(value) && value > 0 ? value : fallback;
}
