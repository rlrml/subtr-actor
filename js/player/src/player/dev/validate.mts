/**
 * Headless validation of the subtr-actor -> SubtrActorPlayer data pipeline.
 * Confirms the coordinate transform lands ball/cars inside RL field dimensions
 * (the #1 correctness risk) without needing a browser. Run: npx tsx src/dev/validate.mts
 */
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import initSubtr from "@rlrml/subtr-actor";
import { SubtrActorPlayer } from "../adapter/SubtrActorPlayer.js";
import { loadReplay } from "../adapter/wasm.js";

// Node can't fetch() the web-target wasm over file://; pre-init with bytes so
// @rlrml/player's later default() init short-circuits to the ready instance.
const wasmUrl = new URL("../../node_modules/@rlrml/subtr-actor/rl_replay_subtr_actor_bg.wasm", import.meta.url);
await initSubtr(readFileSync(fileURLToPath(wasmUrl)));

// RL field half-extents in Unreal Units (THREE space: x=floor width, y=height, z=floor length)
const FIELD = { x: 4096, yCeil: 2044, z: 5120 };

const replayUrl = new URL("../../../../assets/recent-ranked-doubles-2026-03-10.replay", import.meta.url);
const bytes = new Uint8Array(readFileSync(fileURLToPath(replayUrl)));

console.log("parsing via subtr-actor WASM…");
const { replay, raw } = await loadReplay(bytes);
const player = new SubtrActorPlayer(raw as never);

console.log("\n== roster ==");
for (const p of player.playerList) {
  const cam = p.cameraSettings;
  const camStr = cam
    ? `cam{d=${cam.distance} h=${cam.height} a=${cam.angle} stiff=${cam.stiffness} swivel=${cam.swivelSpeed} fov=${cam.fov}}`
    : "cam{none}";
  console.log(`  ${p.name}  team${p.team}  ${p.carName}/${p.hitboxType}  ${camStr}`);
}
console.log(`duration: ${player.duration.toFixed(2)}s`);

const { ballTimeline, playerTimelines } = player.getTimelines();

function bounds(kfs: { position: { x: number; y: number; z: number } }[]) {
  const b = { minx: 1e9, maxx: -1e9, miny: 1e9, maxy: -1e9, minz: 1e9, maxz: -1e9 };
  for (const k of kfs) {
    const p = k.position;
    b.minx = Math.min(b.minx, p.x); b.maxx = Math.max(b.maxx, p.x);
    b.miny = Math.min(b.miny, p.y); b.maxy = Math.max(b.maxy, p.y);
    b.minz = Math.min(b.minz, p.z); b.maxz = Math.max(b.maxz, p.z);
  }
  return b;
}
const fmt = (b: ReturnType<typeof bounds>) =>
  `x[${b.minx.toFixed(0)},${b.maxx.toFixed(0)}] y[${b.miny.toFixed(0)},${b.maxy.toFixed(0)}] z[${b.minz.toFixed(0)},${b.maxz.toFixed(0)}]`;

console.log(`\n== ball (${ballTimeline.length} keyframes) ==`);
const bb = bounds(ballTimeline);
console.log("  bounds:", fmt(bb));

console.log("\n== cars ==");
for (const [name, tl] of Object.entries(playerTimelines)) {
  console.log(`  ${name}: ${tl.length} kf  ${fmt(bounds(tl))}`);
}

// Assertions: in THREE space, y is HEIGHT (small positive, capped near ceiling),
// x and z are the floor plane (can be +/- up to field extents, with margins).
console.log("\n== checks ==");
const margin = 1.4; // allow ball/cars slightly past nominal (corners, ceiling overshoot)
const ok = (label: string, cond: boolean) => console.log(`  ${cond ? "PASS" : "FAIL"}  ${label}`);
ok("roster non-empty", player.playerList.length > 0);
ok("ball has keyframes", ballTimeline.length > 100);
ok(`ball height y >= -50 (not floor-swapped)`, bb.miny >= -50);
ok(`ball height y <= ${FIELD.yCeil * margin} (y is up-axis, not length)`, bb.maxy <= FIELD.yCeil * margin);
ok(`ball |x| <= ${FIELD.x * margin}`, Math.abs(bb.minx) <= FIELD.x * margin && Math.abs(bb.maxx) <= FIELD.x * margin);
ok(`ball |z| <= ${FIELD.z * margin}`, Math.abs(bb.minz) <= FIELD.z * margin && Math.abs(bb.maxz) <= FIELD.z * margin);
ok("duration sane (60-600s)", player.duration > 60 && player.duration < 600);
ok(
  "recorded camera settings present for every player",
  player.playerList.every((p) => p.cameraSettings && p.cameraSettings.stiffness >= 0 && p.cameraSettings.stiffness <= 1),
);

// @rlrml/player parity surface (docs/player/PLAYER_PARITY.md): stable ids + frame timeline.
ok(
  "player ids present and unique",
  player.playerList.every((p) => p.id.length > 0) &&
    new Set(player.playerList.map((p) => p.id)).size === player.playerList.length,
);
ok("frameTimes non-empty", player.frameTimes.length > 100);
ok(
  "frameTimes monotonic non-decreasing",
  player.frameTimes.every((t, i) => i === 0 || t >= player.frameTimes[i - 1]),
);
const midTime = player.duration / 2;
const midIdx = player.frameIndexAt(midTime);
ok(
  "frameIndexAt finds the last frame at-or-before t",
  player.frameTimes[midIdx] <= midTime &&
    (midIdx === player.frameTimes.length - 1 || player.frameTimes[midIdx + 1] > midTime),
);
ok(
  "frameIndexAt clamps to ends",
  player.frameIndexAt(-5) === 0 && player.frameIndexAt(player.duration + 5) === player.frameTimes.length - 1,
);

// Phase 2 (docs/player/PLAYER_PARITY.md): the adapter and @rlrml/player's ReplayModel
// are two views over the same raw output — ids and time axis must agree.
const adapterIds = new Set(player.playerList.map((p) => p.id));
const modelIds = new Set(replay.players.map((p) => p.id));
ok(
  "adapter ids == ReplayModel ids",
  adapterIds.size === modelIds.size && [...adapterIds].every((id) => modelIds.has(id)),
);
ok("time axis starts at 0", player.frameTimes[0] === 0);
ok(
  `rawStartTime matches ReplayModel (${player.rawStartTime.toFixed(3)}s)`,
  player.rawStartTime === replay.rawStartTime,
);
ok(
  "duration matches ReplayModel",
  Math.abs(player.duration - replay.duration) < 0.001,
);
ok("frame count matches ReplayModel", player.frameTimes.length === replay.frameCount);
