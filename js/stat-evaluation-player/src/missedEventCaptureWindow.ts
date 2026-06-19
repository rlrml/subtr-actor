import type { StatsReplayPlayer } from "./statsReplayPlayer.ts";
import {
  MISSED_EVENT_MECHANIC_OPTIONS,
  captureMissedEventFromPlayer,
  resolveCaptureReplayId,
  uploadMissedEvent,
  type MissedEventCaptureRecord,
} from "./missedEventCapture.ts";

export interface MissedEventCaptureElements {
  readonly mechanic: HTMLSelectElement;
  readonly capture: HTMLButtonElement;
  readonly list: HTMLOListElement;
  readonly export: HTMLButtonElement;
  readonly upload: HTMLButtonElement;
  readonly clear: HTMLButtonElement;
  readonly status: HTMLElement;
}

export interface MissedEventCaptureOptions {
  readonly elements: MissedEventCaptureElements;
  getReplayPlayer(): StatsReplayPlayer | null;
  /** Fallback replay id when no `replayId` query param is present. */
  getReplayId?(): string | null;
  /** Reveal the missed-events window (e.g. when capturing via the hotkey). */
  showWindow(): void;
}

/** Capture-at-playhead hotkey. */
const CAPTURE_KEY = "m";

function isTextEntryFocused(): boolean {
  const active = document.activeElement;
  if (!(active instanceof HTMLElement)) {
    return false;
  }
  if (active.isContentEditable) {
    return true;
  }
  const tag = active.tagName;
  return tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT";
}

function downloadJson(filename: string, value: unknown): void {
  const blob = new Blob([JSON.stringify(value, null, 2)], { type: "application/json" });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = filename;
  document.body.appendChild(anchor);
  anchor.click();
  anchor.remove();
  URL.revokeObjectURL(url);
}

/**
 * Drives the "Missed events" window: an `M` hotkey records the selected mechanic
 * at the current playhead (attached player as subject), the window lists the
 * captures, and Export-JSON / Upload-all act on them. Upload targets the
 * rocket-sense missed-event endpoint; export always works even with no replay id
 * / backend. The window itself (chrome, drag, show/hide) is owned by the shared
 * floating-window system; this controller only owns its body content.
 */
export class MissedEventCaptureController {
  private readonly records: MissedEventCaptureRecord[] = [];
  private localIdSeq = 0;

  constructor(private readonly options: MissedEventCaptureOptions) {}

  installEventListeners(signal: AbortSignal): void {
    const { elements } = this.options;

    for (const mechanic of MISSED_EVENT_MECHANIC_OPTIONS) {
      const option = document.createElement("option");
      option.value = mechanic;
      option.textContent = mechanic.replaceAll("_", " ");
      elements.mechanic.appendChild(option);
    }

    elements.capture.addEventListener("click", () => this.capture(), { signal });
    elements.export.addEventListener("click", () => this.exportJson(), { signal });
    elements.upload.addEventListener("click", () => void this.uploadAll(), { signal });
    elements.clear.addEventListener(
      "click",
      () => {
        this.records.length = 0;
        this.render();
        this.setStatus("Cleared.");
      },
      { signal },
    );

    window.addEventListener(
      "keydown",
      (event) => {
        if (event.key.toLowerCase() !== CAPTURE_KEY || event.repeat) {
          return;
        }
        if (event.metaKey || event.ctrlKey || event.altKey || isTextEntryFocused()) {
          return;
        }
        event.preventDefault();
        this.capture();
      },
      { signal },
    );

    this.render();
  }

  capture(): void {
    const player = this.options.getReplayPlayer();
    if (!player) {
      this.setStatus("No replay loaded.");
      return;
    }
    this.localIdSeq += 1;
    const record = captureMissedEventFromPlayer(player, {
      mechanic: this.options.elements.mechanic.value || "flick",
      replayId: this.resolveReplayId(),
      localId: `missed-${this.localIdSeq}`,
    });
    this.records.push(record);
    this.options.showWindow();
    this.render();
    this.setStatus(
      `Captured ${record.mechanic} @ ${record.time.toFixed(2)}s` +
        (record.replayId ? "." : " (no replay id — export only)."),
    );
  }

  private exportJson(): void {
    if (this.records.length === 0) {
      this.setStatus("Nothing to export.");
      return;
    }
    downloadJson("missed-events.json", {
      capturedFrom: "stat-evaluation-player",
      replayId: this.resolveReplayId(),
      missedEvents: this.records,
    });
    this.setStatus(`Exported ${this.records.length}.`);
  }

  private async uploadAll(): Promise<void> {
    if (this.records.length === 0) {
      this.setStatus("Nothing to upload.");
      return;
    }
    let uploaded = 0;
    const failures: string[] = [];
    for (const record of [...this.records]) {
      const result = await uploadMissedEvent(record);
      if (result.ok) {
        uploaded += 1;
        const index = this.records.findIndex((entry) => entry.localId === record.localId);
        if (index >= 0) {
          this.records.splice(index, 1);
        }
      } else {
        failures.push(`${record.mechanic}@${record.time.toFixed(1)}s: ${result.message}`);
      }
    }
    this.render();
    this.setStatus(
      failures.length === 0
        ? `Uploaded ${uploaded}.`
        : `Uploaded ${uploaded}, ${failures.length} failed — ${failures[0]}`,
    );
  }

  private render(): void {
    const { list } = this.options.elements;
    list.replaceChildren();
    for (const record of this.records) {
      const item = document.createElement("li");
      const subject = record.playerName ?? record.subjectId ?? "no subject";
      const detail = document.createElement("span");
      detail.className = "missed-event-row";
      detail.textContent =
        `${record.mechanic} @ ${record.time.toFixed(2)}s · f${record.frame} · ${subject}` +
        (record.replayId ? "" : " · no replay id");

      const remove = document.createElement("button");
      remove.type = "button";
      remove.textContent = "✕";
      remove.title = "Remove";
      remove.addEventListener("click", () => {
        const index = this.records.findIndex((entry) => entry.localId === record.localId);
        if (index >= 0) {
          this.records.splice(index, 1);
          this.render();
        }
      });

      item.append(detail, remove);
      list.appendChild(item);
    }
  }

  private setStatus(message: string): void {
    this.options.elements.status.textContent = message;
  }

  private resolveReplayId(): string | null {
    return resolveCaptureReplayId(window.location.search, this.options.getReplayId?.() ?? null);
  }
}

export function createMissedEventCaptureController(
  options: MissedEventCaptureOptions,
): MissedEventCaptureController {
  return new MissedEventCaptureController(options);
}
