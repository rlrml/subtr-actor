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
import EventEmitter from "eventemitter3";
import { vec3RlToThree, quatRlToThree, boostToPercent, type Vec3, type Quat } from "./coords.js";

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
      };
    };
interface RawReplayData {
  frame_data: {
    ball_data: { frames: RawBallFrame[] };
    players: Array<[unknown, { frames: RawPlayerFrame[] }]>;
    metadata_frames: Array<{ time: number; seconds_remaining: number }>;
  };
  meta: {
    team_zero: RawPlayerInfo[];
    team_one: RawPlayerInfo[];
  };
}
interface RawPlayerInfo {
  remote_id: unknown;
  name: string;
  car_body_name?: string | null;
  car_hitbox_family?: string | null;
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
}

export interface ViewerPlayerInfo {
  name: string;
  team: number;
  carName: string;
  hitboxType: string;
  loadout?: undefined;
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

class PlayerEntity extends EventEmitter {
  position: Vec3 = { x: 0, y: 0, z: 0 };
  rotation: Quat = { x: 0, y: 0, z: 0, w: 1 };
  velocity: Vec3 = { x: 0, y: 0, z: 0 };
  angularVelocity: Vec3 = { x: 0, y: 0, z: 0 };
  sleeping = false;
  steer = 0;
  boost = 0; // 0-100
  isBoosting = false;
  isVisible = true;
  isBallCam = true;
  constructor(
    public name: string,
    public team: number,
    public carName: string,
    public hitboxType: string,
  ) {
    super();
  }
}

export class SubtrActorPlayer extends EventEmitter {
  duration = 0;
  playerList: ViewerPlayerInfo[] = [];
  ball = new BallEntity();
  players = new Map<string, PlayerEntity>();
  boostPads = new Map<string, never>(); // stubbed for v0

  private _currentTime = 0;
  private _ballTimeline: MotionKeyframe[] = [];
  private _playerTimelines: Record<string, MotionKeyframe[]> = {};
  private _ballFlags: FlagsKeyframe[] = []; // ball has none, kept for symmetry
  private _playerFlags: Record<string, FlagsKeyframe[]> = {};
  private _teams: Record<string, number> = {};

  constructor(private raw: RawReplayData) {
    super();
    this._compile();
  }

  // ── Compilation: raw ReplayData -> ballcam-space timelines + entities. ──────
  private _compile(): void {
    const fd = this.raw.frame_data;
    const meta = this.raw.meta;
    const metaFrames = fd.metadata_frames;
    this.duration = metaFrames.length ? metaFrames[metaFrames.length - 1].time : 0;

    // remote_id -> { name, team, car } lookup from meta roster
    const infoByKey = new Map<string, { info: RawPlayerInfo; team: number }>();
    meta.team_zero.forEach((p) => infoByKey.set(this._idKey(p.remote_id), { info: p, team: 0 }));
    meta.team_one.forEach((p) => infoByKey.set(this._idKey(p.remote_id), { info: p, team: 1 }));

    // Ball motion timeline
    fd.ball_data.frames.forEach((f, i) => {
      if (f === "Empty" || !("Data" in f)) return;
      const mk = this._rbToKeyframe(f.Data.rigid_body, metaFrames[i]?.time ?? 0, i);
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

      const carName = matched?.info.car_body_name ?? "Octane";
      const hitboxType = matched?.info.car_hitbox_family ?? "Octane";

      const motion: MotionKeyframe[] = [];
      const flags: FlagsKeyframe[] = [];
      pdata.frames.forEach((f, i) => {
        const time = metaFrames[i]?.time ?? 0;
        if (f === "Empty" || !("Data" in f)) return;
        const mk = this._rbToKeyframe(f.Data.rigid_body, time, i);
        if (mk) motion.push(mk);
        flags.push({
          time,
          boost: boostToPercent(f.Data.boost_amount ?? 0),
          isBoosting: !!f.Data.boost_active,
          present: true,
        });
      });

      this._playerTimelines[name] = motion;
      this._playerFlags[name] = flags;
      this._teams[name] = team;
      this.playerList.push({ name, team, carName, hitboxType });
      this.players.set(name, new PlayerEntity(name, team, carName, hitboxType));
    });

    this.seek(0);
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

  private _idKey(remoteId: unknown): string {
    // RemoteIdTs is a tagged union ({Steam: "..."} etc.) or scalar; JSON is a
    // stable enough key for matching roster <-> frame players.
    return typeof remoteId === "string" ? remoteId : JSON.stringify(remoteId);
  }

  // ── Renderer-facing API ────────────────────────────────────────────────────
  getTimelines(): { ballTimeline: MotionKeyframe[]; playerTimelines: Record<string, MotionKeyframe[]> } {
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
      }
      const fl = lastBefore(this._playerFlags[name] ?? [], time);
      if (fl) {
        entity.boost = fl.boost;
        entity.isBoosting = fl.isBoosting;
      }
      // Presence: visible while we have motion data near `time`. A demolished
      // car produces Empty frames -> no nearby keyframe -> hidden.
      const tl = this._playerTimelines[name] ?? [];
      entity.isVisible = tl.length > 0 && time >= tl[0].time - 0.001 && time <= tl[tl.length - 1].time + 1.0;
    }
  }

  getBall(): BallEntity {
    return this.ball;
  }
  getPlayer(name: string): PlayerEntity | undefined {
    return this.players.get(name);
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
