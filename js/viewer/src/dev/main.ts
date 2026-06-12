/**
 * Bring-up harness: .replay -> subtr-actor WASM -> SubtrActorPlayer -> the real
 * ballcam renderer (SceneManager + ArenaManager + ActorManager).
 *
 * This drives ballcam's actual mesh/interpolation code from subtr-actor data, to
 * validate the pipeline + coordinate transform end-to-end. It deliberately does
 * NOT boot the full GameEngine (cameras/effects/UI/backend) yet — that comes
 * after the vertical slice renders. See INTEGRATION.md.
 */
import * as THREE from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
import { SceneManager } from "../managers/SceneManager.js";
import { ArenaManager } from "../managers/ArenaManager.js";
import { ActorManager } from "../managers/ActorManager.js";
import { SubtrActorPlayer } from "../adapter/SubtrActorPlayer.js";
import { parseReplay } from "../adapter/wasm.js";

const hud = document.getElementById("hud") as HTMLDivElement;
const log = (msg: string) => {
  hud.textContent = msg;
  console.log("[viewer]", msg);
};

// Any effects call from ActorManager is a no-op during bring-up.
const effectsStub = new Proxy({}, { get: () => () => {} });

async function main() {
  const container = document.getElementById("app") as HTMLDivElement;

  log("parsing replay via subtr-actor WASM…");
  const bytes = new Uint8Array(await (await fetch("/sample.replay")).arrayBuffer());
  const raw = await parseReplay(bytes);

  log("compiling timelines…");
  const player = new SubtrActorPlayer(raw as never);
  log(
    `players: ${player.playerList.map((p) => `${p.name}(${p.team})`).join(", ")}\n` +
      `duration: ${player.duration.toFixed(1)}s`,
  );

  // Scene + arena + actors (ballcam's real renderer code)
  const sceneManager = new SceneManager(container);
  const scene = sceneManager.scene as THREE.Scene;
  const camera = sceneManager.camera as THREE.PerspectiveCamera;
  const renderer = sceneManager.renderer as THREE.WebGLRenderer;

  const arena = new ArenaManager(scene);
  await arena.loadArenaMeshes().catch((e: unknown) => console.warn("arena load failed", e));

  const actors = new ActorManager(scene, effectsStub);
  actors.initFromFramework(player);
  actors.initInterpolants(player.getTimelines());

  // Simple orbit camera for the demo.
  const controls = new OrbitControls(camera, renderer.domElement);
  camera.position.set(0, 4000, 6000);
  controls.target.set(0, 200, 0);
  controls.update();

  window.addEventListener("resize", () => {
    camera.aspect = window.innerWidth / window.innerHeight;
    camera.updateProjectionMatrix();
    renderer.setSize(window.innerWidth, window.innerHeight);
  });

  // Playback loop
  let t = 0;
  let last = performance.now();
  const speed = 1.0;
  function frame(now: number) {
    const dt = Math.min(0.1, (now - last) / 1000) * speed;
    last = now;
    t += dt;
    if (t > player.duration) t = 0;

    player.seek(t);
    actors.updateFromFramework(player, t);
    controls.update();
    renderer.render(scene, camera);

    const b = player.ball.position;
    log(`t=${t.toFixed(1)}/${player.duration.toFixed(1)}s  ball=(${b.x.toFixed(0)},${b.y.toFixed(0)},${b.z.toFixed(0)})`);
    requestAnimationFrame(frame);
  }
  requestAnimationFrame(frame);
}

main().catch((e) => {
  console.error(e);
  log("ERROR: " + (e instanceof Error ? e.message : String(e)));
});
