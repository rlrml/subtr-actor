import * as THREE from "three";
import type { ReplayModel, ReplayScene } from "@rlrml/player";
import type { DodgeEvent, StatsTimeline } from "./statsTimeline.ts";

const STYLE_ID = "subtr-actor-dodge-impulse-overlay-styles";
const BLUE_COLOR = 0x59c3ff;
const ORANGE_COLOR = 0xffc15c;
const ARROW_LENGTH_MIN = 260;
const ARROW_LENGTH_MAX = 760;
const LABEL_HEIGHT = 260;
const DEFAULT_DECAY_SECONDS = 2.5;

export interface DodgeImpulseMarker {
  id: string;
  time: number;
  frame: number;
  isTeamZero: boolean;
  playerId: string | null;
  playerName: string;
  position: THREE.Vector3;
  direction: THREE.Vector3;
  magnitude: number;
  confidence: number;
  directionLabel: string;
}

interface DodgeImpulseMarkerView {
  marker: DodgeImpulseMarker;
  arrow: THREE.ArrowHelper;
  label: HTMLDivElement;
}

function playerIdToString(playerId: Record<string, unknown> | undefined): string | null {
  if (!playerId) {
    return null;
  }

  const [kind, value] = Object.entries(playerId)[0] ?? ["Unknown", "unknown"];
  const normalizedValue = typeof value === "string" ? value : JSON.stringify(value);
  return `${kind}:${normalizedValue}`;
}

function findPlayerName(
  replay: ReplayModel,
  playerId: Record<string, unknown> | undefined,
): string {
  const normalizedId = playerIdToString(playerId);
  if (!normalizedId) {
    return "Unknown";
  }

  return replay.players.find((player) => player.id === normalizedId)?.name ?? normalizedId;
}

function titleCaseLabel(value: string): string {
  return value
    .split("_")
    .filter(Boolean)
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");
}

export function buildDodgeImpulseMarkers(
  statsTimeline: StatsTimeline,
  replay: ReplayModel,
): DodgeImpulseMarker[] {
  return (statsTimeline.events.dodge ?? []).flatMap((event: DodgeEvent, index) => {
    const dodgeImpulse = event.dodge_impulse;
    if (!dodgeImpulse) {
      return [];
    }
    const playerName = findPlayerName(replay, event.player);
    const playerId = playerIdToString(event.player);
    const time = replay.frames[event.frame]?.time ?? event.time;
    const direction = new THREE.Vector3(
      dodgeImpulse.estimated_direction[0],
      dodgeImpulse.estimated_direction[1],
      dodgeImpulse.estimated_direction[2],
    );
    if (direction.lengthSq() <= Number.EPSILON) {
      direction.set(1, 0, 0);
    }
    direction.normalize();

    return {
      id: `dodge-impulse:${event.frame}:${playerId}:${index}`,
      time,
      frame: event.frame,
      isTeamZero: event.is_team_0,
      playerId,
      playerName,
      position: new THREE.Vector3(
        dodgeImpulse.start_position[0],
        dodgeImpulse.start_position[1],
        dodgeImpulse.start_position[2] + 44,
      ),
      direction,
      magnitude: dodgeImpulse.estimated_impulse_magnitude,
      confidence: dodgeImpulse.confidence,
      directionLabel: dodgeImpulse.direction_label,
    };
  });
}

export function getVisibleDodgeImpulseMarkers(
  markers: DodgeImpulseMarker[],
  currentTime: number,
  decaySeconds: number,
): DodgeImpulseMarker[] {
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
    .sap-dodge-impulse-overlay-root {
      position: absolute;
      inset: 0;
      z-index: 2;
      pointer-events: none;
      overflow: hidden;
      font-family: "IBM Plex Sans", "Avenir Next", sans-serif;
    }

    .sap-dodge-impulse-overlay-label {
      position: absolute;
      min-width: max-content;
      padding: 0.24rem 0.58rem;
      border-radius: 999px;
      border: 1px solid rgba(255, 255, 255, 0.18);
      background: rgba(6, 12, 18, 0.82);
      color: #f5fbff;
      font-size: 0.7rem;
      font-weight: 700;
      transform: translate(-50%, -100%);
      text-shadow: 0 1px 2px rgba(0, 0, 0, 0.7);
      box-shadow: 0 10px 24px rgba(0, 0, 0, 0.24);
      backdrop-filter: blur(8px);
      will-change: transform, opacity;
    }

    .sap-dodge-impulse-overlay-label-blue {
      border-color: rgba(89, 195, 255, 0.5);
      background: rgba(17, 47, 73, 0.84);
    }

    .sap-dodge-impulse-overlay-label-orange {
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

export class DodgeImpulseOverlay {
  private readonly scene: ReplayScene;
  private readonly container: HTMLElement;
  private readonly group = new THREE.Group();
  private readonly labelRoot: HTMLDivElement;
  private readonly projectedPosition = new THREE.Vector3();
  private readonly worldPosition = new THREE.Vector3();
  private readonly labelOffset = new THREE.Vector3(0, 0, LABEL_HEIGHT);
  private readonly markers: DodgeImpulseMarker[];
  private readonly views = new Map<string, DodgeImpulseMarkerView>();
  private changedContainerPosition = false;
  private originalContainerPosition = "";
  private decaySeconds = DEFAULT_DECAY_SECONDS;

  constructor(
    scene: ReplayScene,
    container: HTMLElement,
    replay: ReplayModel,
    statsTimeline: StatsTimeline,
  ) {
    ensureStyles();
    this.scene = scene;
    this.container = container;
    this.markers = buildDodgeImpulseMarkers(statsTimeline, replay);

    if (getComputedStyle(container).position === "static") {
      this.changedContainerPosition = true;
      this.originalContainerPosition = container.style.position;
      container.style.position = "relative";
    }

    this.group.name = "dodge-impulse-overlay";
    this.scene.replayRoot.add(this.group);

    this.labelRoot = document.createElement("div");
    this.labelRoot.className = "sap-dodge-impulse-overlay-root";
    this.container.append(this.labelRoot);
  }

  update(currentTime: number): void {
    const visibleMarkers = getVisibleDodgeImpulseMarkers(
      this.markers,
      currentTime,
      this.decaySeconds,
    );
    const visibleIds = new Set(visibleMarkers.map((marker) => marker.id));

    for (const [id, view] of this.views.entries()) {
      if (visibleIds.has(id)) {
        continue;
      }
      view.arrow.removeFromParent();
      view.arrow.dispose();
      view.label.remove();
      this.views.delete(id);
    }

    for (const marker of visibleMarkers) {
      const age = Math.max(0, currentTime - marker.time);
      const life = Math.max(0, 1 - age / this.decaySeconds);
      const view = this.ensureView(marker);
      const opacity = 0.24 + 0.72 * life;
      const length =
        ARROW_LENGTH_MIN +
        Math.min(1, marker.magnitude / 450) * (ARROW_LENGTH_MAX - ARROW_LENGTH_MIN);

      view.arrow.position.copy(marker.position);
      view.arrow.setDirection(marker.direction);
      view.arrow.setLength(length, 70, 38);
      (view.arrow.cone.material as THREE.MeshBasicMaterial).opacity = opacity;
      (view.arrow.line.material as THREE.LineBasicMaterial).opacity = opacity;

      this.worldPosition.copy(marker.position).add(this.labelOffset);
      const visible = projectToContainer(
        this.worldPosition,
        this.scene.camera,
        this.container,
        this.projectedPosition,
      );
      view.label.style.display = visible ? "block" : "none";
      if (visible) {
        view.label.style.left = `${this.projectedPosition.x}px`;
        view.label.style.top = `${this.projectedPosition.y}px`;
        view.label.style.opacity = `${0.42 + 0.58 * life}`;
      }
    }
  }

  dispose(): void {
    for (const view of this.views.values()) {
      view.arrow.removeFromParent();
      view.arrow.dispose();
      view.label.remove();
    }
    this.views.clear();
    this.group.removeFromParent();
    this.labelRoot.remove();

    if (this.changedContainerPosition) {
      this.container.style.position = this.originalContainerPosition;
    }
  }

  private ensureView(marker: DodgeImpulseMarker): DodgeImpulseMarkerView {
    const existing = this.views.get(marker.id);
    if (existing) {
      return existing;
    }

    const color = marker.isTeamZero ? BLUE_COLOR : ORANGE_COLOR;
    const arrow = new THREE.ArrowHelper(marker.direction, marker.position, ARROW_LENGTH_MIN, color);
    arrow.renderOrder = 35;
    arrow.line.material = new THREE.LineBasicMaterial({
      color,
      transparent: true,
      opacity: 0.9,
      depthWrite: false,
      depthTest: false,
    });
    arrow.cone.material = new THREE.MeshBasicMaterial({
      color,
      transparent: true,
      opacity: 0.9,
      depthWrite: false,
      depthTest: false,
    });
    this.group.add(arrow);

    const label = document.createElement("div");
    label.className = `sap-dodge-impulse-overlay-label ${
      marker.isTeamZero
        ? "sap-dodge-impulse-overlay-label-blue"
        : "sap-dodge-impulse-overlay-label-orange"
    }`;
    label.textContent = `${marker.playerName} ${titleCaseLabel(marker.directionLabel)} ${Math.round(
      marker.confidence * 100,
    )}%`;
    this.labelRoot.append(label);

    const view = { marker, arrow, label };
    this.views.set(marker.id, view);
    return view;
  }
}
