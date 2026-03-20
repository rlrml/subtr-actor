# Stat Evaluation Player Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a stat-evaluation-player app on top of `js/player/` that visualizes per-frame stat accumulation synced to replay playback, with custom 3D overlays (e.g., most-back/most-forward role indicators on cars).

**Architecture:** The player library (`js/player/`) gets two additions: public scene access and a per-render callback hook. A new `js/stat-evaluation-player/` Vite app consumes the player library (via direct source import in dev, npm package when published) and the WASM `get_stats_timeline()` binding. It renders the 3D replay with overlay objects and a sidebar stat panel that updates each frame. The Rust positioning reducer is updated with threshold-based "even" classification for most-back/most-forward stats.

**Tech Stack:** TypeScript, Three.js, Vite, Rust (stats reducer), wasm-bindgen

---

### Task 1: Expose scene and render hook from player library

**Files:**
- Modify: `js/player/src/player.ts`
- Modify: `js/player/src/lib.ts`
- Modify: `js/player/src/types.ts`

**Step 1: Add BeforeRenderCallback type and public scene accessor to types**

In `js/player/src/types.ts`, add at the end:

```typescript
export interface FrameRenderInfo {
  frameIndex: number;
  nextFrameIndex: number;
  alpha: number;
  currentTime: number;
}

export type BeforeRenderCallback = (info: FrameRenderInfo) => void;
```

**Step 2: Add public getters and callback registration to ReplayPlayer**

In `js/player/src/player.ts`:

1. Change `private readonly sceneState` to `readonly sceneState` (remove `private`).

2. Add a callbacks array field:
```typescript
private readonly beforeRenderCallbacks: BeforeRenderCallback[] = [];
```

3. Add registration method:
```typescript
onBeforeRender(callback: BeforeRenderCallback): () => void {
  this.beforeRenderCallbacks.push(callback);
  return () => {
    const index = this.beforeRenderCallbacks.indexOf(callback);
    if (index >= 0) {
      this.beforeRenderCallbacks.splice(index, 1);
    }
  };
}
```

4. In the `render()` method, just before the final `this.sceneState.renderer.render(...)` call, add:
```typescript
const renderInfo: FrameRenderInfo = {
  frameIndex: frameWindow.frameIndex,
  nextFrameIndex: frameWindow.nextFrameIndex,
  alpha: frameWindow.alpha,
  currentTime: this.currentTime,
};
for (const callback of this.beforeRenderCallbacks) {
  callback(renderInfo);
}
```

**Step 3: Export new types from lib.ts**

Add to `js/player/src/lib.ts` exports:
```typescript
export type { BeforeRenderCallback, FrameRenderInfo } from "./types";
```

Also export the `ReplayScene` type:
```typescript
export type { ReplayScene } from "./scene";
```

**Step 4: Verify player library still type-checks**

Run: `cd js/player && npx tsc --noEmit`
Expected: No errors

**Step 5: Commit**

```bash
git add js/player/src/player.ts js/player/src/lib.ts js/player/src/types.ts
git commit -m "feat: expose scene state and before-render callback hook from player library"
```

---

### Task 2: Update Rust positioning reducer with threshold-based even classification

**Files:**
- Modify: `src/stats/reducers.rs`

**Step 1: Add threshold constant and `time_even` field to PositioningStats**

In `src/stats/reducers.rs`:

1. Add constant near the other positioning constants:
```rust
const MOST_BACK_FORWARD_THRESHOLD_Y: f32 = 800.0;
```

2. Add field to `PositioningStats` struct (after `time_farthest_from_ball`):
```rust
pub time_even: f32,
```

**Step 2: Update the most-back/most-forward logic in PositioningReducer::on_sample**

Replace the existing most_back/most_forward/closest/farthest block (approximately lines 1781-1825) with threshold-aware logic:

```rust
// Sort team players by normalized Y
let mut sorted_team: Vec<_> = team_players
    .iter()
    .map(|(info, pos)| (info.player_id.clone(), normalized_y(is_team_0, *pos)))
    .collect();
sorted_team.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap());

let team_spread = sorted_team.last().map(|(_, y)| *y).unwrap_or(0.0)
    - sorted_team.first().map(|(_, y)| *y).unwrap_or(0.0);

if team_spread <= MOST_BACK_FORWARD_THRESHOLD_Y {
    // All players are even
    for (player_id, _) in &sorted_team {
        self.player_stats
            .entry(player_id.clone())
            .or_default()
            .time_even += sample.dt;
    }
} else {
    let min_y = sorted_team.first().map(|(_, y)| *y).unwrap_or(0.0);
    let max_y = sorted_team.last().map(|(_, y)| *y).unwrap_or(0.0);

    for (player_id, y) in &sorted_team {
        let near_back = (*y - min_y) <= MOST_BACK_FORWARD_THRESHOLD_Y;
        let near_front = (max_y - *y) <= MOST_BACK_FORWARD_THRESHOLD_Y;

        if near_back && !near_front {
            self.player_stats
                .entry(player_id.clone())
                .or_default()
                .time_most_back += sample.dt;
        } else if near_front && !near_back {
            self.player_stats
                .entry(player_id.clone())
                .or_default()
                .time_most_forward += sample.dt;
        }
        // If near both or neither, player gets no role credit (mid)
    }
}

// closest/farthest to ball (unchanged)
```

Keep the existing closest_to_ball and farthest_from_ball logic intact.

**Step 3: Add `time_even` to the PositioningStats percentage helpers**

```rust
pub fn even_pct(&self) -> f32 {
    self.pct(self.time_even)
}
```

**Step 4: Export `time_even` in `src/stats/export/positioning.rs`**

Add to the `visit_stat_fields` implementation for PositioningStats:
```rust
visitor(ExportedStat::float(
    "positioning",
    "time_even",
    StatUnit::Seconds,
    self.time_even,
));
visitor(ExportedStat::float(
    "positioning",
    "percent_even",
    StatUnit::Percent,
    self.even_pct(),
));
```

**Step 5: Verify Rust compiles**

Run: `cargo check`
Expected: No errors

**Step 6: Run existing tests**

Run: `cargo test`
Expected: All tests pass (existing tests may need updating if they snapshot positioning stats)

**Step 7: Commit**

```bash
git add src/stats/reducers.rs src/stats/export/positioning.rs
git commit -m "feat: add threshold-based even classification for most-back/most-forward positioning stats"
```

---

### Task 3: Scaffold the stat-evaluation-player package

**Files:**
- Create: `js/stat-evaluation-player/package.json`
- Create: `js/stat-evaluation-player/tsconfig.json`
- Create: `js/stat-evaluation-player/vite.config.ts`
- Create: `js/stat-evaluation-player/index.html`
- Create: `js/stat-evaluation-player/src/main.ts`
- Create: `js/stat-evaluation-player/src/styles.css`

**Step 1: Create package.json**

```json
{
  "name": "subtr-actor-stat-evaluation-player",
  "private": true,
  "version": "0.0.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc --noEmit && vite build",
    "check": "tsc --noEmit"
  },
  "dependencies": {
    "three": "^0.180.0"
  },
  "devDependencies": {
    "@types/three": "^0.180.0",
    "typescript": "^5.9.2",
    "vite": "^7.1.5"
  }
}
```

Note: no explicit dependency on `subtr-actor-player` or `rl-replay-subtr-actor` — in dev mode we import directly from source paths. For publishable builds, these would be added as dependencies later.

**Step 2: Create tsconfig.json**

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "useDefineForClassFields": true,
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "strict": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "esModuleInterop": true,
    "lib": ["ES2022", "DOM", "DOM.Iterable"],
    "skipLibCheck": true
  },
  "include": ["src"]
}
```

**Step 3: Create vite.config.ts**

Model after `js/example/vite.config.js` — reuse the `ensureWasmBindingsPlugin` and allow access to parent dirs for source imports:

```typescript
import { defineConfig } from "vite";
import path from "node:path";
import {
  ensureWasmPackageFresh,
  getWasmWatchTargets,
  isWasmSourcePath,
} from "../scripts/ensure-wasm-package.mjs";

function ensureWasmBindingsPlugin() {
  let rebuild = Promise.resolve();

  const queueRebuild = (force = false) => {
    rebuild = rebuild.then(() =>
      ensureWasmPackageFresh({
        force,
        log: (message: string) => console.log(message),
      })
    );
    return rebuild;
  };

  return {
    name: "ensure-wasm-bindings",
    async buildStart() {
      await queueRebuild();
    },
    async configureServer(server: any) {
      await queueRebuild();
      server.watcher.add(getWasmWatchTargets());

      const rebuildOnChange = async (filePath: string) => {
        if (!isWasmSourcePath(filePath)) {
          return;
        }
        try {
          await queueRebuild(true);
          server.ws.send({ type: "full-reload" });
        } catch (error: unknown) {
          server.config.logger.error(
            error instanceof Error ? error.message : String(error)
          );
        }
      };

      server.watcher.on("change", rebuildOnChange);
      server.watcher.on("add", rebuildOnChange);
      server.watcher.on("unlink", rebuildOnChange);
    },
  };
}

export default defineConfig({
  plugins: [ensureWasmBindingsPlugin()],
  server: {
    fs: {
      allow: [path.resolve(__dirname, "..")],
    },
  },
  optimizeDeps: {
    exclude: ["rl-replay-subtr-actor"],
  },
  assetsInclude: ["**/*.wasm"],
});
```

**Step 4: Create index.html**

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>stat evaluation player</title>
</head>
<body>
  <div id="app"></div>
  <script type="module" src="./src/main.ts"></script>
</body>
</html>
```

**Step 5: Create src/styles.css**

Minimal starter styles (dark theme matching the example app):

```css
*,
*::before,
*::after {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}

html, body {
  height: 100%;
  font-family: system-ui, -apple-system, sans-serif;
  background: #0c1520;
  color: #d0dce8;
}

#app {
  height: 100%;
}

.shell {
  display: grid;
  grid-template-columns: 1fr 360px;
  grid-template-rows: auto 1fr;
  height: 100%;
  gap: 12px;
  padding: 12px;
}

.header {
  grid-column: 1 / -1;
  display: flex;
  align-items: center;
  gap: 16px;
}

.header h1 {
  font-size: 1.1rem;
  font-weight: 600;
}

.viewport {
  width: 100%;
  height: 100%;
  min-height: 400px;
  border-radius: 8px;
  overflow: hidden;
  position: relative;
}

.sidebar {
  display: flex;
  flex-direction: column;
  gap: 12px;
  overflow-y: auto;
}

.panel {
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  padding: 12px;
}

.panel h2 {
  font-size: 0.85rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: #7a8fa3;
  margin-bottom: 8px;
}

.stat-row {
  display: flex;
  justify-content: space-between;
  padding: 2px 0;
  font-size: 0.85rem;
}

.stat-row .label {
  color: #7a8fa3;
}

.stat-row .value {
  font-weight: 600;
  font-variant-numeric: tabular-nums;
}

.player-stats-group {
  margin-top: 8px;
}

.player-stats-group h3 {
  font-size: 0.8rem;
  margin-bottom: 4px;
}

select, button, input[type="file"] {
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 4px;
  color: #d0dce8;
  padding: 4px 8px;
  font-size: 0.85rem;
}

button {
  cursor: pointer;
}

button:hover {
  background: rgba(255, 255, 255, 0.1);
}

input[type="range"] {
  width: 100%;
}

.transport-row {
  display: flex;
  gap: 8px;
  align-items: center;
  margin-bottom: 8px;
}

.role-indicator {
  font-size: 0.75rem;
  font-weight: 700;
  padding: 1px 5px;
  border-radius: 3px;
  text-transform: uppercase;
}

.role-back { background: #b33; color: #fff; }
.role-forward { background: #3b3; color: #fff; }
.role-even { background: #888; color: #fff; }
```

**Step 6: Create src/main.ts with skeleton**

```typescript
import "./styles.css";
import {
  ReplayPlayer,
  loadReplayFromBytes,
} from "../../player/src/lib.ts";
import type {
  ReplayModel,
  FrameRenderInfo,
  ReplayPlayerState,
} from "../../player/src/lib.ts";

// WASM stats timeline binding
import init, {
  get_stats_timeline,
} from "../../pkg/rl_replay_subtr_actor.js";

interface StatsTimeline {
  replay_meta: unknown;
  timeline_events: unknown[];
  frames: StatsFrame[];
}

interface StatsFrame {
  frame_number: number;
  time: number;
  dt: number;
  players: PlayerStatsSnapshot[];
  [key: string]: unknown;
}

interface PlayerStatsSnapshot {
  player_id: Record<string, string>;
  name: string;
  is_team_0: boolean;
  positioning?: {
    time_most_back: number;
    time_most_forward: number;
    time_even: number;
    [key: string]: unknown;
  };
  [key: string]: unknown;
}

const app = document.getElementById("app")!;

app.innerHTML = `
  <div class="shell">
    <div class="header">
      <h1>Stat Evaluation Player</h1>
      <input id="replay-file" type="file" accept=".replay" />
      <button id="toggle-playback" disabled>Play</button>
      <select id="playback-rate" disabled>
        <option value="0.25">0.25x</option>
        <option value="0.5">0.5x</option>
        <option value="1" selected>1.0x</option>
        <option value="2">2.0x</option>
      </select>
      <input id="timeline" type="range" min="0" max="0" step="0.01" value="0" disabled style="flex:1" />
    </div>
    <div id="viewport" class="viewport"></div>
    <aside class="sidebar">
      <div class="panel">
        <h2>Playback</h2>
        <div class="stat-row">
          <span class="label">Time</span>
          <span class="value" id="time-readout">0.00s</span>
        </div>
        <div class="stat-row">
          <span class="label">Frame</span>
          <span class="value" id="frame-readout">0</span>
        </div>
      </div>
      <div id="stats-container" class="panel">
        <h2>Player Stats</h2>
        <div id="player-stats">Load a replay to see stats.</div>
      </div>
    </aside>
  </div>
`;

let replayPlayer: ReplayPlayer | null = null;
let statsTimeline: StatsTimeline | null = null;
let unsubscribe: (() => void) | null = null;

const fileInput = document.getElementById("replay-file") as HTMLInputElement;
const viewport = document.getElementById("viewport")!;
const togglePlayback = document.getElementById("toggle-playback") as HTMLButtonElement;
const playbackRate = document.getElementById("playback-rate") as HTMLSelectElement;
const timeline = document.getElementById("timeline") as HTMLInputElement;
const timeReadout = document.getElementById("time-readout")!;
const frameReadout = document.getElementById("frame-readout")!;
const playerStatsEl = document.getElementById("player-stats")!;

function renderStats(frameIndex: number): void {
  if (!statsTimeline) return;

  const statsFrame = statsTimeline.frames[frameIndex];
  if (!statsFrame) return;

  const lines: string[] = [];
  for (const player of statsFrame.players) {
    const pos = player.positioning;
    lines.push(`<div class="player-stats-group">`);
    lines.push(`<h3>${player.name} ${player.is_team_0 ? "(Blue)" : "(Orange)"}</h3>`);
    if (pos) {
      lines.push(`<div class="stat-row"><span class="label">Most back</span><span class="value">${pos.time_most_back?.toFixed(1) ?? "?"}s</span></div>`);
      lines.push(`<div class="stat-row"><span class="label">Most forward</span><span class="value">${pos.time_most_forward?.toFixed(1) ?? "?"}s</span></div>`);
      lines.push(`<div class="stat-row"><span class="label">Even</span><span class="value">${pos.time_even?.toFixed(1) ?? "?"}s</span></div>`);
    }
    lines.push(`</div>`);
  }
  playerStatsEl.innerHTML = lines.join("\n");
}

function onStateChange(state: ReplayPlayerState): void {
  timeReadout.textContent = `${state.currentTime.toFixed(2)}s`;
  frameReadout.textContent = `${state.frameIndex}`;
  timeline.value = `${state.currentTime}`;
  togglePlayback.textContent = state.playing ? "Pause" : "Play";
  renderStats(state.frameIndex);
}

async function loadReplay(file: File): Promise<void> {
  if (unsubscribe) {
    unsubscribe();
    unsubscribe = null;
  }
  replayPlayer?.destroy();
  replayPlayer = null;

  await init();
  const bytes = new Uint8Array(await file.arrayBuffer());

  const [{ replay }, rawStatsTimeline] = await Promise.all([
    loadReplayFromBytes(bytes),
    Promise.resolve(get_stats_timeline(bytes) as StatsTimeline),
  ]);

  statsTimeline = rawStatsTimeline;

  replayPlayer = new ReplayPlayer(viewport, replay, {
    initialCameraDistanceScale: 2.25,
  });

  unsubscribe = replayPlayer.subscribe(onStateChange);

  timeline.min = "0";
  timeline.max = `${replay.duration}`;

  togglePlayback.disabled = false;
  playbackRate.disabled = false;
  timeline.disabled = false;
}

fileInput.addEventListener("change", async () => {
  const file = fileInput.files?.[0];
  if (file) {
    try {
      await loadReplay(file);
    } catch (error) {
      console.error("Failed to load replay:", error);
    }
  }
});

togglePlayback.addEventListener("click", () => replayPlayer?.togglePlayback());
playbackRate.addEventListener("change", () => replayPlayer?.setPlaybackRate(Number(playbackRate.value)));
timeline.addEventListener("input", () => replayPlayer?.seek(Number(timeline.value)));
```

**Step 7: Install dependencies and verify it builds**

Run: `cd js/stat-evaluation-player && npm install && npx tsc --noEmit`
Expected: No errors

**Step 8: Commit**

```bash
git add js/stat-evaluation-player/
git commit -m "feat: scaffold stat-evaluation-player package with basic stat display"
```

---

### Task 4: Add 3D overlay system for role visualization

**Files:**
- Create: `js/stat-evaluation-player/src/overlays.ts`
- Modify: `js/stat-evaluation-player/src/main.ts`

**Step 1: Create overlays.ts with role indicator overlay**

This module creates Three.js objects that attach to car meshes and update per-frame based on positioning role classification.

```typescript
import * as THREE from "three";
import type { ReplayModel } from "../../player/src/types.ts";
import type { ReplayScene } from "../../player/src/scene.ts";
import type { FrameRenderInfo } from "../../player/src/types.ts";

const ROLE_COLORS = {
  back: 0xff3333,
  forward: 0x33ff33,
  even: 0x888888,
  mid: 0xffaa33,
} as const;

// Must match Rust constant MOST_BACK_FORWARD_THRESHOLD_Y
const MOST_BACK_FORWARD_THRESHOLD_Y = 800.0;

type Role = "back" | "forward" | "even" | "mid";

interface PlayerRoleRing {
  ring: THREE.Mesh;
  material: THREE.MeshBasicMaterial;
}

export class RoleOverlay {
  private rings = new Map<string, PlayerRoleRing>();
  private replay: ReplayModel;

  constructor(
    sceneState: ReplayScene,
    replay: ReplayModel,
  ) {
    this.replay = replay;

    for (const player of replay.players) {
      const mesh = sceneState.playerMeshes.get(player.id);
      if (!mesh) continue;

      const material = new THREE.MeshBasicMaterial({
        color: ROLE_COLORS.even,
        transparent: true,
        opacity: 0.6,
        side: THREE.DoubleSide,
        depthWrite: false,
      });

      const geometry = new THREE.RingGeometry(140, 180, 24);
      geometry.rotateX(Math.PI / 2);
      const ring = new THREE.Mesh(geometry, material);
      ring.position.set(0, 0, -40);
      mesh.add(ring);

      this.rings.set(player.id, { ring, material });
    }
  }

  update(info: FrameRenderInfo): void {
    const { frameIndex } = info;

    // Group players by team and get their Y positions
    const teams = new Map<boolean, Array<{ id: string; y: number }>>();

    for (const player of this.replay.players) {
      const frame = player.frames[frameIndex];
      if (!frame?.position) continue;

      // normalized_y: for team 0, use raw Y; for team 1, negate Y
      const normalizedY = player.isTeamZero
        ? frame.position.y
        : -frame.position.y;

      const team = teams.get(player.isTeamZero) ?? [];
      team.push({ id: player.id, y: normalizedY });
      teams.set(player.isTeamZero, team);
    }

    for (const [, teamPlayers] of teams) {
      teamPlayers.sort((a, b) => a.y - b.y);

      const minY = teamPlayers[0]?.y ?? 0;
      const maxY = teamPlayers[teamPlayers.length - 1]?.y ?? 0;
      const spread = maxY - minY;

      const roles = new Map<string, Role>();

      if (spread <= MOST_BACK_FORWARD_THRESHOLD_Y) {
        for (const p of teamPlayers) {
          roles.set(p.id, "even");
        }
      } else {
        for (const p of teamPlayers) {
          const nearBack = (p.y - minY) <= MOST_BACK_FORWARD_THRESHOLD_Y;
          const nearFront = (maxY - p.y) <= MOST_BACK_FORWARD_THRESHOLD_Y;

          if (nearBack && !nearFront) {
            roles.set(p.id, "back");
          } else if (nearFront && !nearBack) {
            roles.set(p.id, "forward");
          } else {
            roles.set(p.id, "mid");
          }
        }
      }

      for (const [playerId, role] of roles) {
        const entry = this.rings.get(playerId);
        if (!entry) continue;
        entry.material.color.setHex(ROLE_COLORS[role]);
      }
    }
  }

  dispose(): void {
    for (const [, { ring, material }] of this.rings) {
      ring.geometry.dispose();
      material.dispose();
      ring.removeFromParent();
    }
    this.rings.clear();
  }
}
```

**Step 2: Wire overlay into main.ts**

In `main.ts`, after creating the `ReplayPlayer`:

```typescript
import { RoleOverlay } from "./overlays.ts";

// In loadReplay(), after creating replayPlayer:
const roleOverlay = new RoleOverlay(replayPlayer.sceneState, replay);
const removeRenderHook = replayPlayer.onBeforeRender((info) => {
  roleOverlay.update(info);
});

// Store for cleanup
// (update the cleanup section in loadReplay to call roleOverlay.dispose() and removeRenderHook())
```

**Step 3: Verify dev server runs**

Run: `cd js/stat-evaluation-player && npm run dev`
Expected: Dev server starts, loads in browser, file picker works

**Step 4: Commit**

```bash
git add js/stat-evaluation-player/src/overlays.ts js/stat-evaluation-player/src/main.ts
git commit -m "feat: add role overlay visualization for most-back/most-forward/even"
```

---

### Task 5: Add zone boundary lines to the 3D scene

**Files:**
- Modify: `js/stat-evaluation-player/src/overlays.ts`

**Step 1: Add zone boundary line overlay**

Add to `overlays.ts` a function that draws the defensive/neutral/offensive zone boundary lines at Y = ±2300 (matching Rust `FIELD_ZONE_BOUNDARY_Y`). These are static lines added to the scene root.

```typescript
const FIELD_ZONE_BOUNDARY_Y = 2300.0;

export function createZoneBoundaryLines(
  scene: THREE.Scene,
  fieldScale: number,
): THREE.Group {
  const group = new THREE.Group();
  const FIELD_HALF_WIDTH = 4120 * fieldScale;

  const material = new THREE.LineBasicMaterial({
    color: 0xffffff,
    transparent: true,
    opacity: 0.25,
  });

  for (const ySign of [-1, 1]) {
    const y = ySign * FIELD_ZONE_BOUNDARY_Y * fieldScale;
    const points = [
      new THREE.Vector3(-FIELD_HALF_WIDTH, y, 2),
      new THREE.Vector3(FIELD_HALF_WIDTH, y, 2),
    ];
    const geometry = new THREE.BufferGeometry().setFromPoints(points);
    const line = new THREE.Line(geometry, material);
    group.add(line);
  }

  // Midfield line
  const midPoints = [
    new THREE.Vector3(-FIELD_HALF_WIDTH, 0, 2),
    new THREE.Vector3(FIELD_HALF_WIDTH, 0, 2),
  ];
  const midGeometry = new THREE.BufferGeometry().setFromPoints(midPoints);
  const midMaterial = new THREE.LineBasicMaterial({
    color: 0xffffff,
    transparent: true,
    opacity: 0.15,
  });
  group.add(new THREE.Line(midGeometry, midMaterial));

  scene.add(group);
  return group;
}
```

**Step 2: Call from main.ts after creating player**

```typescript
import { RoleOverlay, createZoneBoundaryLines } from "./overlays.ts";

// After creating replayPlayer:
createZoneBoundaryLines(
  replayPlayer.sceneState.scene,
  replayPlayer.options.fieldScale ?? 1,
);
```

**Step 3: Commit**

```bash
git add js/stat-evaluation-player/src/overlays.ts js/stat-evaluation-player/src/main.ts
git commit -m "feat: add zone boundary lines to stat evaluation player"
```

---

### Task 6: Rebuild WASM bindings and end-to-end test

**Step 1: Rebuild WASM bindings with updated Rust code**

Run: `cd js && npm run build`
Expected: wasm-pack build succeeds

**Step 2: Start stat-evaluation-player dev server**

Run: `cd js/stat-evaluation-player && npm run dev`

**Step 3: Manual test**

1. Open browser to dev server URL
2. Load a `.replay` file
3. Verify: 3D scene renders with replay
4. Verify: colored rings appear on cars (red=back, green=forward, gray=even, orange=mid)
5. Verify: rings update color as replay plays
6. Verify: zone boundary lines visible on field
7. Verify: sidebar stats panel shows per-player positioning stats updating live

**Step 4: Commit any fixes**

---
