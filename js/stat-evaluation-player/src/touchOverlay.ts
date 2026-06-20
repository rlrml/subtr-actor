import * as THREE from "three";
import type { ReplayModel, ReplayScene } from "@rlrml/player";
import type { PlayerStatsSnapshot, StatsFrame, StatsTimeline } from "./statsTimeline.ts";
import { statsEventPayloads } from "./statsTimeline.ts";

const STYLE_ID = "subtr-actor-touch-overlay-styles";
const BLUE_TOUCH_COLOR = 0x59c3ff;
const ORANGE_TOUCH_COLOR = 0xffc15c;
const TOUCH_RING_INNER_RADIUS = 120;
const TOUCH_RING_BAND_GAP = 4;
const TOUCH_RING_OUTER_RADIUS = 196;
const TOUCH_RING_HEIGHT = 24;
const TOUCH_LABEL_HEIGHT = 210;
const DEFAULT_DECAY_SECONDS = 5;
const TOUCH_CREDIT_EPSILON = 0.1;
const ADVANCEMENT_ARROW_MIN_LENGTH = 48;

export type TouchOverlayMode = "markers" | "advancement";

export type TouchOverlayColorMode =
  | "team"
  | "intention"
  | "kind"
  | "height_band"
  | "surface"
  | "dodge_state"
  | "flag";

export const TOUCH_OVERLAY_COLOR_MODE_ORDER: TouchOverlayColorMode[] = [
  "team",
  "intention",
  "kind",
  "height_band",
  "surface",
  "dodge_state",
  "flag",
];

export interface TouchColorLegendEntry {
  label: string;
  color: number;
}

export interface TouchColorLegendGroup {
  title: string;
  entries: TouchColorLegendEntry[];
}

export interface TouchMarkerClassification {
  key: string;
  value: string;
  label: string;
  color: number;
}

export const TOUCH_TEAM_COLOR_LEGEND: TouchColorLegendEntry[] = [
  { label: "Blue team", color: BLUE_TOUCH_COLOR },
  { label: "Orange team", color: ORANGE_TOUCH_COLOR },
];

export const TOUCH_INTENTION_COLOR_LEGEND: TouchColorLegendEntry[] = [
  { label: "Shot", color: 0xff5d6c },
  { label: "Save", color: 0x4ade80 },
  { label: "Clear", color: 0xfacc15 },
  { label: "Pass", color: 0x22d3ee },
  { label: "Challenge", color: 0xc084fc },
  { label: "Control", color: 0x000000 },
  { label: "Neutral", color: 0x9aa5b1 },
];

export const TOUCH_KIND_COLOR_LEGEND: TouchColorLegendEntry[] = [
  { label: "Control", color: 0x000000 },
  { label: "Medium hit", color: 0xfacc15 },
  { label: "Hard hit", color: 0xff5d6c },
];

export const TOUCH_HEIGHT_BAND_COLOR_LEGEND: TouchColorLegendEntry[] = [
  { label: "Ground", color: 0xa3e635 },
  { label: "Low air", color: 0x38bdf8 },
  { label: "High air", color: 0x818cf8 },
];

export const TOUCH_SURFACE_COLOR_LEGEND: TouchColorLegendEntry[] = [
  { label: "Ground", color: 0x84cc16 },
  { label: "Air", color: 0x60a5fa },
  { label: "Wall", color: 0xf97316 },
];

export const TOUCH_DODGE_STATE_COLOR_LEGEND: TouchColorLegendEntry[] = [
  { label: "No dodge", color: 0x94a3b8 },
  { label: "Dodge", color: 0xe879f9 },
];

export const TOUCH_FLAG_COLOR_LEGEND: TouchColorLegendEntry[] = [
  { label: "First touch", color: 0xffffff },
  { label: "Contested", color: 0xef4444 },
];

export const TOUCH_COLOR_LEGEND_GROUPS: TouchColorLegendGroup[] = [
  { title: "Team", entries: TOUCH_TEAM_COLOR_LEGEND },
  { title: "Intention", entries: TOUCH_INTENTION_COLOR_LEGEND },
  { title: "Hit strength", entries: TOUCH_KIND_COLOR_LEGEND },
  { title: "Height", entries: TOUCH_HEIGHT_BAND_COLOR_LEGEND },
  { title: "Surface", entries: TOUCH_SURFACE_COLOR_LEGEND },
  { title: "Dodge", entries: TOUCH_DODGE_STATE_COLOR_LEGEND },
  { title: "Flags", entries: TOUCH_FLAG_COLOR_LEGEND },
];

function colorRecord(entries: TouchColorLegendEntry[]): Record<string, number> {
  return Object.fromEntries(
    entries.map((entry) => [entry.label.toLowerCase().replaceAll(" ", "_"), entry.color]),
  );
}

const INTENTION_COLORS = colorRecord(TOUCH_INTENTION_COLOR_LEGEND);
const KIND_COLORS: Record<string, number> = {
  ...colorRecord(TOUCH_KIND_COLOR_LEGEND),
  medium_hit: TOUCH_KIND_COLOR_LEGEND[1]!.color,
  hard_hit: TOUCH_KIND_COLOR_LEGEND[2]!.color,
};
const HEIGHT_BAND_COLORS: Record<string, number> = {
  ...colorRecord(TOUCH_HEIGHT_BAND_COLOR_LEGEND),
  low_air: TOUCH_HEIGHT_BAND_COLOR_LEGEND[1]!.color,
  high_air: TOUCH_HEIGHT_BAND_COLOR_LEGEND[2]!.color,
};
const SURFACE_COLORS = colorRecord(TOUCH_SURFACE_COLOR_LEGEND);
const DODGE_STATE_COLORS: Record<string, number> = {
  ...colorRecord(TOUCH_DODGE_STATE_COLOR_LEGEND),
  no_dodge: TOUCH_DODGE_STATE_COLOR_LEGEND[0]!.color,
};
const BOOLEAN_CLASSIFICATION_COLORS: Record<string, number> = {
  first_touch: TOUCH_FLAG_COLOR_LEGEND[0]!.color,
  contested: TOUCH_FLAG_COLOR_LEGEND[1]!.color,
};

const UNKNOWN_CLASSIFICATION_COLOR = 0x9aa5b1;

export interface TouchMarker {
  id: string;
  time: number;
  frame: number;
  isTeamZero: boolean;
  playerId: string | null;
  playerName: string;
  kind: string | null;
  intention: string | null;
  heightBand: string | null;
  surface: string | null;
  dodgeState: string | null;
  firstTouch: boolean;
  contested: boolean;
  classifications: TouchMarkerClassification[];
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
  ring: THREE.Group;
  ringSegments: THREE.Mesh[];
  ringColorsKey: string;
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

export function isTouchOverlayColorMode(value: unknown): value is TouchOverlayColorMode {
  return (
    value === "team" ||
    value === "intention" ||
    value === "kind" ||
    value === "height_band" ||
    value === "surface" ||
    value === "dodge_state" ||
    value === "flag"
  );
}

export function normalizeTouchOverlayColorModes(
  value: TouchOverlayColorMode | readonly TouchOverlayColorMode[] | undefined,
): TouchOverlayColorMode[] {
  const rawModes = Array.isArray(value) ? value : value ? [value] : ["team"];
  const seen = new Set<TouchOverlayColorMode>();
  const modes: TouchOverlayColorMode[] = [];
  for (const mode of rawModes) {
    if (isTouchOverlayColorMode(mode) && !seen.has(mode)) {
      seen.add(mode);
      modes.push(mode);
    }
  }
  return modes.length > 0 ? modes : ["team"];
}

function primaryColorMode(colorModes: readonly TouchOverlayColorMode[]): TouchOverlayColorMode {
  return colorModes[colorModes.length - 1] ?? "team";
}

function touchCreditLabel(
  marker: TouchMarker,
  mode: TouchOverlayMode,
  colorModes: readonly TouchOverlayColorMode[],
): string {
  const classification = touchMarkerClassification(marker, primaryColorMode(colorModes));
  const suffix = classification ? ` · ${classification.replaceAll("_", " ")}` : "";
  if (mode === "markers") {
    return `${marker.playerName}${suffix}`;
  }

  const advance = Math.round(marker.totalBallAdvanceDistance);
  const retreat = Math.round(marker.totalBallRetreatDistance);
  if (advance > 0 && retreat > 0) {
    return `${marker.playerName} +${advance} / -${retreat} uu${suffix}`;
  }
  if (retreat > 0) {
    return `${marker.playerName} -${retreat} uu${suffix}`;
  }
  return `${marker.playerName} +${advance} uu${suffix}`;
}

function touchMarkerClassification(
  marker: TouchMarker,
  colorMode: TouchOverlayColorMode,
): string | null {
  if (colorMode === "intention") {
    return marker.intention;
  }
  if (colorMode === "kind") {
    return marker.kind;
  }
  if (colorMode === "height_band") {
    return marker.heightBand;
  }
  if (colorMode === "surface") {
    return marker.surface;
  }
  if (colorMode === "dodge_state") {
    return marker.dodgeState;
  }
  if (colorMode === "flag") {
    return marker.contested ? "contested" : marker.firstTouch ? "first_touch" : null;
  }
  return null;
}

function formatClassificationValue(value: string): string {
  return value
    .split("_")
    .filter(Boolean)
    .map((part) => `${part.slice(0, 1).toUpperCase()}${part.slice(1)}`)
    .join(" ");
}

function stringClassification(
  key: string,
  value: unknown,
  colors: Record<string, number>,
): TouchMarkerClassification | null {
  if (typeof value !== "string" || value.length === 0) {
    return null;
  }

  return {
    key,
    value,
    label: formatClassificationValue(value),
    color: colors[value] ?? UNKNOWN_CLASSIFICATION_COLOR,
  };
}

function booleanClassification(
  key: string,
  enabled: unknown,
  label: string,
): TouchMarkerClassification | null {
  if (enabled !== true) {
    return null;
  }

  return {
    key,
    value: "true",
    label,
    color: BOOLEAN_CLASSIFICATION_COLORS[key] ?? UNKNOWN_CLASSIFICATION_COLOR,
  };
}

function buildTouchMarkerClassifications(event: {
  kind?: unknown;
  intention?: unknown;
  height_band?: unknown;
  surface?: unknown;
  dodge_state?: unknown;
  first_touch?: unknown;
  contested?: unknown;
}): TouchMarkerClassification[] {
  return [
    stringClassification("intention", event.intention, INTENTION_COLORS),
    stringClassification("kind", event.kind, KIND_COLORS),
    stringClassification("height_band", event.height_band, HEIGHT_BAND_COLORS),
    stringClassification("surface", event.surface, SURFACE_COLORS),
    stringClassification("dodge_state", event.dodge_state, DODGE_STATE_COLORS),
    booleanClassification("first_touch", event.first_touch, "First touch"),
    booleanClassification("contested", event.contested, "Contested"),
  ].filter((classification): classification is TouchMarkerClassification => classification != null);
}

export function touchMarkerColor(marker: TouchMarker, colorMode: TouchOverlayColorMode): number {
  if (colorMode === "intention") {
    return INTENTION_COLORS[marker.intention ?? ""] ?? UNKNOWN_CLASSIFICATION_COLOR;
  }
  if (colorMode === "kind") {
    return KIND_COLORS[marker.kind ?? ""] ?? UNKNOWN_CLASSIFICATION_COLOR;
  }
  if (colorMode === "height_band") {
    return HEIGHT_BAND_COLORS[marker.heightBand ?? ""] ?? UNKNOWN_CLASSIFICATION_COLOR;
  }
  if (colorMode === "surface") {
    return SURFACE_COLORS[marker.surface ?? ""] ?? UNKNOWN_CLASSIFICATION_COLOR;
  }
  if (colorMode === "dodge_state") {
    return DODGE_STATE_COLORS[marker.dodgeState ?? ""] ?? UNKNOWN_CLASSIFICATION_COLOR;
  }
  if (colorMode === "flag") {
    if (marker.contested) {
      return BOOLEAN_CLASSIFICATION_COLORS.contested ?? UNKNOWN_CLASSIFICATION_COLOR;
    }
    if (marker.firstTouch) {
      return BOOLEAN_CLASSIFICATION_COLORS.first_touch ?? UNKNOWN_CLASSIFICATION_COLOR;
    }
    return UNKNOWN_CLASSIFICATION_COLOR;
  }
  return marker.isTeamZero ? BLUE_TOUCH_COLOR : ORANGE_TOUCH_COLOR;
}

export function touchMarkerRingColors(
  marker: TouchMarker,
  colorModes: readonly TouchOverlayColorMode[],
): number[] {
  const modes = colorModes.length > 0 ? colorModes : (["team"] satisfies TouchOverlayColorMode[]);
  return modes.map((colorMode) => touchMarkerColor(marker, colorMode));
}

export function buildTouchMarkers(
  statsTimeline: StatsTimeline,
  replay: ReplayModel,
): TouchMarker[] {
  const markers: TouchMarker[] = [];
  const events = [...statsEventPayloads(statsTimeline, "touch")].sort((left, right) => {
    if (left.frame !== right.frame) {
      return left.frame - right.frame;
    }
    if (left.time !== right.time) {
      return left.time - right.time;
    }
    return 0;
  });

  for (const event of events) {
    const playerId = playerIdToString(event.player);
    const ballPosition = replay.ballFrames[event.frame]?.position;
    if (!ballPosition) {
      continue;
    }
    const movement = event.ball_movement;
    const movementEndPosition = movement ? replay.ballFrames[movement.end_frame]?.position : null;
    const endBallPosition = movementEndPosition ?? ballPosition;
    const markerIndex = markers.length;
    const classifications = buildTouchMarkerClassifications(event);
    markers.push({
      id: `touch-stat:${event.frame}:${playerId}:${markerIndex + 1}`,
      time: replay.frames[event.frame]?.time ?? event.time,
      frame: event.frame,
      isTeamZero: event.is_team_0,
      playerId,
      playerName: replay.players.find((player) => player.id === playerId)?.name ?? playerId,
      kind: typeof event.kind === "string" ? event.kind : null,
      intention: typeof event.intention === "string" ? event.intention : null,
      heightBand: typeof event.height_band === "string" ? event.height_band : null,
      surface: typeof event.surface === "string" ? event.surface : null,
      dodgeState: typeof event.dodge_state === "string" ? event.dodge_state : null,
      firstTouch: event.first_touch === true,
      contested: event.contested === true,
      classifications,
      position: {
        x: ballPosition.x,
        y: ballPosition.y,
        z: ballPosition.z,
      },
      endPosition: {
        x: endBallPosition.x,
        y: endBallPosition.y,
        z: endBallPosition.z,
      },
      totalBallTravelDistance: movement ? positiveDelta(movement.travel_distance, 0) : 0,
      totalBallAdvanceDistance: movement ? positiveDelta(movement.advance_distance, 0) : 0,
      totalBallRetreatDistance: movement ? positiveDelta(movement.retreat_distance, 0) : 0,
    });
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
      width: max-content;
      max-width: min(26rem, calc(100% - 1rem));
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
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
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

function disposeRingSegments(view: TouchMarkerView): void {
  for (const segment of view.ringSegments) {
    segment.removeFromParent();
    segment.geometry.dispose();
    const materials = Array.isArray(segment.material) ? segment.material : [segment.material];
    for (const material of materials) {
      material.dispose();
    }
  }
  view.ringSegments = [];
  view.ringColorsKey = "";
}

function createRingSegment(color: number, innerRadius: number, outerRadius: number): THREE.Mesh {
  const geometry = new THREE.RingGeometry(innerRadius, outerRadius, 48, 1);
  const material = new THREE.MeshBasicMaterial({
    color,
    transparent: true,
    opacity: 0.7,
    side: THREE.DoubleSide,
    depthWrite: false,
    depthTest: false,
  });
  const segment = new THREE.Mesh(geometry, material);
  segment.rotation.x = -Math.PI / 2;
  segment.renderOrder = 40;
  return segment;
}

function setRingSegments(view: TouchMarkerView, colors: readonly number[]): void {
  const normalizedColors = colors.length > 0 ? colors : [UNKNOWN_CLASSIFICATION_COLOR];
  const colorsKey = normalizedColors.join("|");
  if (view.ringColorsKey === colorsKey) {
    return;
  }

  disposeRingSegments(view);
  const ringCount = normalizedColors.length;
  const totalGap = TOUCH_RING_BAND_GAP * Math.max(0, ringCount - 1);
  const bandWidth = (TOUCH_RING_OUTER_RADIUS - TOUCH_RING_INNER_RADIUS - totalGap) / ringCount;
  const segments = normalizedColors.map((color, index) => {
    const innerRadius = TOUCH_RING_INNER_RADIUS + index * (bandWidth + TOUCH_RING_BAND_GAP);
    return createRingSegment(color, innerRadius, innerRadius + bandWidth);
  });
  for (const segment of segments) {
    view.ring.add(segment);
    view.ringSegments.push(segment);
  }
  view.ringColorsKey = colorsKey;
}

function setRingOpacity(view: TouchMarkerView, opacity: number): void {
  for (const segment of view.ringSegments) {
    const materials = Array.isArray(segment.material) ? segment.material : [segment.material];
    for (const material of materials) {
      material.opacity = opacity;
    }
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
  private colorModes: TouchOverlayColorMode[] = ["team"];

  constructor(
    scene: ReplayScene,
    container: HTMLElement,
    replay: ReplayModel,
    statsTimeline: StatsTimeline,
    options?: {
      decaySeconds?: number;
      mode?: TouchOverlayMode;
      colorMode?: TouchOverlayColorMode;
      colorModes?: TouchOverlayColorMode[];
    },
  ) {
    ensureStyles();
    this.scene = scene;
    this.container = container;
    this.decaySeconds = Math.max(0.1, options?.decaySeconds ?? DEFAULT_DECAY_SECONDS);
    this.mode = options?.mode ?? "markers";
    this.colorModes = normalizeTouchOverlayColorModes(options?.colorModes ?? options?.colorMode);
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

  getColorModes(): TouchOverlayColorMode[] {
    return [...this.colorModes];
  }

  setColorMode(colorMode: TouchOverlayColorMode): void {
    this.setColorModes([colorMode]);
  }

  setColorModes(colorModes: readonly TouchOverlayColorMode[]): void {
    this.colorModes = normalizeTouchOverlayColorModes([...colorModes]);
  }

  update(currentTime: number): void {
    const visibleMarkers = getVisibleTouchMarkers(this.markers, currentTime, this.decaySeconds);
    const visibleIds = new Set(visibleMarkers.map((marker) => marker.id));

    for (const [id, view] of this.views.entries()) {
      if (visibleIds.has(id)) {
        continue;
      }
      view.ring.removeFromParent();
      disposeRingSegments(view);
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

      const primaryMode = primaryColorMode(this.colorModes);
      const color = touchMarkerColor(marker, primaryMode);
      const ringColors = touchMarkerRingColors(marker, this.colorModes);
      setRingSegments(view, ringColors);
      setRingOpacity(view, baseOpacity);
      view.arrow.setColor(color);
      view.ring.position.set(
        marker.position.x,
        marker.position.y,
        marker.position.z + TOUCH_RING_HEIGHT,
      );
      view.ring.scale.setScalar(scale);
      view.label.textContent = touchCreditLabel(marker, this.mode, this.colorModes);
      view.label.classList.toggle(
        "sap-touch-overlay-label-advancement",
        this.mode === "advancement",
      );
      const teamTinted = primaryMode === "team";
      view.label.classList.toggle("sap-touch-overlay-label-blue", teamTinted && marker.isTeamZero);
      view.label.classList.toggle(
        "sap-touch-overlay-label-orange",
        teamTinted && !marker.isTeamZero,
      );
      view.label.style.borderColor = teamTinted ? "" : `#${color.toString(16).padStart(6, "0")}cc`;
      view.label.style.background = teamTinted ? "" : `#${color.toString(16).padStart(6, "0")}66`;

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
      disposeRingSegments(view);
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

    const ring = new THREE.Group();
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
      ringSegments: [],
      ringColorsKey: "",
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
