import type { PlayerSample, ReplayPlayerTrack } from "@rlrml/player";
import { DEFAULT_TRAINING_SHOT_TIME_LIMIT_SECONDS } from "@rlrml/player";
import type { StatsReplayPlayer } from "./statsReplayPlayer.ts";
import { TrainingPackSession } from "./trainingPackSession.ts";
import { formatTime } from "./statsWindows.ts";

export interface TrainingPackWindowElements {
  readonly name: HTMLInputElement;
  readonly creator: HTMLInputElement;
  readonly description: HTMLInputElement;
  readonly difficulty: HTMLSelectElement;
  readonly shooter: HTMLSelectElement;
  readonly timeLimit: HTMLInputElement;
  readonly capture: HTMLButtonElement;
  readonly load: HTMLButtonElement;
  readonly loadInput: HTMLInputElement;
  readonly newPack: HTMLButtonElement;
  readonly download: HTMLButtonElement;
  readonly shotList: HTMLOListElement;
  readonly status: HTMLElement;
}

export interface TrainingPackWindowOptions {
  readonly elements: TrainingPackWindowElements;
  getReplayPlayer(): StatsReplayPlayer | null;
}

function downloadBytes(bytes: Uint8Array, fileName: string): void {
  const buffer = bytes.buffer.slice(
    bytes.byteOffset,
    bytes.byteOffset + bytes.byteLength,
  ) as ArrayBuffer;
  const blob = new Blob([buffer], { type: "application/octet-stream" });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = fileName;
  document.body.appendChild(anchor);
  anchor.click();
  anchor.remove();
  window.setTimeout(() => URL.revokeObjectURL(url), 0);
}

/**
 * Drives the "Training pack" window: captures the current replay frame's
 * ball + shooter car as a training shot, accumulates shots into an
 * in-memory pack (optionally seeded by loading an existing `.tem`, so
 * captures append), edits pack metadata, and downloads the result as a
 * GUID-named `.Tem` file. The window chrome (drag, show/hide) is owned by
 * the shared floating-window system; this controller only owns the body.
 */
export class TrainingPackWindowController {
  private session: TrainingPackSession | null = null;
  private sessionPromise: Promise<TrainingPackSession> | null = null;

  constructor(private readonly options: TrainingPackWindowOptions) {}

  installEventListeners(signal: AbortSignal): void {
    const { elements } = this.options;

    elements.capture.addEventListener("click", () => void this.captureShot(), { signal });
    elements.newPack.addEventListener("click", () => void this.newPack(), { signal });
    elements.download.addEventListener("click", () => void this.download(), { signal });
    elements.load.addEventListener("click", () => elements.loadInput.click(), { signal });
    elements.loadInput.addEventListener(
      "change",
      () => {
        const file = elements.loadInput.files?.[0] ?? null;
        elements.loadInput.value = "";
        if (file) {
          void this.loadPackFile(file);
        }
      },
      { signal },
    );

    elements.name.addEventListener(
      "change",
      () => this.session?.file.setName(elements.name.value || null),
      { signal },
    );
    elements.creator.addEventListener(
      "change",
      () => this.session?.file.setCreatorName(elements.creator.value || null),
      { signal },
    );
    elements.description.addEventListener(
      "change",
      () => this.session?.file.setDescription(elements.description.value || null),
      { signal },
    );
    elements.difficulty.addEventListener(
      "change",
      () => this.session?.file.setDifficulty(elements.difficulty.value),
      { signal },
    );

    this.sync();
  }

  /** Refreshes enablement and the shooter dropdown for the loaded replay. */
  sync(): void {
    const { elements } = this.options;
    const player = this.options.getReplayPlayer();
    elements.capture.disabled = !player;
    elements.shooter.disabled = !player;
    this.populateShooterOptions(player);
    this.renderShotList();
  }

  private populateShooterOptions(player: StatsReplayPlayer | null): void {
    const { shooter } = this.options.elements;
    const previous = shooter.value;
    shooter.replaceChildren();
    shooter.append(new Option("Followed player (camera)", ""));
    for (const track of player?.replay.players ?? []) {
      shooter.append(new Option(track.name, track.id));
    }
    if ([...shooter.options].some((option) => option.value === previous)) {
      shooter.value = previous;
    }
  }

  private async ensureSession(): Promise<TrainingPackSession> {
    if (this.session) {
      return this.session;
    }
    this.sessionPromise ??= TrainingPackSession.createNew();
    const session = await this.sessionPromise;
    // A load/new that resolved while we awaited wins.
    if (!this.session) {
      this.adoptSession(session);
    }
    return this.session ?? session;
  }

  private adoptSession(session: TrainingPackSession): void {
    this.session = session;
    this.sessionPromise = null;
    const { elements } = this.options;
    elements.name.value = session.file.name ?? "";
    elements.creator.value = session.file.creatorName ?? "";
    elements.description.value = session.file.description ?? "";
    const difficulty = session.file.difficulty;
    if ([...elements.difficulty.options].some((option) => option.value === difficulty)) {
      elements.difficulty.value = difficulty;
    }
    this.renderShotList();
  }

  private resolveShooter(
    player: StatsReplayPlayer,
    frameIndex: number,
  ): { track: ReplayPlayerTrack; sample: PlayerSample } | null {
    const requestedId = this.options.elements.shooter.value || null;
    const attachedId = player.getState().attachedPlayerId;
    const candidates = player.replay.players;
    const ordered = [
      ...candidates.filter((track) => track.id === (requestedId ?? attachedId)),
      ...candidates,
    ];
    for (const track of ordered) {
      const sample = track.frames[frameIndex];
      if (sample?.position) {
        if (requestedId && track.id !== requestedId) {
          // The requested shooter has no state on this frame; don't
          // silently substitute another car.
          return null;
        }
        return { track, sample };
      }
    }
    return null;
  }

  private timeLimitSeconds(): number {
    const raw = Number(this.options.elements.timeLimit.value);
    if (!Number.isFinite(raw) || raw < 0) {
      return DEFAULT_TRAINING_SHOT_TIME_LIMIT_SECONDS;
    }
    return raw;
  }

  async captureShot(): Promise<void> {
    const player = this.options.getReplayPlayer();
    if (!player) {
      this.setStatus("No replay loaded.");
      return;
    }
    const state = player.getState();
    const ballSample = player.replay.ballFrames[state.frameIndex];
    if (!ballSample?.position) {
      this.setStatus("No ball state on the current frame.");
      return;
    }
    const shooter = this.resolveShooter(player, state.frameIndex);
    if (!shooter) {
      this.setStatus("Selected shooter has no car state on the current frame.");
      return;
    }
    const session = await this.ensureSession();
    const index = session.captureShot(
      {
        ball: {
          position: ballSample.position,
          linearVelocity: ballSample.linearVelocity,
        },
        shooter: {
          position: shooter.sample.position!,
          rotation: shooter.sample.rotation,
        },
        timeLimit: this.timeLimitSeconds(),
      },
      state.currentTime,
    );
    this.renderShotList();
    this.setStatus(
      `Captured shot ${index + 1} (${shooter.track.name} at ${formatTime(state.currentTime)}).`,
    );
  }

  async loadPackFile(file: File): Promise<void> {
    if (
      this.session?.hasUnsavedShots &&
      !window.confirm("Discard unsaved captured shots and open this pack?")
    ) {
      return;
    }
    try {
      const session = await TrainingPackSession.loadFromBytes(await file.arrayBuffer());
      this.adoptSession(session);
      this.setStatus(
        `Opened ${file.name} (${session.shotCount} shot${session.shotCount === 1 ? "" : "s"}); captures will append.`,
      );
    } catch (error) {
      console.error("Failed to load training pack:", error);
      this.setStatus(error instanceof Error ? error.message : "Failed to load training pack.");
    }
  }

  async newPack(): Promise<void> {
    if (
      this.session?.hasUnsavedShots &&
      !window.confirm("Discard unsaved captured shots and start a new pack?")
    ) {
      return;
    }
    try {
      this.adoptSession(await TrainingPackSession.createNew());
      this.setStatus("Started a new pack.");
    } catch (error) {
      console.error("Failed to create training pack:", error);
      this.setStatus(error instanceof Error ? error.message : "Failed to create training pack.");
    }
  }

  async download(): Promise<void> {
    const session = await this.ensureSession();
    if (session.shotCount === 0) {
      this.setStatus("No shots to download; capture one first.");
      return;
    }
    try {
      downloadBytes(session.toBytes(), session.downloadFileName());
      this.setStatus(`Downloaded ${session.downloadFileName()}.`);
    } catch (error) {
      console.error("Failed to serialize training pack:", error);
      this.setStatus(error instanceof Error ? error.message : "Failed to serialize pack.");
    }
  }

  private renderShotList(): void {
    const { shotList } = this.options.elements;
    shotList.replaceChildren();
    const session = this.session;
    if (!session) {
      return;
    }
    for (const shot of session.shots()) {
      const item = document.createElement("li");

      const label = document.createElement("span");
      const source =
        shot.sourceReplayTime === null ? "loaded" : `at ${formatTime(shot.sourceReplayTime)}`;
      const limit = shot.timeLimit === 0 ? "no limit" : `${shot.timeLimit}s`;
      label.textContent = `Shot ${shot.index + 1} — ${source} — ${limit}`;

      const remove = document.createElement("button");
      remove.type = "button";
      remove.textContent = "Remove";
      remove.addEventListener("click", () => {
        session.removeShot(shot.index);
        this.renderShotList();
        this.setStatus(`Removed shot ${shot.index + 1}.`);
      });

      item.append(label, remove);
      shotList.appendChild(item);
    }
  }

  private setStatus(message: string): void {
    this.options.elements.status.textContent = message;
  }
}

export function createTrainingPackWindowController(
  options: TrainingPackWindowOptions,
): TrainingPackWindowController {
  return new TrainingPackWindowController(options);
}
