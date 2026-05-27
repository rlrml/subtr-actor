const DEFAULT_MIME_TYPES = ["video/webm;codecs=vp9", "video/webm;codecs=vp8", "video/webm"];

export const DEFAULT_CANVAS_RECORDER_FPS = 60;

export function chooseCanvasRecorderMimeType(requested: string | undefined): string {
  if (requested && MediaRecorder.isTypeSupported(requested)) {
    return requested;
  }

  for (const candidate of DEFAULT_MIME_TYPES) {
    if (MediaRecorder.isTypeSupported(candidate)) {
      return candidate;
    }
  }

  return "";
}

export function getRecorderErrorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
