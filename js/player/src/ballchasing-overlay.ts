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
      align-items: center;
      min-width: max-content;
      transform: translate(-50%, -100%);
      will-change: transform;
    }

    .sap-bc-player-selectable {
      pointer-events: auto;
      cursor: pointer;
    }

    .sap-bc-player-selectable:focus-visible {
      outline: 2px solid rgba(255, 255, 255, 0.88);
      outline-offset: 2px;
    }

    .sap-bc-floating-track[hidden] {
      display: none;
    }

    .sap-bc-boost-bar {
      position: relative;
      display: inline-flex;
      align-items: center;
      justify-content: center;
      min-width: 8rem;
      max-width: 14rem;
      min-height: 1.45rem;
      border-radius: 999px;
      overflow: hidden;
      border: 1px solid rgba(255, 255, 255, 0.3);
      background: rgba(6, 11, 17, 0.42);
      box-shadow: 0 10px 24px rgba(0, 0, 0, 0.18);
      backdrop-filter: blur(6px);
      transition:
        border-color 0.12s ease-out,
        box-shadow 0.12s ease-out,
        transform 0.12s ease-out;
    }

    .sap-bc-boost-bar-blue {
      background: rgba(18, 39, 68, 0.68);
      border-color: rgba(109, 169, 255, 0.5);
    }

    .sap-bc-boost-bar-orange {
      background: rgba(71, 35, 8, 0.72);
      border-color: rgba(255, 189, 110, 0.5);
    }

    .sap-bc-boost-fill {
      position: absolute;
      left: 0;
      top: 0;
      height: 100%;
      width: 0%;
      border-radius: 999px;
      transition: width 0.08s ease-out;
    }

    .sap-bc-boost-fill-blue {
      background:
        linear-gradient(90deg, rgba(123, 185, 255, 0.94), rgba(59, 130, 246, 0.96));
    }

    .sap-bc-boost-fill-orange {
      background:
        linear-gradient(90deg, rgba(255, 201, 118, 0.94), rgba(245, 158, 11, 0.96));
    }

    .sap-bc-boost-text {
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 0.35rem;
      position: relative;
      z-index: 1;
      min-width: 0;
      max-width: 100%;
      overflow: hidden;
      text-overflow: ellipsis;
      padding: 0.22rem 0.72rem;
      color: #ffffff;
      font-size: 0.72rem;
      font-weight: 700;
      text-shadow: 0 1px 3px rgba(0, 0, 0, 0.7);
      white-space: nowrap;
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

    .sap-bc-hud-boost-bar {
      position: relative;
      display: inline-flex;
      align-items: center;
      justify-content: center;
      min-width: 8.25rem;
      max-width: 12rem;
      min-height: 1.2rem;
      border-radius: 999px;
      overflow: hidden;
      border: 1px solid rgba(255, 255, 255, 0.26);
      background: rgba(0, 0, 0, 0.44);
      transition:
        border-color 0.12s ease-out,
        box-shadow 0.12s ease-out,
        transform 0.12s ease-out;
    }

    .sap-bc-hud-boost-fill {
      position: absolute;
      left: 0;
      top: 0;
      height: 100%;
      width: 0%;
      border-radius: 999px;
      transition: width 0.08s ease-out;
    }

    .sap-bc-hud-boost-text {
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 0.35rem;
      position: relative;
      z-index: 1;
      min-width: 0;
      max-width: 100%;
      overflow: hidden;
      text-overflow: ellipsis;
      padding: 0.14rem 0.65rem;
      color: #ffffff;
      font-size: 0.72rem;
      font-weight: 700;
      text-shadow: 0 1px 2px rgba(0, 0, 0, 0.75);
      white-space: nowrap;
      font-variant-numeric: tabular-nums;
    }

    .sap-bc-hud-player-inactive {
      opacity: 0.45;
    }

    .sap-bc-player-selectable:hover .sap-bc-boost-bar,
    .sap-bc-player-selectable:hover .sap-bc-hud-boost-bar,
    .sap-bc-player-selectable:focus-visible .sap-bc-boost-bar,
    .sap-bc-player-selectable:focus-visible .sap-bc-hud-boost-bar {
      transform: translateY(-1px);
      border-color: rgba(255, 255, 255, 0.56);
      box-shadow: 0 10px 22px rgba(0, 0, 0, 0.24);
    }

    .sap-bc-player-following .sap-bc-boost-bar,
    .sap-bc-player-following .sap-bc-hud-boost-bar {
      border-color: rgba(255, 255, 255, 0.82);
      box-shadow:
        0 0 0 2px rgba(255, 255, 255, 0.22),
        0 12px 28px rgba(0, 0, 0, 0.28);
    }

    @media (max-width: 900px) {
      .sap-bc-team-hud {
        display: none;
      }
    }

    @media (max-width: 640px) {
      .sap-bc-boost-bar {
        min-width: 6.7rem;
        max-width: 11rem;
        min-height: 1.2rem;
      }

      .sap-bc-boost-text {
        font-size: 0.64rem;
        padding-inline: 0.58rem;
      }
    }
  `;
  document.head.append(style);
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
  amount: number,
  playerName: string
): void {
  if (!fill || !text) {
    return;
  }

  const percent = Math.max(0, Math.min(100, Math.round((amount / 255) * 100)));
  fill.style.width = `${percent}%`;
  text.textContent = `${percent} ${playerName}`;
}

function makePlayerSelectable(
  element: HTMLDivElement | null,
  context: ReplayPlayerPluginContext,
  playerId: string,
  playerName: string
): void {
  if (!element) {
    return;
  }

  const followPlayer = (): void => {
    context.player.setAttachedPlayer(playerId);
  };

  element.classList.add("sap-bc-player-selectable");
  element.tabIndex = 0;
  element.setAttribute("role", "button");
  element.setAttribute("aria-label", `Follow ${playerName}`);
  element.title = `Follow ${playerName}`;
  element.addEventListener("click", followPlayer);
  element.addEventListener("keydown", (event) => {
    if (event.key !== "Enter" && event.key !== " ") {
      return;
    }

    event.preventDefault();
    followPlayer();
  });
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

  function syncAttachedPlayer(attachedPlayerId: string | null): void {
    for (const [playerId, elements] of playerElements.entries()) {
      const isAttached = playerId === attachedPlayerId;
      elements.floatingRoot?.classList.toggle("sap-bc-player-following", isAttached);
      elements.teamHudEntry?.classList.toggle("sap-bc-player-following", isAttached);
      elements.floatingRoot?.setAttribute("aria-pressed", isAttached ? "true" : "false");
      elements.teamHudEntry?.setAttribute("aria-pressed", isAttached ? "true" : "false");
    }
  }

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
      let floatingBoostBar: HTMLDivElement | null = null;
      let floatingBoostFill: HTMLDivElement | null = null;
      let floatingBoostText: HTMLSpanElement | null = null;
      if (floatingLayer) {
        floatingRoot = document.createElement("div");
        floatingRoot.className = "sap-bc-floating-track";
        floatingRoot.hidden = true;

        if (showFloatingNames || showFloatingBoostBars) {
          floatingBoostBar = document.createElement("div");
          floatingBoostBar.className = `sap-bc-boost-bar ${
            track.isTeamZero ? "sap-bc-boost-bar-blue" : "sap-bc-boost-bar-orange"
          }`;
          floatingBoostFill = document.createElement("div");
          floatingBoostFill.className = `sap-bc-boost-fill ${
            track.isTeamZero ? "sap-bc-boost-fill-blue" : "sap-bc-boost-fill-orange"
          }`;
          floatingBoostText = document.createElement("span");
          floatingBoostText.className = "sap-bc-boost-text";
          floatingBoostBar.append(floatingBoostFill, floatingBoostText);
          floatingRoot.append(floatingBoostBar);
        }

        makePlayerSelectable(floatingRoot, context, track.id, track.name);
        floatingLayer.append(floatingRoot);
      }

      let teamHudEntry: HTMLDivElement | null = null;
      let teamHudFill: HTMLDivElement | null = null;
      let teamHudText: HTMLSpanElement | null = null;
      if (showTeamBoostHud) {
        teamHudEntry = document.createElement("div");
        teamHudEntry.className = "sap-bc-hud-player";

        const teamHudBar = document.createElement("div");
        teamHudBar.className = `sap-bc-hud-boost-bar ${
          track.isTeamZero ? "sap-bc-boost-bar-blue" : "sap-bc-boost-bar-orange"
        }`;
        teamHudFill = document.createElement("div");
        teamHudFill.className = `sap-bc-hud-boost-fill ${
          track.isTeamZero ? "sap-bc-boost-fill-blue" : "sap-bc-boost-fill-orange"
        }`;
        teamHudText = document.createElement("span");
        teamHudText.className = "sap-bc-hud-boost-text";
        teamHudBar.append(teamHudFill, teamHudText);
        teamHudEntry.append(teamHudBar);
        makePlayerSelectable(teamHudEntry, context, track.id, track.name);

        const hudContainer = track.isTeamZero ? blueHud : orangeHud;
        hudContainer?.append(teamHudEntry);
      }

      playerElements.set(track.id, {
        floatingRoot,
        floatingBoostFill,
        floatingBoostText,
        teamHudEntry,
        teamHudFill,
        teamHudText,
      });
    }

    floatingOffset.set(0, 0, 255 * (context.options.fieldScale ?? 1));
    container.append(root);
    syncAttachedPlayer(context.player.getState().attachedPlayerId);
  }

  return {
    id: "ballchasing-overlay",
    setup(context): void {
      buildHud(context, context.container);
    },
    onStateChange(context): void {
      syncAttachedPlayer(context.state.attachedPlayerId);
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
        setBoostBar(
          elements.floatingBoostFill,
          elements.floatingBoostText,
          boostAmount,
          player.track.name
        );
        setBoostBar(
          elements.teamHudFill,
          elements.teamHudText,
          boostAmount,
          player.track.name
        );

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
