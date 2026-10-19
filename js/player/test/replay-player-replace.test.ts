import test from "node:test";
import assert from "node:assert/strict";

import { ReplayPlayer } from "../src/player/ReplayPlayer.ts";
import type { SubtrActorPlayer } from "../src/player/adapter/SubtrActorPlayer.ts";

const noopManager = new Proxy({}, { get: () => () => {} });

function stubAdapter(): SubtrActorPlayer {
  const vector = { x: 0, y: 0, z: 0 };
  return {
    duration: 10,
    frameTimes: [],
    playerList: [],
    ball: { position: vector, rotation: { ...vector, w: 1 }, velocity: vector, visible: true },
    frameIndexAt: () => 0,
    seek() {},
    getAllPlayers: () => [],
    getTimelines: () => ({}),
  } as unknown as SubtrActorPlayer;
}

// Builds a ReplayPlayer around stub managers (no WebGL) so replaceReplay's
// plugin lifecycle can be exercised directly.
function createHeadlessPlayer(ballModelLoads: Array<() => void>) {
  const pluginLog: string[] = [];
  let pluginActive = true;
  const plugin = {
    id: "probe",
    setup() {
      pluginActive = true;
      pluginLog.push("setup");
    },
    teardown() {
      pluginActive = false;
      pluginLog.push("teardown");
    },
    beforeRender() {
      if (!pluginActive) pluginLog.push("beforeRender while torn down");
    },
    onStateChange() {
      if (!pluginActive) pluginLog.push("onStateChange while torn down");
    },
  };
  const actorManager = new Proxy(
    {},
    {
      get: (_target, key) =>
        key === "waitForBallModel"
          ? () => new Promise<void>((resolve) => ballModelLoads.push(resolve))
          : () => {},
    },
  );
  const player = Object.assign(Object.create(ReplayPlayer.prototype), {
    adapter: stubAdapter(),
    replay: null,
    options: {},
    container: {},
    sceneManager: { scene: {}, camera: {}, renderer: { render() {} }, updateSkyboxAnimation() {} },
    controls: { update() {} },
    actorManager,
    effectsManager: noopManager,
    hitboxManager: noopManager,
    effectsEnabled: false,
    plugins: [{ definition: plugin, plugin, active: true }],
    beforeRenderCallbacks: [],
    disposed: false,
    playing: false,
    readyResolved: true,
    speed: 1,
    currentTime: 0,
    replaceGeneration: 0,
    pendingReplayPlayer: false,
    freeCameraTransition: null,
    cameraDistanceScaleValue: 1,
    customCameraSettingsValue: null,
    cameraViewModeValue: "free",
    attachedPlayerIdValue: null,
    ballCamEnabledValue: null,
    hitboxWireframesEnabledValue: false,
    hitboxOnlyModeEnabledValue: false,
    hitboxesActive: false,
    skipPostGoalTransitionsEnabledValue: false,
    skipKickoffsEnabledValue: false,
    attachmentTouched: false,
    dispatchEvent: () => true,
  }) as ReplayPlayer;
  return { player, pluginLog };
}

test("overlapping replaceReplay calls set plugins up once", async () => {
  const ballModelLoads: Array<() => void> = [];
  const { player, pluginLog } = createHeadlessPlayer(ballModelLoads);
  const internals = player as unknown as { readyResolved: boolean };

  const first = player.replaceReplay(stubAdapter(), null);
  const second = player.replaceReplay(stubAdapter(), null);
  assert.equal(ballModelLoads.length, 2);

  ballModelLoads[0]();
  await first;
  assert.equal(internals.readyResolved, false);
  player.renderFrame();
  player.seek(1);
  assert.deepEqual(pluginLog, ["teardown"]);

  ballModelLoads[1]();
  await second;
  assert.equal(internals.readyResolved, true);
  assert.deepEqual(pluginLog, ["teardown", "setup"]);
});
