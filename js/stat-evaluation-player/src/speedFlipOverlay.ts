import * as THREE from "three";
import type { ReplayModel, ReplayScene } from "subtr-actor-player";
import type { StatsTimeline } from "./statsTimeline.ts";

const STYLE_ID = "subtr-actor-speed-flip-overlay-styles";
const BLUE_COLOR = 0x59c3ff;
const ORANGE_COLOR = 0xffc15c;
const HIGH_CONFIDENCE_COLOR = 0xf6f6f3;
const RING_INNER_RADIUS = 150;
const RING_OUTER_RADIUS = 230;
const LABEL_HEIGHT = 220;
const DEFAULT_DECAY_SECONDS = 4;

export interface SpeedFlipMarker {
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
  quality: number;
  qualityLabel: string;
}

interface SpeedFlipMarkerView {
  marker: SpeedFlipMarker;
  ring: THREE.Mesh;
  material: THREE.MeshBasicMaterial;
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

export function buildSpeedFlipMarkers(
  statsTimeline: StatsTimeline,
  replay: ReplayModel,
): SpeedFlipMarker[] {
  return statsTimeline.events.speed_flip.map((event) => {
    const playerName = findPlayerName(replay, event.player);
    const playerId = playerIdToString(event.player);
    const time = replay.frames[event.frame]?.time ?? event.time;
    const quality = event.confidence;

    return {
      id: `speed-flip:${event.frame}:${playerId}:${Math.round(quality * 1000)}`,
      time,
      frame: event.frame,
      isTeamZero: event.is_team_0,
      playerId,
      playerName,
      position: {
        x: event.start_position[0],
        y: event.start_position[1],
        z: event.start_position[2],
      },
      quality,
      qualityLabel: `${Math.round(quality * 100)}%`,
    };
  });
}

export function getVisibleSpeedFlipMarkers(
  markers: SpeedFlipMarker[],
  currentTime: number,
  decaySeconds: number,
): SpeedFlipMarker[] {
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
    .sap-speed-flip-overlay-root {
      position: absolute;
      inset: 0;
      z-index: 2;
      pointer-events: none;
      overflow: hidden;
      font-family: "IBM Plex Sans", "Avenir Next", sans-serif;
    }

    .sap-speed-flip-overlay-label {
      position: absolute;
      min-width: max-content;
      padding: 0.24rem 0.6rem;
      border-radius: 999px;
      border: 1px solid rgba(255, 255, 255, 0.18);
      background: rgba(6, 12, 18, 0.82);
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

    .sap-speed-flip-overlay-label-blue {
      border-color: rgba(89, 195, 255, 0.5);
      background: rgba(17, 47, 73, 0.84);
    }

    .sap-speed-flip-overlay-label-orange {
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

export class SpeedFlipOverlay {
  private readonly scene: ReplayScene;
  private readonly container: HTMLElement;
  private readonly group = new THREE.Group();
  private readonly labelRoot: HTMLDivElement;
  private readonly projectedPosition = new THREE.Vector3();
  private readonly worldPosition = new THREE.Vector3();
  private readonly labelOffset = new THREE.Vector3(0, 0, LABEL_HEIGHT);
  private readonly markers: SpeedFlipMarker[];
  private readonly views = new Map<string, SpeedFlipMarkerView>();
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
    this.markers = buildSpeedFlipMarkers(statsTimeline, replay);

    if (getComputedStyle(container).position === "static") {
      this.changedContainerPosition = true;
      this.originalContainerPosition = container.style.position;
      container.style.position = "relative";
    }

    this.group.name = "speed-flip-overlay";
    this.scene.replayRoot.add(this.group);

    this.labelRoot = document.createElement("div");
    this.labelRoot.className = "sap-speed-flip-overlay-root";
    this.container.append(this.labelRoot);
  }

  update(currentTime: number): void {
    const visibleMarkers = getVisibleSpeedFlipMarkers(
      this.markers,
      currentTime,
      this.decaySeconds,
    );
    const visibleIds = new Set(visibleMarkers.map((marker) => marker.id));

    for (const [id, view] of this.views.entries()) {
      if (visibleIds.has(id)) {
        continue;
      }
      view.ring.removeFromParent();
      view.ring.geometry.dispose();
      view.material.dispose();
      view.label.remove();
      this.views.delete(id);
    }

    for (const marker of visibleMarkers) {
      const age = Math.max(0, currentTime - marker.time);
      const life = Math.max(0, 1 - age / this.decaySeconds);
      const view = this.ensureView(marker);
      const opacity = 0.16 + 0.56 * life;
      const scale = 0.96 + (1 - life) * 0.22;

      view.material.opacity = opacity;
      view.ring.position.set(
        marker.position.x,
        marker.position.y,
        marker.position.z + 14,
      );
      view.ring.scale.setScalar(scale + marker.quality * 0.08);

      this.worldPosition.set(
        marker.position.x,
        marker.position.y,
        marker.position.z,
      ).add(this.labelOffset);
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
      view.ring.removeFromParent();
      view.ring.geometry.dispose();
      view.material.dispose();
      view.label.remove();
    }
    this.views.clear();
    this.group.removeFromParent();
    this.labelRoot.remove();

    if (this.changedContainerPosition) {
      this.container.style.position = this.originalContainerPosition;
    }
  }

  private ensureView(marker: SpeedFlipMarker): SpeedFlipMarkerView {
    const existing = this.views.get(marker.id);
    if (existing) {
      return existing;
    }

    const material = new THREE.MeshBasicMaterial({
      color: marker.quality >= 0.75
        ? HIGH_CONFIDENCE_COLOR
        : marker.isTeamZero
          ? BLUE_COLOR
          : ORANGE_COLOR,
      transparent: true,
      opacity: 0.8,
      side: THREE.DoubleSide,
      depthWrite: false,
      depthTest: false,
    });
    const geometry = new THREE.RingGeometry(RING_INNER_RADIUS, RING_OUTER_RADIUS, 48);
    const ring = new THREE.Mesh(geometry, material);
    ring.renderOrder = 30;
    this.group.add(ring);

    const label = document.createElement("div");
    label.className = `sap-speed-flip-overlay-label ${
      marker.isTeamZero
        ? "sap-speed-flip-overlay-label-blue"
        : "sap-speed-flip-overlay-label-orange"
    }`;
    label.textContent = `${marker.playerName} speed flip ${marker.qualityLabel}`;
    this.labelRoot.append(label);

    const view = { marker, ring, material, label };
    this.views.set(marker.id, view);
    return view;
  }
}
