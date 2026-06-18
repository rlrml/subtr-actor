import * as THREE from "three";
import { HDRLoader } from "three/examples/jsm/loaders/HDRLoader.js";
import { RoomEnvironment } from "three/examples/jsm/environments/RoomEnvironment.js";
import { resolvePlayerAssetUrl } from "../asset-url.js";

export class SceneManager {
  constructor(container, options = {}) {
    this.container = typeof container === "string" ? document.getElementById(container) : container;
    this.assetBase = options.assetBase;
    if (!this.container) {
      console.error("Invalid container passed to SceneManager");
      return;
    }

    this.scene = new THREE.Scene();

    // No skybox in the constructor: ReplayPlayer calls initDefaultEnvironment()
    // for instant neutral lighting, then applyEnvironment() loads the HDR lazily.
    this.scene.background = new THREE.Color(0x87ceeb); // Temporary until environment loads

    const width = this.container.clientWidth;
    const height = this.container.clientHeight;

    // Camera - far plane set for Unreal Units (arena is ~10000 UU)
    this.camera = new THREE.PerspectiveCamera(75, width / height, 10, 50000);
    this.camera.position.set(0, 2000, 5000);

    // Renderer - NOTE: logarithmicDepthBuffer causes shader recompilation issues
    // It was added to prevent z-fighting, but causes massive freeze on first explosion
    // TODO: Find alternative z-fighting solution (adjust near/far planes, polygon offset, etc.)
    this.renderer = new THREE.WebGLRenderer({ antialias: true });
    this.renderer.setSize(width, height);
    this.renderer.shadowMap.enabled = true;
    this.renderer.shadowMap.type = THREE.PCFSoftShadowMap;
    this.renderer.toneMapping = THREE.ACESFilmicToneMapping;
    this.renderer.toneMappingExposure = 1.0;
    this.renderer.outputColorSpace = THREE.SRGBColorSpace;
    this.container.appendChild(this.renderer.domElement);

    // Resize handler
    window.addEventListener("resize", () => this.onWindowResize());
  }

  /**
   * Asset-free default lighting. The original ballcam app lit everything via
   * an HDR skybox (scene.environment -> IBL on the PBR materials); those HDRs
   * were never vendored into this package, so without this the scene renders
   * nearly black. RoomEnvironment + PMREM gives equivalent neutral IBL from
   * code, and a directional key light adds definition.
   */
  initDefaultEnvironment() {
    if (!this._neutralEnvTexture) {
      const pmrem = new THREE.PMREMGenerator(this.renderer);
      this._neutralEnvTexture = pmrem.fromScene(new RoomEnvironment(), 0.04).texture;
      pmrem.dispose();
    }
    this.scene.environment = this._neutralEnvTexture;

    // Add the helper lights once; re-running this method (e.g. reverting from an
    // HDR environment) must not stack duplicate sun/ambient lights.
    if (!this._defaultLightsAdded) {
      const sun = new THREE.DirectionalLight(0xffffff, 1.5);
      sun.position.set(3000, 8000, 4000);
      this.scene.add(sun);

      const ambient = new THREE.AmbientLight(0xffffff, 0.4);
      this.scene.add(ambient);
      this._defaultLightsAdded = true;
    }
  }

  /**
   * Load and apply a {@link PlayerEnvironment}: an HDR skybox that drives both
   * the visible background and the image-based lighting (reflections/ambient) on
   * every PBR material. Async and non-blocking — call it without awaiting so the
   * neutral `initDefaultEnvironment()` lighting renders immediately and the HDR
   * swaps in once decoded. Resolves `true` on success, `false` on load failure
   * (the neutral default is left in place).
   *
   * @param {import("../environments.js").PlayerEnvironment} env
   * @returns {Promise<boolean>}
   */
  applyEnvironment(env) {
    return new Promise((resolve) => {
      const hdrLoader = new HDRLoader();
      const path = resolvePlayerAssetUrl(env.skyboxUrl, this.assetBase);

      hdrLoader.load(
        path,
        (texture) => {
          // Dispose of previous HDR skybox texture if one is mounted.
          if (this.scene.background && this.scene.background.dispose) {
            this.scene.background.dispose();
          }

          texture.mapping = THREE.EquirectangularReflectionMapping;
          this.scene.background = texture;
          this.scene.environment = texture;
          this.currentEnvironmentId = env.id;

          // Static skybox tilt (degrees → radians), applied to both the visible
          // background and the IBL so reflections stay aligned with the sky.
          const rot = env.rotation ?? {};
          this._skyboxBaseRotation = {
            x: THREE.MathUtils.degToRad(rot.x ?? 0),
            y: THREE.MathUtils.degToRad(rot.y ?? 0),
            z: THREE.MathUtils.degToRad(rot.z ?? 0),
          };
          this._skyboxAnimatedY = 0;
          this._skyboxAnimation = env.animation ?? null;
          this._applySkyboxRotation();

          if (typeof env.exposure === "number") {
            this.renderer.toneMappingExposure = env.exposure;
          }

          console.log(`[SceneManager] environment applied: ${env.id}`);
          resolve(true);
        },
        undefined,
        (error) => {
          console.error(`[SceneManager] Failed to load environment "${env.id}":`, error);
          // Keep the neutral default lighting; never leave the scene black.
          resolve(false);
        },
      );
    });
  }

  /** Apply base tilt + accumulated animation to background/environment rotation. */
  _applySkyboxRotation() {
    const base = this._skyboxBaseRotation;
    if (!base) return;
    const y = base.y + (this._skyboxAnimatedY ?? 0);
    if (this.scene.backgroundRotation) {
      this.scene.backgroundRotation.set(base.x, y, base.z);
    }
    if (this.scene.environmentRotation) {
      this.scene.environmentRotation.set(base.x, y, base.z);
    }
  }

  /**
   * Advance the slow skybox drift, if the active environment enables it. Cheap
   * no-op otherwise. `dt` is in seconds (already scaled by playback speed).
   */
  updateSkyboxAnimation(dt) {
    const anim = this._skyboxAnimation;
    if (!anim || !anim.enabled || !dt) return;
    this._skyboxAnimatedY =
      (this._skyboxAnimatedY ?? 0) + THREE.MathUtils.degToRad(anim.speed * dt);
    this._applySkyboxRotation();
  }

  /**
   * Revert to the neutral default: a flat background plus the asset-free
   * RoomEnvironment IBL (so PBR materials stay lit). Used when no environment is
   * selected (`environment: false`) or when switching away from an HDR skybox.
   */
  setDefaultBackground() {
    // Dispose of the HDR skybox texture if one is mounted.
    if (this.scene.background && this.scene.background.dispose) {
      this.scene.background.dispose();
    }
    this.scene.background = new THREE.Color(0x1a1a2e);
    this.initDefaultEnvironment(); // restore neutral IBL (idempotent)
    this.renderer.toneMappingExposure = 1.0;
    this._skyboxAnimation = null;
    this._skyboxBaseRotation = null;
    this.currentEnvironmentId = null;
    console.log("[SceneManager] Using neutral default environment (no skybox)");
  }

  setExposure(value) {
    if (this.renderer) {
      this.renderer.toneMappingExposure = value;
    }
  }

  onWindowResize() {
    if (!this.container || !this.renderer) return;

    const width = this.container.clientWidth;
    const height = this.container.clientHeight;

    this.camera.aspect = width / height;
    this.camera.updateProjectionMatrix();
    this.renderer.setSize(width, height);
  }

  render() {
    if (this.renderer) {
      this.renderer.render(this.scene, this.camera);
    }
  }

  dispose() {
    if (this.renderer) {
      this.renderer.dispose();
      if (this.renderer.domElement && this.renderer.domElement.parentNode) {
        this.renderer.domElement.parentNode.removeChild(this.renderer.domElement);
      }
      this.renderer = null;
    }
  }
}
