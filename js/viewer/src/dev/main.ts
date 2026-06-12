/**
 * Dev harness: exercises the real public API (`createViewer` from lib.ts) with
 * the sample replay — the same path an embedding consumer uses. The HUD readout
 * and name tags are driven entirely through the public seams (subscribe + the
 * name-tag plugin), keeping the core bare.
 */
import * as THREE from "three";
import {
  createViewer,
  createNameTagPlugin,
  createBoostPadsPlugin,
  createFpsOverlayPlugin,
  createBoostPickupAnimationPlugin,
  createCameraPlugin,
  createCanvasRecorderPlugin,
  createTimelineOverlayPlugin,
  fromReplayPlayerPlugin,
} from "../lib.js";

const hud = document.getElementById("hud") as HTMLDivElement;
const log = (msg: string) => {
  hud.textContent = msg;
  console.log("[viewer]", msg);
};

async function main() {
  const container = document.getElementById("app") as HTMLDivElement;

  log("parsing replay via subtr-actor WASM…");
  const bytes = new Uint8Array(await (await fetch("/sample.replay")).arrayBuffer());

  const camPlugin = createCameraPlugin();
  // @rlrml/player's canvas recorder, bridged: extra members (start/stop/
  // getStatus/recordRange) survive fromReplayPlayerPlugin.
  const recorder = fromReplayPlayerPlugin(createCanvasRecorderPlugin());
  const viewer = await createViewer(container, bytes, {
    autoplay: true,
    loop: true,
    plugins: [
      createFpsOverlayPlugin(),
      createNameTagPlugin(),
      createBoostPadsPlugin(),
      camPlugin,
      // @rlrml/player's own timeline overlay (goals/saves markers, skip
      // toggles, scrubber), mounted through the Phase 3 bridge.
      fromReplayPlayerPlugin(createTimelineOverlayPlugin()),
      // beforeRender plugins run on the bridge's synthesized render context.
      fromReplayPlayerPlugin(createBoostPickupAnimationPlugin()),
      recorder,
    ],
  });

  const roster = viewer.adapter.playerList
    .map((p) => `${p.name}(${p.team}:${p.carName})`)
    .join(", ");
  console.log("[viewer] roster:", roster);

  // ── Camera bar: follow-player selector + ball cam toggle (dev-only UI). ─────
  const camBar = document.createElement("div");
  camBar.style.cssText =
    "position:fixed;top:8px;right:8px;z-index:10;display:flex;gap:8px;" +
    "align-items:center;font:12px monospace;color:#fff;background:rgba(0,0,0,.55);" +
    "padding:6px 8px;border-radius:6px;";
  const select = document.createElement("select");
  select.append(new Option("orbit camera", ""));
  select.append(new Option("free camera (WASD + right-drag)", "__free"));
  select.append(new Option("ball orbit", "__ballOrbit"));
  for (const p of viewer.adapter.playerList) {
    select.append(new Option(`follow ${p.name} (team ${p.team})`, p.name));
  }
  select.onchange = () => {
    if (select.value === "") camPlugin.release();
    else if (select.value === "__free") camPlugin.setMode("free");
    else if (select.value === "__ballOrbit") camPlugin.setMode("ballOrbit");
    else {
      camPlugin.follow(select.value);
      console.log(
        `[viewer] following ${select.value}; recorded camera settings:`,
        camPlugin.getRecordedSettings(),
      );
    }
    syncStiffness();
    select.blur(); // keep arrow keys on the scrubber, not the dropdown
  };
  const ballCamLabel = document.createElement("label");
  const ballCamBox = document.createElement("input");
  ballCamBox.type = "checkbox";
  ballCamBox.checked = camPlugin.getBallCam();
  ballCamBox.onchange = () => camPlugin.setBallCam(ballCamBox.checked);
  ballCamLabel.append(ballCamBox, " ball cam");
  // Stiffness slider: shows the effective value (recorded preset when
  // following); dragging it sets an explicit override that wins over recorded.
  const stiffnessLabel = document.createElement("label");
  const stiffness = document.createElement("input");
  stiffness.type = "range";
  stiffness.min = "0";
  stiffness.max = "1";
  stiffness.step = "0.05";
  stiffness.style.width = "70px";
  const stiffnessValue = document.createElement("span");
  const syncStiffness = () => {
    const s = camPlugin.getCameraSettings().stiffness ?? 0.45;
    stiffness.value = String(s);
    stiffnessValue.textContent = s.toFixed(2);
  };
  stiffness.oninput = () => {
    camPlugin.setCameraSettings({ stiffness: Number(stiffness.value) });
    stiffnessValue.textContent = Number(stiffness.value).toFixed(2);
  };
  stiffnessLabel.append("stiff ", stiffness, stiffnessValue);
  syncStiffness();
  camBar.append(select, ballCamLabel, stiffnessLabel);
  document.body.append(camBar);

  // URL params for headless/dev bring-up:
  //   ?follow=<player name>  ?cam=free|ballOrbit  ?t=<seconds>
  const params = new URLSearchParams(location.search);
  const camParam = params.get("cam");
  if (camParam === "free" || camParam === "ballOrbit") {
    camPlugin.setMode(camParam);
    select.value = `__${camParam}`;
  }
  const followParam = params.get("follow");
  if (followParam) {
    const match = viewer.adapter.playerList.find(
      (p) => p.name.toLowerCase() === followParam.toLowerCase(),
    );
    if (match) {
      camPlugin.follow(match.name);
      select.value = match.name;
      console.log(
        `[viewer] following ${match.name}; recorded camera settings:`,
        camPlugin.getRecordedSettings(),
      );
      syncStiffness();
    } else {
      console.warn(`[viewer] ?follow=${followParam}: no such player`);
    }
  }
  const tParam = params.get("t");
  if (tParam) viewer.seek(Number(tParam));
  // ?paritycheck: exercise the @rlrml/player-parity surface (docs/PLAYER_PARITY.md)
  // and log PASS/FAIL lines for headless verification.
  if (params.get("paritycheck")) {
    const ok = (label: string, cond: boolean) =>
      console.log(`[paritycheck] ${cond ? "PASS" : "FAIL"} ${label}`);
    const first = viewer.adapter.playerList[0];
    let renders = 0;
    const offBeforeRender = viewer.onBeforeRender((info) => {
      renders += 1;
      if (renders === 1) {
        ok("FrameRenderInfo sane", info.frameIndex >= 0 && info.alpha >= 0 && info.alpha <= 1);
        offBeforeRender();
      }
    });
    viewer.setState({ playing: false, currentTime: 45, speed: 2 });
    let s = viewer.getState();
    ok("setState applied", !s.playing && Math.abs(s.currentTime - 45) < 0.001 && s.speed === 2);
    ok("frameIndex matches adapter", s.frameIndex === viewer.adapter.frameIndexAt(45));
    const beforeIdx = s.frameIndex;
    viewer.stepFrames(3);
    ok("stepFrames(3) advances", viewer.getState().frameIndex === beforeIdx + 3);
    viewer.stepBackwardFrame();
    ok("stepBackwardFrame", viewer.getState().frameIndex === beforeIdx + 2);
    viewer.setAttachedPlayer(first.id);
    s = viewer.getState();
    ok(
      "setAttachedPlayer by id → follow",
      s.cameraViewMode === "follow" && s.attachedPlayerId === first.id,
    );
    ok("camera plugin followed", camPlugin.getMode() === "follow" && camPlugin.getTarget() === first.name);
    viewer.setCustomCameraSettings({ pitch: -7, distance: 300 });
    ok("pitch alias → angle", camPlugin.getCameraSettings().angle === -7);
    viewer.setCameraDistanceScale(2);
    ok("distance scale applied", camPlugin.getCameraSettings().distance === 600);
    viewer.setCustomCameraSettings(null);
    viewer.setCameraDistanceScale(1);
    ok("settings cleared", viewer.getState().customCameraSettings === null);
    viewer.setBallCamEnabled(false);
    ok("ball cam forced off", viewer.getState().ballCamEnabled === false);
    viewer.setCameraViewMode("free");
    s = viewer.getState();
    ok("view mode free releases", s.cameraViewMode === "free" && camPlugin.getMode() === "orbit");
    ok("snapshot equals state", JSON.stringify(viewer.getSnapshot()) === JSON.stringify(viewer.getState()));
    // Phase 2: shared data layer (viewer.replay is @rlrml/player's ReplayModel).
    const model = viewer.replay;
    ok("replay model present", !!model && model.frameCount > 0);
    ok(
      "replay model ids/time axis match adapter",
      !!model &&
        model.players.some((p) => p.id === first.id) &&
        Math.abs(model.duration - viewer.getState().duration) < 0.001,
    );
    // Phase 3: timeline projection / skip windows / bridged @rlrml/player plugin.
    ok(
      "timeline duration matches model",
      !!model && Math.abs(viewer.getTimelineDuration() - model.duration) < 0.001,
    );
    ok(
      "timeline current time = projection of currentTime",
      Math.abs(
        viewer.getTimelineCurrentTime() -
          viewer.projectReplayTimeToTimeline(viewer.getState().currentTime).timelineTime,
      ) < 0.001,
    );
    viewer.setSkipKickoffsEnabled(true);
    const proj0 = viewer.projectReplayTimeToTimeline(0);
    ok(
      "t=0 hidden by kickoff skip, seekTime jumps past it",
      proj0.hiddenBySkip && proj0.seekTime > 0,
    );
    ok("skip segments computed", viewer.getTimelineSegments().length > 0);
    viewer.setSkipKickoffsEnabled(false);
    const kickoffFrame = model?.frames.find((f) => f.kickoffCountdown > 0);
    if (kickoffFrame) {
      viewer.setState({ playing: false, currentTime: kickoffFrame.time });
      ok(
        "activeMetadata during kickoff countdown",
        viewer.getState().activeMetadata?.kind === "kickoff-countdown",
      );
      viewer.setState({ currentTime: 45 });
      ok("activeMetadata null outside kickoff", viewer.getState().activeMetadata === null);
    } else {
      ok("activeMetadata during kickoff countdown (no countdown frames in model)", false);
    }
    ok(
      "bridged timeline overlay mounted",
      !!document.querySelector(".sap-tl-root") &&
        document.querySelector<HTMLInputElement>(".sap-tl-root input[type=range]")?.max ===
          `${model?.duration}`,
    );
    // sceneState (ReplayScene parity): replayRoot maps UE coords → world
    // exactly like adapter/coords.ts (x→x, z→y, y→z).
    {
      const probe = new THREE.Object3D();
      probe.position.set(1000, 2000, 300); // UE coords
      viewer.sceneState.replayRoot.add(probe);
      viewer.scene.updateMatrixWorld(true);
      const world = probe.getWorldPosition(new THREE.Vector3());
      ok(
        "sceneState.replayRoot is UE-coordinate space",
        world.x === 1000 && world.y === 300 && world.z === 2000,
      );
      viewer.sceneState.replayRoot.remove(probe);
      const meshes = viewer.sceneState.playerMeshes;
      ok(
        "sceneState.playerMeshes keyed by roster ids",
        viewer.adapter.playerList.every((p) => meshes.get(p.id) !== undefined),
      );
      ok("sceneState.ballMesh present", viewer.sceneState.ballMesh.isObject3D === true);
    }
    // Phase 3b: beforeRender plugins on the bridge's synthesized
    // ReplayPlayerRenderContext (boost-pickup animation + canvas recorder).
    {
      // The pickup plugin parents one group per pickup event on replayRoot
      // (renderOrder 60) during setup…
      const pickupGroups = () =>
        viewer.sceneState.replayRoot.children.filter((child) => child.renderOrder === 60);
      ok("pickup animation groups installed in replayRoot", pickupGroups().length > 0);
      ok("canvas recorder bridged, idle", recorder.getStatus().state === "idle");
      const firstPickup = (model?.boostPads ?? [])
        .flatMap((pad) => pad.events.filter((e) => !e.available && e.playerId))
        .sort((a, b) => a.time - b.time)[0];
      if (!firstPickup) {
        ok("bridged beforeRender animates pickup (no pickup events in model)", false);
        viewer.play();
      } else {
        // …and flips them visible from beforeRender, which only runs inside
        // the render loop — so finish this check a couple of frames later.
        viewer.setState({
          playing: false,
          currentTime: firstPickup.time + 0.2,
          boostPickupAnimationEnabled: true,
        });
        requestAnimationFrame(() =>
          requestAnimationFrame(() => {
            ok(
              "bridged beforeRender animates pickup",
              pickupGroups().some((group) => group.visible),
            );
            viewer.setState({ playing: true, currentTime: 45 });
          }),
        );
      }
    }
    // Phase 3c: hitbox display toggles drive HitboxManager (created lazily on
    // the first enabled render — so check a couple of frames after toggling).
    // Delayed past the pickup block's rAF chain to keep the checks ordered.
    setTimeout(() => {
      viewer.setHitboxWireframesEnabled(true);
      requestAnimationFrame(() =>
        requestAnimationFrame(() => {
          const hitboxes = viewer.hitboxManager.hitboxes as Map<
            unknown,
            { mesh: THREE.Object3D }
          >;
          ok("hitbox wireframes created for cars", hitboxes.size > 0);
          ok(
            "hitbox wireframes visible when enabled",
            [...hitboxes.values()].some((entry) => entry.mesh.visible),
          );
          viewer.setHitboxOnlyModeEnabled(true);
          requestAnimationFrame(() =>
            requestAnimationFrame(() => {
              ok(
                "hitbox-only mode hides car bodies",
                viewer.adapter.playerList.every((p) => {
                  const mesh = viewer.sceneState.playerMeshes.get(p.id);
                  return !mesh || !mesh.visible;
                }),
              );
              viewer.setHitboxOnlyModeEnabled(false);
              viewer.setHitboxWireframesEnabled(false);
              requestAnimationFrame(() =>
                requestAnimationFrame(() => {
                  ok(
                    "disabling hitboxes restores car bodies",
                    viewer.adapter.playerList.some((p) => {
                      const mesh = viewer.sceneState.playerMeshes.get(p.id);
                      return mesh ? mesh.visible : false;
                    }) && [...hitboxes.values()].every((entry) => !entry.mesh.visible),
                  );
                }),
              );
            }),
          );
        }),
      );
    }, 400);
  }
  if (params.get("paused")) viewer.pause();
  // ?pauseat=<seconds>: pause once playback reaches this time (deterministic
  // screenshots — both A/B runs freeze on the identical frame).
  const pauseAt = params.get("pauseat");
  if (pauseAt) {
    const at = Number(pauseAt);
    const unsub = viewer.subscribe((state) => {
      if (state.currentTime >= at) {
        viewer.pause();
        unsub();
      }
    });
  }

  viewer.subscribe((state) => {
    const b = viewer.adapter.ball.position;
    log(
      `t=${state.currentTime.toFixed(1)}/${state.duration.toFixed(1)}s` +
        `  ball=(${b.x.toFixed(0)},${b.y.toFixed(0)},${b.z.toFixed(0)})` +
        (state.playing ? "" : "  [paused]"),
    );
  });

  // Space toggles playback; ←/→ scrub 5s; B toggles ball cam.
  // In free-cam mode the CameraManager owns Space/arrows (fly controls).
  window.addEventListener("keydown", (e) => {
    const freeCam = camPlugin.getMode() === "free";
    if (e.code === "Space" && !freeCam) {
      e.preventDefault();
      viewer.togglePlayback();
    } else if (e.code === "ArrowRight" && !freeCam) {
      viewer.seek(viewer.getState().currentTime + 5);
    } else if (e.code === "ArrowLeft" && !freeCam) {
      viewer.seek(viewer.getState().currentTime - 5);
    } else if (e.code === "KeyB") {
      ballCamBox.checked = !ballCamBox.checked;
      camPlugin.setBallCam(ballCamBox.checked);
    }
  });

  await viewer.ready;
  console.log("[viewer] assets ready (arena + ball model)");
}

main().catch((e) => {
  console.error(e);
  log("ERROR: " + (e instanceof Error ? e.message : String(e)));
});
