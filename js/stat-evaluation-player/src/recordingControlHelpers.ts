import type { CanvasRecorderStatus } from "@rlrml/subtr-actor-player";

export function formatBytes(bytes: number): string {
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

export function recordingLabel(status: CanvasRecorderStatus | null): string {
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

export function getRecordingOptions(elements: {
  fps: HTMLInputElement;
  playbackRate: HTMLInputElement | HTMLSelectElement;
}): { fps: number; playbackRate: number } {
  const fps = Number(elements.fps.value);
  const playbackRate = Number(elements.playbackRate.value);
  return {
    fps: Number.isFinite(fps) ? Math.max(1, Math.min(120, Math.trunc(fps))) : 60,
    playbackRate: Number.isFinite(playbackRate) ? Math.max(0.1, playbackRate) : 1,
  };
}

export function recordingFileName(sourceName: string | null): string {
  const source = sourceName?.replace(/\.replay$/i, "") || "replay";
  const safeSource = source.replace(/[^a-zA-Z0-9._-]+/g, "-").replace(/^-+|-+$/g, "");
  const timestamp = new Date().toISOString().replace(/[:.]/g, "-");
  return `${safeSource || "replay"}-${timestamp}.webm`;
}

export function downloadRecording(blob: Blob, fileName: string): void {
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = fileName;
  document.body.append(link);
  link.click();
  link.remove();
  window.setTimeout(() => URL.revokeObjectURL(url), 0);
}
