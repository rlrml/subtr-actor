import {
  DEFAULT_TRAINING_SHOT_TIME_LIMIT_SECONDS,
  TrainingPackFile,
  appendCapturedRound,
  capturedTrainingPackDefaults,
  trainingPackFileName,
  type TrainingCaptureOptions,
  type TrainingPackBindings,
} from "@rlrml/player";

export { DEFAULT_TRAINING_SHOT_TIME_LIMIT_SECONDS };

/** One row of the training-pack panel's shot list. */
export interface TrainingPackShotView {
  readonly index: number;
  readonly timeLimit: number;
  /**
   * Replay time (viewer seconds) the shot was captured at, or `null` for
   * rounds that came in by loading an existing `.tem` file.
   */
  readonly sourceReplayTime: number | null;
}

/**
 * The in-memory training pack being assembled by the training-pack panel:
 * a `TrainingPackFile` plus per-shot capture annotations and an
 * unsaved-shots flag.
 *
 * Loading an existing pack seeds the session, so captures APPEND to the
 * loaded rounds — round edits go through `TrainingPackFile`'s lossless
 * property-tree operations and never drop or rewrite pre-existing rounds.
 */
export class TrainingPackSession {
  private sourceTimes: (number | null)[];
  private dirty = false;

  private constructor(
    readonly file: TrainingPackFile,
    sourceTimes: (number | null)[],
  ) {
    this.sourceTimes = sourceTimes;
  }

  /**
   * Starts a fresh pack with capture defaults (random GUID, current
   * timestamps, striker/medium/Park_P) — the browser counterpart of the
   * BakkesMod plugin's new-pack flow.
   */
  static async createNew(options?: {
    bindings?: TrainingPackBindings;
  }): Promise<TrainingPackSession> {
    const file = await TrainingPackFile.create(capturedTrainingPackDefaults(), options);
    return new TrainingPackSession(file, []);
  }

  /** Opens an existing `.tem` file so captured shots append to it. */
  static async loadFromBytes(
    data: Uint8Array | ArrayBuffer,
    options?: { bindings?: TrainingPackBindings },
  ): Promise<TrainingPackSession> {
    const file = await TrainingPackFile.load(data, options);
    return new TrainingPackSession(file, new Array<number | null>(file.roundCount).fill(null));
  }

  get shotCount(): number {
    return this.file.roundCount;
  }

  /** Whether shots were captured/removed since the last download. */
  get hasUnsavedShots(): boolean {
    return this.dirty;
  }

  shots(): TrainingPackShotView[] {
    return this.file.rounds.map((round, index) => ({
      index,
      timeLimit: round.time_limit,
      sourceReplayTime: this.sourceTimes[index] ?? null,
    }));
  }

  /**
   * Appends a captured replay frame as a new shot and returns its index.
   */
  captureShot(capture: TrainingCaptureOptions, sourceReplayTime: number | null = null): number {
    const index = appendCapturedRound(this.file, capture);
    this.sourceTimes[index] = sourceReplayTime;
    this.dirty = true;
    return index;
  }

  removeShot(index: number): void {
    this.file.removeRound(index);
    this.sourceTimes.splice(index, 1);
    this.dirty = true;
  }

  setShotTimeLimit(index: number, timeLimit: number): void {
    this.file.setRoundTimeLimit(index, timeLimit);
    this.dirty = true;
  }

  /**
   * Download filename: `<guid-hex>.Tem`, the GUID-based name the game
   * scans for (and the BakkesMod plugin writes), so a downloaded pack is
   * drop-in usable in the game's training folder.
   */
  downloadFileName(): string {
    return trainingPackFileName(this.file.guid);
  }

  /**
   * Serializes to `.tem` bytes, refreshing `UpdatedAt` (mirroring the
   * plugin's save) and clearing the unsaved-shots flag.
   */
  toBytes(nowSeconds: number = Math.floor(Date.now() / 1000)): Uint8Array {
    this.file.updateMetadata({ updated_at: BigInt(nowSeconds) });
    const bytes = this.file.toBytes();
    this.dirty = false;
    return bytes;
  }
}
