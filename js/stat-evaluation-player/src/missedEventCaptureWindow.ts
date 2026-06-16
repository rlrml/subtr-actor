import type { StatsReplayPlayer } from "./statsReplayPlayer.ts";
import {
  MISSED_EVENT_MECHANIC_OPTIONS,
  captureMissedEventFromPlayer,
  resolveCaptureReplayId,
  uploadMissedEvent,
  type MissedEventCaptureRecord,
} from "./missedEventCapture.ts";

export interface MissedEventCaptureOptions {
  getReplayPlayer(): StatsReplayPlayer | null;
  signal: AbortSignal;
  /** Fallback replay id when no `replayId` query param is present. */
  getReplayId?(): string | null;
  /** Defaults to `document.body`. */
  mountParent?: HTMLElement;
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
 * Install the "missed event" capture affordance: an `M` hotkey that records the
 * selected mechanic at the current playhead (attached player as subject), a
 * compact panel listing captures, and Export-JSON / Upload-all actions. Upload
 * targets the rocket-sense missed-event endpoint; export always works even with
 * no replay id / backend.
 */
export function installMissedEventCapture(options: MissedEventCaptureOptions): {
  capture(): void;
  records(): readonly MissedEventCaptureRecord[];
} {
  const { getReplayPlayer, signal } = options;
  const parent = options.mountParent ?? document.body;
  const records: MissedEventCaptureRecord[] = [];
  let localIdSeq = 0;

  const panel = document.createElement("section");
  panel.className = "missed-capture";
  panel.hidden = true;
  panel.innerHTML = `
    <header class="missed-capture__head">
      <span class="missed-capture__title">Missed events (<span data-count>0</span>)</span>
      <button type="button" data-action="hide" title="Hide">×</button>
    </header>
    <div class="missed-capture__controls">
      <label>Mechanic
        <select data-mechanic></select>
      </label>
      <button type="button" data-action="capture">Capture (M)</button>
    </div>
    <ol class="missed-capture__list" data-list></ol>
    <div class="missed-capture__actions">
      <button type="button" data-action="export">Export JSON</button>
      <button type="button" data-action="upload">Upload all</button>
      <button type="button" data-action="clear">Clear</button>
    </div>
    <p class="missed-capture__status" data-status></p>
  `;
  parent.appendChild(panel);

  const select = panel.querySelector<HTMLSelectElement>("[data-mechanic]")!;
  for (const mechanic of MISSED_EVENT_MECHANIC_OPTIONS) {
    const option = document.createElement("option");
    option.value = mechanic;
    option.textContent = mechanic.replaceAll("_", " ");
    select.appendChild(option);
  }
  const listEl = panel.querySelector<HTMLOListElement>("[data-list]")!;
  const countEl = panel.querySelector<HTMLElement>("[data-count]")!;
  const statusEl = panel.querySelector<HTMLElement>("[data-status]")!;

  const setStatus = (message: string): void => {
    statusEl.textContent = message;
  };

  const resolveReplayId = (): string | null =>
    resolveCaptureReplayId(window.location.search, options.getReplayId?.() ?? null);

  const render = (): void => {
    countEl.textContent = String(records.length);
    listEl.replaceChildren();
    for (const record of records) {
      const item = document.createElement("li");
      const subject = record.playerName ?? record.subjectId ?? "no subject";
      item.innerHTML = `
        <span class="missed-capture__row">
          <strong>${record.mechanic}</strong>
          @ ${record.time.toFixed(2)}s · f${record.frame} · ${subject}
          ${record.replayId ? "" : "· <em>no replay id</em>"}
        </span>
      `;
      const remove = document.createElement("button");
      remove.type = "button";
      remove.textContent = "✕";
      remove.title = "Remove";
      remove.addEventListener(
        "click",
        () => {
          const index = records.findIndex((entry) => entry.localId === record.localId);
          if (index >= 0) {
            records.splice(index, 1);
            render();
          }
        },
        { signal },
      );
      item.appendChild(remove);
      listEl.appendChild(item);
    }
    panel.hidden = records.length === 0;
  };

  const capture = (): void => {
    const player = getReplayPlayer();
    if (!player) {
      setStatus("No replay loaded.");
      return;
    }
    localIdSeq += 1;
    const record = captureMissedEventFromPlayer(player, {
      mechanic: select.value || "flick",
      replayId: resolveReplayId(),
      localId: `missed-${localIdSeq}`,
    });
    records.push(record);
    render();
    setStatus(
      `Captured ${record.mechanic} @ ${record.time.toFixed(2)}s` +
        (record.replayId ? "." : " (no replay id — export only)."),
    );
  };

  const uploadAll = async (): Promise<void> => {
    if (records.length === 0) {
      setStatus("Nothing to upload.");
      return;
    }
    let uploaded = 0;
    const failures: string[] = [];
    for (const record of [...records]) {
      const result = await uploadMissedEvent(record);
      if (result.ok) {
        uploaded += 1;
        const index = records.findIndex((entry) => entry.localId === record.localId);
        if (index >= 0) {
          records.splice(index, 1);
        }
      } else {
        failures.push(`${record.mechanic}@${record.time.toFixed(1)}s: ${result.message}`);
      }
    }
    render();
    setStatus(
      failures.length === 0
        ? `Uploaded ${uploaded}.`
        : `Uploaded ${uploaded}, ${failures.length} failed — ${failures[0]}`,
    );
  };

  panel.querySelector("[data-action='capture']")?.addEventListener("click", capture, { signal });
  panel.querySelector("[data-action='hide']")?.addEventListener(
    "click",
    () => {
      panel.hidden = true;
    },
    { signal },
  );
  panel.querySelector("[data-action='export']")?.addEventListener(
    "click",
    () => {
      if (records.length === 0) {
        setStatus("Nothing to export.");
        return;
      }
      downloadJson("missed-events.json", {
        capturedFrom: "stat-evaluation-player",
        replayId: resolveReplayId(),
        missedEvents: records,
      });
      setStatus(`Exported ${records.length}.`);
    },
    { signal },
  );
  panel
    .querySelector("[data-action='upload']")
    ?.addEventListener("click", () => void uploadAll(), { signal });
  panel.querySelector("[data-action='clear']")?.addEventListener(
    "click",
    () => {
      records.length = 0;
      render();
      setStatus("Cleared.");
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
      capture();
    },
    { signal },
  );

  return {
    capture,
    records: () => records,
  };
}
