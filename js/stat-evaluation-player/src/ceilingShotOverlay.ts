import * as THREE from "three";
import type { ReplayModel, ReplayScene } from "subtr-actor-player";
import type { StatsTimeline } from "./statsTimeline.ts";

const STYLE_ID = "subtr-actor-ceiling-shot-overlay-styles";
const BLUE_COLOR = 0x59c3ff;
const ORANGE_COLOR = 0xffc15c;
const HIGH_CONFIDENCE_COLOR = 0xf6f6f3;
const RING_INNER_RADIUS = 140;
const RING_OUTER_RADIUS = 215;
const LABEL_HEIGHT = 220;
const DEFAULT_DECAY_SECONDS = 4.5;

export interface CeilingShotMarker {
  id: string;
  time: number;
  frame: number;
  isTeamZero: boolean;
  playerId: string | null;
  playerName: string;
  ceilingContactPosition: {
    x: number;
    y: number;
    z: number;
  };
  touchPosition: {
    x: number;
    y: number;
    z: number;
  };
  quality: number;
  qualityLabel: string;
}

interface CeilingShotMarkerView {
  marker: CeilingShotMarker;
  ring: THREE.Mesh;
  ringMaterial: THREE.MeshBasicMaterial;
  beam: THREE.Line;
  beamGeometry: THREE.BufferGeometry;
  beamMaterial: THREE.LineBasicMaterial;
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

export function buildCeilingShotMarkers(
  statsTimeline: StatsTimeline,
  replay: ReplayModel,
): CeilingShotMarker[] {
  return statsTimeline.events.ceiling_shot.map((event) => {
    const playerName = findPlayerName(replay, event.player);
    const playerId = playerIdToString(event.player);
    const time = replay.frames[event.frame]?.time ?? event.time;
    const quality = event.confidence;

    return {
      id: `ceiling-shot:${event.frame}:${playerId}:${Math.round(quality * 1000)}`,
      time,
      frame: event.frame,
      isTeamZero: event.is_team_0,
      playerId,
      playerName,
      ceilingContactPosition: {
        x: event.ceiling_contact_position[0],
        y: event.ceiling_contact_position[1],
        z: event.ceiling_contact_position[2],
      },
      touchPosition: {
        x: event.touch_position[0],
        y: event.touch_position[1],
        z: event.touch_position[2],
      },
      quality,
      qualityLabel: `${Math.round(quality * 100)}%`,
    };
  });
}

export function getVisibleCeilingShotMarkers(
  markers: CeilingShotMarker[],
  currentTime: number,
  decaySeconds: number,
): CeilingShotMarker[] {
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
    .sap-ceiling-shot-overlay-root {
      position: absolute;
      inset: 0;
      z-index: 2;
      pointer-events: none;
      overflow: hidden;
      font-family: "IBM Plex Sans", "Avenir Next", sans-serif;
    }

    .sap-ceiling-shot-overlay-label {
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

    .sap-ceiling-shot-overlay-label-blue {
      border-color: rgba(89, 195, 255, 0.5);
      background: rgba(17, 47, 73, 0.84);
    }

    .sap-ceiling-shot-overlay-label-orange {
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

export class CeilingShotOverlay {
  private readonly scene: ReplayScene;
  private readonly container: HTMLElement;
  private readonly group = new THREE.Group();
  private readonly labelRoot: HTMLDivElement;
  private readonly projectedPosition = new THREE.Vector3();
  private readonly worldPosition = new THREE.Vector3();
  private readonly labelOffset = new THREE.Vector3(0, 0, LABEL_HEIGHT);
  private readonly markers: CeilingShotMarker[];
  private readonly views = new Map<string, CeilingShotMarkerView>();
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
    this.markers = buildCeilingShotMarkers(statsTimeline, replay);

    if (getComputedStyle(container).position === "static") {
      this.changedContainerPosition = true;
      this.originalContainerPosition = container.style.position;
      container.style.position = "relative";
    }

    this.group.name = "ceiling-shot-overlay";
    this.scene.replayRoot.add(this.group);

    this.labelRoot = document.createElement("div");
    this.labelRoot.className = "sap-ceiling-shot-overlay-root";
    this.container.append(this.labelRoot);
  }

  update(currentTime: number): void {
    const visibleMarkers = getVisibleCeilingShotMarkers(
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
      view.ringMaterial.dispose();
      view.beam.removeFromParent();
      view.beamGeometry.dispose();
      view.beamMaterial.dispose();
      view.label.remove();
      this.views.delete(id);
    }

    for (const marker of visibleMarkers) {
      const age = Math.max(0, currentTime - marker.time);
      const life = Math.max(0, 1 - age / this.decaySeconds);
      const view = this.ensureView(marker);
      const opacity = 0.14 + 0.60 * life;
      const scale = 0.94 + (1 - life) * 0.18;

      view.ringMaterial.opacity = opacity;
      view.beamMaterial.opacity = 0.18 + 0.55 * life;
      view.ring.position.set(
        marker.touchPosition.x,
        marker.touchPosition.y,
        marker.touchPosition.z + 12,
      );
      view.ring.scale.setScalar(scale + marker.quality * 0.08);

      this.worldPosition.set(
        marker.touchPosition.x,
        marker.touchPosition.y,
        marker.touchPosition.z,
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
      view.ringMaterial.dispose();
      view.beam.removeFromParent();
      view.beamGeometry.dispose();
      view.beamMaterial.dispose();
      view.label.remove();
    }
    this.views.clear();
    this.group.removeFromParent();
    this.labelRoot.remove();

    if (this.changedContainerPosition) {
      this.container.style.position = this.originalContainerPosition;
    }
  }

  private ensureView(marker: CeilingShotMarker): CeilingShotMarkerView {
    const existing = this.views.get(marker.id);
    if (existing) {
      return existing;
    }

    const color = marker.quality >= 0.8
      ? HIGH_CONFIDENCE_COLOR
      : marker.isTeamZero
        ? BLUE_COLOR
        : ORANGE_COLOR;

    const ringMaterial = new THREE.MeshBasicMaterial({
      color,
      transparent: true,
      opacity: 0.8,
      side: THREE.DoubleSide,
      depthWrite: false,
      depthTest: false,
    });
    const ringGeometry = new THREE.RingGeometry(RING_INNER_RADIUS, RING_OUTER_RADIUS, 48);
    const ring = new THREE.Mesh(ringGeometry, ringMaterial);
    ring.renderOrder = 30;
    this.group.add(ring);

    const beamGeometry = new THREE.BufferGeometry().setFromPoints([
      new THREE.Vector3(
        marker.ceilingContactPosition.x,
        marker.ceilingContactPosition.y,
        marker.ceilingContactPosition.z,
      ),
      new THREE.Vector3(
        marker.touchPosition.x,
        marker.touchPosition.y,
        marker.touchPosition.z,
      ),
    ]);
    const beamMaterial = new THREE.LineBasicMaterial({
      color,
      transparent: true,
      opacity: 0.7,
      depthWrite: false,
      depthTest: false,
    });
    const beam = new THREE.Line(beamGeometry, beamMaterial);
    beam.renderOrder = 29;
    this.group.add(beam);

    const label = document.createElement("div");
    label.className = `sap-ceiling-shot-overlay-label ${
      marker.isTeamZero
        ? "sap-ceiling-shot-overlay-label-blue"
        : "sap-ceiling-shot-overlay-label-orange"
    }`;
    label.textContent = `${marker.playerName} ceiling shot ${marker.qualityLabel}`;
    this.labelRoot.append(label);

    const view = { marker, ring, ringMaterial, beam, beamGeometry, beamMaterial, label };
    this.views.set(marker.id, view);
    return view;
  }
}
