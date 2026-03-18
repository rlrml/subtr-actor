import * as THREE from "three";
import type { ReplayModel } from "../../player/src/types.ts";
import type { ReplayScene } from "../../player/src/scene.ts";
import type {
  FiftyFiftyEvent,
  StatsTimeline,
} from "./statsTimeline.ts";

const STYLE_ID = "subtr-actor-fifty-fifty-overlay-styles";
const BLUE_COLOR = 0x59c3ff;
const ORANGE_COLOR = 0xffc15c;
const NEUTRAL_COLOR = 0xf3f6f8;
const LABEL_HEIGHT = 180;
const DEFAULT_DECAY_SECONDS = 4;

export interface FiftyFiftyMarker {
  id: string;
  time: number;
  label: string;
  labelClassName: string;
  axisStart: THREE.Vector3;
  axisEnd: THREE.Vector3;
  midpoint: THREE.Vector3;
  winnerIsTeamZero: boolean | null;
}

interface FiftyFiftyMarkerView {
  marker: FiftyFiftyMarker;
  line: THREE.Line;
  material: THREE.LineBasicMaterial;
  label: HTMLDivElement;
}

function playerIdToString(playerId: Record<string, string> | undefined): string | null {
  if (!playerId) {
    return null;
  }

  const [kind, value] = Object.entries(playerId)[0] ?? ["Unknown", "unknown"];
  return `${kind}:${value}`;
}

function findPlayerName(replay: ReplayModel, playerId: Record<string, string> | undefined): string {
  const normalizedId = playerIdToString(playerId);
  if (!normalizedId) {
    return "Unknown";
  }

  return replay.players.find((player) => player.id === normalizedId)?.name ?? normalizedId;
}

function formatWinnerLabel(
  event: FiftyFiftyEvent,
  replay: ReplayModel,
): { text: string; className: string; winnerIsTeamZero: boolean | null } {
  const blueName = findPlayerName(replay, event.team_zero_player);
  const orangeName = findPlayerName(replay, event.team_one_player);
  const prefix = event.is_kickoff ? "Kickoff 50/50" : "50/50";
  const winner = event.winning_team_is_team_0 === undefined
    ? null
    : event.winning_team_is_team_0;
  const possession = event.possession_team_is_team_0 === undefined
    ? null
    : event.possession_team_is_team_0;

  const winnerLabel = winner === null
    ? "neutral"
    : winner
      ? "blue win"
      : "orange win";
  const possessionLabel = possession === null
    ? "neutral poss"
    : possession
      ? "blue poss"
      : "orange poss";
  const className = winner === null
    ? "sap-fifty-fifty-overlay-label-neutral"
    : winner
      ? "sap-fifty-fifty-overlay-label-blue"
      : "sap-fifty-fifty-overlay-label-orange";

  return {
    text: `${prefix}: ${blueName} vs ${orangeName} | ${winnerLabel} | ${possessionLabel}`,
    className,
    winnerIsTeamZero: winner,
  };
}

export function buildFiftyFiftyMarkers(
  statsTimeline: StatsTimeline,
  replay: ReplayModel,
): FiftyFiftyMarker[] {
  return (statsTimeline.fifty_fifty_events ?? []).map((event) => {
    const label = formatWinnerLabel(event, replay);
    const teamZeroPosition = new THREE.Vector3(...event.team_zero_position);
    const teamOnePosition = new THREE.Vector3(...event.team_one_position);
    const midpoint = new THREE.Vector3(...event.midpoint);

    return {
      id: `fifty-fifty:${event.start_frame}:${playerIdToString(event.team_zero_player)}:${playerIdToString(event.team_one_player)}`,
      time: event.resolve_time,
      label: label.text,
      labelClassName: label.className,
      axisStart: teamZeroPosition,
      axisEnd: teamOnePosition,
      midpoint,
      winnerIsTeamZero: label.winnerIsTeamZero,
    };
  });
}

export function getVisibleFiftyFiftyMarkers(
  markers: FiftyFiftyMarker[],
  currentTime: number,
  decaySeconds: number,
): FiftyFiftyMarker[] {
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
    .sap-fifty-fifty-overlay-root {
      position: absolute;
      inset: 0;
      z-index: 2;
      pointer-events: none;
      overflow: hidden;
      font-family: "IBM Plex Sans", "Avenir Next", sans-serif;
    }

    .sap-fifty-fifty-overlay-label {
      position: absolute;
      min-width: max-content;
      padding: 0.24rem 0.6rem;
      border-radius: 999px;
      border: 1px solid rgba(255, 255, 255, 0.16);
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

    .sap-fifty-fifty-overlay-label-blue {
      border-color: rgba(89, 195, 255, 0.5);
      background: rgba(17, 47, 73, 0.84);
    }

    .sap-fifty-fifty-overlay-label-orange {
      border-color: rgba(255, 193, 92, 0.5);
      background: rgba(76, 41, 7, 0.84);
    }

    .sap-fifty-fifty-overlay-label-neutral {
      border-color: rgba(243, 246, 248, 0.4);
      background: rgba(34, 41, 47, 0.86);
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

export class FiftyFiftyOverlay {
  private readonly scene: ReplayScene;
  private readonly container: HTMLElement;
  private readonly group = new THREE.Group();
  private readonly labelRoot: HTMLDivElement;
  private readonly projectedPosition = new THREE.Vector3();
  private readonly worldPosition = new THREE.Vector3();
  private readonly labelOffset = new THREE.Vector3(0, 0, LABEL_HEIGHT);
  private readonly markers: FiftyFiftyMarker[];
  private readonly views = new Map<string, FiftyFiftyMarkerView>();
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
    this.markers = buildFiftyFiftyMarkers(statsTimeline, replay);

    if (getComputedStyle(container).position === "static") {
      this.changedContainerPosition = true;
      this.originalContainerPosition = container.style.position;
      container.style.position = "relative";
    }

    this.group.name = "fifty-fifty-overlay";
    this.scene.replayRoot.add(this.group);

    this.labelRoot = document.createElement("div");
    this.labelRoot.className = "sap-fifty-fifty-overlay-root";
    this.container.append(this.labelRoot);
  }

  update(currentTime: number): void {
    const visibleMarkers = getVisibleFiftyFiftyMarkers(
      this.markers,
      currentTime,
      this.decaySeconds,
    );
    const visibleIds = new Set(visibleMarkers.map((marker) => marker.id));

    for (const [id, view] of this.views.entries()) {
      if (visibleIds.has(id)) {
        continue;
      }
      view.line.removeFromParent();
      view.line.geometry.dispose();
      view.material.dispose();
      view.label.remove();
      this.views.delete(id);
    }

    for (const marker of visibleMarkers) {
      const age = Math.max(0, currentTime - marker.time);
      const life = Math.max(0, 1 - age / this.decaySeconds);
      const view = this.ensureView(marker);
      const opacity = 0.12 + 0.78 * life;

      view.material.opacity = opacity;
      const positions = view.line.geometry.getAttribute("position");
      positions.setXYZ(0, marker.axisStart.x, marker.axisStart.y, marker.axisStart.z + 24);
      positions.setXYZ(1, marker.axisEnd.x, marker.axisEnd.y, marker.axisEnd.z + 24);
      positions.needsUpdate = true;

      this.worldPosition.copy(marker.midpoint).add(this.labelOffset);
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
        view.label.style.opacity = `${0.24 + 0.76 * life}`;
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
      view.line.removeFromParent();
      view.line.geometry.dispose();
      view.material.dispose();
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

  private ensureView(marker: FiftyFiftyMarker): FiftyFiftyMarkerView {
    const existing = this.views.get(marker.id);
    if (existing) {
      return existing;
    }

    const geometry = new THREE.BufferGeometry().setFromPoints([
      marker.axisStart,
      marker.axisEnd,
    ]);
    const material = new THREE.LineBasicMaterial({
      color: marker.winnerIsTeamZero === null
        ? NEUTRAL_COLOR
        : marker.winnerIsTeamZero
          ? BLUE_COLOR
          : ORANGE_COLOR,
      transparent: true,
      opacity: 0.9,
    });
    const line = new THREE.Line(geometry, material);
    line.renderOrder = 3;
    this.group.add(line);

    const label = document.createElement("div");
    label.className = `sap-fifty-fifty-overlay-label ${marker.labelClassName}`;
    label.textContent = marker.label;
    this.labelRoot.append(label);

    const view: FiftyFiftyMarkerView = {
      marker,
      line,
      material,
      label,
    };
    this.views.set(marker.id, view);
    return view;
  }
}
