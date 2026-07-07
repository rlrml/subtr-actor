import type { Archetype } from "./generated/Archetype";
import type { BallSpawn } from "./generated/BallSpawn";
import type { CarSpawn } from "./generated/CarSpawn";
import type { Guid } from "./generated/Guid";
import type { PlayerCarSpawn } from "./generated/PlayerCarSpawn";
import type { Round } from "./generated/Round";
import type { TrainingPack } from "./generated/TrainingPack";

export type { Archetype, BallSpawn, CarSpawn, PlayerCarSpawn };

/**
 * The subset of the `@rlrml/subtr-actor` WASM exports used for training pack
 * (`.tem`) files.
 *
 * Every editing entry point exchanges a *lossless* JSON string: the full
 * property tree of the underlying training file, which preserves unknown
 * properties so a parse -> edit -> serialize round trip only changes the
 * fields that were explicitly edited. JS code should treat that string as an
 * opaque token.
 */
export interface TrainingPackBindings {
  parse_training_pack(data: Uint8Array): unknown;
  parse_training_pack_lossless(data: Uint8Array): string;
  serialize_training_pack(lossless: string): Uint8Array;
  training_pack_from_lossless(lossless: string): unknown;
  new_training_pack(typedPack: unknown): string;
  update_training_pack_metadata(lossless: string, typedPack: unknown): string;
  training_pack_add_round(lossless: string, round: unknown): string;
  training_pack_insert_round(lossless: string, index: number, round: unknown): string;
  training_pack_remove_round(lossless: string, index: number): string;
  training_pack_move_round(lossless: string, from: number, to: number): string;
  training_pack_duplicate_round(lossless: string, index: number): string;
  training_pack_append_rounds(lossless: string, otherLossless: string): string;
  training_pack_round_archetypes(lossless: string, roundIndex: number): unknown;
  training_pack_set_round_archetype(
    lossless: string,
    roundIndex: number,
    archetypeIndex: number,
    archetype: unknown,
  ): string;
  training_pack_add_round_archetype(
    lossless: string,
    roundIndex: number,
    archetype: unknown,
  ): string;
  training_pack_remove_round_archetype(
    lossless: string,
    roundIndex: number,
    archetypeIndex: number,
  ): string;
  training_pack_set_round_ball(lossless: string, roundIndex: number, ball: unknown): string;
  training_pack_set_round_time_limit(
    lossless: string,
    roundIndex: number,
    timeLimit: number,
  ): string;
}

export interface TrainingPackFileOptions {
  /**
   * Explicit WASM bindings to use. When omitted, the `@rlrml/subtr-actor`
   * package is loaded and initialized on demand. Passing bindings explicitly
   * is useful in Node (e.g. a `pkg-node` build) or tests.
   */
  bindings?: TrainingPackBindings;
}

/** A fresh typed training pack with every field at its default value. */
export function defaultTrainingPack(): TrainingPack {
  return {
    guid: { a: 0, b: 0, c: 0, d: 0 },
    code: null,
    name: null,
    training_type: "Training_None",
    difficulty: "D_Easy",
    creator_name: null,
    description: null,
    tags: [],
    map_name: null,
    created_at: 0n,
    updated_at: 0n,
    creator_player_id: {
      uid: 0n,
      epic_account_id: null,
      platform: null,
      splitscreen_id: 0,
    },
    rounds: [],
    player_team_number: 0,
    unowned: false,
    perfect_completed: false,
    shots_completed: 0,
  };
}

/**
 * The ball of a freshly created editor round (centered in front of the
 * orange goal, lobbed toward it), mirroring the Rust `BallSpawn::default()`.
 */
export function defaultBallSpawn(): BallSpawn {
  return {
    start_location_x: 0,
    start_location_y: 4120,
    start_location_z: 100.4872,
    velocity_start_rotation_p: 8191,
    velocity_start_rotation_y: -16384,
    velocity_start_rotation_r: 0,
    velocity_start_speed: 1500,
    extras: {},
  };
}

/**
 * The car spawn point of a freshly created editor round (center field,
 * facing the orange goal), mirroring the Rust `CarSpawn::default()`.
 */
export function defaultCarSpawn(): CarSpawn {
  return {
    location_x: 0,
    location_y: 0,
    location_z: 30,
    rotation_p: 0,
    rotation_y: 16384,
    rotation_r: 0,
    velocity_start_speed: 0,
    extras: {},
  };
}

/**
 * A player car entry with a zeroed transform, mirroring the Rust
 * `PlayerCarSpawn::default()`.
 */
export function defaultPlayerCarSpawn(): PlayerCarSpawn {
  return {
    is_pc: true,
    location_x: 0,
    location_y: 0,
    location_z: 0,
    rotation_p: 0,
    rotation_y: 0,
    rotation_r: 0,
    extras: {},
  };
}

let defaultBindingsPromise: Promise<TrainingPackBindings> | null = null;

async function defaultTrainingPackBindings(): Promise<TrainingPackBindings> {
  if (!defaultBindingsPromise) {
    defaultBindingsPromise = (async () => {
      const module = await import("@rlrml/subtr-actor");
      const maybeInit = (module as { default?: (moduleOrPath?: unknown) => Promise<unknown> })
        .default;
      if (typeof maybeInit === "function") {
        await maybeInit();
      }
      return module as unknown as TrainingPackBindings;
    })();
  }
  return defaultBindingsPromise;
}

async function resolveBindings(options?: TrainingPackFileOptions): Promise<TrainingPackBindings> {
  return options?.bindings ?? defaultTrainingPackBindings();
}

function toUint8Array(data: Uint8Array | ArrayBuffer): Uint8Array {
  return data instanceof Uint8Array ? data : new Uint8Array(data);
}

function rethrowAsError<T>(run: () => T): T {
  try {
    return run();
  } catch (error) {
    throw error instanceof Error ? error : new Error(String(error));
  }
}

/**
 * A loaded Rocket League custom training pack (`.tem`) file.
 *
 * Internally holds the lossless property-tree JSON produced by the WASM
 * bindings and re-enters WASM for each mutation, so unknown properties in the
 * original file survive edits:
 *
 * - metadata setters write typed fields back into the existing tree in place;
 * - round remove/move/duplicate and {@link appendRoundsFrom} move whole round
 *   property lists (unknown per-round properties included);
 * - only newly added rounds are built from the typed {@link Round} shape,
 *   which is lossless because a new round has no unknown data.
 *
 * An untouched pack serializes back to byte-identical `.tem` output.
 *
 * 64-bit fields (`created_at`, `updated_at`, `creator_player_id.uid`) are
 * `bigint`, matching the generated {@link TrainingPack} type.
 */
export class TrainingPackFile {
  private lossless: string;
  private cachedPack: TrainingPack | null = null;

  private constructor(
    private readonly bindings: TrainingPackBindings,
    lossless: string,
  ) {
    this.lossless = lossless;
  }

  /** Load a `.tem` file from bytes. */
  static async load(
    data: Uint8Array | ArrayBuffer,
    options?: TrainingPackFileOptions,
  ): Promise<TrainingPackFile> {
    return TrainingPackFile.fromBytes(data, await resolveBindings(options));
  }

  /** Load a `.tem` file from bytes with explicit bindings, synchronously. */
  static fromBytes(data: Uint8Array | ArrayBuffer, bindings: TrainingPackBindings) {
    const lossless = rethrowAsError(() =>
      bindings.parse_training_pack_lossless(toUint8Array(data)),
    );
    return new TrainingPackFile(bindings, lossless);
  }

  /** Create a new pack from scratch; `pack` overrides the defaults. */
  static async create(
    pack?: Partial<TrainingPack>,
    options?: TrainingPackFileOptions,
  ): Promise<TrainingPackFile> {
    return TrainingPackFile.createWithBindings(await resolveBindings(options), pack);
  }

  /** Create a new pack from scratch with explicit bindings, synchronously. */
  static createWithBindings(bindings: TrainingPackBindings, pack?: Partial<TrainingPack>) {
    const typedPack: TrainingPack = { ...defaultTrainingPack(), ...pack };
    const lossless = rethrowAsError(() => bindings.new_training_pack(typedPack));
    return new TrainingPackFile(bindings, lossless);
  }

  /** Restore a pack from a previously captured {@link losslessJson} string. */
  static fromLosslessJson(lossless: string, bindings: TrainingPackBindings) {
    const file = new TrainingPackFile(bindings, lossless);
    file.view(); // validate eagerly
    return file;
  }

  private view(): TrainingPack {
    if (!this.cachedPack) {
      this.cachedPack = rethrowAsError(
        () => this.bindings.training_pack_from_lossless(this.lossless) as TrainingPack,
      );
    }
    return this.cachedPack;
  }

  private apply(edit: () => string): void {
    this.lossless = rethrowAsError(edit);
    this.cachedPack = null;
  }

  /**
   * The lossless property-tree JSON for this pack (an opaque token; see
   * {@link TrainingPackBindings}). Stable to store and later restore with
   * {@link TrainingPackFile.fromLosslessJson}.
   */
  get losslessJson(): string {
    return this.lossless;
  }

  /** The typed view of this pack (a defensive copy). */
  get pack(): TrainingPack {
    return structuredClone(this.view());
  }

  toJSON(): TrainingPack {
    return this.pack;
  }

  get name(): string | null {
    return this.view().name;
  }

  get code(): string | null {
    return this.view().code;
  }

  get description(): string | null {
    return this.view().description;
  }

  get creatorName(): string | null {
    return this.view().creator_name;
  }

  /** `ETrainingType` value name, e.g. `Training_Striker`. */
  get trainingType(): string {
    return this.view().training_type;
  }

  /** `EDifficulty` value name, e.g. `D_Medium`. */
  get difficulty(): string {
    return this.view().difficulty;
  }

  get mapName(): string | null {
    return this.view().map_name;
  }

  get guid(): Guid {
    return { ...this.view().guid };
  }

  get tags(): number[] {
    return [...this.view().tags];
  }

  get rounds(): Round[] {
    return structuredClone(this.view().rounds);
  }

  get roundCount(): number {
    return this.view().rounds.length;
  }

  /**
   * Write typed metadata fields back into the pack, preserving unknown
   * properties. `rounds` and `creator_player_id` in the patch are ignored:
   * rounds are edited through the round operations and the creator player id
   * is read-only.
   */
  updateMetadata(patch: Partial<Omit<TrainingPack, "rounds" | "creator_player_id">>): void {
    const next = { ...this.view(), ...patch };
    this.apply(() => this.bindings.update_training_pack_metadata(this.lossless, next));
  }

  setName(name: string | null): void {
    this.updateMetadata({ name });
  }

  setCode(code: string | null): void {
    this.updateMetadata({ code });
  }

  setDescription(description: string | null): void {
    this.updateMetadata({ description });
  }

  setCreatorName(creatorName: string | null): void {
    this.updateMetadata({ creator_name: creatorName });
  }

  /** Set the `ETrainingType` value name, e.g. `Training_Striker`. */
  setTrainingType(trainingType: string): void {
    this.updateMetadata({ training_type: trainingType });
  }

  /** Set the `EDifficulty` value name, e.g. `D_Medium`. */
  setDifficulty(difficulty: string): void {
    this.updateMetadata({ difficulty });
  }

  setMapName(mapName: string): void {
    this.updateMetadata({ map_name: mapName });
  }

  setTags(tags: number[]): void {
    this.updateMetadata({ tags });
  }

  setGuid(guid: Guid): void {
    this.updateMetadata({ guid });
  }

  /** Append a round. */
  addRound(round: Round): void {
    this.apply(() => this.bindings.training_pack_add_round(this.lossless, round));
  }

  /** Insert a round at `index` (clamped to the round count). */
  insertRound(index: number, round: Round): void {
    this.apply(() => this.bindings.training_pack_insert_round(this.lossless, index, round));
  }

  /** Remove the round at `index`, returning its typed view. */
  removeRound(index: number): Round {
    const removed = this.view().rounds[index];
    this.apply(() => this.bindings.training_pack_remove_round(this.lossless, index));
    return structuredClone(removed);
  }

  /** Move the round at `from` to position `to`. */
  moveRound(from: number, to: number): void {
    this.apply(() => this.bindings.training_pack_move_round(this.lossless, from, to));
  }

  /** Duplicate the round at `index`, inserting the copy right after it. */
  duplicateRound(index: number): void {
    this.apply(() => this.bindings.training_pack_duplicate_round(this.lossless, index));
  }

  /**
   * The parsed archetypes of the round at `roundIndex` (ball, car spawn
   * point, player car — or `kind: "Unknown"` carrying the raw string for
   * anything unrecognized). Parsing is on demand; the round keeps its
   * original strings until an archetype is edited.
   */
  getRoundArchetypes(roundIndex: number): Archetype[] {
    return rethrowAsError(
      () => this.bindings.training_pack_round_archetypes(this.lossless, roundIndex) as Archetype[],
    );
  }

  /**
   * Replace the archetype at `archetypeIndex` of round `roundIndex`. Only
   * that one serialized string is regenerated; all other strings in the
   * file stay byte-identical.
   */
  setRoundArchetype(roundIndex: number, archetypeIndex: number, archetype: Archetype): void {
    this.apply(() =>
      this.bindings.training_pack_set_round_archetype(
        this.lossless,
        roundIndex,
        archetypeIndex,
        archetype,
      ),
    );
  }

  /** Append an archetype to the round at `roundIndex`. */
  addRoundArchetype(roundIndex: number, archetype: Archetype): void {
    this.apply(() =>
      this.bindings.training_pack_add_round_archetype(this.lossless, roundIndex, archetype),
    );
  }

  /**
   * Remove the archetype at `archetypeIndex` of round `roundIndex`,
   * returning its parsed view.
   */
  removeRoundArchetype(roundIndex: number, archetypeIndex: number): Archetype {
    const removed = this.getRoundArchetypes(roundIndex)[archetypeIndex];
    this.apply(() =>
      this.bindings.training_pack_remove_round_archetype(this.lossless, roundIndex, archetypeIndex),
    );
    return removed;
  }

  /**
   * Set the ball of the round at `roundIndex`: replaces the round's first
   * ball archetype, or inserts one at position 0 if the round has none.
   */
  setRoundBall(roundIndex: number, ball: BallSpawn): void {
    this.apply(() => this.bindings.training_pack_set_round_ball(this.lossless, roundIndex, ball));
  }

  /** Append a car spawn point to the round at `roundIndex`. */
  addRoundCar(roundIndex: number, car: CarSpawn): void {
    this.addRoundArchetype(roundIndex, { kind: "CarSpawnPoint", ...car });
  }

  /**
   * Set the time limit of the round at `roundIndex` in place (0 removes the
   * property, matching the game's omit-default convention).
   */
  setRoundTimeLimit(roundIndex: number, timeLimit: number): void {
    this.apply(() =>
      this.bindings.training_pack_set_round_time_limit(this.lossless, roundIndex, timeLimit),
    );
  }

  /**
   * Append every round of `other`, preserving unknown per-round properties.
   * Returns the number of rounds appended.
   */
  appendRoundsFrom(other: TrainingPackFile): number {
    const appended = other.view().rounds.length;
    this.apply(() => this.bindings.training_pack_append_rounds(this.lossless, other.lossless));
    return appended;
  }

  /** Serialize back to encrypted `.tem` bytes. */
  toBytes(): Uint8Array {
    return rethrowAsError(() => this.bindings.serialize_training_pack(this.lossless));
  }
}
