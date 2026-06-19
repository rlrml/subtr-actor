/**
 * SubtrActorPlayer — a `Player`-compatible facade backed by subtr-actor.
 *
 * Ballcam's three.js renderer (GameEngine + managers) only ever talks to one
 * object: the `Player` from framework/dist/Player.js. This class implements the
 * subset of that interface the renderer actually reads (see INTEGRATION.md), but
 * sourced from subtr-actor's raw ReplayData instead of ballcam's JS boxcars
 * compilers.
 *
 * Two data channels feed the renderer (confirmed in ActorManager.updateFromFramework):
 *   1. getTimelines()  -> { ballTimeline, playerTimelines } — drives smooth
 *      position/rotation interpolation (PRIMARY motion source).
 *   2. live entities (ball, getAllPlayers()) — updated on seek(); supply
 *      velocity / sleeping / visible / boost each frame.
 *
 * v0 scope: ball + cars move correctly. Analytics getters are stubbed empty;
 * game-phase / gap-removal / boost-pad polish are deferred (see INTEGRATION.md).
 */
import EventEmitter from "../util/EventEmitter.js";
import { vec3RlToThree, quatRlToThree, boostToPercent, type Vec3, type Quat } from "./coords.js";
import { getCarHitboxInfo } from "../data/hitboxes.js";

// ── Minimal view of subtr-actor's raw ReplayData (ts-rs). Loosely typed on
//    purpose: the ts-rs union types (BallFrame = "Empty" | {Data:{...}}) are
//    awkward to consume directly; we narrow defensively at runtime instead.
interface RawRigidBody {
  sleeping?: boolean;
  location: { x: number; y: number; z: number };
  rotation: { x: number; y: number; z: number; w: number };
  linear_velocity?: { x: number; y: number; z: number } | null;
  angular_velocity?: { x: number; y: number; z: number } | null;
}
type RawBallFrame = "Empty" | { Data: { rigid_body: RawRigidBody } };
type RawPlayerFrame =
  | "Empty"
  | {
      Data: {
        rigid_body: RawRigidBody;
        boost_amount: number;
        boost_active: boolean;
        player_name: string | null;
        team: number | null;
        is_team_0: boolean | null;
        camera?: {
          pitch?: number | null;
          yaw?: number | null;
        };
        input?: {
          throttle?: number | null;
          steer?: number | null;
          dodge_impulse?: [number, number, number] | null;
          dodge_torque?: [number, number, number] | null;
        };
      };
    };
type RawBoostPadEventKind = "Available" | { PickedUp: { sequence: number } };
interface RawBoostPadEvent {
  time: number;
  frame: number;
  pad_id: string;
  kind: RawBoostPadEventKind;
}
interface RawResolvedBoostPad {
  index: number;
  pad_id: string | null;
  size: "Big" | "Small";
  position: { x: number; y: number; z: number };
}
interface RawReplayData {
  frame_data: {
    ball_data: { frames: RawBallFrame[] };
    players: Array<[unknown, { frames: RawPlayerFrame[] }]>;
    metadata_frames: Array<{
      time: number;
      seconds_remaining: number;
      replicated_game_state_name?: number;
      replicated_game_state_time_remaining?: number;
    }>;
  };
  meta: {
    team_zero: RawPlayerInfo[];
    team_one: RawPlayerInfo[];
  };
  boost_pads?: RawResolvedBoostPad[];
  boost_pad_events?: RawBoostPadEvent[];
  goal_events?: Array<{ time: number; frame: number }>;
  player_camera_events?: Array<
    [
      unknown,
      Array<{
        frame: number;
        ball_cam_active: boolean | null;
        behind_view_active: boolean | null;
        driving: boolean | null;
      }>,
    ]
  >;
}
interface RawPlayerInfo {
  remote_id: unknown;
  name: string;
  car_body_id?: number | null;
  car_body_name?: string | null;
  car_hitbox_family?: string | null;
  camera_settings?: RawCameraSettings | null;
}
/** subtr-actor's PlayerCameraSettings (snake_case, RL menu units). */
interface RawCameraSettings {
  fov: number;
  height: number;
  angle: number;
  distance: number;
  stiffness: number;
  swivel_speed: number;
  transition_speed?: number | null;
}

// ── Timeline keyframe shapes the renderer expects (ballcam THREE space).
export interface MotionKeyframe {
  time: number;
  frame: number;
  position: Vec3;
  rotation: Quat | null;
  velocity: Vec3;
  angularVelocity?: Vec3 | null;
  sleeping: boolean;
}
interface FlagsKeyframe {
  time: number;
  boost: number; // 0-100
  isBoosting: boolean;
  present: boolean;
  /** Normalized steer input (-1 left .. 1 right); 0 when not replicated. */
  steer: number;
}

/** A coalesced ball-cam change for a player, resolved via last-before on seek. */
interface CameraEventKeyframe {
  time: number;
  ballCam: boolean | null;
}

export interface ReplayPlayerInfo {
  /** Stable player id derived from the replay's remote id (Steam/Epic/…). */
  id: string;
  name: string;
  team: number;
  carName: string;
  hitboxType: string;
  loadout?: undefined;
  /** The player's recorded RL camera preset, when the replay carries one. */
  cameraSettings: RecordedCameraSettings | null;
}

/**
 * A player's recorded Rocket League camera preset (in-game menu units; `fov`
 * is the HORIZONTAL field of view). Key names match the camera plugin's
 * `CameraSettings` so a recorded preset can be applied directly.
 */
export interface RecordedCameraSettings {
  fov: number;
  height: number;
  angle: number;
  distance: number;
  stiffness: number;
  swivelSpeed: number;
  transitionSpeed?: number;
}

function toRecordedCameraSettings(
  raw: RawCameraSettings | null | undefined,
): RecordedCameraSettings | null {
  if (!raw) return null;
  const settings: RecordedCameraSettings = {
    fov: raw.fov,
    height: raw.height,
    angle: raw.angle,
    distance: raw.distance,
    stiffness: raw.stiffness,
    swivelSpeed: raw.swivel_speed,
  };
  // Only set when present so spreading a recorded preset over defaults never
  // clobbers transitionSpeed with `undefined`.
  if (raw.transition_speed != null) settings.transitionSpeed = raw.transition_speed;
  return settings;
}

/** Supersonic threshold in UU/s (game value ~2200). */
const SUPERSONIC_SPEED = 2200;
const DEFAULT_POSITION_SMOOTHING = true;
const DEFAULT_TIMELINE_COMPACTION = false;
const POSITION_SMOOTHING_BLEND_FACTOR = 0.15;
const POSITION_SMOOTHING_ANCHOR_INTERVAL = 10;
const MAX_POSITION_CORRECTION_DT_SECONDS = 0.1;
const MAX_POSITION_CORRECTION_DRIFT_UU = 10;
const FILTER_VELOCITY_THRESHOLD = 0.1;
const FILTER_POSITION_THRESHOLD = 0.15;
const MIN_FILTER_SPEED_UU_PER_SECOND = 10;

export interface SubtrActorPlayerOptions {
  /**
   * Preprocess compiled ball/player timelines with the same style of
   * velocity-based correction Ballcam applies before serializing its replay
   * artifact. Defaults to true; set false for raw sample inspection.
   */
  motionSmoothing?: boolean;
  /** Blend toward the measured replay sample during velocity correction. */
  smoothingBlendFactor?: number;
  /** Every N corrected samples, use a stronger measured-sample anchor. */
  smoothingAnchorInterval?: number;
  /**
   * Remove pre-kickoff idle time and post-goal replay gaps from the adapter's
   * motion timelines, matching Ballcam's compiled .rlrf time axis. Defaults to
   * false because it intentionally diverges from @rlrml/player's raw normalized
   * ReplayModel time axis.
   */
  timelineCompaction?: boolean;
  /** Skip Ballcam-style velocity/position consistency filtering. */
  disableFrameFiltering?: boolean;
}

interface TimelineProcessingOptions {
  motionSmoothing: boolean;
  smoothingBlendFactor: number;
  smoothingAnchorInterval: number;
  timelineCompaction: boolean;
  disableFrameFiltering: boolean;
}

interface ReplayGap {
  beforeFrame: number;
  afterFrame: number;
  beforeTime: number;
  afterTime: number;
  duration: number;
}

interface TimelineCompaction {
  gaps: ReplayGap[];
  prematchEndTime: number | null;
  removedDuration: number;
  compactedDuration: number;
}

function lastBefore<T extends { time: number }>(arr: T[], time: number): T | null {
  if (arr.length === 0) return null;
  let lo = 0;
  let hi = arr.length - 1;
  if (time <= arr[0].time) return arr[0];
  if (time >= arr[hi].time) return arr[hi];
  while (lo < hi) {
    const mid = (lo + hi + 1) >> 1;
    if (arr[mid].time <= time) lo = mid;
    else hi = mid - 1;
  }
  return arr[lo];
}

/** Live mutable entity read by the renderer each frame after seek(). */
class BallEntity {
  position: Vec3 = { x: 0, y: 0, z: 0 };
  rotation: Quat = { x: 0, y: 0, z: 0, w: 1 };
  velocity: Vec3 = { x: 0, y: 0, z: 0 };
  angularVelocity: Vec3 = { x: 0, y: 0, z: 0 };
  sleeping = false;
  visible = true;
}

/**
 * Boost pad in the exact shape the original GameEngine read off framework's
 * Player.boostPads: position in raw Unreal coords (the renderer does its own
 * Y/Z swap at the mesh level) + live `isAvailable` updated on seek().
 */
export class BoostPadEntity {
  isAvailable = true;
  constructor(
    public isBig: boolean,
    /** Unreal coords: x, y = along field length, z = height. */
    public position: Vec3,
    /** Sorted availability timeline compiled from boost_pad_events. */
    public events: Array<{ time: number; available: boolean }>,
  ) {}
}

class PlayerEntity extends EventEmitter {
  position: Vec3 = { x: 0, y: 0, z: 0 };
  rotation: Quat = { x: 0, y: 0, z: 0, w: 1 };
  velocity: Vec3 = { x: 0, y: 0, z: 0 };
  angularVelocity: Vec3 = { x: 0, y: 0, z: 0 };
  sleeping = false;
  steer = 0;
  boost = 0; // 0-100
  isBoosting = false;
  isSupersonic = false;
  /** True while boost is being reset for a kickoff (suppresses boost particles). */
  isKickoffReset = false;
  isVisible = true;
  isBallCam = true;
  constructor(
    /** Stable player id derived from the replay's remote id. */
    public id: string,
    public name: string,
    public team: number,
    public carName: string,
    public hitboxType: string,
    /** The player's recorded RL camera preset, when the replay carries one. */
    public cameraSettings: RecordedCameraSettings | null = null,
  ) {
    super();
  }
}

export class SubtrActorPlayer extends EventEmitter {
  duration = 0;
  playerList: ReplayPlayerInfo[] = [];
  /** Monotonic per-frame timestamps (s) — the replay's frame timeline. */
  frameTimes: number[] = [];
  /**
   * The raw replay clock value at the first frame. All adapter times (frame
   * timeline, boost-pad events, duration) are shifted by this so t=0 is the
   * first frame — matching @rlrml/player's ReplayModel time axis exactly.
   */
  rawStartTime = 0;
  ball = new BallEntity();
  players = new Map<string, PlayerEntity>();
  boostPads = new Map<number, BoostPadEntity>();

  private _currentTime = 0;
  private _ballTimeline: MotionKeyframe[] = [];
  private _playerTimelines: Record<string, MotionKeyframe[]> = {};
  private _ballFlags: FlagsKeyframe[] = []; // ball has none, kept for symmetry
  private _playerFlags: Record<string, FlagsKeyframe[]> = {};
  /** Coalesced ball-cam change timeline per player name (last-before on seek). */
  private _playerCameraEvents: Record<string, CameraEventKeyframe[]> = {};
  private _teams: Record<string, number> = {};
  private _timelineCompaction: TimelineCompaction | null = null;

  constructor(
    private raw: RawReplayData,
    private options: SubtrActorPlayerOptions = {},
  ) {
    super();
    this._compile();
  }

  // ── Compilation: raw ReplayData -> ballcam-space timelines + entities. ──────
  private _compile(): void {
    const fd = this.raw.frame_data;
    const meta = this.raw.meta;
    const metaFrames = fd.metadata_frames;
    // Replay clocks don't start at 0 (the raw first-frame time is several
    // seconds in). Shift everything so t=0 is the first frame — the exact
    // normalization @rlrml/player's normalizeReplayData applies
    // (normalizeReplayTime = max(0, raw - startTime)) — so player playback
    // time is directly comparable to ReplayModel times (PLAYER_PARITY Phase 2).
    const startTime = metaFrames[0]?.time ?? 0;
    this.rawStartTime = startTime;
    const t = (rawTime: number) => Math.max(0, rawTime - startTime);
    this.duration = metaFrames.length ? t(metaFrames[metaFrames.length - 1].time) : 0;
    this.frameTimes = metaFrames.map((f) => t(f.time));

    // remote_id -> { name, team, car } lookup from meta roster
    const infoByKey = new Map<string, { info: RawPlayerInfo; team: number }>();
    meta.team_zero.forEach((p) => infoByKey.set(this._idKey(p.remote_id), { info: p, team: 0 }));
    meta.team_one.forEach((p) => infoByKey.set(this._idKey(p.remote_id), { info: p, team: 1 }));

    // Ball motion timeline
    fd.ball_data.frames.forEach((f, i) => {
      if (f === "Empty" || !("Data" in f)) return;
      const mk = this._rbToKeyframe(f.Data.rigid_body, t(metaFrames[i]?.time ?? startTime), i);
      if (mk) this._ballTimeline.push(mk);
    });

    // Per-player motion + flags timelines
    fd.players.forEach(([remoteId, pdata]) => {
      const key = this._idKey(remoteId);
      const matched = infoByKey.get(key);
      // Resolve a display name even if roster lookup misses (use first frame's name)
      let name = matched?.info.name ?? null;
      let team = matched?.team ?? 0;
      if (!name) {
        for (const f of pdata.frames) {
          if (f !== "Empty" && "Data" in f && f.Data.player_name) {
            name = f.Data.player_name;
            if (f.Data.is_team_0 != null) team = f.Data.is_team_0 ? 0 : 1;
            break;
          }
        }
      }
      if (!name) name = `Player_${key}`;

      // Prefer subtr-actor's resolved fields; fall back to the body-id table when
      // the replay header omits the body name (it often carries only car_body_id).
      const info = matched?.info;
      const byId = info?.car_body_id != null ? getCarHitboxInfo(info.car_body_id) : null;
      const carName = info?.car_body_name ?? byId?.name ?? "Octane";
      const hitboxType = info?.car_hitbox_family ?? byId?.hitboxType ?? "Octane";

      const motion: MotionKeyframe[] = [];
      const flags: FlagsKeyframe[] = [];
      pdata.frames.forEach((f, i) => {
        const time = t(metaFrames[i]?.time ?? startTime);
        if (f === "Empty" || !("Data" in f)) return;
        const mk = this._rbToKeyframe(f.Data.rigid_body, time, i);
        if (mk) motion.push(mk);
        const rawSteer = f.Data.input?.steer;
        flags.push({
          time,
          boost: boostToPercent(f.Data.boost_amount ?? 0),
          isBoosting: !!f.Data.boost_active,
          present: true,
          // ReplicatedSteer is a byte (~128 neutral); normalize to -1..1 for
          // the renderer's wheel steering (ActorManager scales by max angle).
          steer: rawSteer == null ? 0 : Math.max(-1, Math.min(1, (rawSteer - 128) / 128)),
        });
      });

      const cameraSettings = toRecordedCameraSettings(info?.camera_settings);

      this._playerTimelines[name] = motion;
      this._playerFlags[name] = flags;
      this._teams[name] = team;
      this.playerList.push({ id: key, name, team, carName, hitboxType, cameraSettings });
      this.players.set(
        name,
        new PlayerEntity(key, name, team, carName, hitboxType, cameraSettings),
      );
    });

    // Coalesced ball-cam changes arrive grouped by remote id; map them onto the
    // display names the rest of the adapter uses, deriving each change's time
    // from its frame, so seek() can resolve ball-cam state via last-before.
    const nameByKey = new Map<string, string>();
    this.playerList.forEach((player) => nameByKey.set(player.id, player.name));
    for (const [player, changes] of this.raw.player_camera_events ?? []) {
      const name = nameByKey.get(this._idKey(player));
      if (!name) continue;
      this._playerCameraEvents[name] = changes.map((change) => ({
        time: t(metaFrames[change.frame]?.time ?? startTime),
        ballCam: change.ball_cam_active,
      }));
    }

    this._preprocessMotionTimelines();
    this._compileBoostPads();

    this.seek(0);
  }

  private _timelineProcessingOptions(): TimelineProcessingOptions {
    return {
      motionSmoothing: this.options.motionSmoothing ?? DEFAULT_POSITION_SMOOTHING,
      smoothingBlendFactor: this.options.smoothingBlendFactor ?? POSITION_SMOOTHING_BLEND_FACTOR,
      smoothingAnchorInterval:
        this.options.smoothingAnchorInterval ?? POSITION_SMOOTHING_ANCHOR_INTERVAL,
      timelineCompaction: this.options.timelineCompaction ?? DEFAULT_TIMELINE_COMPACTION,
      disableFrameFiltering: this.options.disableFrameFiltering ?? false,
    };
  }

  private _preprocessMotionTimelines(): void {
    const options = this._timelineProcessingOptions();
    if (options.motionSmoothing) {
      this._applyVelocityBasedPositionCorrection(options);
    }
    if (options.timelineCompaction) {
      this._applyTimelineCompaction();
    }
    if (!options.disableFrameFiltering) {
      this._filterInconsistentFrames();
    }
  }

  private _applyTimelineCompaction(): void {
    const compaction = this._buildTimelineCompaction();
    if (!compaction || (compaction.gaps.length === 0 && compaction.prematchEndTime === null)) {
      return;
    }

    this._timelineCompaction = compaction;
    this._ballTimeline = this._compactTimeline(this._ballTimeline, compaction);
    Object.entries(this._playerTimelines).forEach(([name, timeline]) => {
      this._playerTimelines[name] = this._compactTimeline(timeline, compaction);
    });
    Object.entries(this._playerFlags).forEach(([name, timeline]) => {
      this._playerFlags[name] = this._compactTimeline(timeline, compaction);
    });

    this.frameTimes = this.frameTimes.map((time) => this._compactTime(time, compaction));
    this.duration = compaction.compactedDuration;
  }

  private _buildTimelineCompaction(): TimelineCompaction | null {
    if (this.frameTimes.length === 0) return null;

    const gaps = this._detectPostGoalTimeGaps();
    const prematchRawEndTime = this._detectFirstKickoffGoTime();
    const prematchEndTime =
      prematchRawEndTime == null ? null : remapGapTime(prematchRawEndTime, gaps);
    const gapRemovedDuration = gaps.reduce((total, gap) => total + gap.duration, 0);
    const removedDuration = gapRemovedDuration + (prematchEndTime ?? 0);
    if (removedDuration <= 0) return null;

    return {
      gaps,
      prematchEndTime,
      removedDuration,
      compactedDuration: Math.max(0, this.duration - removedDuration),
    };
  }

  private _detectPostGoalTimeGaps(): ReplayGap[] {
    const gaps: ReplayGap[] = [];
    for (const goal of this.raw.goal_events ?? []) {
      const goalFrame = goal.frame;
      if (!Number.isInteger(goalFrame) || goalFrame < 0 || goalFrame >= this.frameTimes.length) {
        continue;
      }
      const goalTime = this.frameTimes[goalFrame]!;
      for (let frame = goalFrame + 1; frame < this.frameTimes.length; frame += 1) {
        const beforeTime = this.frameTimes[frame - 1]!;
        const afterTime = this.frameTimes[frame]!;
        if (beforeTime - goalTime > 10) break;
        const duration = afterTime - beforeTime;
        if (duration > 0.3) {
          gaps.push({
            beforeFrame: frame - 1,
            afterFrame: frame,
            beforeTime,
            afterTime,
            duration,
          });
          break;
        }
      }
    }
    return gaps;
  }

  private _detectFirstKickoffGoTime(): number | null {
    const frames = this.raw.frame_data.metadata_frames;
    let sawCountdown = false;
    for (let index = 0; index < frames.length; index += 1) {
      const remaining = frames[index]?.replicated_game_state_time_remaining;
      if (remaining != null && remaining > 0) sawCountdown = true;
      if (sawCountdown && remaining === 0) return this.frameTimes[index] ?? null;
    }

    const firstActiveFrame = frames.findIndex((frame) => frame.replicated_game_state_name === 54);
    return firstActiveFrame === -1 ? null : (this.frameTimes[firstActiveFrame] ?? null);
  }

  private _compactTimeline<T extends { time: number }>(
    timeline: T[],
    compaction: TimelineCompaction,
  ): T[] {
    const afterGaps = this._remapReplayGaps(timeline, compaction.gaps);
    if (compaction.prematchEndTime === null) return afterGaps;
    return this._remapPrematch(afterGaps, compaction.prematchEndTime);
  }

  private _remapReplayGaps<T extends { time: number }>(timeline: T[], gaps: ReplayGap[]): T[] {
    if (gaps.length === 0) return timeline;

    const inserted: T[] = [];
    gaps.forEach((gap, gapIndex) => {
      const entry = timeline.find((frame) => frame.time >= gap.afterTime);
      if (!entry) return;
      inserted.push({
        ...entry,
        time: remapGapTime(gap.afterTime, gaps.slice(0, gapIndex + 1)),
      });
    });

    const remapped = timeline
      .filter((frame) => !isInReplayGap(frame.time, gaps))
      .map((frame) => ({ ...frame, time: remapGapTime(frame.time, gaps) }));

    for (const entry of inserted) {
      if (remapped.some((frame) => Math.abs(frame.time - entry.time) < 1e-3)) continue;
      let insertAt = remapped.findIndex((frame) => frame.time > entry.time);
      if (insertAt === -1) insertAt = remapped.length;
      remapped.splice(insertAt, 0, entry);
    }

    return remapped;
  }

  private _remapPrematch<T extends { time: number }>(timeline: T[], prematchEndTime: number): T[] {
    let lastPrematchFrame: T | null = null;
    for (const frame of timeline) {
      if (frame.time < prematchEndTime) lastPrematchFrame = frame;
      else break;
    }

    const remapped = timeline
      .filter((frame) => frame.time >= prematchEndTime)
      .map((frame) => ({ ...frame, time: frame.time - prematchEndTime }));

    if (lastPrematchFrame && (remapped.length === 0 || remapped[0]!.time > 1e-3)) {
      remapped.unshift({ ...lastPrematchFrame, time: 0 });
    }

    return remapped;
  }

  private _compactTime(time: number, compaction: TimelineCompaction): number {
    const afterGaps = remapGapTime(time, compaction.gaps);
    if (compaction.prematchEndTime === null) return afterGaps;
    return Math.max(0, afterGaps - compaction.prematchEndTime);
  }

  private _applyVelocityBasedPositionCorrection(options: TimelineProcessingOptions): void {
    const correctTimeline = (timeline: MotionKeyframe[]): void => {
      if (timeline.length < 3) return;

      let startIndex = 0;
      while (
        startIndex < timeline.length &&
        (!timeline[startIndex].position || !timeline[startIndex].velocity)
      ) {
        startIndex += 1;
      }
      if (startIndex >= timeline.length - 1) return;

      let smoothed = { ...timeline[startIndex].position };

      for (let index = startIndex + 1; index < timeline.length; index += 1) {
        const previous = timeline[index - 1]!;
        const current = timeline[index]!;
        if (!previous.position || !current.position) continue;
        if (!previous.velocity || !current.velocity) {
          smoothed = { ...current.position };
          continue;
        }

        const dt = current.time - previous.time;
        if (dt <= 0 || dt > MAX_POSITION_CORRECTION_DT_SECONDS) {
          smoothed = { ...current.position };
          continue;
        }

        if (distance(smoothed, current.position) > MAX_POSITION_CORRECTION_DRIFT_UU) {
          smoothed = { ...current.position };
          continue;
        }

        const averageVelocity = {
          x: (previous.velocity.x + current.velocity.x) / 2,
          y: (previous.velocity.y + current.velocity.y) / 2,
          z: (previous.velocity.z + current.velocity.z) / 2,
        };
        const predicted = {
          x: smoothed.x + averageVelocity.x * dt,
          y: smoothed.y + averageVelocity.y * dt,
          z: smoothed.z + averageVelocity.z * dt,
        };
        const blend =
          (index - startIndex) % options.smoothingAnchorInterval === 0
            ? 0.5
            : options.smoothingBlendFactor;

        smoothed = {
          x: predicted.x * (1 - blend) + current.position.x * blend,
          y: predicted.y * (1 - blend) + current.position.y * blend,
          z: predicted.z * (1 - blend) + current.position.z * blend,
        };
        current.position = { ...smoothed };
      }
    };

    correctTimeline(this._ballTimeline);
    Object.values(this._playerTimelines).forEach(correctTimeline);
  }

  private _filterInconsistentFrames(): void {
    this._ballTimeline = this._filterInconsistentTimeline(this._ballTimeline);
    Object.entries(this._playerTimelines).forEach(([name, timeline]) => {
      this._playerTimelines[name] = this._filterInconsistentTimeline(timeline);
    });
  }

  private _filterInconsistentTimeline(timeline: MotionKeyframe[]): MotionKeyframe[] {
    if (timeline.length < 2) return timeline;

    const filtered = [timeline[0]!];
    let lastKeptIndex = 0;

    for (let index = 1; index < timeline.length; index += 1) {
      const current = timeline[index]!;
      const previous = timeline[lastKeptIndex]!;
      if (!current.position || !current.velocity || !previous.position || !previous.velocity) {
        filtered.push(current);
        lastKeptIndex = index;
        continue;
      }

      const previousSpeed = magnitude(previous.velocity);
      const currentSpeed = magnitude(current.velocity);
      if (previousSpeed < MIN_FILTER_SPEED_UU_PER_SECOND) {
        filtered.push(current);
        lastKeptIndex = index;
        continue;
      }

      if (Math.abs(currentSpeed - previousSpeed) / previousSpeed < FILTER_VELOCITY_THRESHOLD) {
        const dt = current.time - previous.time;
        if (dt > 0.001) {
          const expectedDistance = previousSpeed * dt;
          const actualDistance = distance(previous.position, current.position);
          const positionError = Math.abs(actualDistance - expectedDistance) / expectedDistance;
          if (Number.isFinite(positionError) && positionError > FILTER_POSITION_THRESHOLD) {
            continue;
          }
        }
      }

      filtered.push(current);
      lastKeptIndex = index;
    }

    return filtered;
  }

  /**
   * subtr-actor resolves the standard soccar pad layout (with replay pad ids
   * when known) and emits exact pickup/availability events; fold the events
   * into per-pad timelines so seek() can resolve `isAvailable` at any time.
   */
  private _compileBoostPads(): void {
    const eventsByPadId = new Map<string, Array<{ time: number; available: boolean }>>();
    (this.raw.boost_pad_events ?? []).forEach((e) => {
      const available =
        e.kind === "Available"
          ? true
          : e.kind && typeof e.kind === "object" && "PickedUp" in e.kind
            ? false
            : null;
      if (available === null) return;
      const time = Math.max(0, e.time - this.rawStartTime); // same shift as frame times
      if (this._timelineCompaction && this._isRemovedByTimelineCompaction(time)) return;
      const compactedTime = this._timelineCompaction
        ? this._compactTime(time, this._timelineCompaction)
        : time;
      const bucket = eventsByPadId.get(e.pad_id);
      if (bucket) bucket.push({ time: compactedTime, available });
      else eventsByPadId.set(e.pad_id, [{ time: compactedTime, available }]);
    });

    (this.raw.boost_pads ?? []).forEach((pad) => {
      const events = (pad.pad_id ? eventsByPadId.get(pad.pad_id) : undefined) ?? [];
      events.sort((a, b) => a.time - b.time);
      this.boostPads.set(pad.index, new BoostPadEntity(pad.size === "Big", pad.position, events));
    });
  }

  private _rbToKeyframe(rb: RawRigidBody, time: number, frame: number): MotionKeyframe | null {
    const position = vec3RlToThree(rb.location);
    if (!position) return null;
    return {
      time,
      frame,
      position,
      rotation: quatRlToThree(rb.rotation),
      velocity: vec3RlToThree(rb.linear_velocity) ?? { x: 0, y: 0, z: 0 },
      angularVelocity: vec3RlToThree(rb.angular_velocity),
      sleeping: !!rb.sleeping,
    };
  }

  private _isRemovedByTimelineCompaction(time: number): boolean {
    const compaction = this._timelineCompaction;
    if (!compaction) return false;
    if (isInReplayGap(time, compaction.gaps)) return true;
    const afterGaps = remapGapTime(time, compaction.gaps);
    return compaction.prematchEndTime !== null && afterGaps < compaction.prematchEndTime;
  }

  private _idKey(remoteId: unknown): string {
    // RemoteIdTs is a tagged union ({Steam: "..."} etc.) or scalar. Mirrors
    // @rlrml/player's playerIdToString (replay-data-helpers.ts) so adapter ids
    // are byte-identical to ReplayModel `players[].id` — required for the
    // shared data layer (docs/player/PLAYER_PARITY.md Phase 2); validate.mts
    // cross-checks the two id sets to catch drift.
    if (typeof remoteId === "string" || typeof remoteId === "number") return String(remoteId);
    if (remoteId && typeof remoteId === "object") {
      const [kind, value] = Object.entries(remoteId)[0] ?? ["Unknown", "unknown"];
      if (typeof value === "string" || typeof value === "number") return `${kind}:${value}`;
      return `${kind}:${JSON.stringify(value)}`;
    }
    return JSON.stringify(remoteId);
  }

  // ── Renderer-facing API ────────────────────────────────────────────────────
  getTimelines(): {
    ballTimeline: MotionKeyframe[];
    playerTimelines: Record<string, MotionKeyframe[]>;
  } {
    return { ballTimeline: this._ballTimeline, playerTimelines: this._playerTimelines };
  }

  get currentTime(): number {
    return this._currentTime;
  }

  seek(time: number): void {
    this._currentTime = Math.max(0, Math.min(time, this.duration));
    this._updateEntities(this._currentTime);
    this.emit("seek", this._currentTime);
  }

  private _updateEntities(time: number): void {
    const b = lastBefore(this._ballTimeline, time);
    if (b) {
      this.ball.position = b.position;
      this.ball.rotation = b.rotation ?? this.ball.rotation;
      this.ball.velocity = b.velocity;
      this.ball.angularVelocity = b.angularVelocity ?? { x: 0, y: 0, z: 0 };
      this.ball.sleeping = b.sleeping;
      this.ball.visible = true;
    }
    for (const [name, entity] of this.players) {
      const m = lastBefore(this._playerTimelines[name] ?? [], time);
      if (m) {
        entity.position = m.position;
        entity.rotation = m.rotation ?? entity.rotation;
        entity.velocity = m.velocity;
        entity.angularVelocity = m.angularVelocity ?? { x: 0, y: 0, z: 0 };
        entity.sleeping = m.sleeping;
        const v = m.velocity;
        entity.isSupersonic = Math.hypot(v.x, v.y, v.z) >= SUPERSONIC_SPEED;
      }
      const fl = lastBefore(this._playerFlags[name] ?? [], time);
      if (fl) {
        entity.boost = fl.boost;
        entity.isBoosting = fl.isBoosting;
        // Replay-driven wheel steering (ActorManager reads entity.steer).
        entity.steer = fl.steer;
      }
      // Replay-driven ball cam from the coalesced event stream; keep the
      // previous value when the replay never replicated it for this player.
      const cameraEvent = lastBefore(this._playerCameraEvents[name] ?? [], time);
      if (cameraEvent && cameraEvent.ballCam != null) {
        entity.isBallCam = cameraEvent.ballCam;
      }
      // Presence: visible while we have motion data near `time`. A demolished
      // car produces Empty frames -> no nearby keyframe -> hidden.
      const tl = this._playerTimelines[name] ?? [];
      entity.isVisible =
        tl.length > 0 && time >= tl[0].time - 0.001 && time <= tl[tl.length - 1].time + 1.0;
    }
    for (const pad of this.boostPads.values()) {
      if (pad.events.length === 0) continue; // no events recorded -> always available
      const e = lastBefore(pad.events, time);
      // Before the first event the pad is in its initial (available) state.
      pad.isAvailable = e && e.time <= time ? e.available : true;
    }
  }

  /** Index of the last frame at or before `time` (binary search over frameTimes). */
  frameIndexAt(time: number): number {
    const times = this.frameTimes;
    if (times.length === 0) return 0;
    if (time <= times[0]) return 0;
    let lo = 0;
    let hi = times.length - 1;
    if (time >= times[hi]) return hi;
    while (lo < hi) {
      const mid = (lo + hi + 1) >> 1;
      if (times[mid] <= time) lo = mid;
      else hi = mid - 1;
    }
    return lo;
  }

  getBall(): BallEntity {
    return this.ball;
  }
  getPlayer(name: string): PlayerEntity | undefined {
    return this.players.get(name);
  }
  getPlayerById(id: string): PlayerEntity | undefined {
    for (const entity of this.players.values()) {
      if (entity.id === id) return entity;
    }
    return undefined;
  }
  getAllPlayers(): PlayerEntity[] {
    return Array.from(this.players.values());
  }
  getPlayerTeams(): Record<string, number> {
    return { ...this._teams };
  }

  // ── Analytics / overlay getters: stubbed empty for v0 (renderer guards them).
  getGameTimeMap(): unknown[] {
    return [];
  }
  getCountdownEvents(): unknown[] {
    return [];
  }
  getPlayerStatsTimelines(): Record<string, unknown[]> {
    return {};
  }
  getGameEventTimeline(): unknown[] {
    return [];
  }
  getAdvancedStats(): null {
    return null;
  }
  getEvents(): unknown[] {
    return [];
  }
  getEventsInRange(): unknown[] {
    return [];
  }
  getTextOverlaysAt(): unknown[] {
    return [];
  }
  getGamePhaseAt(): null {
    return null;
  }
}

function magnitude(vector: Vec3): number {
  return Math.sqrt(vector.x * vector.x + vector.y * vector.y + vector.z * vector.z);
}

function distance(left: Vec3, right: Vec3): number {
  const dx = right.x - left.x;
  const dy = right.y - left.y;
  const dz = right.z - left.z;
  return Math.sqrt(dx * dx + dy * dy + dz * dz);
}

function remapGapTime(time: number, gaps: ReplayGap[]): number {
  let cumulativeRemoved = 0;
  for (const gap of gaps) {
    if (time < gap.beforeTime) break;
    if (time >= gap.afterTime) {
      cumulativeRemoved += gap.duration;
      continue;
    }
    return gap.beforeTime - cumulativeRemoved;
  }
  return time - cumulativeRemoved;
}

function isInReplayGap(time: number, gaps: ReplayGap[]): boolean {
  return gaps.some((gap) => time > gap.beforeTime && time < gap.afterTime);
}
