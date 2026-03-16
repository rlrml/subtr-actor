import * as THREE from "three";
import type {
  ReplayPlayerPlugin,
  ReplayPlayerPluginContext,
  ReplayPlayerRenderContext,
} from "./types";

export interface BallchasingOverlayPluginOptions {
  showFloatingNames?: boolean;
  showFloatingBoostBars?: boolean;
  showTeamBoostHud?: boolean;
}

interface PlayerOverlayElements {
  floatingRoot: HTMLDivElement | null;
  floatingName: HTMLDivElement | null;
  floatingBoostBar: HTMLDivElement | null;
  floatingBoostFill: HTMLDivElement | null;
  floatingBoostText: HTMLSpanElement | null;
  teamHudEntry: HTMLDivElement | null;
  teamHudFill: HTMLDivElement | null;
  teamHudText: HTMLSpanElement | null;
}

const STYLE_ID = "subtr-actor-ballchasing-overlay-styles";
const TEAM_BLUE = "#3b82f6";
const TEAM_ORANGE = "#f59e0b";

function ensureStyles(): void {
  if (document.getElementById(STYLE_ID)) {
    return;
  }

  const style = document.createElement("style");
  style.id = STYLE_ID;
  style.textContent = `
    .sap-bc-overlay-root {
      position: absolute;
      inset: 0;
      z-index: 3;
      pointer-events: none;
      overflow: hidden;
      font-family: "IBM Plex Sans", "Segoe UI", Roboto, sans-serif;
    }

    .sap-bc-floating-layer {
      position: absolute;
      inset: 0;
      pointer-events: none;
    }

    .sap-bc-floating-track {
      position: absolute;
      display: flex;
      flex-direction: column;
      align-items: center;
      gap: 0.35rem;
      min-width: max-content;
      transform: translate(-50%, -100%);
      will-change: transform;
    }

    .sap-bc-floating-track[hidden] {
      display: none;
    }

    .sap-bc-name {
      display: inline-flex;
      align-items: center;
      min-height: 1.5rem;
      padding: 0.2rem 0.5rem 0.2rem 0.55rem;
      border-radius: 0.45rem;
      background: rgba(21, 28, 36, 0.72);
      color: #f7fbff;
      font-size: 0.78rem;
      font-weight: 600;
      line-height: 1;
      text-shadow: 0 1px 2px rgba(0, 0, 0, 0.55);
      white-space: nowrap;
      backdrop-filter: blur(6px);
      box-shadow: 0 10px 24px rgba(0, 0, 0, 0.18);
    }

    .sap-bc-name-blue {
      border-left: 3px solid ${TEAM_BLUE};
    }

    .sap-bc-name-orange {
      border-left: 3px solid ${TEAM_ORANGE};
    }

    .sap-bc-boost-bar {
      position: relative;
      width: 4.6rem;
      height: 1.2rem;
      border-radius: 999px;
      overflow: hidden;
      border: 2px solid rgba(255, 255, 255, 0.44);
      background: rgba(6, 11, 17, 0.42);
      box-shadow: 0 10px 24px rgba(0, 0, 0, 0.18);
      backdrop-filter: blur(6px);
    }

    .sap-bc-boost-fill {
      position: absolute;
      left: 0;
      top: 0;
      height: 100%;
      width: 0%;
      border-radius: 999px;
      transition: width 0.08s ease-out, background-color 0.12s ease-out;
    }

    .sap-bc-boost-text {
      position: absolute;
      inset: 0;
      display: flex;
      align-items: center;
      justify-content: center;
      color: #ffffff;
      font-size: 0.72rem;
      font-weight: 700;
      text-shadow: 0 1px 3px rgba(0, 0, 0, 0.7);
      font-variant-numeric: tabular-nums;
    }

    .sap-bc-team-hud {
      position: absolute;
      top: 5.4rem;
      display: flex;
      flex-direction: column;
      gap: 0.7rem;
      padding: 0.8rem 0.9rem;
      border-radius: 0.9rem;
      background: rgba(9, 14, 21, 0.52);
      backdrop-filter: blur(8px);
      box-shadow: 0 14px 36px rgba(0, 0, 0, 0.2);
    }

    .sap-bc-team-hud-blue {
      left: 0.7rem;
      border-left: 4px solid ${TEAM_BLUE};
    }

    .sap-bc-team-hud-orange {
      right: 0.7rem;
      border-right: 4px solid ${TEAM_ORANGE};
    }

    .sap-bc-hud-player {
      display: flex;
      flex-direction: column;
      gap: 0.25rem;
    }

    .sap-bc-team-hud-orange .sap-bc-hud-player {
      align-items: flex-end;
    }

    .sap-bc-hud-player-name {
      max-width: 9rem;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
      color: #f7fbff;
      font-size: 0.85rem;
      font-weight: 600;
      text-shadow: 0 1px 2px rgba(0, 0, 0, 0.75);
    }

    .sap-bc-hud-boost-bar {
      position: relative;
      width: 7.5rem;
      height: 1.1rem;
      border-radius: 999px;
      overflow: hidden;
      border: 1px solid rgba(255, 255, 255, 0.26);
      background: rgba(0, 0, 0, 0.44);
    }

    .sap-bc-hud-boost-fill {
      position: absolute;
      left: 0;
      top: 0;
      height: 100%;
      width: 0%;
      border-radius: 999px;
      transition: width 0.08s ease-out, background-color 0.12s ease-out;
    }

    .sap-bc-hud-boost-text {
      position: absolute;
      inset: 0;
      display: flex;
      align-items: center;
      justify-content: center;
      color: #ffffff;
      font-size: 0.72rem;
      font-weight: 700;
      text-shadow: 0 1px 2px rgba(0, 0, 0, 0.75);
      font-variant-numeric: tabular-nums;
    }

    .sap-bc-hud-player-inactive {
      opacity: 0.45;
    }

    @media (max-width: 900px) {
      .sap-bc-team-hud {
        display: none;
      }
    }

    @media (max-width: 640px) {
      .sap-bc-name {
        font-size: 0.68rem;
        padding: 0.16rem 0.45rem 0.16rem 0.5rem;
      }

      .sap-bc-boost-bar {
        width: 3.7rem;
        height: 1rem;
      }

      .sap-bc-boost-text {
        font-size: 0.64rem;
      }
    }
  `;
  document.head.append(style);
}

function getBoostColor(boost: number): string {
  if (boost <= 33) {
    const t = boost / 33;
    const r = 200;
    const g = Math.round(50 + t * 80);
    return `rgb(${r}, ${g}, 30)`;
  }

  if (boost <= 66) {
    const t = (boost - 33) / 33;
    const r = Math.round(200 - t * 20);
    const g = Math.round(130 + t * 50);
    return `rgb(${r}, ${g}, 30)`;
  }

  const t = (boost - 66) / 34;
  const r = Math.round(180 - t * 140);
  const g = Math.round(180 - t * 50);
  return `rgb(${r}, ${g}, 30)`;
}

function lerpBoostAmount(context: ReplayPlayerRenderContext, playerIndex: number): number {
  const player = context.players[playerIndex];
  const currentAmount = player.frame?.boostAmount ?? 0;
  const nextAmount = player.nextFrame?.boostAmount ?? currentAmount;
  return THREE.MathUtils.lerp(currentAmount, nextAmount, context.alpha);
}

function setBoostBar(
  fill: HTMLElement | null,
  text: HTMLElement | null,
  amount: number
): void {
  if (!fill || !text) {
    return;
  }

  const percent = Math.max(0, Math.min(100, Math.round((amount / 255) * 100)));
  fill.style.width = `${percent}%`;
  fill.style.backgroundColor = getBoostColor(percent);
  text.textContent = `${percent}`;
}

function projectToContainer(
  mesh: THREE.Object3D,
  worldOffset: THREE.Vector3,
  camera: THREE.Camera,
  container: HTMLElement,
  out: THREE.Vector3
): boolean {
  mesh.getWorldPosition(out);
  out.add(worldOffset);
  out.project(camera);

  if (out.z < -1 || out.z > 1) {
    return false;
  }

  const width = container.clientWidth || 1;
  const height = container.clientHeight || 1;
  out.x = ((out.x + 1) * width) / 2;
  out.y = ((1 - out.y) * height) / 2;

  if (out.x < -80 || out.x > width + 80 || out.y < -80 || out.y > height + 80) {
    return false;
  }

  return true;
}

export function createBallchasingOverlayPlugin(
  options: BallchasingOverlayPluginOptions = {}
): ReplayPlayerPlugin {
  const showFloatingNames = options.showFloatingNames ?? true;
  const showFloatingBoostBars = options.showFloatingBoostBars ?? true;
  const showTeamBoostHud = options.showTeamBoostHud ?? true;

  let root: HTMLDivElement | null = null;
  let floatingLayer: HTMLDivElement | null = null;
  let blueHud: HTMLDivElement | null = null;
  let orangeHud: HTMLDivElement | null = null;
  let changedContainerPosition = false;
  let originalContainerPosition = "";
  const playerElements = new Map<string, PlayerOverlayElements>();
  const projected = new THREE.Vector3();
  const floatingOffset = new THREE.Vector3(0, 0, 255);

  function buildHud(
    context: ReplayPlayerPluginContext,
    container: HTMLElement
  ): void {
    ensureStyles();
    if (getComputedStyle(container).position === "static") {
      changedContainerPosition = true;
      originalContainerPosition = container.style.position;
      container.style.position = "relative";
    }

    root = document.createElement("div");
    root.className = "sap-bc-overlay-root";

    if (showFloatingNames || showFloatingBoostBars) {
      floatingLayer = document.createElement("div");
      floatingLayer.className = "sap-bc-floating-layer";
      root.append(floatingLayer);
    } else {
      floatingLayer = null;
    }

    if (showTeamBoostHud) {
      blueHud = document.createElement("div");
      blueHud.className = "sap-bc-team-hud sap-bc-team-hud-blue";
      orangeHud = document.createElement("div");
      orangeHud.className = "sap-bc-team-hud sap-bc-team-hud-orange";
      root.append(blueHud, orangeHud);
    } else {
      blueHud = null;
      orangeHud = null;
    }

    for (const track of context.replay.players) {
      let floatingRoot: HTMLDivElement | null = null;
      let floatingName: HTMLDivElement | null = null;
      let floatingBoostBar: HTMLDivElement | null = null;
      let floatingBoostFill: HTMLDivElement | null = null;
      let floatingBoostText: HTMLSpanElement | null = null;
      if (floatingLayer) {
        floatingRoot = document.createElement("div");
        floatingRoot.className = "sap-bc-floating-track";
        floatingRoot.hidden = true;

        if (showFloatingNames) {
          floatingName = document.createElement("div");
          floatingName.className = `sap-bc-name ${
            track.isTeamZero ? "sap-bc-name-blue" : "sap-bc-name-orange"
          }`;
          floatingName.textContent = track.name;
          floatingRoot.append(floatingName);
        }

        if (showFloatingBoostBars) {
          floatingBoostBar = document.createElement("div");
          floatingBoostBar.className = "sap-bc-boost-bar";
          floatingBoostFill = document.createElement("div");
          floatingBoostFill.className = "sap-bc-boost-fill";
          floatingBoostText = document.createElement("span");
          floatingBoostText.className = "sap-bc-boost-text";
          floatingBoostBar.append(floatingBoostFill, floatingBoostText);
          floatingRoot.append(floatingBoostBar);
        }

        floatingLayer.append(floatingRoot);
      }

      let teamHudEntry: HTMLDivElement | null = null;
      let teamHudFill: HTMLDivElement | null = null;
      let teamHudText: HTMLSpanElement | null = null;
      if (showTeamBoostHud) {
        teamHudEntry = document.createElement("div");
        teamHudEntry.className = "sap-bc-hud-player";

        const teamHudName = document.createElement("span");
        teamHudName.className = "sap-bc-hud-player-name";
        teamHudName.textContent = track.name;

        const teamHudBar = document.createElement("div");
        teamHudBar.className = "sap-bc-hud-boost-bar";
        teamHudFill = document.createElement("div");
        teamHudFill.className = "sap-bc-hud-boost-fill";
        teamHudText = document.createElement("span");
        teamHudText.className = "sap-bc-hud-boost-text";
        teamHudBar.append(teamHudFill, teamHudText);
        teamHudEntry.append(teamHudName, teamHudBar);

        const hudContainer = track.isTeamZero ? blueHud : orangeHud;
        hudContainer?.append(teamHudEntry);
      }

      playerElements.set(track.id, {
        floatingRoot,
        floatingName,
        floatingBoostBar,
        floatingBoostFill,
        floatingBoostText,
        teamHudEntry,
        teamHudFill,
        teamHudText,
      });
    }

    floatingOffset.set(0, 0, 255 * (context.options.fieldScale ?? 1));
    container.append(root);
  }

  return {
    id: "ballchasing-overlay",
    setup(context): void {
      buildHud(context, context.container);
    },
    teardown(context): void {
      root?.remove();
      root = null;
      floatingLayer = null;
      blueHud = null;
      orangeHud = null;
      playerElements.clear();
      if (changedContainerPosition) {
        context.container.style.position = originalContainerPosition;
        changedContainerPosition = false;
      }
    },
    beforeRender(context): void {
      if (!root) {
        return;
      }

      for (const [playerIndex, player] of context.players.entries()) {
        const elements = playerElements.get(player.track.id);
        if (!elements) {
          continue;
        }

        const boostAmount = lerpBoostAmount(context, playerIndex);
        setBoostBar(elements.floatingBoostFill, elements.floatingBoostText, boostAmount);
        setBoostBar(elements.teamHudFill, elements.teamHudText, boostAmount);

        const mesh = player.mesh;
        const active = mesh !== null && player.interpolatedPosition !== null;
        elements.teamHudEntry?.classList.toggle("sap-bc-hud-player-inactive", !active);

        if (!elements.floatingRoot) {
          continue;
        }

        if (
          !active ||
          !projectToContainer(
            mesh,
            floatingOffset,
            context.scene.camera,
            context.container,
            projected
          )
        ) {
          elements.floatingRoot.hidden = true;
          continue;
        }

        elements.floatingRoot.hidden = false;
        elements.floatingRoot.style.transform =
          `translate(${projected.x.toFixed(1)}px, ${projected.y.toFixed(1)}px) ` +
          "translate(-50%, -100%)";
      }
    },
  };
}
