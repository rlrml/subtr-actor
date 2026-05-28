import * as THREE from "three";
import type {
  ReplayPlayerPlugin,
  ReplayPlayerPluginContext,
  ReplayPlayerRenderContext,
} from "./types";
import { ensureBallchasingOverlayStyles } from "./ballchasing-overlay-styles";

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
  playerName: string,
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
  playerName: string,
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
  out: THREE.Vector3,
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
  options: BallchasingOverlayPluginOptions = {},
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

  function buildHud(context: ReplayPlayerPluginContext, container: HTMLElement): void {
    ensureBallchasingOverlayStyles();
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
          player.track.name,
        );
        setBoostBar(elements.teamHudFill, elements.teamHudText, boostAmount, player.track.name);

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
            projected,
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
