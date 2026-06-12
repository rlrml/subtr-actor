import * as THREE from 'three';
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js';
import { DRACOLoader } from 'three/examples/jsm/loaders/DRACOLoader.js';
import { RGBELoader } from 'three/examples/jsm/loaders/RGBELoader.js';
import { environmentApi } from '../services/environment.api';
import { assetApi } from '../services/asset.api';
import type { Environment, EnvironmentMesh, EnvironmentLight, ShadowParams } from '../types/environment';

export interface EnvironmentManagerOptions {
  scene: THREE.Scene;
  renderer: THREE.WebGLRenderer;
}

interface LoadedMesh {
  id: string;
  object: THREE.Object3D;
  assetId: string;
}

interface LoadedLight {
  id: string;
  light: THREE.Light;
  helper?: THREE.Object3D;
}

/**
 * EnvironmentManager handles loading and applying custom environments
 * from the database to the Three.js scene.
 */
export class EnvironmentManager {
  private scene: THREE.Scene;
  private renderer: THREE.WebGLRenderer;
  private gltfLoader: GLTFLoader;
  private rgbeLoader: RGBELoader;
  private dracoLoader: DRACOLoader;

  private loadedMeshes: LoadedMesh[] = [];
  private loadedLights: LoadedLight[] = [];
  private groundMesh: THREE.Mesh | null = null;
  private currentEnvironment: Environment | null = null;

  // Asset URL cache to avoid re-downloading
  private assetCache: Map<string, THREE.Object3D | THREE.Texture> = new Map();

  // Loading state
  private isLoading = false;
  private loadingProgress = 0;
  private _groundLoadingLock = false; // Prevent race conditions when loading ground

  // Skybox rotation state (Dev Mode Lot 3 - US1/US2)
  private _skyboxRotationX = 0;
  private _skyboxRotationY = 0;
  private _skyboxRotationZ = 0;
  private _skyboxAnimationEnabled = false;
  private _skyboxAnimationSpeed = 1.0; // degrees per second
  private _currentAnimatedRotation = 0; // Applied to Y axis during animation

  // Ground/Terrain constants (Dev Mode Lot 3 - US3/US4)
  // Arena is ~10240 x 8192 UU, terrain should cover far beyond for environment design
  private static readonly TERRAIN_SIZE = 100000;
  private static readonly TERRAIN_SEGMENTS = 128;
  // @ts-expect-error - Reserved for future use
  private static readonly _TERRAIN_VERTICES = 65 * 65; // 4225 vertices total

  // Callbacks
  public onLoadingStart?: () => void;
  public onLoadingProgress?: (progress: number) => void;
  public onLoadingComplete?: () => void;
  public onError?: (error: Error) => void;

  constructor(options: EnvironmentManagerOptions) {
    this.scene = options.scene;
    this.renderer = options.renderer;

    // Initialize loaders
    this.gltfLoader = new GLTFLoader();
    this.dracoLoader = new DRACOLoader();
    this.dracoLoader.setDecoderPath('/draco/');
    this.gltfLoader.setDRACOLoader(this.dracoLoader);

    this.rgbeLoader = new RGBELoader();
  }

  /**
   * Get the current environment
   */
  getCurrentEnvironment(): Environment | null {
    return this.currentEnvironment;
  }

  /**
   * Check if currently loading
   */
  getIsLoading(): boolean {
    return this.isLoading;
  }

  /**
   * Get loading progress (0-100)
   */
  getLoadingProgress(): number {
    return this.loadingProgress;
  }

  /**
   * Load an environment by ID
   */
  async loadEnvironment(environmentId: string): Promise<void> {
    if (this.isLoading) {
      console.warn('EnvironmentManager: Already loading an environment');
      return;
    }

    this.isLoading = true;
    this.loadingProgress = 0;
    this.onLoadingStart?.();

    try {
      // Fetch environment data
      const environment = await environmentApi.get(environmentId);
      if (!environment) {
        throw new Error(`Environment not found: ${environmentId}`);
      }

      // Clear current environment
      await this.clearEnvironment();

      // Load all elements
      const totalSteps =
        (environment.skyboxAsset ? 1 : 0) +
        (environment.groundEnabled && environment.groundTexture ? 1 : 0) +
        (environment.meshes?.length || 0) +
        (environment.lights?.length || 0);

      let completedSteps = 0;
      const updateProgress = () => {
        completedSteps++;
        this.loadingProgress = Math.round((completedSteps / totalSteps) * 100);
        this.onLoadingProgress?.(this.loadingProgress);
      };

      // Load skybox
      if (environment.skyboxAsset) {
        await this.loadSkybox(environment.skyboxAsset.id, environment.skyboxExposure);
        updateProgress();
      }

      // Apply skybox rotation (Dev Mode Lot 3 - US1/US2)
      this._skyboxRotationX = environment.skyboxRotationX ?? 0;
      this._skyboxRotationY = environment.skyboxRotationY ?? 0;
      this._skyboxRotationZ = environment.skyboxRotationZ ?? 0;
      this._skyboxAnimationEnabled = environment.skyboxAnimationEnabled ?? false;
      this._skyboxAnimationSpeed = environment.skyboxAnimationSpeed ?? 1.0;
      this._currentAnimatedRotation = 0;
      this._applySkyboxRotation();

      // Load ground
      if (environment.groundEnabled && environment.groundTexture) {
        await this.loadGround(
          environment.groundTexture.id,
          environment.groundRepeatX,
          environment.groundRepeatY,
          environment.groundHeight ?? 0,
          environment.groundHeightmap ?? null
        );
        updateProgress();
      }

      // Load meshes in parallel
      if (environment.meshes && environment.meshes.length > 0) {
        await Promise.all(
          environment.meshes.map(async (mesh) => {
            await this.loadMesh(mesh);
            updateProgress();
          })
        );
      }

      // Create lights
      if (environment.lights && environment.lights.length > 0) {
        for (const light of environment.lights) {
          this.createLight(light);
          updateProgress();
        }
      }

      this.currentEnvironment = environment;
      this.loadingProgress = 100;
      this.onLoadingComplete?.();
    } catch (error) {
      const err = error instanceof Error ? error : new Error(String(error));
      console.error('EnvironmentManager: Failed to load environment', err);
      this.onError?.(err);
      throw err;
    } finally {
      this.isLoading = false;
    }
  }

  /**
   * Load the default environment
   */
  async loadDefaultEnvironment(): Promise<void> {
    try {
      const defaultEnv = await environmentApi.getDefault();
      if (defaultEnv) {
        await this.loadEnvironment(defaultEnv.id);
      }
    } catch (error) {
      console.warn('EnvironmentManager: No default environment available');
    }
  }

  /**
   * Clear the current environment
   * @param clearSkybox - If true, also clears the skybox (default: true)
   */
  async clearEnvironment(clearSkybox: boolean = true): Promise<void> {
    // Remove meshes
    for (const { object } of this.loadedMeshes) {
      this.scene.remove(object);
      this.disposeObject(object);
    }
    this.loadedMeshes = [];

    // Remove lights
    for (const { light, helper } of this.loadedLights) {
      this.scene.remove(light);
      if (helper) {
        this.scene.remove(helper);
      }
      light.dispose?.();
    }
    this.loadedLights = [];

    // Remove ground
    if (this.groundMesh) {
      this.scene.remove(this.groundMesh);
      this.disposeObject(this.groundMesh);
      this.groundMesh = null;
    }

    // Reset skybox rotation state (Dev Mode Lot 3 - US1/US2)
    this.resetSkyboxRotation();

    // Clear skybox if requested
    if (clearSkybox) {
      this.clearSkybox();
    }

    this.currentEnvironment = null;
  }

  /**
   * Clear the skybox from the scene
   */
  clearSkybox(): void {
    // Dispose existing environment/background textures
    if (this.scene.environment instanceof THREE.Texture) {
      this.scene.environment.dispose();
    }
    if (this.scene.background instanceof THREE.Texture) {
      (this.scene.background as THREE.Texture).dispose();
    }

    this.scene.environment = null;
    this.scene.background = null;

    // Reset exposure to default
    this.renderer.toneMappingExposure = 1.0;
  }

  /**
   * Load a mesh from an asset
   */
  async loadMesh(meshData: EnvironmentMesh): Promise<THREE.Object3D | null> {
    const assetId = meshData.assetId;
    const url = assetApi.getDownloadUrl(assetId);

    try {
      let object: THREE.Object3D;

      // Check cache
      const cached = this.assetCache.get(url);
      if (cached && cached instanceof THREE.Object3D) {
        object = cached.clone();
      } else {
        // Load the GLTF
        const gltf = await new Promise<THREE.Object3D>((resolve, reject) => {
          this.gltfLoader.load(
            url,
            (gltf) => resolve(gltf.scene),
            undefined,
            reject
          );
        });
        this.assetCache.set(url, gltf);
        object = gltf.clone();
      }

      // Apply transform
      object.position.set(
        meshData.position.x,
        meshData.position.y,
        meshData.position.z
      );
      object.rotation.set(
        meshData.rotation.x,
        meshData.rotation.y,
        meshData.rotation.z
      );
      object.scale.set(
        meshData.scale.x,
        meshData.scale.y,
        meshData.scale.z
      );

      object.visible = meshData.visible;
      object.userData.environmentMeshId = meshData.id;
      object.userData.assetId = assetId;

      // Add to scene
      this.scene.add(object);
      this.loadedMeshes.push({
        id: meshData.id,
        object,
        assetId,
      });

      return object;
    } catch (error) {
      console.error(`EnvironmentManager: Failed to load mesh ${assetId}`, error);
      return null;
    }
  }

  /**
   * Create a light from configuration
   */
  createLight(lightData: EnvironmentLight): THREE.Light | null {
    let light: THREE.Light;
    const params = lightData.params as Record<string, unknown>;

    const color = new THREE.Color(lightData.color);

    switch (lightData.lightType) {
      case 'ambient':
        light = new THREE.AmbientLight(color, lightData.intensity);
        break;

      case 'hemisphere': {
        const groundColor = new THREE.Color((params.groundColor as string) || '#444444');
        light = new THREE.HemisphereLight(color, groundColor, lightData.intensity);
        break;
      }

      case 'point': {
        const pointLight = new THREE.PointLight(
          color,
          lightData.intensity,
          (params.distance as number) || 0,
          (params.decay as number) || 2
        );
        const pos = params.position as { x: number; y: number; z: number } | undefined;
        if (pos) {
          pointLight.position.set(pos.x, pos.y, pos.z);
        }
        light = pointLight;
        break;
      }

      case 'spot': {
        const spotLight = new THREE.SpotLight(
          color,
          lightData.intensity,
          (params.distance as number) || 0,
          (params.angle as number) || Math.PI / 4,
          (params.penumbra as number) || 0.1,
          (params.decay as number) || 2
        );
        const spotPos = params.position as { x: number; y: number; z: number } | undefined;
        if (spotPos) {
          spotLight.position.set(spotPos.x, spotPos.y, spotPos.z);
        }
        const target = params.target as { x: number; y: number; z: number } | undefined;
        if (target) {
          spotLight.target.position.set(target.x, target.y, target.z);
          this.scene.add(spotLight.target);
        }
        // Apply shadow configuration
        const spotShadow = params.shadow as ShadowParams | undefined;
        if (spotShadow?.enabled) {
          spotLight.castShadow = true;
          spotLight.shadow.mapSize.width = spotShadow.mapSize || 2048;
          spotLight.shadow.mapSize.height = spotShadow.mapSize || 2048;
          spotLight.shadow.bias = spotShadow.bias ?? -0.0001;
          spotLight.shadow.normalBias = spotShadow.normalBias ?? 0;
          spotLight.shadow.camera.near = spotShadow.cameraNear ?? 10;
          spotLight.shadow.camera.far = spotShadow.cameraFar ?? 10000;
        }
        light = spotLight;
        break;
      }

      case 'directional': {
        const dirLight = new THREE.DirectionalLight(color, lightData.intensity);
        const dirPos = params.position as { x: number; y: number; z: number } | undefined;
        if (dirPos) {
          dirLight.position.set(dirPos.x, dirPos.y, dirPos.z);
        }
        const dirTarget = params.target as { x: number; y: number; z: number } | undefined;
        if (dirTarget) {
          dirLight.target.position.set(dirTarget.x, dirTarget.y, dirTarget.z);
          this.scene.add(dirLight.target);
        }
        // Apply shadow configuration
        const dirShadow = params.shadow as ShadowParams | undefined;
        if (dirShadow?.enabled) {
          dirLight.castShadow = true;
          dirLight.shadow.mapSize.width = dirShadow.mapSize || 2048;
          dirLight.shadow.mapSize.height = dirShadow.mapSize || 2048;
          dirLight.shadow.bias = dirShadow.bias ?? -0.0001;
          dirLight.shadow.normalBias = dirShadow.normalBias ?? 0.02;
          // Orthographic camera bounds for directional light
          dirLight.shadow.camera.left = dirShadow.cameraLeft ?? -10000;
          dirLight.shadow.camera.right = dirShadow.cameraRight ?? 10000;
          dirLight.shadow.camera.top = dirShadow.cameraTop ?? 10000;
          dirLight.shadow.camera.bottom = dirShadow.cameraBottom ?? -10000;
          dirLight.shadow.camera.near = dirShadow.cameraNear ?? 100;
          dirLight.shadow.camera.far = dirShadow.cameraFar ?? 15000;
        }
        light = dirLight;
        break;
      }

      default:
        console.warn(`EnvironmentManager: Unknown light type ${lightData.lightType}`);
        return null;
    }

    light.visible = lightData.enabled;
    light.userData.environmentLightId = lightData.id;

    this.scene.add(light);
    this.loadedLights.push({
      id: lightData.id,
      light,
    });

    return light;
  }

  /**
   * Load a skybox from an asset
   */
  async loadSkybox(assetId: string, exposure: number): Promise<void> {
    const url = assetApi.getDownloadUrl(assetId);

    try {
      const texture = await new Promise<THREE.Texture>((resolve, reject) => {
        this.rgbeLoader.load(
          url,
          (texture) => {
            texture.mapping = THREE.EquirectangularReflectionMapping;
            resolve(texture);
          },
          undefined,
          reject
        );
      });

      this.scene.environment = texture;
      this.scene.background = texture;

      // Apply exposure
      this.renderer.toneMappingExposure = exposure;
    } catch (error) {
      console.error(`EnvironmentManager: Failed to load skybox ${assetId}`, error);
    }
  }

  // ============================================
  // Skybox Rotation Methods (Dev Mode Lot 3 - US1/US2)
  // ============================================

  /**
   * Set the static skybox rotation on a single axis (0-360 degrees)
   * US1: Static Skybox Rotation
   * @param degrees Rotation value in degrees
   * @param axis Which axis to rotate: 'x', 'y', or 'z'
   */
  setSkyboxRotation(degrees: number, axis: 'x' | 'y' | 'z' = 'y'): void {
    // Normalize to 0-360 range
    const normalized = ((degrees % 360) + 360) % 360;
    switch (axis) {
      case 'x':
        this._skyboxRotationX = normalized;
        break;
      case 'y':
        this._skyboxRotationY = normalized;
        break;
      case 'z':
        this._skyboxRotationZ = normalized;
        break;
    }
    this._applySkyboxRotation();
  }

  /**
   * Set all skybox rotations at once
   * @param x X-axis rotation in degrees
   * @param y Y-axis rotation in degrees
   * @param z Z-axis rotation in degrees
   */
  setSkyboxRotationXYZ(x: number, y: number, z: number): void {
    this._skyboxRotationX = ((x % 360) + 360) % 360;
    this._skyboxRotationY = ((y % 360) + 360) % 360;
    this._skyboxRotationZ = ((z % 360) + 360) % 360;
    this._applySkyboxRotation();
  }

  /**
   * Get the current skybox rotation in degrees for a specific axis
   * @param axis Which axis: 'x', 'y', or 'z'
   */
  getSkyboxRotation(axis: 'x' | 'y' | 'z' = 'y'): number {
    switch (axis) {
      case 'x':
        return this._skyboxRotationX;
      case 'y':
        return this._skyboxRotationY;
      case 'z':
        return this._skyboxRotationZ;
    }
  }

  /**
   * Get all skybox rotations
   */
  getSkyboxRotationXYZ(): { x: number; y: number; z: number } {
    return {
      x: this._skyboxRotationX,
      y: this._skyboxRotationY,
      z: this._skyboxRotationZ,
    };
  }

  /**
   * Apply the skybox rotation to the scene
   */
  private _applySkyboxRotation(): void {
    // Y-axis includes animated rotation
    const totalRotationY = this._skyboxRotationY + this._currentAnimatedRotation;

    const radiansX = THREE.MathUtils.degToRad(this._skyboxRotationX);
    const radiansY = THREE.MathUtils.degToRad(totalRotationY);
    const radiansZ = THREE.MathUtils.degToRad(this._skyboxRotationZ);

    // Access scene with extended type for backgroundRotation/environmentRotation
    // These properties exist in Three.js r152+ but types may not be updated
    const sceneExt = this.scene as unknown as {
      backgroundRotation: THREE.Euler;
      environmentRotation: THREE.Euler;
    };

    // Use .set() method to modify the existing Euler objects
    // backgroundRotation/environmentRotation added in Three.js r162+
    if (sceneExt.backgroundRotation) {
      sceneExt.backgroundRotation.set(radiansX, radiansY, radiansZ);
    }
    if (sceneExt.environmentRotation) {
      sceneExt.environmentRotation.set(radiansX, radiansY, radiansZ);
    }
  }

  /**
   * Start animated skybox rotation
   * US2: Animated Skybox Rotation
   */
  startSkyboxAnimation(speed?: number): void {
    if (speed !== undefined) {
      this._skyboxAnimationSpeed = speed;
    }
    this._skyboxAnimationEnabled = true;
  }

  /**
   * Stop animated skybox rotation
   */
  stopSkyboxAnimation(): void {
    this._skyboxAnimationEnabled = false;
  }

  /**
   * Check if skybox animation is enabled
   */
  isSkyboxAnimationEnabled(): boolean {
    return this._skyboxAnimationEnabled;
  }

  /**
   * Set skybox animation speed (degrees per second)
   */
  setSkyboxAnimationSpeed(speed: number): void {
    this._skyboxAnimationSpeed = speed;
  }

  /**
   * Get skybox animation speed
   */
  getSkyboxAnimationSpeed(): number {
    return this._skyboxAnimationSpeed;
  }

  /**
   * Update skybox animation - call this from render loop
   * @param deltaTime Time since last frame in seconds
   */
  updateSkyboxAnimation(deltaTime: number): void {
    if (!this._skyboxAnimationEnabled) return;

    // Update animated rotation based on speed and time
    this._currentAnimatedRotation += this._skyboxAnimationSpeed * deltaTime;

    // Normalize to prevent overflow
    this._currentAnimatedRotation = this._currentAnimatedRotation % 360;

    this._applySkyboxRotation();
  }

  /**
   * Reset skybox rotation to default state
   */
  resetSkyboxRotation(): void {
    this._skyboxRotationX = 0;
    this._skyboxRotationY = 0;
    this._skyboxRotationZ = 0;
    this._currentAnimatedRotation = 0;
    this._skyboxAnimationEnabled = false;
    this._skyboxAnimationSpeed = 1.0;
    this._applySkyboxRotation();
  }

  // ============================================
  // Ground Height Methods (Dev Mode Lot 3 - US3)
  // ============================================

  /**
   * Set ground plane height
   * US3: Ground Plane with Texture
   */
  setGroundHeight(height: number): void {
    if (this.groundMesh) {
      this.groundMesh.position.y = height;
    }
  }

  /**
   * Get current ground height
   */
  getGroundHeight(): number {
    return this.groundMesh?.position.y ?? 0;
  }

  // ============================================
  // Terraforming Methods (Dev Mode Lot 3 - US4)
  // ============================================

  /**
   * Apply brush to terrain at given position
   * US4: Ground Terraforming
   * @param worldX World X coordinate
   * @param worldZ World Z coordinate
   * @param radius Brush radius in world units
   * @param strength Brush strength (positive = raise, negative = lower)
   * @param falloff Falloff type: 'linear', 'smooth', 'constant'
   */
  applyBrush(
    worldX: number,
    worldZ: number,
    radius: number,
    strength: number,
    falloff: 'linear' | 'smooth' | 'constant' = 'smooth'
  ): void {
    if (!this.groundMesh) return;

    const geometry = this.groundMesh.geometry as THREE.PlaneGeometry;
    const positions = geometry.attributes.position;

    if (!positions) return;

    for (let i = 0; i < positions.count; i++) {
      // Get vertex position (plane is rotated -90 deg on X, so Y becomes Z)
      const vx = positions.getX(i);
      const vz = positions.getY(i); // Y in local space = Z in world after rotation

      // Convert to world coordinates
      const wx = vx;
      const wz = -vz; // Flip because of rotation

      // Calculate distance from brush center
      const dx = wx - worldX;
      const dz = wz - worldZ;
      const distance = Math.sqrt(dx * dx + dz * dz);

      if (distance <= radius) {
        // Calculate falloff factor
        let factor = 1.0;
        const normalizedDist = distance / radius;

        switch (falloff) {
          case 'linear':
            factor = 1.0 - normalizedDist;
            break;
          case 'smooth':
            // Smooth hermite interpolation
            factor = 1.0 - (3 * normalizedDist * normalizedDist - 2 * normalizedDist * normalizedDist * normalizedDist);
            break;
          case 'constant':
            factor = 1.0;
            break;
        }

        // Apply height change
        const currentHeight = positions.getZ(i);
        positions.setZ(i, currentHeight + strength * factor);
      }
    }

    positions.needsUpdate = true;
    this.recalculateGroundNormals();
  }

  /**
   * Recalculate ground mesh normals after terrain modification
   */
  recalculateGroundNormals(): void {
    if (!this.groundMesh) return;

    const geometry = this.groundMesh.geometry;
    geometry.computeVertexNormals();
  }

  /**
   * Serialize heightmap for persistence
   * @returns Array of height values or null if no ground
   */
  serializeHeightmap(): number[] | null {
    if (!this.groundMesh) return null;

    const geometry = this.groundMesh.geometry as THREE.PlaneGeometry;
    const positions = geometry.attributes.position;

    if (!positions) return null;

    const heightmap: number[] = [];
    for (let i = 0; i < positions.count; i++) {
      heightmap.push(positions.getZ(i));
    }

    return heightmap;
  }

  /**
   * Deserialize and apply heightmap from persistence
   * @param heightmap Array of height values
   */
  deserializeHeightmap(heightmap: number[] | null): void {
    if (!heightmap || !this.groundMesh) return;

    const geometry = this.groundMesh.geometry as THREE.PlaneGeometry;
    const positions = geometry.attributes.position;

    if (!positions || heightmap.length !== positions.count) {
      console.warn('EnvironmentManager: Heightmap size mismatch');
      return;
    }

    for (let i = 0; i < positions.count; i++) {
      positions.setZ(i, heightmap[i]);
    }

    positions.needsUpdate = true;
    this.recalculateGroundNormals();
  }

  /**
   * Flatten terrain to a specific height
   */
  flattenTerrain(targetHeight: number = 0): void {
    if (!this.groundMesh) return;

    const geometry = this.groundMesh.geometry as THREE.PlaneGeometry;
    const positions = geometry.attributes.position;

    if (!positions) return;

    for (let i = 0; i < positions.count; i++) {
      positions.setZ(i, targetHeight);
    }

    positions.needsUpdate = true;
    this.recalculateGroundNormals();
  }

  /**
   * Get ground mesh for external access (e.g., raycasting)
   */
  getGroundMesh(): THREE.Mesh | null {
    return this.groundMesh;
  }

  /**
   * Remove the current ground mesh and dispose resources
   */
  removeGround(): void {
    if (this.groundMesh) {
      this.scene.remove(this.groundMesh);
      this.disposeObject(this.groundMesh);
      this.groundMesh = null;
    }
  }

  /**
   * Load the ground texture
   * @param textureAssetId Texture asset ID
   * @param repeatX Texture repeat X
   * @param repeatY Texture repeat Y
   * @param height Ground plane height (default 0)
   * @param heightmap Optional heightmap for terraforming
   */
  async loadGround(
    textureAssetId: string,
    repeatX: number,
    repeatY: number,
    height: number = 0,
    heightmap: number[] | null = null
  ): Promise<void> {
    // Prevent race conditions - if already loading, skip this call
    if (this._groundLoadingLock) {
      return;
    }

    this._groundLoadingLock = true;

    // Remove any existing ground before loading new one
    this.removeGround();

    const url = assetApi.getDownloadUrl(textureAssetId);

    try {
      const textureLoader = new THREE.TextureLoader();
      const texture = await new Promise<THREE.Texture>((resolve, reject) => {
        textureLoader.load(url, resolve, undefined, reject);
      });

      // Check if we were cancelled during async load
      if (this.groundMesh) {
        texture.dispose();
        this._groundLoadingLock = false;
        return;
      }

      texture.wrapS = THREE.RepeatWrapping;
      texture.wrapT = THREE.RepeatWrapping;
      texture.repeat.set(repeatX, repeatY);

      // Create ground plane with subdivisions for terraforming (US4)
      const geometry = new THREE.PlaneGeometry(
        EnvironmentManager.TERRAIN_SIZE,
        EnvironmentManager.TERRAIN_SIZE,
        EnvironmentManager.TERRAIN_SEGMENTS,
        EnvironmentManager.TERRAIN_SEGMENTS
      );
      const material = new THREE.MeshStandardMaterial({
        map: texture,
        roughness: 0.8,
        metalness: 0.2,
      });

      this.groundMesh = new THREE.Mesh(geometry, material);
      this.groundMesh.rotation.x = -Math.PI / 2;
      this.groundMesh.position.y = height;
      this.groundMesh.receiveShadow = true;
      this.groundMesh.userData.isGround = true;

      this.scene.add(this.groundMesh);

      // Apply heightmap if provided (must be after adding to scene)
      if (heightmap) {
        this.deserializeHeightmap(heightmap);
      }
    } catch (error) {
      console.error(`EnvironmentManager: Failed to load ground texture ${textureAssetId}`, error);
    } finally {
      this._groundLoadingLock = false;
    }
  }

  /**
   * Dispose an object and its resources
   */
  private disposeObject(object: THREE.Object3D): void {
    object.traverse((child) => {
      if (child instanceof THREE.Mesh) {
        child.geometry?.dispose();
        if (Array.isArray(child.material)) {
          child.material.forEach((mat) => this.disposeMaterial(mat));
        } else if (child.material) {
          this.disposeMaterial(child.material);
        }
      }
    });
  }

  /**
   * Dispose a material and its textures
   */
  private disposeMaterial(material: THREE.Material): void {
    if ('map' in material && material.map) {
      (material.map as THREE.Texture).dispose();
    }
    if ('normalMap' in material && material.normalMap) {
      (material.normalMap as THREE.Texture).dispose();
    }
    if ('roughnessMap' in material && material.roughnessMap) {
      (material.roughnessMap as THREE.Texture).dispose();
    }
    if ('metalnessMap' in material && material.metalnessMap) {
      (material.metalnessMap as THREE.Texture).dispose();
    }
    material.dispose();
  }

  /**
   * Clear the asset cache
   */
  clearCache(): void {
    for (const [, value] of this.assetCache) {
      if (value instanceof THREE.Texture) {
        value.dispose();
      } else if (value instanceof THREE.Object3D) {
        this.disposeObject(value);
      }
    }
    this.assetCache.clear();
  }

  /**
   * Dispose all resources
   */
  dispose(): void {
    this.clearEnvironment();
    this.clearCache();
    this.dracoLoader.dispose();
  }
}
