import * as THREE from "three";
import type { ReplayModel, ReplayScene } from "subtr-actor-player";
import type { PlayerStatsSnapshot, StatsFrame, StatsTimeline } from "./statsTimeline.ts";

const STYLE_ID = "subtr-actor-touch-overlay-styles";
const BLUE_TOUCH_COLOR = 0x59c3ff;
const ORANGE_TOUCH_COLOR = 0xffc15c;
const TOUCH_RING_INNER_RADIUS = 120;
const TOUCH_RING_OUTER_RADIUS = 196;
const TOUCH_RING_HEIGHT = 24;
const TOUCH_LABEL_HEIGHT = 210;
const DEFAULT_DECAY_SECONDS = 5;
const TOUCH_CREDIT_EPSILON = 0.1;
const ADVANCEMENT_ARROW_MIN_LENGTH = 48;

export type TouchOverlayMode = "markers" | "advancement";

export interface TouchMarker {
  id: string;
  time: number;
  frame: number;
  isTeamZero: boolean;
  playerId: string | null;
  playerName: string;
  position: {
    x: number;
    y: number;
    z: number;
  };
  endPosition: {
    x: number;
    y: number;
    z: number;
  };
  totalBallTravelDistance: number;
  totalBallAdvanceDistance: number;
  totalBallRetreatDistance: number;
}

interface TouchMarkerView {
  marker: TouchMarker;
  ring: THREE.Mesh;
  material: THREE.MeshBasicMaterial;
  arrow: THREE.ArrowHelper;
  label: HTMLDivElement;
}

export function getLastTouchPlayer(statsFrame: StatsFrame): PlayerStatsSnapshot | null {
  return statsFrame.players.find((player) => player.touch?.is_last_touch) ?? null;
}

export function playerIdToString(playerId: Record<string, unknown>): string {
  const [kind, value] = Object.entries(playerId)[0] ?? ["Unknown", "unknown"];
  const normalizedValue = typeof value === "string" ? value : JSON.stringify(value);
  return `${kind}:${normalizedValue}`;
}

function positiveDelta(current: number, previous: number): number {
  return Math.max(0, current - previous);
}

function touchCreditLabel(marker: TouchMarker, mode: TouchOverlayMode): string {
  if (mode === "markers") {
    return marker.playerName;
  }

  const advance = Math.round(marker.totalBallAdvanceDistance);
  const retreat = Math.round(marker.totalBallRetreatDistance);
  if (advance > 0 && retreat > 0) {
    return `${marker.playerName} +${advance} / -${retreat} uu`;
  }
  if (retreat > 0) {
    return `${marker.playerName} -${retreat} uu`;
  }
  return `${marker.playerName} +${advance} uu`;
}

export function buildTouchMarkers(
  statsTimeline: StatsTimeline,
  replay: ReplayModel,
): TouchMarker[] {
  const activeMarkerByPlayer = new Map<string, number>();
  const markers: TouchMarker[] = [];
  const events = [
    ...(statsTimeline.events?.touch ?? []).map((event, index) => ({
      kind: "touch" as const,
      frame: event.frame,
      time: event.time,
      index,
      event,
    })),
    ...(statsTimeline.events?.touch_ball_movement ?? []).map((event, index) => ({
      kind: "movement" as const,
      frame: event.frame,
      time: event.time,
      index,
      event,
    })),
  ].sort((left, right) => {
    if (left.frame !== right.frame) {
      return left.frame - right.frame;
    }
    if (left.time !== right.time) {
      return left.time - right.time;
    }
    if (left.kind !== right.kind) {
      return left.kind === "touch" ? -1 : 1;
    }
    return left.index - right.index;
  });

  for (const item of events) {
    if (item.kind === "touch") {
      const event = item.event;
      const playerId = playerIdToString(event.player);
      const ballPosition = replay.ballFrames[event.frame]?.position;
      if (!ballPosition) {
        continue;
      }
      const markerIndex = markers.length;
      markers.push({
        id: `touch-stat:${event.frame}:${playerId}:${markerIndex + 1}`,
        time: replay.frames[event.frame]?.time ?? event.time,
        frame: event.frame,
        isTeamZero: event.is_team_0,
        playerId,
        playerName: replay.players.find((player) => player.id === playerId)?.name ?? playerId,
        position: {
          x: ballPosition.x,
          y: ballPosition.y,
          z: ballPosition.z,
        },
        endPosition: {
          x: ballPosition.x,
          y: ballPosition.y,
          z: ballPosition.z,
        },
        totalBallTravelDistance: 0,
        totalBallAdvanceDistance: 0,
        totalBallRetreatDistance: 0,
      });
      activeMarkerByPlayer.set(playerId, markerIndex);
      continue;
    }

    const event = item.event;
    const playerId = playerIdToString(event.player);
    const activeMarkerIndex = activeMarkerByPlayer.get(playerId);
    const frameBallPosition = replay.ballFrames[event.frame]?.position;
    if (activeMarkerIndex === undefined || !frameBallPosition) {
      continue;
    }
    const activeMarker = markers[activeMarkerIndex];
    if (!activeMarker) {
      continue;
    }
    activeMarker.totalBallTravelDistance += positiveDelta(event.travel_distance, 0);
    activeMarker.totalBallAdvanceDistance += positiveDelta(event.advance_distance, 0);
    activeMarker.totalBallRetreatDistance += positiveDelta(event.retreat_distance, 0);
    activeMarker.endPosition = {
      x: frameBallPosition.x,
      y: frameBallPosition.y,
      z: frameBallPosition.z,
    };
  }

  return markers;
}

export function getVisibleTouchMarkers(
  markers: TouchMarker[],
  currentTime: number,
  decaySeconds: number,
): TouchMarker[] {
  const effectiveDecay = Math.max(0.1, decaySeconds);
  return markers.filter((marker) => {
    const age = currentTime - marker.time;
    return age >= 0 && age <= effectiveDecay;
  });
}

function ensureStyles(): void {
  if (document.getElementById(STYLE_ID)) {
    return;
  }

  const style = document.createElement("style");
  style.id = STYLE_ID;
  style.textContent = `
    .sap-touch-overlay-root {
      position: absolute;
      inset: 0;
      z-index: 2;
      pointer-events: none;
      overflow: hidden;
      font-family: "IBM Plex Sans", "Avenir Next", sans-serif;
    }

    .sap-touch-overlay-label {
      position: absolute;
      min-width: max-content;
      padding: 0.22rem 0.55rem;
      border-radius: 999px;
      border: 1px solid rgba(255, 255, 255, 0.16);
      background: rgba(6, 12, 18, 0.8);
      color: #f5fbff;
      font-size: 0.72rem;
      font-weight: 700;
      letter-spacing: 0.01em;
      transform: translate(-50%, -100%);
      text-shadow: 0 1px 2px rgba(0, 0, 0, 0.7);
      box-shadow: 0 10px 24px rgba(0, 0, 0, 0.24);
      backdrop-filter: blur(8px);
      will-change: transform, opacity;
    }

    .sap-touch-overlay-label-advancement {
      min-width: 4.8rem;
      text-align: center;
    }

    .sap-touch-overlay-label-blue {
      border-color: rgba(89, 195, 255, 0.5);
      background: rgba(17, 47, 73, 0.84);
    }

    .sap-touch-overlay-label-orange {
      border-color: rgba(255, 193, 92, 0.5);
      background: rgba(76, 41, 7, 0.84);
    }
  `;
  document.head.append(style);
}

function projectToContainer(
  worldPosition: THREE.Vector3,
  camera: THREE.Camera,
  container: HTMLElement,
  out: THREE.Vector3,
): boolean {
  out.copy(worldPosition).project(camera);

  if (out.z < -1 || out.z > 1) {
    return false;
  }

  const width = container.clientWidth || 1;
  const height = container.clientHeight || 1;
  out.x = ((out.x + 1) * width) / 2;
  out.y = ((1 - out.y) * height) / 2;

  if (out.x < -100 || out.x > width + 100 || out.y < -100 || out.y > height + 100) {
    return false;
  }

  return true;
}

function arrowMaterials(arrow: THREE.ArrowHelper): THREE.Material[] {
  return [arrow.line.material, arrow.cone.material].flatMap((material) =>
    Array.isArray(material) ? material : [material],
  );
}

function setArrowOpacity(arrow: THREE.ArrowHelper, opacity: number): void {
  for (const material of arrowMaterials(arrow)) {
    material.transparent = true;
    material.opacity = opacity;
    material.depthWrite = false;
    material.depthTest = false;
  }
}

function disposeArrow(arrow: THREE.ArrowHelper): void {
  arrow.removeFromParent();
  arrow.line.geometry.dispose();
  arrow.cone.geometry.dispose();
  for (const material of arrowMaterials(arrow)) {
    material.dispose();
  }
}

export class TouchEventOverlay {
  private readonly scene: ReplayScene;
  private readonly container: HTMLElement;
  private readonly group = new THREE.Group();
  private readonly labelRoot: HTMLDivElement;
  private readonly projectedPosition = new THREE.Vector3();
  private readonly worldPosition = new THREE.Vector3();
  private readonly arrowStart = new THREE.Vector3();
  private readonly arrowEnd = new THREE.Vector3();
  private readonly arrowDirection = new THREE.Vector3();
  private readonly labelOffset = new THREE.Vector3(0, 0, TOUCH_LABEL_HEIGHT);
  private readonly markers: TouchMarker[];
  private readonly views = new Map<string, TouchMarkerView>();
  private changedContainerPosition = false;
  private originalContainerPosition = "";
  private decaySeconds = DEFAULT_DECAY_SECONDS;
  private mode: TouchOverlayMode = "markers";

  constructor(
    scene: ReplayScene,
    container: HTMLElement,
    replay: ReplayModel,
    statsTimeline: StatsTimeline,
    options?: {
      decaySeconds?: number;
      mode?: TouchOverlayMode;
    },
  ) {
    ensureStyles();
    this.scene = scene;
    this.container = container;
    this.decaySeconds = Math.max(0.1, options?.decaySeconds ?? DEFAULT_DECAY_SECONDS);
    this.mode = options?.mode ?? "markers";
    this.labelOffset.set(0, 0, TOUCH_LABEL_HEIGHT);
    this.markers = buildTouchMarkers(statsTimeline, replay);

    if (getComputedStyle(container).position === "static") {
      this.changedContainerPosition = true;
      this.originalContainerPosition = container.style.position;
      container.style.position = "relative";
    }

    this.group.name = "touch-event-overlay";
    this.scene.replayRoot.add(this.group);

    this.labelRoot = document.createElement("div");
    this.labelRoot.className = "sap-touch-overlay-root";
    this.container.append(this.labelRoot);
  }

  getDecaySeconds(): number {
    return this.decaySeconds;
  }

  setDecaySeconds(value: number): void {
    this.decaySeconds = Math.max(0.1, value);
  }

  getMode(): TouchOverlayMode {
    return this.mode;
  }

  setMode(mode: TouchOverlayMode): void {
    this.mode = mode;
  }

  update(currentTime: number): void {
    const visibleMarkers = getVisibleTouchMarkers(this.markers, currentTime, this.decaySeconds);
    const visibleIds = new Set(visibleMarkers.map((marker) => marker.id));

    for (const [id, view] of this.views.entries()) {
      if (visibleIds.has(id)) {
        continue;
      }
      view.ring.removeFromParent();
      view.ring.geometry.dispose();
      view.material.dispose();
      disposeArrow(view.arrow);
      view.label.remove();
      this.views.delete(id);
    }

    for (const marker of visibleMarkers) {
      const age = Math.max(0, currentTime - marker.time);
      const life = Math.max(0, 1 - age / this.decaySeconds);
      const view = this.ensureView(marker);
      const baseOpacity = 0.1 + 0.6 * life;
      const scale = 0.95 + (1 - life) * 0.28;

      view.material.opacity = baseOpacity;
      view.ring.position.set(
        marker.position.x,
        marker.position.y,
        marker.position.z + TOUCH_RING_HEIGHT,
      );
      view.ring.scale.setScalar(scale);
      view.label.textContent = touchCreditLabel(marker, this.mode);
      view.label.classList.toggle(
        "sap-touch-overlay-label-advancement",
        this.mode === "advancement",
      );

      this.updateArrow(view, marker, baseOpacity);

      this.worldPosition.set(marker.position.x, marker.position.y, marker.position.z);
      this.worldPosition.add(this.labelOffset);
      this.scene.replayRoot.localToWorld(this.worldPosition);

      if (
        projectToContainer(
          this.worldPosition,
          this.scene.camera,
          this.container,
          this.projectedPosition,
        )
      ) {
        view.label.hidden = false;
        view.label.style.opacity = `${0.22 + 0.78 * life}`;
        view.label.style.transform =
          `translate(${this.projectedPosition.x.toFixed(1)}px, ` +
          `${this.projectedPosition.y.toFixed(1)}px) translate(-50%, -100%)`;
      } else {
        view.label.hidden = true;
      }
    }
  }

  dispose(): void {
    for (const view of this.views.values()) {
      view.ring.removeFromParent();
      view.ring.geometry.dispose();
      view.material.dispose();
      disposeArrow(view.arrow);
      view.label.remove();
    }
    this.views.clear();
    this.group.removeFromParent();
    this.labelRoot.remove();
    if (this.changedContainerPosition) {
      this.container.style.position = this.originalContainerPosition;
      this.changedContainerPosition = false;
    }
  }

  private ensureView(marker: TouchMarker): TouchMarkerView {
    const existing = this.views.get(marker.id);
    if (existing) {
      return existing;
    }

    const material = new THREE.MeshBasicMaterial({
      color: marker.isTeamZero ? BLUE_TOUCH_COLOR : ORANGE_TOUCH_COLOR,
      transparent: true,
      opacity: 0.7,
      side: THREE.DoubleSide,
      depthWrite: false,
      depthTest: false,
    });
    const ring = new THREE.Mesh(
      new THREE.RingGeometry(TOUCH_RING_INNER_RADIUS, TOUCH_RING_OUTER_RADIUS, 48),
      material,
    );
    ring.rotation.x = -Math.PI / 2;
    ring.renderOrder = 40;
    this.group.add(ring);

    const arrow = new THREE.ArrowHelper(
      new THREE.Vector3(0, 1, 0),
      new THREE.Vector3(),
      1,
      marker.isTeamZero ? BLUE_TOUCH_COLOR : ORANGE_TOUCH_COLOR,
      1,
      1,
    );
    arrow.visible = false;
    arrow.renderOrder = 45;
    arrow.line.renderOrder = 45;
    arrow.cone.renderOrder = 45;
    setArrowOpacity(arrow, 0.7);
    this.group.add(arrow);

    const label = document.createElement("div");
    label.className = `sap-touch-overlay-label ${
      marker.isTeamZero ? "sap-touch-overlay-label-blue" : "sap-touch-overlay-label-orange"
    }`;
    label.textContent = marker.playerName;
    label.hidden = true;
    this.labelRoot.append(label);

    const view = {
      marker,
      ring,
      material,
      arrow,
      label,
    };
    this.views.set(marker.id, view);
    return view;
  }

  private updateArrow(view: TouchMarkerView, marker: TouchMarker, opacity: number): void {
    if (this.mode !== "advancement" || marker.totalBallTravelDistance <= TOUCH_CREDIT_EPSILON) {
      view.arrow.visible = false;
      return;
    }

    this.arrowStart.set(
      marker.position.x,
      marker.position.y,
      marker.position.z + TOUCH_RING_HEIGHT * 2,
    );
    this.arrowEnd.set(
      marker.endPosition.x,
      marker.endPosition.y,
      marker.endPosition.z + TOUCH_RING_HEIGHT * 2,
    );
    this.arrowDirection.copy(this.arrowEnd).sub(this.arrowStart);
    const length = this.arrowDirection.length();
    if (length < ADVANCEMENT_ARROW_MIN_LENGTH) {
      view.arrow.visible = false;
      return;
    }

    this.arrowDirection.normalize();
    view.arrow.visible = true;
    view.arrow.position.copy(this.arrowStart);
    view.arrow.setDirection(this.arrowDirection);
    view.arrow.setLength(
      length,
      Math.min(140, Math.max(42, length * 0.18)),
      Math.min(86, Math.max(24, length * 0.1)),
    );
    setArrowOpacity(view.arrow, Math.min(0.86, opacity + 0.12));
  }
}
