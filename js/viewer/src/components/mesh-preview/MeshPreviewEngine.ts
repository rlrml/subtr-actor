import * as THREE from 'three';
import CameraControls from 'camera-controls';
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js';
import { DRACOLoader } from 'three/examples/jsm/loaders/DRACOLoader.js';
import { TransformControls } from 'three/examples/jsm/controls/TransformControls.js';
import { RGBELoader } from 'three/examples/jsm/loaders/RGBELoader.js';
import { EXRLoader } from 'three/examples/jsm/loaders/EXRLoader.js';

// API URL for production/development
const API_URL = import.meta.env.VITE_API_URL || '/api';

// Install CameraControls with THREE
CameraControls.install({ THREE });

// Types for material management
export interface MaterialInfo {
  id: string;
  name: string;
  material: THREE.MeshStandardMaterial | THREE.MeshPhysicalMaterial;
  meshName: string;
  original: {
    color: string;
    roughness: number;
    metalness: number;
    emissive: string;
    emissiveIntensity: number;
    opacity: number;
    envMapIntensity: number;
  };
}

// Types for mesh hierarchy
export interface MeshNode {
  id: string;
  name: string;
  type: 'group' | 'mesh' | 'bone' | 'light' | 'camera' | 'other';
  object: THREE.Object3D;
  parentId: string | null;
  childIds: string[];
  materialIds?: string[];
  visible: boolean;
}

// Types for custom lights
export interface SceneLightInfo {
  id: string;
  type: 'ambient' | 'directional' | 'point' | 'spot';
  name: string;
  color: string;
  intensity: number;
  enabled: boolean;
  position?: THREE.Vector3;
  light: THREE.Light;
  helper: THREE.Object3D | null;
}

// Loading progress callback
export type LoadProgressCallback = (progress: number) => void;

/**
 * MeshPreviewEngine - Three.js scene manager for mesh preview tool
 * Handles rendering, camera, lighting, mesh loading, and material management
 */
export class MeshPreviewEngine {
  // Core Three.js objects
  private renderer: THREE.WebGLRenderer;
  private scene: THREE.Scene;
  private camera: THREE.PerspectiveCamera;
  private controls: CameraControls;

  // Loaders
  private gltfLoader: GLTFLoader;
  private dracoLoader: DRACOLoader;

  // Loaded mesh state
  private loadedMesh: THREE.Group | null = null;
  private meshBoundingBox: THREE.Box3 | null = null;
  private meshCenter: THREE.Vector3 = new THREE.Vector3();

  // Material management
  private materials: Map<string, MaterialInfo> = new Map();

  // Hierarchy
  private hierarchy: MeshNode[] = [];

  // Custom lights
  private customLights: Map<string, SceneLightInfo> = new Map();
  private defaultAmbient: THREE.AmbientLight;
  private defaultDirectional: THREE.DirectionalLight;

  // Ground plane (hidden by default to avoid z-fighting)
  private groundPlane: THREE.Mesh;

  // Transform controls
  private transformControls: TransformControls | null = null;
  private transformMode: 'translate' | 'rotate' | 'scale' = 'translate';

  // Animation
  private animationFrameId: number | null = null;
  private clock: THREE.Clock;

  // Canvas container
  private container: HTMLElement;

  // Raycaster for selection
  private raycaster: THREE.Raycaster;
  private mouse: THREE.Vector2;

  // Hover highlight
  private hoveredMesh: THREE.Mesh | null = null;
  private hoveredMaterialOriginalEmissive: THREE.Color | null = null;
  private lastHoverCheck: number = 0;
  private hoverThrottleMs: number = 50; // ~20fps for hover checks

  // Callbacks
  public onMaterialsUpdate?: (materials: MaterialInfo[]) => void;
  public onHierarchyUpdate?: (hierarchy: MeshNode[]) => void;
  public onMaterialSelect?: (materialId: string | null) => void;

  constructor(container: HTMLElement) {
    this.container = container;
    this.clock = new THREE.Clock();

    // Initialize renderer - ISO with main viewer (SceneManager.js)
    // Note: logarithmicDepthBuffer enabled here to prevent z-fighting
    // The main viewer can't use it due to shader recompilation issues with explosions
    // but MeshPreview doesn't have those effects, so we can safely enable it
    this.renderer = new THREE.WebGLRenderer({
      antialias: true,
      logarithmicDepthBuffer: true,
    });
    this.renderer.setSize(container.clientWidth, container.clientHeight);
    this.renderer.shadowMap.enabled = true;
    this.renderer.shadowMap.type = THREE.PCFSoftShadowMap;
    this.renderer.toneMapping = THREE.ACESFilmicToneMapping;
    this.renderer.toneMappingExposure = 1.0;
    this.renderer.outputColorSpace = THREE.SRGBColorSpace;
    container.appendChild(this.renderer.domElement);

    // Initialize scene with neutral gray background
    this.scene = new THREE.Scene();
    this.scene.background = new THREE.Color(0x1a1a1a);

    // Initialize camera - ISO with main viewer
    // FOV 75, near 10, far 50000 (same as SceneManager.js)
    this.camera = new THREE.PerspectiveCamera(
      75,
      container.clientWidth / container.clientHeight,
      10,
      50000
    );
    this.camera.position.set(0, 100, 500);

    // Initialize camera controls
    this.controls = new CameraControls(this.camera, this.renderer.domElement);
    this.controls.dollyToCursor = true;
    this.controls.infinityDolly = true;
    this.controls.smoothTime = 0.1;
    this.controls.draggingSmoothTime = 0.1;
    this.controls.minDistance = 10;
    this.controls.maxDistance = 50000;
    this.controls.minPolarAngle = 0;
    this.controls.maxPolarAngle = Math.PI;

    // Setup GLTF loader with DRACO
    this.dracoLoader = new DRACOLoader();
    this.dracoLoader.setDecoderPath('/draco/');
    this.gltfLoader = new GLTFLoader();
    this.gltfLoader.setDRACOLoader(this.dracoLoader);

    // Setup default lighting
    this.defaultAmbient = new THREE.AmbientLight(0xffffff, 0.4);
    this.scene.add(this.defaultAmbient);

    this.defaultDirectional = new THREE.DirectionalLight(0xffffff, 1.0);
    this.defaultDirectional.position.set(100, 200, 100);
    this.defaultDirectional.castShadow = true;
    this.defaultDirectional.shadow.mapSize.width = 2048;
    this.defaultDirectional.shadow.mapSize.height = 2048;
    this.defaultDirectional.shadow.camera.near = 0.5;
    this.defaultDirectional.shadow.camera.far = 1000;
    this.scene.add(this.defaultDirectional);

    // Ground plane for shadow receiving - disabled by default to avoid z-fighting
    // with meshes that have their own floor (like the arena)
    const groundGeometry = new THREE.PlaneGeometry(10000, 10000);
    const groundMaterial = new THREE.ShadowMaterial({ opacity: 0.3 });
    this.groundPlane = new THREE.Mesh(groundGeometry, groundMaterial);
    this.groundPlane.rotation.x = -Math.PI / 2;
    this.groundPlane.position.y = -1; // Below most mesh floors
    this.groundPlane.receiveShadow = true;
    this.groundPlane.name = '__ground__';
    this.groundPlane.visible = false; // Hidden by default
    this.scene.add(this.groundPlane);

    // Initialize raycaster for selection
    this.raycaster = new THREE.Raycaster();
    this.mouse = new THREE.Vector2();

    // Handle resize
    window.addEventListener('resize', this.handleResize);

    // Handle click for material selection
    this.renderer.domElement.addEventListener('click', this.handleClick);

    // Handle hover for mesh highlight
    this.renderer.domElement.addEventListener('mousemove', this.handleMouseMove);

    // Start render loop
    this.startRenderLoop();
  }

  /**
   * Handle click for material selection via raycasting
   */
  private handleClick = (event: MouseEvent) => {
    if (!this.loadedMesh) return;

    // Calculate mouse position in normalized device coordinates (-1 to +1)
    const rect = this.renderer.domElement.getBoundingClientRect();
    this.mouse.x = ((event.clientX - rect.left) / rect.width) * 2 - 1;
    this.mouse.y = -((event.clientY - rect.top) / rect.height) * 2 + 1;

    // Update raycaster
    this.raycaster.setFromCamera(this.mouse, this.camera);

    // Find intersected objects (only meshes from the loaded model)
    const intersects = this.raycaster.intersectObject(this.loadedMesh, true);

    if (intersects.length > 0) {
      const hit = intersects[0];
      const mesh = hit.object as THREE.Mesh;

      if (mesh.material) {
        // Get the material index if it's a multi-material mesh
        let materialIndex = 0;
        if (hit.face && Array.isArray(mesh.material)) {
          materialIndex = hit.face.materialIndex;
        }

        const materialId = `${mesh.uuid}_${materialIndex}`;

        // Notify listeners
        if (this.materials.has(materialId)) {
          this.onMaterialSelect?.(materialId);
        }
      }
    }
  };

  /**
   * Handle mouse move for hover highlight (throttled)
   */
  private handleMouseMove = (event: MouseEvent) => {
    // Throttle hover checks to reduce CPU usage
    const now = performance.now();
    if (now - this.lastHoverCheck < this.hoverThrottleMs) {
      return;
    }
    this.lastHoverCheck = now;

    if (!this.loadedMesh) {
      this.clearHoverHighlight();
      return;
    }

    // Calculate mouse position in normalized device coordinates (-1 to +1)
    const rect = this.renderer.domElement.getBoundingClientRect();
    this.mouse.x = ((event.clientX - rect.left) / rect.width) * 2 - 1;
    this.mouse.y = -((event.clientY - rect.top) / rect.height) * 2 + 1;

    // Update raycaster
    this.raycaster.setFromCamera(this.mouse, this.camera);

    // Find intersected objects
    const intersects = this.raycaster.intersectObject(this.loadedMesh, true);

    if (intersects.length > 0) {
      const hit = intersects[0];
      const mesh = hit.object as THREE.Mesh;

      // If hovering a different mesh, update highlight
      if (mesh !== this.hoveredMesh) {
        this.clearHoverHighlight();
        this.applyHoverHighlight(mesh);
      }
    } else {
      this.clearHoverHighlight();
    }
  };

  /**
   * Apply hover highlight to a mesh
   */
  private applyHoverHighlight(mesh: THREE.Mesh) {
    const material = mesh.material as THREE.MeshStandardMaterial;
    if (!material || !material.emissive) return;

    this.hoveredMesh = mesh;
    this.hoveredMaterialOriginalEmissive = material.emissive.clone();

    // Apply highlight - orange/amber emissive glow for visibility
    material.emissive.setHex(0x995522);

    // Change cursor to pointer
    this.renderer.domElement.style.cursor = 'pointer';
  }

  /**
   * Clear hover highlight from current mesh
   */
  private clearHoverHighlight() {
    if (this.hoveredMesh && this.hoveredMaterialOriginalEmissive) {
      const material = this.hoveredMesh.material as THREE.MeshStandardMaterial;
      if (material && material.emissive) {
        material.emissive.copy(this.hoveredMaterialOriginalEmissive);
      }
    }

    this.hoveredMesh = null;
    this.hoveredMaterialOriginalEmissive = null;

    // Reset cursor
    this.renderer.domElement.style.cursor = 'default';
  }

  /**
   * Select material at screen coordinates (alternative method)
   */
  selectMaterialAt(x: number, y: number): string | null {
    if (!this.loadedMesh) return null;

    const rect = this.renderer.domElement.getBoundingClientRect();
    this.mouse.x = ((x - rect.left) / rect.width) * 2 - 1;
    this.mouse.y = -((y - rect.top) / rect.height) * 2 + 1;

    this.raycaster.setFromCamera(this.mouse, this.camera);
    const intersects = this.raycaster.intersectObject(this.loadedMesh, true);

    if (intersects.length > 0) {
      const hit = intersects[0];
      const mesh = hit.object as THREE.Mesh;

      if (mesh.material) {
        let materialIndex = 0;
        if (hit.face && Array.isArray(mesh.material)) {
          materialIndex = hit.face.materialIndex;
        }

        const materialId = `${mesh.uuid}_${materialIndex}`;
        return this.materials.has(materialId) ? materialId : null;
      }
    }

    return null;
  }

  /**
   * Handle window/container resize
   */
  private handleResize = () => {
    const width = this.container.clientWidth;
    const height = this.container.clientHeight;

    this.camera.aspect = width / height;
    this.camera.updateProjectionMatrix();
    this.renderer.setSize(width, height);
  };

  /**
   * Start the render loop
   */
  private startRenderLoop() {
    const animate = () => {
      this.animationFrameId = requestAnimationFrame(animate);

      const delta = this.clock.getDelta();
      this.controls.update(delta);

      // Dynamic near plane adjustment to reduce z-fighting
      // Adjusts near plane based on camera distance from origin/target
      const cameraDistance = this.camera.position.length();
      // Near plane is 0.1% of distance, clamped between 0.1 and 100
      const dynamicNear = Math.max(0.1, Math.min(100, cameraDistance * 0.001));
      if (Math.abs(this.camera.near - dynamicNear) > 0.01) {
        this.camera.near = dynamicNear;
        this.camera.updateProjectionMatrix();
      }

      this.renderer.render(this.scene, this.camera);
    };
    animate();
  }

  /**
   * Stop the render loop
   */
  private stopRenderLoop() {
    if (this.animationFrameId !== null) {
      cancelAnimationFrame(this.animationFrameId);
      this.animationFrameId = null;
    }
  }

  /**
   * Format error messages for user display
   */
  private formatLoadError(error: unknown): string {
    const errorString = error instanceof Error ? error.message : String(error);

    // DRACO decoder errors
    if (errorString.includes('DRACOLoader') || errorString.includes('draco')) {
      return 'Failed to decode mesh compression. The DRACO decoder may be missing or corrupted.';
    }

    // GLTF parsing errors
    if (errorString.includes('JSON') || errorString.includes('parse')) {
      return 'Invalid GLTF file format. The file may be corrupted or not a valid GLTF/GLB.';
    }

    // Network/loading errors
    if (errorString.includes('Failed to fetch') || errorString.includes('NetworkError')) {
      return 'Failed to load the file. Please check your connection and try again.';
    }

    // Buffer/memory errors
    if (errorString.includes('ArrayBuffer') || errorString.includes('buffer')) {
      return 'Failed to read file data. The file may be corrupted.';
    }

    // Unknown Three.js errors
    if (errorString.includes('THREE') || errorString.includes('WebGL')) {
      return 'WebGL rendering error. Please try a different mesh or refresh the page.';
    }

    // Generic error
    return errorString || 'An unknown error occurred while loading the mesh.';
  }

  /**
   * Load a mesh from a File object
   */
  async loadMesh(file: File, onProgress?: LoadProgressCallback): Promise<THREE.Group> {
    // Validate file size (50MB limit)
    const maxSize = 50 * 1024 * 1024;
    if (file.size > maxSize) {
      throw new Error(`File size exceeds 50MB limit (${(file.size / 1024 / 1024).toFixed(1)}MB)`);
    }

    // Validate file extension
    const validExtensions = ['.glb', '.gltf'];
    const ext = file.name.toLowerCase().substring(file.name.lastIndexOf('.'));
    if (!validExtensions.includes(ext)) {
      throw new Error(`Invalid file format. Accepted formats: GLB, GLTF`);
    }

    // Clear existing mesh
    this.clearMesh();

    // Create object URL for loading
    const url = URL.createObjectURL(file);

    try {
      const gltf = await new Promise<{ scene: THREE.Group }>((resolve, reject) => {
        this.gltfLoader.load(
          url,
          (gltf) => resolve(gltf),
          (event) => {
            if (onProgress && event.lengthComputable) {
              onProgress(event.loaded / event.total);
            }
          },
          (error) => {
            // Format the error before rejecting
            reject(new Error(this.formatLoadError(error)));
          }
        );
      });

      this.loadedMesh = gltf.scene;
      this.loadedMesh.name = file.name;

      // Enable shadows on all meshes
      this.loadedMesh.traverse((child) => {
        if (child instanceof THREE.Mesh) {
          child.castShadow = true;
          child.receiveShadow = true;
        }
      });

      // Add to scene
      this.scene.add(this.loadedMesh);

      // Compute bounding box and center
      this.meshBoundingBox = new THREE.Box3().setFromObject(this.loadedMesh);
      this.meshBoundingBox.getCenter(this.meshCenter);

      // Auto-frame the mesh
      this.focusOnObject(this.loadedMesh);

      // Scan materials
      this.scanMaterials();

      // Build hierarchy
      this.buildHierarchy();

      return this.loadedMesh;
    } finally {
      URL.revokeObjectURL(url);
    }
  }

  /**
   * Clear the currently loaded mesh
   */
  clearMesh() {
    if (this.loadedMesh) {
      this.scene.remove(this.loadedMesh);
      this.loadedMesh.traverse((child) => {
        if (child instanceof THREE.Mesh) {
          child.geometry.dispose();
          if (Array.isArray(child.material)) {
            child.material.forEach((m) => m.dispose());
          } else {
            child.material.dispose();
          }
        }
      });
      this.loadedMesh = null;
    }
    this.meshBoundingBox = null;
    this.meshCenter.set(0, 0, 0);
    this.materials.clear();
    this.hierarchy = [];
  }

  /**
   * Focus camera on an object
   */
  focusOnObject(object: THREE.Object3D) {
    const box = new THREE.Box3().setFromObject(object);
    const center = new THREE.Vector3();
    const size = new THREE.Vector3();
    box.getCenter(center);
    box.getSize(size);

    const maxDim = Math.max(size.x, size.y, size.z);
    const fov = this.camera.fov * (Math.PI / 180);
    const distance = maxDim / (2 * Math.tan(fov / 2)) * 1.5;

    this.controls.setLookAt(
      center.x + distance * 0.5,
      center.y + distance * 0.5,
      center.z + distance,
      center.x,
      center.y,
      center.z,
      true
    );
  }

  /**
   * Scan and collect all PBR materials from the loaded mesh
   */
  scanMaterials() {
    this.materials.clear();

    if (!this.loadedMesh) return;

    this.loadedMesh.traverse((object) => {
      if (object instanceof THREE.Mesh && object.material) {
        const materials = Array.isArray(object.material) ? object.material : [object.material];

        materials.forEach((material, index) => {
          if (
            material instanceof THREE.MeshStandardMaterial ||
            material instanceof THREE.MeshPhysicalMaterial
          ) {
            const id = `${object.uuid}_${index}`;

            // Skip if already added (same material used by multiple meshes)
            if (this.materials.has(id)) return;

            this.materials.set(id, {
              id,
              name: material.name || `Material_${this.materials.size + 1}`,
              material,
              meshName: object.name || 'Unnamed Mesh',
              original: {
                color: '#' + material.color.getHexString(),
                roughness: material.roughness,
                metalness: material.metalness,
                emissive: '#' + material.emissive.getHexString(),
                emissiveIntensity: material.emissiveIntensity,
                opacity: material.opacity,
                envMapIntensity: material.envMapIntensity ?? 1,
              },
            });
          }
        });
      }
    });

    this.onMaterialsUpdate?.(Array.from(this.materials.values()));
  }

  /**
   * Get all materials
   */
  getMaterials(): MaterialInfo[] {
    return Array.from(this.materials.values());
  }

  /**
   * Update a material property
   */
  updateMaterial(id: string, property: string, value: unknown) {
    const info = this.materials.get(id);
    if (!info) return;

    const material = info.material;

    switch (property) {
      case 'color':
        material.color.set(value as string);
        break;
      case 'roughness':
        material.roughness = value as number;
        break;
      case 'metalness':
        material.metalness = value as number;
        break;
      case 'emissive':
        material.emissive.set(value as string);
        break;
      case 'emissiveIntensity':
        material.emissiveIntensity = value as number;
        break;
      case 'opacity':
        material.opacity = value as number;
        if ((value as number) < 1 && !material.transparent) {
          material.transparent = true;
        }
        break;
      case 'envMapIntensity':
        material.envMapIntensity = value as number;
        break;
    }

    material.needsUpdate = true;
  }

  /**
   * Reset a material to original values
   */
  resetMaterial(id: string) {
    const info = this.materials.get(id);
    if (!info) return;

    const material = info.material;
    const original = info.original;

    material.color.set(original.color);
    material.roughness = original.roughness;
    material.metalness = original.metalness;
    material.emissive.set(original.emissive);
    material.emissiveIntensity = original.emissiveIntensity;
    material.opacity = original.opacity;
    material.envMapIntensity = original.envMapIntensity;
    material.needsUpdate = true;

    this.onMaterialsUpdate?.(Array.from(this.materials.values()));
  }

  /**
   * Reset all materials to original values
   */
  resetAllMaterials() {
    this.materials.forEach((info) => {
      const material = info.material;
      const original = info.original;

      material.color.set(original.color);
      material.roughness = original.roughness;
      material.metalness = original.metalness;
      material.emissive.set(original.emissive);
      material.emissiveIntensity = original.emissiveIntensity;
      material.opacity = original.opacity;
      material.envMapIntensity = original.envMapIntensity;
      material.needsUpdate = true;
    });

    this.onMaterialsUpdate?.(Array.from(this.materials.values()));
  }

  /**
   * Build hierarchy tree from loaded mesh
   */
  buildHierarchy() {
    this.hierarchy = [];

    if (!this.loadedMesh) return;

    const buildNode = (object: THREE.Object3D, parentId: string | null): MeshNode => {
      let type: MeshNode['type'] = 'other';
      if (object instanceof THREE.Group) type = 'group';
      else if (object instanceof THREE.Mesh) type = 'mesh';
      else if (object instanceof THREE.Bone) type = 'bone';
      else if (object instanceof THREE.Light) type = 'light';
      else if (object instanceof THREE.Camera) type = 'camera';

      const materialIds: string[] = [];
      if (object instanceof THREE.Mesh && object.material) {
        const mats = Array.isArray(object.material) ? object.material : [object.material];
        mats.forEach((_, index) => {
          materialIds.push(`${object.uuid}_${index}`);
        });
      }

      const node: MeshNode = {
        id: object.uuid,
        name: object.name || `${type}_${object.uuid.slice(0, 8)}`,
        type,
        object,
        parentId,
        childIds: object.children.map((c) => c.uuid),
        materialIds: materialIds.length > 0 ? materialIds : undefined,
        visible: object.visible,
      };

      return node;
    };

    const traverse = (object: THREE.Object3D, parentId: string | null) => {
      const node = buildNode(object, parentId);
      this.hierarchy.push(node);
      object.children.forEach((child) => traverse(child, node.id));
    };

    traverse(this.loadedMesh, null);

    this.onHierarchyUpdate?.(this.hierarchy);
  }

  /**
   * Get hierarchy
   */
  getHierarchy(): MeshNode[] {
    return this.hierarchy;
  }

  /**
   * Get loaded mesh
   */
  getLoadedMesh(): THREE.Group | null {
    return this.loadedMesh;
  }

  /**
   * Get scene
   */
  getScene(): THREE.Scene {
    return this.scene;
  }

  /**
   * Get camera
   */
  getCamera(): THREE.PerspectiveCamera {
    return this.camera;
  }

  /**
   * Get camera controls
   */
  getControls(): CameraControls {
    return this.controls;
  }

  /**
   * Get renderer
   */
  getRenderer(): THREE.WebGLRenderer {
    return this.renderer;
  }

  /**
   * Initialize TransformControls (lazy)
   */
  private initTransformControls() {
    if (this.transformControls) return;

    this.transformControls = new TransformControls(this.camera, this.renderer.domElement);
    this.transformControls.setSize(0.75);

    // Disable camera controls while dragging transform
    this.transformControls.addEventListener('dragging-changed', (event) => {
      this.controls.enabled = !event.value;
    });

    // Handle uniform scaling
    this.transformControls.addEventListener('objectChange', () => {
      if (this.transformMode === 'scale' && this.loadedMesh) {
        // Apply uniform scale based on average
        const scale = this.loadedMesh.scale;
        const avgScale = (scale.x + scale.y + scale.z) / 3;
        this.loadedMesh.scale.setScalar(avgScale);
      }
    });

    this.scene.add(this.transformControls.getHelper());
  }

  /**
   * Attach TransformControls to loaded mesh
   */
  attachTransformControls() {
    if (!this.loadedMesh) return;

    this.initTransformControls();

    if (this.transformControls) {
      this.transformControls.attach(this.loadedMesh);
      this.transformControls.setMode(this.transformMode);
    }
  }

  /**
   * Detach TransformControls
   */
  detachTransformControls() {
    if (this.transformControls) {
      this.transformControls.detach();
    }
  }

  /**
   * Set transform mode
   */
  setTransformMode(mode: 'translate' | 'rotate' | 'scale') {
    this.transformMode = mode;

    if (this.transformControls) {
      this.transformControls.setMode(mode);
    }
  }

  /**
   * Get transform controls
   */
  getTransformControls(): TransformControls | null {
    return this.transformControls;
  }

  /**
   * Load default environment (neutral gray + studio lighting)
   */
  loadDefaultEnvironment() {
    // Reset scene background to neutral gray
    this.scene.background = new THREE.Color(0x1a1a1a);

    // Reset default lights to studio setup
    this.defaultAmbient.intensity = 0.4;
    this.defaultAmbient.color.setHex(0xffffff);

    this.defaultDirectional.intensity = 1.0;
    this.defaultDirectional.color.setHex(0xffffff);
    this.defaultDirectional.position.set(100, 200, 100);

    // Clear any environment map
    this.scene.environment = null;
  }

  /**
   * Load environment from backend API
   * Loads skybox and IBL only (no arena meshes)
   */
  async loadEnvironment(environmentId: string): Promise<void> {
    try {
      // Fetch environment data from API
      const response = await fetch(`${API_URL}/environments/${environmentId}`);
      if (!response.ok) {
        throw new Error('Environment not found');
      }

      const env = await response.json();

      // Load skybox if available (skyboxAsset contains the asset data)
      if (env.skyboxAsset?.id) {
        const skyboxUrl = `${API_URL}/assets/${env.skyboxAsset.id}/download`;
        const mimeType = env.skyboxAsset.mimeType || 'image/hdr';
        await this.loadSkybox(skyboxUrl, mimeType);

        // Apply exposure if available
        if (env.skyboxExposure !== undefined) {
          this.renderer.toneMappingExposure = env.skyboxExposure;
        }
      } else {
        // No skybox - use dark background
        this.scene.background = new THREE.Color(0x1a1a1a);
        this.scene.environment = null;
      }

      // Apply light settings from environment lights if available
      if (env.lights && Array.isArray(env.lights)) {
        for (const light of env.lights) {
          if (light.lightType === 'ambient') {
            this.defaultAmbient.intensity = light.intensity ?? 0.4;
            this.defaultAmbient.color.set(light.color || '#ffffff');
          } else if (light.lightType === 'directional') {
            this.defaultDirectional.intensity = light.intensity ?? 1.0;
            this.defaultDirectional.color.set(light.color || '#ffffff');
            if (light.params?.position) {
              this.defaultDirectional.position.set(
                light.params.position.x,
                light.params.position.y,
                light.params.position.z
              );
            }
          }
        }
      }

      console.log(`Environment loaded: ${env.name || environmentId}`);
    } catch (error) {
      console.error('Failed to load environment:', error);
      // Fallback to default
      this.loadDefaultEnvironment();
      throw error;
    }
  }

  /**
   * Load HDR/EXR skybox and set as background + environment map
   */
  private async loadSkybox(url: string, mimeType?: string): Promise<void> {
    return new Promise((resolve, reject) => {
      // Determine loader based on mime type or URL extension
      const isEXR = mimeType?.includes('exr') || url.toLowerCase().endsWith('.exr');
      const loader = isEXR ? new EXRLoader() : new RGBELoader();

      loader.load(
        url,
        (texture) => {
          texture.mapping = THREE.EquirectangularReflectionMapping;
          this.scene.background = texture;
          this.scene.environment = texture;
          resolve();
        },
        undefined,
        (error) => {
          console.error('Failed to load skybox:', error);
          reject(error);
        }
      );
    });
  }

  /**
   * Cleanup and dispose all resources
   */
  dispose() {
    this.stopRenderLoop();
    window.removeEventListener('resize', this.handleResize);
    this.renderer.domElement.removeEventListener('click', this.handleClick);
    this.renderer.domElement.removeEventListener('mousemove', this.handleMouseMove);

    this.clearHoverHighlight();
    this.clearMesh();

    // Dispose transform controls
    if (this.transformControls) {
      this.scene.remove(this.transformControls.getHelper());
      this.transformControls.dispose();
      this.transformControls = null;
    }

    // Dispose custom lights
    this.customLights.forEach((light) => {
      this.scene.remove(light.light);
      if (light.helper) this.scene.remove(light.helper);
    });
    this.customLights.clear();

    // Dispose default lights
    this.scene.remove(this.defaultAmbient);
    this.scene.remove(this.defaultDirectional);

    // Dispose renderer
    this.renderer.dispose();
    this.renderer.domElement.remove();

    // Dispose DRACO loader
    this.dracoLoader.dispose();
  }
}
