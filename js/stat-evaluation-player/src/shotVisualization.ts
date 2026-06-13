import * as THREE from "three";
import type { ReplayPlayerState, ReplayTimelineEvent } from "@rlrml/player";
import type { StatsReplayPlayer } from "./statsReplayPlayer.ts";

const FIELD_HALF_WIDTH = 4096;
const FIELD_HALF_LENGTH = 5120;
const GOAL_HALF_WIDTH = 893;
const GOAL_HEIGHT = 642;
const FIELD_SCALE = 1 / 105;

type ShotEvent = ReplayTimelineEvent & {
  kind: "shot";
  shot: NonNullable<ReplayTimelineEvent["shot"]>;
};

interface Vec3Like {
  readonly x: number;
  readonly y: number;
  readonly z: number;
}

export interface ShotVisualizationControllerOptions {
  readonly body: HTMLElement;
  getReplayPlayer(): StatsReplayPlayer | null;
  cueTimelineEvent(event: ReplayTimelineEvent): void;
}

export class ShotVisualizationController {
  private readonly chartCanvas: HTMLCanvasElement;
  private readonly shotList: HTMLDivElement;
  private readonly summary: HTMLElement;
  private readonly details: HTMLElement;
  private readonly sceneRoot: HTMLDivElement;
  private readonly emptyState: HTMLElement;
  private renderer: THREE.WebGLRenderer | null = null;
  private scene: THREE.Scene | null = null;
  private camera: THREE.PerspectiveCamera | null = null;
  private resizeObserver: ResizeObserver | null = null;
  private selectedShotId: string | null = null;
  private cachedReplay: StatsReplayPlayer["replay"] | null = null;
  private cachedShots: ShotEvent[] = [];
  private lastShotsSignature = "";
  private lastListSelectionKey: string | null = null;
  private lastSelectedShotKey: string | null = null;
  private readonly shotButtons = new Map<string, HTMLButtonElement>();

  constructor(private readonly options: ShotVisualizationControllerOptions) {
    this.options.body.innerHTML = `
      <div class="shot-visualization">
        <div class="shot-visualization-summary" id="shot-visualization-summary"></div>
        <canvas class="shot-chart-canvas" id="shot-chart-canvas" width="720" height="520"></canvas>
        <div class="shot-visualization-layout">
          <div class="shot-list" id="shot-list" data-no-drag></div>
          <div class="shot-demo-panel">
            <div class="shot-demo-scene" id="shot-demo-scene" data-no-drag></div>
            <dl class="shot-details" id="shot-details"></dl>
          </div>
        </div>
        <p class="shot-visualization-empty" id="shot-visualization-empty">Load a replay with shot events.</p>
      </div>
    `;
    this.chartCanvas = this.mustElement<HTMLCanvasElement>("#shot-chart-canvas");
    this.shotList = this.mustElement<HTMLDivElement>("#shot-list");
    this.summary = this.mustElement<HTMLElement>("#shot-visualization-summary");
    this.details = this.mustElement<HTMLElement>("#shot-details");
    this.sceneRoot = this.mustElement<HTMLDivElement>("#shot-demo-scene");
    this.emptyState = this.mustElement<HTMLElement>("#shot-visualization-empty");
    this.chartCanvas.addEventListener("click", (event) => this.handleChartClick(event));
  }

  destroy(): void {
    this.resizeObserver?.disconnect();
    this.renderer?.dispose();
    this.renderer = null;
    this.scene = null;
    this.camera = null;
  }

  render(
    state: ReplayPlayerState | null = this.options.getReplayPlayer()?.getState() ?? null,
  ): void {
    const shots = this.shotEvents();
    const currentTime = state?.currentTime ?? null;
    if (!this.findSelectedShot(shots)) {
      this.selectedShotId = shots[0] ? this.shotKey(shots[0], 0) : null;
    }
    const selectedShot = this.findSelectedShot(shots);
    const selectedShotKey = selectedShot?.key ?? null;
    const shotsSignature = this.shotsSignature(shots);
    const shotsChanged = shotsSignature !== this.lastShotsSignature;

    this.emptyState.hidden = shots.length > 0;
    if (shotsChanged) {
      this.summary.textContent = this.summaryText(shots);
    }
    if (shotsChanged || selectedShotKey !== this.lastListSelectionKey) {
      this.renderShotList(shots, currentTime, selectedShotKey);
      this.lastListSelectionKey = selectedShotKey;
    } else {
      this.updateShotListActiveState(shots, currentTime);
    }
    this.renderChart(shots, currentTime);
    if (shotsChanged || selectedShotKey !== this.lastSelectedShotKey) {
      this.renderSelectedShot(selectedShot?.shot ?? null);
      this.lastSelectedShotKey = selectedShotKey;
    }
    this.lastShotsSignature = shotsSignature;
  }

  private shotEvents(): ShotEvent[] {
    const replay = this.options.getReplayPlayer()?.replay;
    if (!replay) {
      this.cachedReplay = null;
      this.cachedShots = [];
      return [];
    }
    if (replay === this.cachedReplay) {
      return this.cachedShots;
    }

    this.cachedReplay = replay;
    this.cachedShots = replay.timelineEvents.filter(isShotEvent);
    return this.cachedShots;
  }

  private summaryText(shots: readonly ShotEvent[]): string {
    if (shots.length === 0) {
      return "No shots";
    }

    const saved = shots.filter((shot) => shot.shot.resulting_save).length;
    const speeds = shots
      .map((shot) => shot.shot.ball_speed)
      .filter((speed): speed is number => Number.isFinite(speed));
    const averageSpeed =
      speeds.length === 0
        ? null
        : speeds.reduce((total, speed) => total + speed, 0) / speeds.length;
    return `${shots.length} shots | ${saved} saved | avg ${formatSpeed(averageSpeed)}`;
  }

  private renderShotList(
    shots: readonly ShotEvent[],
    currentTime: number | null,
    selectedShotKey: string | null,
  ): void {
    this.shotButtons.clear();
    this.shotList.replaceChildren(
      ...shots.map((shot, index) => {
        const key = this.shotKey(shot, index);
        const button = document.createElement("button");
        button.type = "button";
        button.className = "shot-list-item";
        button.dataset.selected = String(key === selectedShotKey);
        button.dataset.active = String(isShotActive(shot, currentTime));
        button.addEventListener("click", () => {
          this.selectedShotId = key;
          this.options.cueTimelineEvent(shot);
          this.render();
        });

        const title = document.createElement("span");
        title.className = "shot-list-title";
        title.textContent = shot.playerName ?? shot.playerId ?? `Shot ${index + 1}`;
        const meta = document.createElement("span");
        meta.className = "shot-list-meta";
        meta.textContent = `${formatTime(shot.time)} | ${formatSpeed(shot.shot.ball_speed)} | ${
          shot.shot.resulting_save ? "saved" : "unsaved"
        }`;
        button.append(title, meta);
        this.shotButtons.set(key, button);
        return button;
      }),
    );
  }

  private updateShotListActiveState(shots: readonly ShotEvent[], currentTime: number | null): void {
    shots.forEach((shot, index) => {
      const button = this.shotButtons.get(this.shotKey(shot, index));
      if (button) {
        button.dataset.active = String(isShotActive(shot, currentTime));
      }
    });
  }

  private renderChart(shots: readonly ShotEvent[], currentTime: number | null): void {
    const context = this.chartCanvas.getContext("2d");
    if (!context) {
      return;
    }

    const ratio = window.devicePixelRatio || 1;
    const rect = this.chartCanvas.getBoundingClientRect();
    const width = Math.max(320, Math.round(rect.width * ratio));
    const height = Math.max(220, Math.round(rect.height * ratio));
    if (this.chartCanvas.width !== width || this.chartCanvas.height !== height) {
      this.chartCanvas.width = width;
      this.chartCanvas.height = height;
    }
    context.setTransform(ratio, 0, 0, ratio, 0, 0);

    const cssWidth = width / ratio;
    const cssHeight = height / ratio;
    context.clearRect(0, 0, cssWidth, cssHeight);
    drawField(context, cssWidth, cssHeight);

    shots.forEach((shot, index) => {
      const position = shot.shot.shot_touch_position ?? shot.shot.ball_position;
      const point = fieldToChart(position, cssWidth, cssHeight);
      const saved = Boolean(shot.shot.resulting_save);
      const active = isShotActive(shot, currentTime);
      const selected = this.shotKey(shot, index) === this.selectedShotId;
      const radius = selected ? 6 : active ? 5 : 4;

      context.beginPath();
      context.arc(point.x, point.y, radius, 0, Math.PI * 2);
      context.fillStyle = saved ? "#f2c14e" : "#62d2a2";
      context.fill();
      context.lineWidth = selected ? 2.5 : 1.25;
      context.strokeStyle = selected ? "#f8fafc" : active ? "#e11d48" : "rgba(4, 12, 20, 0.85)";
      context.stroke();
    });
  }

  private renderSelectedShot(shot: ShotEvent | null): void {
    if (!shot) {
      this.details.replaceChildren();
      this.clearThreeScene();
      return;
    }

    this.details.replaceChildren(
      detail("Player", shot.playerName ?? shot.playerId ?? "--"),
      detail("Speed", formatSpeed(shot.shot.ball_speed)),
      detail("Toward goal", formatSpeed(shot.shot.ball_speed_toward_goal)),
      detail("Touch", formatVec(shot.shot.shot_touch_position ?? shot.shot.ball_position)),
      detail("Save", shot.shot.resulting_save ? formatTime(shot.shot.resulting_save.time) : "--"),
    );
    this.renderThreeScene(shot);
  }

  private renderThreeScene(shot: ShotEvent): void {
    const renderer = this.ensureRenderer();
    const scene = this.scene!;
    const camera = this.camera!;
    scene.clear();

    scene.add(new THREE.AmbientLight(0xffffff, 1.2));
    const directional = new THREE.DirectionalLight(0xffffff, 1.7);
    directional.position.set(18, 28, 22);
    scene.add(directional);

    const field = new THREE.Mesh(
      new THREE.PlaneGeometry(
        FIELD_HALF_WIDTH * 2 * FIELD_SCALE,
        FIELD_HALF_LENGTH * 2 * FIELD_SCALE,
      ),
      new THREE.MeshBasicMaterial({ color: 0x173326, side: THREE.DoubleSide }),
    );
    field.rotation.x = -Math.PI / 2;
    scene.add(field);

    const lineMaterial = new THREE.LineBasicMaterial({
      color: 0xd9e7e2,
      transparent: true,
      opacity: 0.55,
    });
    scene.add(new THREE.LineSegments(new THREE.EdgesGeometry(field.geometry), lineMaterial));
    scene.add(goalFrame(true));
    scene.add(goalFrame(false));

    const start = toThree(shot.shot.shot_touch_position ?? shot.shot.ball_position);
    const target = toThree(shot.shot.target_goal_position);
    const velocity = shot.shot.ball_velocity;
    const end = velocity
      ? start.clone().add(toThreeDelta(velocity).normalize().multiplyScalar(22))
      : target.clone();
    const path = new THREE.CatmullRomCurve3([
      start,
      start
        .clone()
        .lerp(end, 0.45)
        .setY(Math.max(start.y, end.y) + 2.8),
      end,
    ]);
    const pathGeometry = new THREE.BufferGeometry().setFromPoints(path.getPoints(36));
    scene.add(
      new THREE.Line(
        pathGeometry,
        new THREE.LineBasicMaterial({
          color: shot.shot.resulting_save ? 0xf2c14e : 0x62d2a2,
          linewidth: 2,
        }),
      ),
    );

    const ball = new THREE.Mesh(
      new THREE.SphereGeometry(0.9, 24, 16),
      new THREE.MeshStandardMaterial({ color: 0xf8fafc, roughness: 0.36 }),
    );
    ball.position.copy(start);
    scene.add(ball);

    const targetMarker = new THREE.Mesh(
      new THREE.RingGeometry(1.05, 1.45, 32),
      new THREE.MeshBasicMaterial({ color: 0x87afd4, side: THREE.DoubleSide }),
    );
    targetMarker.position.copy(target);
    targetMarker.rotation.y = Math.PI / 2;
    scene.add(targetMarker);

    camera.position.set(0, 54, 72);
    camera.lookAt(0, 0, 0);
    renderer.render(scene, camera);
  }

  private ensureRenderer(): THREE.WebGLRenderer {
    if (this.renderer && this.scene && this.camera) {
      this.resizeThreeScene();
      return this.renderer;
    }

    const renderer = new THREE.WebGLRenderer({
      antialias: true,
      alpha: true,
      preserveDrawingBuffer: true,
    });
    renderer.setPixelRatio(window.devicePixelRatio || 1);
    this.sceneRoot.replaceChildren(renderer.domElement);
    this.renderer = renderer;
    this.scene = new THREE.Scene();
    this.scene.background = new THREE.Color(0x07111c);
    this.camera = new THREE.PerspectiveCamera(42, 1, 0.1, 500);
    this.resizeObserver = new ResizeObserver(() => {
      this.resizeThreeScene();
      const shot = this.shotEvents().find((event) => event.id === this.selectedShotId) ?? null;
      if (shot) {
        this.renderThreeScene(shot);
      }
    });
    this.resizeObserver.observe(this.sceneRoot);
    this.resizeThreeScene();
    return renderer;
  }

  private resizeThreeScene(): void {
    if (!this.renderer || !this.camera) {
      return;
    }

    const rect = this.sceneRoot.getBoundingClientRect();
    const width = Math.max(240, Math.round(rect.width));
    const height = Math.max(170, Math.round(rect.height));
    this.camera.aspect = width / height;
    this.camera.updateProjectionMatrix();
    this.renderer.setSize(width, height, false);
  }

  private clearThreeScene(): void {
    this.sceneRoot.replaceChildren();
    this.renderer?.dispose();
    this.renderer = null;
    this.scene = null;
    this.camera = null;
  }

  private handleChartClick(event: MouseEvent): void {
    const shots = this.shotEvents();
    const rect = this.chartCanvas.getBoundingClientRect();
    const click = { x: event.clientX - rect.left, y: event.clientY - rect.top };
    let nearest: ShotEvent | null = null;
    let nearestDistance = Number.POSITIVE_INFINITY;
    for (const shot of shots) {
      const point = fieldToChart(
        shot.shot.shot_touch_position ?? shot.shot.ball_position,
        rect.width,
        rect.height,
      );
      const distance = Math.hypot(point.x - click.x, point.y - click.y);
      if (distance < nearestDistance) {
        nearest = shot;
        nearestDistance = distance;
      }
    }
    if (nearest && nearestDistance <= 18) {
      this.selectedShotId = this.shotKey(nearest, shots.indexOf(nearest));
      this.options.cueTimelineEvent(nearest);
      this.render();
    }
  }

  private mustElement<T extends HTMLElement>(selector: string): T {
    const element = this.options.body.querySelector(selector);
    if (!(element instanceof HTMLElement)) {
      throw new Error(`Missing shot visualization element: ${selector}`);
    }
    return element as T;
  }

  private findSelectedShot(
    shots: readonly ShotEvent[],
  ): { shot: ShotEvent; index: number; key: string } | null {
    if (!this.selectedShotId) {
      return null;
    }
    for (let index = 0; index < shots.length; index += 1) {
      const shot = shots[index]!;
      const key = this.shotKey(shot, index);
      if (key === this.selectedShotId) {
        return { shot, index, key };
      }
    }
    return null;
  }

  private shotKey(shot: ShotEvent, index: number): string {
    return shot.id ?? `${shot.frame ?? "time"}:${shot.time}:${shot.playerId ?? "shot"}:${index}`;
  }

  private shotsSignature(shots: readonly ShotEvent[]): string {
    return shots
      .map((shot, index) => `${this.shotKey(shot, index)}:${shot.time}:${shot.shot.ball_speed}`)
      .join("|");
  }
}

function isShotEvent(event: ReplayTimelineEvent): event is ShotEvent {
  return event.kind === "shot" && Boolean(event.shot);
}

function isShotActive(shot: ShotEvent, currentTime: number | null): boolean {
  return currentTime !== null && Math.abs(currentTime - shot.time) < 1.5;
}

function drawField(context: CanvasRenderingContext2D, width: number, height: number): void {
  const pad = 18;
  context.fillStyle = "#0f2b24";
  context.fillRect(0, 0, width, height);
  context.strokeStyle = "rgba(226, 241, 236, 0.74)";
  context.lineWidth = 1.2;
  context.strokeRect(pad, pad, width - pad * 2, height - pad * 2);
  context.beginPath();
  context.moveTo(pad, height / 2);
  context.lineTo(width - pad, height / 2);
  context.stroke();
  context.beginPath();
  context.arc(width / 2, height / 2, 34, 0, Math.PI * 2);
  context.stroke();
  const blueGoal = fieldToChart({ x: 0, y: -FIELD_HALF_LENGTH, z: 0 }, width, height);
  const orangeGoal = fieldToChart({ x: 0, y: FIELD_HALF_LENGTH, z: 0 }, width, height);
  context.fillStyle = "#4fa3ff";
  context.fillRect(blueGoal.x - 28, blueGoal.y - 4, 56, 8);
  context.fillStyle = "#ff8a4c";
  context.fillRect(orangeGoal.x - 28, orangeGoal.y - 4, 56, 8);
}

function fieldToChart(position: Vec3Like, width: number, height: number): { x: number; y: number } {
  const pad = 18;
  const usableWidth = width - pad * 2;
  const usableHeight = height - pad * 2;
  return {
    x: pad + ((position.x + FIELD_HALF_WIDTH) / (FIELD_HALF_WIDTH * 2)) * usableWidth,
    y: pad + (1 - (position.y + FIELD_HALF_LENGTH) / (FIELD_HALF_LENGTH * 2)) * usableHeight,
  };
}

function goalFrame(orange: boolean): THREE.Group {
  const group = new THREE.Group();
  const material = new THREE.LineBasicMaterial({ color: orange ? 0xff8a4c : 0x4fa3ff });
  const y = (orange ? FIELD_HALF_LENGTH : -FIELD_HALF_LENGTH) * FIELD_SCALE;
  const halfWidth = GOAL_HALF_WIDTH * FIELD_SCALE;
  const height = GOAL_HEIGHT * FIELD_SCALE;
  const points = [
    new THREE.Vector3(-halfWidth, 0, y),
    new THREE.Vector3(-halfWidth, height, y),
    new THREE.Vector3(halfWidth, height, y),
    new THREE.Vector3(halfWidth, 0, y),
  ];
  group.add(new THREE.Line(new THREE.BufferGeometry().setFromPoints(points), material));
  return group;
}

function toThree(position: Vec3Like): THREE.Vector3 {
  return new THREE.Vector3(
    position.x * FIELD_SCALE,
    position.z * FIELD_SCALE,
    position.y * FIELD_SCALE,
  );
}

function toThreeDelta(position: Vec3Like): THREE.Vector3 {
  return new THREE.Vector3(
    position.x * FIELD_SCALE,
    position.z * FIELD_SCALE,
    position.y * FIELD_SCALE,
  );
}

function detail(label: string, value: string): HTMLElement {
  const wrapper = document.createElement("div");
  const term = document.createElement("dt");
  term.textContent = label;
  const description = document.createElement("dd");
  description.textContent = value;
  wrapper.append(term, description);
  return wrapper;
}

function formatTime(value: number | null | undefined): string {
  if (!Number.isFinite(value)) {
    return "--";
  }
  return `${value!.toFixed(2)}s`;
}

function formatSpeed(value: number | null | undefined): string {
  if (!Number.isFinite(value)) {
    return "--";
  }
  return `${Math.round(value!)} uu/s`;
}

function formatVec(value: Vec3Like | null | undefined): string {
  if (!value) {
    return "--";
  }
  return `${Math.round(value.x)}, ${Math.round(value.y)}, ${Math.round(value.z)}`;
}
