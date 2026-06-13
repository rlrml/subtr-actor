import * as THREE from "three";
import { RGBELoader } from "three/examples/jsm/loaders/RGBELoader.js";
import { RoomEnvironment } from "three/examples/jsm/environments/RoomEnvironment.js";

export class SceneManager {
  constructor(container) {
    this.container = typeof container === "string" ? document.getElementById(container) : container;
    if (!this.container) {
      console.error("Invalid container passed to SceneManager");
      return;
    }

    this.scene = new THREE.Scene();

    // Don't load skybox in constructor - it must be awaited before shader precompilation
    // Call loadSkybox() from GameEngine.init() instead
    this.scene.background = new THREE.Color(0x87ceeb); // Temporary until skybox loads

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
    const pmrem = new THREE.PMREMGenerator(this.renderer);
    this.scene.environment = pmrem.fromScene(new RoomEnvironment(), 0.04).texture;
    pmrem.dispose();

    const sun = new THREE.DirectionalLight(0xffffff, 1.5);
    sun.position.set(3000, 8000, 4000);
    this.scene.add(sun);

    const ambient = new THREE.AmbientLight(0xffffff, 0.4);
    this.scene.add(ambient);
  }

  loadSkybox(skyboxId = "HighFantasy4k") {
    return new Promise((resolve) => {
      const rgbeLoader = new RGBELoader();
      const path = `/skyboxes/${skyboxId}.hdr`;

      rgbeLoader.load(
        path,
        (texture) => {
          // Dispose of previous skybox texture if exists
          if (this.scene.background && this.scene.background.dispose) {
            this.scene.background.dispose();
          }

          texture.mapping = THREE.EquirectangularReflectionMapping;
          this.scene.background = texture;
          this.scene.environment = texture;
          this.currentSkyboxId = skyboxId;
          console.log(`[SceneManager] HDR skybox loaded: ${skyboxId}`);
          resolve(true);
        },
        undefined,
        (error) => {
          console.error(`[SceneManager] Failed to load HDR skybox (${skyboxId}):`, error);
          // Fallback to solid color
          this.scene.background = new THREE.Color(0x87ceeb);
          resolve(false);
        },
      );
    });
  }

  setSkybox(skyboxId) {
    if (this.currentSkyboxId !== skyboxId) {
      this.loadSkybox(skyboxId);
    }
  }

  /**
   * Set a simple default background (no skybox HDR)
   * Used when no custom environment is selected
   */
  setDefaultBackground() {
    // Dispose of previous skybox texture if exists
    if (this.scene.background && this.scene.background.dispose) {
      this.scene.background.dispose();
    }
    // Use a dark grey gradient-like color for a neutral look
    this.scene.background = new THREE.Color(0x1a1a2e);
    this.scene.environment = null; // No environment lighting from skybox
    this.currentSkyboxId = null;
    console.log("[SceneManager] Using default background (no skybox)");
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
