import * as THREE from 'three';
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js';
import { DRACOLoader } from 'three/examples/jsm/loaders/DRACOLoader.js';
import { OBJLoader } from 'three/examples/jsm/loaders/OBJLoader.js';

export class ArenaManager {
  constructor(scene) {
    this.scene = scene;
    this.arenaMeshes = []; // Store references to arena meshes for raycasting
    this.drawingCollider = null; // Simplified mesh for drawing raycasting
    this.drawingColliderMeshes = []; // Individual meshes from the collider
    this.arenaDecorMesh = null; // Arena decoration mesh (stands, surroundings)
    this.showArenaDecor = true; // Whether to show arena decoration

    // Setup DRACO loader for compressed meshes
    this.dracoLoader = new DRACOLoader();
    this.dracoLoader.setDecoderPath('https://www.gstatic.com/draco/versioned/decoders/1.5.6/');

    // Setup GLTF loader with DRACO support
    this.gltfLoader = new GLTFLoader();
    this.gltfLoader.setDRACOLoader(this.dracoLoader);
  }

  async loadArenaMeshes() {
    try {
      console.log('Loading arena mesh...');

      const gltf = await this.gltfLoader.loadAsync('/models/stadium/stadium.glb');
      const arena = gltf.scene;

      // Rotate arena 180 degrees around Y axis to match replay coordinate system
      // In Rocket League replays, team 0 (blue) spawns at negative Y coordinates
      // and team 1 (orange) spawns at positive Y coordinates
      // This rotation ensures the blue side of the arena mesh faces the blue team's spawn
      // arena.rotation.y = Math.PI; // 180 degrees

      // Materials that need visibility fix (disappear at certain camera angles)
      const visibilityFixMaterials = [
        'Sol_Trait_T0', 'Sol_Trait_T1',
        'Milieu_Forme', 'Milieu_Forme.001',
        'cage_T0', 'cage_T1',
        'Couleur_Hexagone_T0', 'Couleur_Hexagone_T1',
        'wall_gradient_color_2', 'wall_gradient_color_2.001',
        'Fond_BackBoard_Transparent', // For Transparant_BackBoard_+_Cage meshes
        'dégradé_transparent_T0', 'dégradé_transparent_T1', // Glow effects on field edges
        'grid_transperant', // Goal glass mesh
        'Detail_Milieu', 'Detail_Milieu.001', // Field mid details
      ];

      // Mesh name patterns that need frustum culling disabled (glow effects with incorrect bounding box)
      const disableFrustumCullingPatterns = ['Glow', 'Glass'];

      // Meshes that should NOT cast shadows (ceilings, transparent elements)
      const noCastShadowMeshes = [
        'Plafond_Hexagone_T0',
        'Plafond_Hexagone_T1',
        'Plafond_Transparent',
      ];

      // Enable shadow receiving on arena meshes and collect for raycasting
      arena.traverse((child) => {
        if (child.isMesh) {
          child.receiveShadow = true;
          // Disable castShadow for ceiling meshes
          child.castShadow = !noCastShadowMeshes.includes(child.name);

          // Store mesh reference for raycasting (drawing/pings)
          this.arenaMeshes.push(child);

          // Disable frustum culling for glow meshes (they have incorrect bounding boxes)
          const shouldDisableFrustumCulling = disableFrustumCullingPatterns.some(
            pattern => child.name.includes(pattern)
          );
          if (shouldDisableFrustumCulling) {
            console.log(`[ArenaManager] Disabling frustum culling for: ${child.name}`);
            child.frustumCulled = false;
          }

          // Fix visibility issues (disappearing at certain angles)
          if (child.material && child.material.name && visibilityFixMaterials.includes(child.material.name)) {
            console.log(`[ArenaManager] Fixing visibility for: ${child.name} (material: ${child.material.name})`);
            child.material = child.material.clone();
            child.material.side = THREE.DoubleSide; // Render both sides
            child.material.depthWrite = false; // Don't write to depth buffer
            child.renderOrder = 1; // Render after floor
            child.frustumCulled = false; // Also disable frustum culling for these
          }
        }
      });

      console.log(`[ArenaManager] Collected ${this.arenaMeshes.length} meshes for raycasting`);

      this.scene.add(arena);
      console.log('Arena mesh loaded successfully with correct orientation');
    } catch (error) {
      console.error('Error loading arena mesh:', error);
      // Fallback: add a simple floor if arena loading fails
      const planeGeometry = new THREE.PlaneGeometry(10240, 8192);
      const planeMaterial = new THREE.MeshStandardMaterial({
        color: 0x333333,
        side: THREE.DoubleSide,
      });
      const plane = new THREE.Mesh(planeGeometry, planeMaterial);
      plane.rotation.x = -Math.PI / 2;
      plane.receiveShadow = true;
      this.scene.add(plane);
      // Store fallback plane for raycasting
      this.arenaMeshes.push(plane);
    }
  }

  /**
   * Get all arena meshes for raycasting
   * @returns {THREE.Mesh[]}
   */
  getArenaMeshes() {
    return this.arenaMeshes;
  }

  /**
   * Get drawing collider meshes for raycasting (simplified geometry)
   * @returns {THREE.Mesh[]}
   */
  getDrawingColliderMeshes() {
    return this.drawingColliderMeshes;
  }

  /**
   * Load the simplified drawing collider mesh
   * @param {boolean} visible - Whether to show the collider (for debugging/positioning)
   */
  async loadDrawingCollider(visible = false) {
    try {
      console.log('[ArenaManager] Loading drawing collider...');
      const objLoader = new OBJLoader();
      const collider = await objLoader.loadAsync('/models/stadium/DrawingArena.obj');

      // Apply rotations to match arena orientation
      // X rotation: +90 degrees to lay the collider flat (it's modeled vertically)
      // Y rotation: 180 degrees to match arena's coordinate system
      collider.rotation.x = Math.PI / 2;
      collider.rotation.y = Math.PI;

      // Scale down slightly and raise so drawings appear in front of arena surfaces
      collider.scale.setScalar(0.99);
      collider.position.y = 20;

      // Collect meshes and apply debug material if visible
      collider.traverse((child) => {
        if (child.isMesh) {
          this.drawingColliderMeshes.push(child);
          // Drawing collider should never cast shadows
          child.castShadow = false;
          child.receiveShadow = false;

          if (visible) {
            // Debug material - semi-transparent solid green
            child.material = new THREE.MeshBasicMaterial({
              color: 0x00ff00,
              transparent: true,
              opacity: 0.7,
              side: THREE.DoubleSide,
            });
          } else {
            // Invisible material for raycasting only
            child.material = new THREE.MeshBasicMaterial({
              visible: false,
            });
          }
        }
      });

      this.drawingCollider = collider;
      this.scene.add(collider);

      console.log(`[ArenaManager] Drawing collider loaded with ${this.drawingColliderMeshes.length} meshes`);
    } catch (error) {
      console.error('[ArenaManager] Failed to load drawing collider:', error);
    }
  }

  /**
   * Toggle drawing collider visibility (for debugging)
   * @param {boolean} visible
   */
  setDrawingColliderVisible(visible) {
    for (const mesh of this.drawingColliderMeshes) {
      if (visible) {
        mesh.material = new THREE.MeshBasicMaterial({
          color: 0x00ff00,
          wireframe: true,
          transparent: true,
          opacity: 0.5,
        });
      } else {
        mesh.material = new THREE.MeshBasicMaterial({
          visible: false,
        });
      }
    }
  }

  /**
   * Load the arena decoration mesh (stands, stadium surroundings)
   * @param {boolean} show - Whether to show the decoration initially
   */
  async loadArenaDecor(show = true) {
    try {
      console.log('[ArenaManager] Loading arena decoration mesh...');

      const gltf = await this.gltfLoader.loadAsync('/models/stadium/arene.glb');
      this.arenaDecorMesh = gltf.scene;
      this.showArenaDecor = show;

      // Enable shadow receiving on decoration meshes
      this.arenaDecorMesh.traverse((child) => {
        if (child.isMesh) {
          child.receiveShadow = true;
          child.castShadow = true;
        }
      });

      // Set initial visibility
      this.arenaDecorMesh.visible = show;

      this.scene.add(this.arenaDecorMesh);
      console.log(`[ArenaManager] Arena decoration loaded, visible: ${show}`);
    } catch (error) {
      console.error('[ArenaManager] Failed to load arena decoration:', error);
    }
  }

  /**
   * Set the visibility of the arena decoration
   * @param {boolean} visible
   */
  setArenaDecorVisible(visible) {
    this.showArenaDecor = visible;
    if (this.arenaDecorMesh) {
      this.arenaDecorMesh.visible = visible;
      console.log(`[ArenaManager] Arena decoration visibility set to: ${visible}`);
    }
  }

  /**
   * Get the current visibility state of the arena decoration
   * @returns {boolean}
   */
  isArenaDecorVisible() {
    return this.showArenaDecor;
  }
}
