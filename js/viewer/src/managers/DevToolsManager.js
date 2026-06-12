import * as THREE from 'three';
import { TransformControls } from 'three/examples/jsm/controls/TransformControls.js';
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js';
import { DRACOLoader } from 'three/examples/jsm/loaders/DRACOLoader.js';
import { assetApi } from '../services/asset.api';

/**
 * DevToolsManager - Manages scene inspection and manipulation for development
 *
 * Features:
 * - Material inspection and editing (roughness, metalness)
 * - Dynamic light creation, positioning, and removal
 * - Mesh selection via raycasting
 * - Transform controls for positioning lights
 * - DevTools camera mode (orbit + keyboard navigation)
 */
export class DevToolsManager {
  constructor(scene, camera, renderer, cameraManager = null) {
    this.scene = scene;
    this.camera = camera;
    this.renderer = renderer;
    this.cameraManager = cameraManager; // Reference to CameraManager to disable its controls in dev mode

    // Material editing
    this.meshMaterials = new Map(); // Map of mesh name -> material info
    this.selectedMesh = null;

    // Lights management
    this.customLights = new Map(); // Map of light id -> { light, helper, type }
    this.lightIdCounter = 0;
    this.selectedLight = null;

    // Placed meshes management
    this.placedMeshes = new Map(); // Map of mesh id -> { object, assetId, name, transform }
    this.meshIdCounter = 0;
    this.selectedPlacedMesh = null;
    this._meshTransformMode = 'translate'; // 'translate', 'rotate', 'scale'

    // GLTF Loader for meshes
    this.gltfLoader = new GLTFLoader();
    this.dracoLoader = new DRACOLoader();
    this.dracoLoader.setDecoderPath('/draco/');
    this.gltfLoader.setDRACOLoader(this.dracoLoader);

    // Mesh asset cache
    this.meshCache = new Map(); // assetId -> THREE.Object3D

    // Raycaster for mesh selection
    this.raycaster = new THREE.Raycaster();
    this.mouse = new THREE.Vector2();

    // Transform controls for light positioning - lazy initialization
    this.transformControls = null;
    this._transformControlsInitialized = false;

    // Event callbacks
    this.onMaterialsUpdate = null;
    this.onLightsUpdate = null;
    this.onPlacedMeshesUpdate = null; // Callback when placed meshes change
    this.onSelectionChange = null;
    this.onDevModeChange = null; // Callback when dev mode is toggled

    // Bind methods
    this.handlePointerDown = this.handlePointerDown.bind(this);
    this.handleMouseMove = this.handleMouseMove.bind(this);
    this._handleKeyDown = this._handleKeyDown.bind(this);
    this._handleKeyUp = this._handleKeyUp.bind(this);
    this._preventContextMenu = this._preventContextMenu.bind(this);
    this._handleMouseDown = this._handleMouseDown.bind(this);
    this._handleMouseUp = this._handleMouseUp.bind(this);
    this._handleMouseMoveRotate = this._handleMouseMoveRotate.bind(this);
    this._handleWheel = this._handleWheel.bind(this);

    // Selection mode
    this.selectionMode = 'none'; // 'none', 'mesh', 'light-position'

    // Hovering
    this.hoveredMesh = null;
    this.originalMaterials = new Map(); // For hover highlight
    this.outlineMesh = null; // Outline mesh for hover highlight
    this.tooltip = null; // Tooltip element for mesh info

    // Raycast hit cycling (scroll through meshes on raycast path)
    this.raycastHits = []; // Array of valid meshes from last click
    this.currentHitIndex = 0; // Current index in raycastHits

    // Throttle for lights update to prevent freeze
    this._lastLightsUpdate = 0;
    this._lightsUpdateThrottle = 100; // ms

    // DevTools camera mode
    this._devModeActive = false;
    this._savedCameraState = null;
    this._isRightMouseDown = false;
    this._euler = new THREE.Euler(0, 0, 0, 'YXZ');
    this._moveState = { forward: false, backward: false, left: false, right: false, up: false, down: false };
    this._moveSpeed = 3000; // Unreal units per second (1 unit = 1 cm, so 3000 = 30 m/s)

    // Terraforming mode (Dev Mode Lot 3 - US4)
    this._terraformingMode = false;
    this._terraformBrushSize = 500; // World units (increased for larger terrain)
    this._terraformBrushStrength = 10.0; // Increased for visible effect
    this._isTerraforming = false; // Currently painting
    this._terraformRaycaster = new THREE.Raycaster();
    
    // Bind terraforming handlers
    this._handleTerraformMouseDown = this._handleTerraformMouseDown.bind(this);
    this._handleTerraformMouseMove = this._handleTerraformMouseMove.bind(this);
    this._handleTerraformMouseUp = this._handleTerraformMouseUp.bind(this);

    // Brush preview (visual indicator)
    this._brushPreview = null;
    this._brushPreviewVisible = false;

    // Callback for terraforming mode change
    this.onTerraformingModeChange = null;

    // Bind focus-related handlers for camera controls
    this._handleWindowBlur = this._handleWindowBlur.bind(this);
    this._handleVisibilityChange = this._handleVisibilityChange.bind(this);
    this._handlePointerLockChange = this._handlePointerLockChange.bind(this);
  }

  /**
   * Check if dev mode is active
   */
  isDevModeActive() {
    return this._devModeActive;
  }

  /**
   * Enter DevTools camera mode
   * - Saves current camera state
   * - Enables orbit controls for angle changes (middle mouse / right click drag)
   * - Enables keyboard navigation (arrow keys)
   * - Left click for mesh selection
   */
  enterDevMode() {
    if (this._devModeActive) return;

    console.log('[DevTools] Entering dev mode');

    // Exit any pointer lock
    if (document.pointerLockElement) {
      document.exitPointerLock();
    }

    // Disable CameraManager controls to prevent interference
    if (this.cameraManager && this.cameraManager.controls) {
      this.cameraManager.controls.enabled = false;
      console.log('[DevTools] CameraManager controls disabled');
    } else {
      console.warn('[DevTools] CameraManager not available!', {
        cameraManager: !!this.cameraManager,
        controls: this.cameraManager?.controls
      });
    }

    // Save current camera state
    this._savedCameraState = {
      position: this.camera.position.clone(),
      quaternion: this.camera.quaternion.clone(),
      target: new THREE.Vector3(0, 0, -1).applyQuaternion(this.camera.quaternion).add(this.camera.position),
    };

    // Use pointer lock-free camera controls similar to freecam
    // Right-click drag to rotate, scroll to move forward/backward, WASD to move
    this._isRightMouseDown = false;
    this._lastMouseX = 0;
    this._lastMouseY = 0;
    this._euler = new THREE.Euler(0, 0, 0, 'YXZ');
    this._euler.setFromQuaternion(this.camera.quaternion);

    // Add pointer handlers for right-click rotation (already bound in constructor)
    // Use pointer events instead of mouse events for better capture support
    this.renderer.domElement.addEventListener('pointerdown', this._handleMouseDown);
    this.renderer.domElement.addEventListener('pointerup', this._handleMouseUp);
    this.renderer.domElement.addEventListener('pointermove', this._handleMouseMoveRotate);
    this.renderer.domElement.addEventListener('wheel', this._handleWheel);

    // Add keyboard listeners
    document.addEventListener('keydown', this._handleKeyDown);
    document.addEventListener('keyup', this._handleKeyUp);

    // Prevent context menu on right click (for orbit rotation)
    this.renderer.domElement.addEventListener('contextmenu', this._preventContextMenu);

    // Add click listener for selection (always active in dev mode)
    this.renderer.domElement.addEventListener('click', this.handlePointerDown);
    this.renderer.domElement.addEventListener('mousemove', this.handleMouseMove);

    // Add focus-related listeners to reset state when losing focus (like freecam)
    window.addEventListener('blur', this._handleWindowBlur);
    document.addEventListener('visibilitychange', this._handleVisibilityChange);
    document.addEventListener('pointerlockchange', this._handlePointerLockChange);

    this._devModeActive = true;
    this.selectionMode = 'none'; // Selection mode starts disabled, activated via enableMeshSelection
    this.renderer.domElement.style.cursor = 'default';

    if (this.onDevModeChange) {
      this.onDevModeChange(true);
    }
  }

  /**
   * Exit DevTools camera mode
   * - Restores previous camera state (optional)
   * - Removes all event listeners
   */
  exitDevMode(restoreCamera = false) {
    if (!this._devModeActive) return;

    console.log('[DevTools] Exiting dev mode');

    // Remove keyboard listeners
    document.removeEventListener('keydown', this._handleKeyDown);
    document.removeEventListener('keyup', this._handleKeyUp);

    // Remove mouse/pointer listeners
    this.renderer.domElement.removeEventListener('contextmenu', this._preventContextMenu);
    this.renderer.domElement.removeEventListener('click', this.handlePointerDown);
    this.renderer.domElement.removeEventListener('mousemove', this.handleMouseMove);
    this.renderer.domElement.removeEventListener('pointerdown', this._handleMouseDown);
    this.renderer.domElement.removeEventListener('pointerup', this._handleMouseUp);
    this.renderer.domElement.removeEventListener('pointermove', this._handleMouseMoveRotate);
    this.renderer.domElement.removeEventListener('wheel', this._handleWheel);

    // Remove focus-related listeners
    window.removeEventListener('blur', this._handleWindowBlur);
    document.removeEventListener('visibilitychange', this._handleVisibilityChange);
    document.removeEventListener('pointerlockchange', this._handlePointerLockChange);

    // Release pointer lock if active
    if (document.pointerLockElement === this.renderer.domElement) {
      document.exitPointerLock?.();
    }

    // Optionally restore camera
    if (restoreCamera && this._savedCameraState) {
      this.camera.position.copy(this._savedCameraState.position);
      this.camera.quaternion.copy(this._savedCameraState.quaternion);
    }

    // Re-enable CameraManager controls
    if (this.cameraManager && this.cameraManager.controls) {
      this.cameraManager.controls.enabled = true;
    }

    // Reset state
    this._devModeActive = false;
    this._isRightMouseDown = false;
    this.selectionMode = 'none';
    this._moveState = { forward: false, backward: false, left: false, right: false, up: false, down: false };
    this.renderer.domElement.style.cursor = 'auto';
    this.clearHoverHighlight();

    if (this.onDevModeChange) {
      this.onDevModeChange(false);
    }
  }

  /**
   * Handle keyboard input for dev camera movement
   */
  _handleKeyDown(event) {
    // Ignore if typing in input
    if (event.target.tagName === 'INPUT' || event.target.tagName === 'TEXTAREA') return;

    switch (event.code) {
      case 'ArrowUp':
      case 'KeyW':
        this._moveState.forward = true;
        event.preventDefault();
        break;
      case 'ArrowDown':
      case 'KeyS':
        this._moveState.backward = true;
        event.preventDefault();
        break;
      case 'ArrowLeft':
      case 'KeyA':
        this._moveState.left = true;
        event.preventDefault();
        break;
      case 'ArrowRight':
      case 'KeyD':
        this._moveState.right = true;
        event.preventDefault();
        break;
      case 'Space':
        this._moveState.up = true;
        event.preventDefault();
        break;
      case 'ShiftLeft':
      case 'ShiftRight':
        this._moveState.down = true;
        event.preventDefault();
        break;
    }
  }

  _handleKeyUp(event) {
    switch (event.code) {
      case 'ArrowUp':
      case 'KeyW':
        this._moveState.forward = false;
        break;
      case 'ArrowDown':
      case 'KeyS':
        this._moveState.backward = false;
        break;
      case 'ArrowLeft':
      case 'KeyA':
        this._moveState.left = false;
        break;
      case 'ArrowRight':
      case 'KeyD':
        this._moveState.right = false;
        break;
      case 'Space':
        this._moveState.up = false;
        break;
      case 'ShiftLeft':
      case 'ShiftRight':
        this._moveState.down = false;
        break;
    }
  }

  /**
   * Prevent context menu when right-clicking on the canvas
   */
  _preventContextMenu(event) {
    event.preventDefault();
    return false;
  }

  /**
   * Handle mouse down for right-click rotation with pointer lock
   */
  _handleMouseDown(event) {
    if (event.button === 2) { // Right click
      event.preventDefault();
      event.stopPropagation();
      this._isRightMouseDown = true;
      // Request pointer lock for smooth camera rotation (like freecam)
      this.renderer.domElement.requestPointerLock?.();
    }
  }

  /**
   * Handle mouse up - release pointer lock and reset cursor
   */
  _handleMouseUp(event) {
    if (event.button === 2) {
      this._isRightMouseDown = false;
      // Release pointer lock
      if (document.pointerLockElement === this.renderer.domElement) {
        document.exitPointerLock?.();
      }
      this.renderer.domElement.style.cursor = this.selectionMode === 'mesh' ? 'crosshair' : 'default';
    }
  }

  /**
   * Handle window blur - reset all input state
   */
  _handleWindowBlur() {
    if (!this._devModeActive) return;

    // Reset right mouse state
    this._isRightMouseDown = false;

    // Release pointer lock if active
    if (document.pointerLockElement === this.renderer.domElement) {
      document.exitPointerLock?.();
    }

    // Reset all keyboard keys to prevent stuck movement
    this._moveState = { forward: false, backward: false, left: false, right: false, up: false, down: false };

    // Reset cursor
    this.renderer.domElement.style.cursor = this.selectionMode === 'mesh' ? 'crosshair' : 'default';
  }

  /**
   * Handle visibility change - reset state when tab is hidden
   */
  _handleVisibilityChange() {
    if (!this._devModeActive) return;

    if (document.hidden) {
      // Reset right mouse state
      this._isRightMouseDown = false;

      // Release pointer lock if active
      if (document.pointerLockElement === this.renderer.domElement) {
        document.exitPointerLock?.();
      }

      // Reset all keyboard keys to prevent stuck movement
      this._moveState = { forward: false, backward: false, left: false, right: false, up: false, down: false };
    }
  }

  /**
   * Handle pointer lock change - reset drag state if lock was released externally
   */
  _handlePointerLockChange() {
    if (!this._devModeActive) return;

    // If pointer lock was released externally (e.g., Escape key), reset drag state
    if (document.pointerLockElement !== this.renderer.domElement) {
      this._isRightMouseDown = false;
      this.renderer.domElement.style.cursor = this.selectionMode === 'mesh' ? 'crosshair' : 'default';
    }
  }

  /**
   * Handle mouse move for camera rotation (right-click drag with pointer lock)
   * Uses movementX/Y from pointer lock for smooth, unconstrained rotation
   */
  _handleMouseMoveRotate(event) {
    if (!this._devModeActive) {
      return;
    }

    // Only rotate if right mouse is held (pointer lock active)
    if (!this._isRightMouseDown) {
      return;
    }

    // Use movementX/Y when pointer is locked for smooth rotation
    // These provide delta values directly, no need to track last position
    const deltaX = event.movementX || 0;
    const deltaY = event.movementY || 0;

    // Skip if no movement
    if (deltaX === 0 && deltaY === 0) {
      return;
    }

    // Rotation sensitivity
    const sensitivity = 0.002;

    // Update euler angles (like freecam)
    this._euler.y -= deltaX * sensitivity;
    this._euler.x -= deltaY * sensitivity;

    // Clamp vertical rotation to avoid flipping
    this._euler.x = Math.max(-Math.PI / 2 + 0.01, Math.min(Math.PI / 2 - 0.01, this._euler.x));

    // Apply to camera
    this.camera.quaternion.setFromEuler(this._euler);
  }

  /**
   * Handle mouse wheel for forward/backward movement or mesh cycling
   */
  _handleWheel(event) {
    // In mesh selection mode with hits: cycle through meshes
    if (this.selectionMode === 'mesh' && this.raycastHits.length > 1) {
      event.preventDefault();
      event.stopPropagation();

      // Scroll down = next mesh (deeper), scroll up = previous mesh (closer)
      if (event.deltaY > 0) {
        this.currentHitIndex = (this.currentHitIndex + 1) % this.raycastHits.length;
      } else {
        this.currentHitIndex = (this.currentHitIndex - 1 + this.raycastHits.length) % this.raycastHits.length;
      }

      this._selectHitAtIndex(this.currentHitIndex);
      console.log('[DevTools] Cycling to mesh', this.currentHitIndex + 1, '/', this.raycastHits.length);
      return;
    }

    // Dev mode camera movement
    if (!this._devModeActive) return;

    event.preventDefault();

    // Move in the direction the camera is looking
    const forward = new THREE.Vector3();
    this.camera.getWorldDirection(forward);

    // Scroll up = move forward, scroll down = move backward
    const scrollSpeed = 200; // Unreal units per scroll tick
    const delta = event.deltaY > 0 ? -scrollSpeed : scrollSpeed;

    this.camera.position.addScaledVector(forward, delta);
  }

  /**
   * Select a mesh/placed-mesh at a specific index in raycastHits
   */
  _selectHitAtIndex(index) {
    if (index < 0 || index >= this.raycastHits.length) return;

    const hit = this.raycastHits[index];

    // Clear previous hover highlight
    this.clearHoverHighlight();

    if (hit.type === 'placed') {
      console.log('[DevTools] Selected placed mesh:', hit.id, '(', index + 1, '/', this.raycastHits.length, ')');
      this.selectPlacedMesh(hit.id);
    } else {
      console.log('[DevTools] Selected mesh:', hit.object.name || hit.object.uuid, '(', index + 1, '/', this.raycastHits.length, ')');
      this.selectMesh(hit.object);
    }

    // Apply highlight to current selection
    this.applyHoverHighlight(hit.object);

    // Update tooltip with cycling info
    this._updateCyclingTooltip(hit.object, index);
  }

  /**
   * Update tooltip to show current mesh and cycling position
   */
  _updateCyclingTooltip(mesh, index) {
    if (!this.tooltip) {
      this.tooltip = document.createElement('div');
      this.tooltip.style.cssText = `
        position: fixed;
        background: rgba(0, 0, 0, 0.9);
        color: #fff;
        padding: 8px 12px;
        border-radius: 4px;
        font-size: 12px;
        pointer-events: none;
        z-index: 10000;
        max-width: 300px;
      `;
      document.body.appendChild(this.tooltip);
    }

    const meshName = mesh.name || `Mesh_${mesh.uuid.slice(0, 8)}`;
    const materialName = mesh.material?.name || 'unnamed';
    const total = this.raycastHits.length;

    this.tooltip.innerHTML = `
      <div style="color: #4ade80; font-weight: bold; margin-bottom: 4px;">
        [${index + 1}/${total}] - Scroll to cycle
      </div>
      <b>Mesh:</b> ${meshName}<br>
      <b>Material:</b> ${materialName}
    `;
    this.tooltip.style.display = 'block';

    // Position tooltip in corner since we don't have mouse position
    this.tooltip.style.left = '50%';
    this.tooltip.style.top = '100px';
    this.tooltip.style.transform = 'translateX(-50%)';
  }

  /**
   * Update dev mode camera - call this every frame from GameEngine
   */
  update(delta) {
    if (!this._devModeActive) return;

    // Apply keyboard movement (WASD + Space/Shift for up/down)
    const speed = this._moveSpeed * delta;

    // Get camera direction vectors
    const forward = new THREE.Vector3();
    this.camera.getWorldDirection(forward);
    forward.y = 0;
    forward.normalize();

    const right = new THREE.Vector3();
    right.crossVectors(forward, new THREE.Vector3(0, 1, 0)).normalize();

    const movement = new THREE.Vector3();

    if (this._moveState.forward) movement.add(forward.clone().multiplyScalar(speed));
    if (this._moveState.backward) movement.add(forward.clone().multiplyScalar(-speed));
    if (this._moveState.right) movement.add(right.clone().multiplyScalar(speed));
    if (this._moveState.left) movement.add(right.clone().multiplyScalar(-speed));
    if (this._moveState.up) movement.y += speed;
    if (this._moveState.down) movement.y -= speed;

    if (movement.length() > 0) {
      this.camera.position.add(movement);
    }

    // Update all direction helpers every frame (so they stay in sync with light positions)
    this.customLights.forEach((data) => {
      if (data.directionHelpers) {
        this._updateLightDirectionHelpers(data.directionHelpers);
      }
    });
  }

  /**
   * Lazy initialization of TransformControls to avoid immediate freeze
   */
  _ensureTransformControls() {
    if (this._transformControlsInitialized) return;

    this.transformControls = new TransformControls(this.camera, this.renderer.domElement);
    this.transformControls.setMode('translate');
    this.transformControls.setSize(0.8);

    // In Three.js r170+, TransformControls is no longer an Object3D
    // We need to add the helper (gizmo) to the scene instead
    const helper = this.transformControls.getHelper();
    helper.visible = false;
    // Disable shadows on the gizmo
    helper.traverse((child) => {
      child.castShadow = false;
      child.receiveShadow = false;
    });
    this.scene.add(helper);
    this._transformControlsHelper = helper;

    // Throttled update on change
    this.transformControls.addEventListener('change', () => {
      if (this.selectedLight) {
        const lightData = this.customLights.get(this.selectedLight);
        if (lightData) {
          // Update the main helper (SpotLightHelper, etc.)
          if (lightData.helper && lightData.helper.update) {
            lightData.helper.update();
          }
          // Update direction helpers (source marker, target marker, connecting line)
          if (lightData.directionHelpers) {
            this._updateLightDirectionHelpers(lightData.directionHelpers);
          }
        }
        // Throttle updates to prevent freeze
        const now = Date.now();
        if (now - this._lastLightsUpdate > this._lightsUpdateThrottle) {
          this._lastLightsUpdate = now;
          this.notifyLightsUpdate();
        }
      }
    });

    // Track scale at drag start for uniform scaling
    this._scaleAtDragStart = new THREE.Vector3(1, 1, 1);
    this._isDraggingScale = false;

    // Track when dragging starts/ends
    this.transformControls.addEventListener('dragging-changed', (event) => {
      if (event.value) {
        // Drag started
        if (this.selectedPlacedMesh && this._meshTransformMode === 'scale') {
          const meshData = this.placedMeshes.get(this.selectedPlacedMesh);
          if (meshData && meshData.object) {
            this._scaleAtDragStart.copy(meshData.object.scale);
            this._isDraggingScale = true;
          }
        }
      } else {
        // Drag ended
        this._isDraggingScale = false;

        if (this.selectedLight) {
          const lightData = this.customLights.get(this.selectedLight);
          if (lightData && lightData.directionHelpers) {
            this._updateLightDirectionHelpers(lightData.directionHelpers);
          }
          this.notifyLightsUpdate();
        }

        // Update UI when drag ends for placed meshes
        if (this.selectedPlacedMesh) {
          this.notifyPlacedMeshesUpdate();
        }
      }
    });

    // Handle uniform scaling for placed meshes during drag
    this.transformControls.addEventListener('objectChange', () => {
      if (this.selectedPlacedMesh && this._meshTransformMode === 'scale' && this._isDraggingScale) {
        const meshData = this.placedMeshes.get(this.selectedPlacedMesh);
        if (meshData && meshData.object) {
          const scale = meshData.object.scale;

          // Calculate how much each axis changed from the drag start
          const deltaX = Math.abs(scale.x - this._scaleAtDragStart.x);
          const deltaY = Math.abs(scale.y - this._scaleAtDragStart.y);
          const deltaZ = Math.abs(scale.z - this._scaleAtDragStart.z);

          // Find which axis the user is dragging (biggest change from start)
          let uniformScale;
          if (deltaX >= deltaY && deltaX >= deltaZ) {
            uniformScale = scale.x;
          } else if (deltaY >= deltaX && deltaY >= deltaZ) {
            uniformScale = scale.y;
          } else {
            uniformScale = scale.z;
          }

          // Apply uniform scale to all axes
          scale.set(uniformScale, uniformScale, uniformScale);
          // Don't call notifyPlacedMeshesUpdate here - wait for drag end
        }
      }
    });

    this._transformControlsInitialized = true;
  }

  /**
   * Create a simple visual helper for AmbientLight (which has no built-in helper)
   */
  _createAmbientLightHelper(light) {
    // Create a small sphere at scene center to represent ambient light
    const geometry = new THREE.SphereGeometry(0.3, 16, 16);
    const material = new THREE.MeshBasicMaterial({
      color: light.color,
      wireframe: true,
      transparent: true,
      opacity: 0.6,
    });
    const helper = new THREE.Mesh(geometry, material);
    helper.position.set(0, 15, 0); // Position high in the scene

    // Exclude from shadow casting and receiving
    helper.castShadow = false;
    helper.receiveShadow = false;

    // Add update method for consistency with other helpers
    helper.update = () => {
      material.color.copy(light.color);
    };

    return helper;
  }

  /**
   * Configure helper to not cast/receive shadows
   * Note: We don't change layers anymore as it can cause visibility issues
   */
  _configureHelper(helper) {
    if (!helper) return;

    // Recursively disable shadows on helper and all children
    helper.traverse((child) => {
      child.castShadow = false;
      child.receiveShadow = false;
      // Make sure frustumCulled is false so helpers don't disappear
      child.frustumCulled = false;
    });

    helper.castShadow = false;
    helper.receiveShadow = false;
    helper.frustumCulled = false;
  }

  /**
   * Create visual helpers for light source and target (for spot/directional lights)
   * Returns an object with sourceMarker, targetMarker, and connectingLine
   */
  _createLightDirectionHelpers(light, target, color = 0xffff00) {
    // Source marker (small sphere at light position) - yellow/orange
    const sourceGeometry = new THREE.SphereGeometry(0.4, 8, 8);
    const sourceMaterial = new THREE.MeshBasicMaterial({
      color: 0xffff00, // Yellow for source
      wireframe: true,
      transparent: true,
      opacity: 0.9,
    });
    const sourceMarker = new THREE.Mesh(sourceGeometry, sourceMaterial);
    sourceMarker.castShadow = false;
    sourceMarker.receiveShadow = false;

    // Target marker (small sphere at target position) - orange/cyan based on light type
    const targetGeometry = new THREE.SphereGeometry(0.35, 8, 8);
    const targetMaterial = new THREE.MeshBasicMaterial({
      color: color,
      wireframe: true,
      transparent: true,
      opacity: 0.9,
    });
    const targetMarker = new THREE.Mesh(targetGeometry, targetMaterial);
    targetMarker.castShadow = false;
    targetMarker.receiveShadow = false;

    // Connecting line from source to target
    const lineGeometry = new THREE.BufferGeometry();
    const positions = new Float32Array(6); // 2 points x 3 components
    lineGeometry.setAttribute('position', new THREE.BufferAttribute(positions, 3));

    const lineMaterial = new THREE.LineBasicMaterial({
      color: color,
      transparent: true,
      opacity: 0.7,
      linewidth: 2,
    });
    const connectingLine = new THREE.Line(lineGeometry, lineMaterial);
    connectingLine.castShadow = false;
    connectingLine.receiveShadow = false;

    // Group to hold all helpers
    const group = new THREE.Group();
    group.add(sourceMarker);
    group.add(targetMarker);
    group.add(connectingLine);

    // Store references for updates
    group.userData.sourceMarker = sourceMarker;
    group.userData.targetMarker = targetMarker;
    group.userData.connectingLine = connectingLine;
    group.userData.light = light;
    group.userData.target = target;

    return group;
  }

  /**
   * Update light direction helpers (source marker, target marker, and connecting line)
   */
  _updateLightDirectionHelpers(helperGroup) {
    if (!helperGroup || !helperGroup.userData) return;

    const { sourceMarker, targetMarker, connectingLine, light, target } = helperGroup.userData;

    if (!light || !target) return;

    // Get world positions
    const lightWorldPos = new THREE.Vector3();
    const targetWorldPos = new THREE.Vector3();

    light.getWorldPosition(lightWorldPos);
    target.getWorldPosition(targetWorldPos);

    // Update source marker position (in world coords, but group is at origin so local = world)
    if (sourceMarker) {
      sourceMarker.position.copy(lightWorldPos);
    }

    // Update target marker position
    if (targetMarker) {
      targetMarker.position.copy(targetWorldPos);
    }

    // Update connecting line
    if (connectingLine) {
      const positions = connectingLine.geometry.attributes.position.array;
      positions[0] = lightWorldPos.x;
      positions[1] = lightWorldPos.y;
      positions[2] = lightWorldPos.z;
      positions[3] = targetWorldPos.x;
      positions[4] = targetWorldPos.y;
      positions[5] = targetWorldPos.z;
      connectingLine.geometry.attributes.position.needsUpdate = true;
    }
  }

  /**
   * Get the hierarchy path of a mesh (parent names chain)
   */
  _getMeshHierarchyPath(mesh) {
    const parts = [];
    let current = mesh;

    while (current) {
      if (current.name && current.name !== '') {
        parts.unshift(current.name);
      }
      current = current.parent;
      // Stop at scene level
      if (current === this.scene || !current) break;
    }

    return parts.length > 0 ? parts.join(' > ') : null;
  }

  /**
   * Scan the scene for all meshes with materials
   */
  scanMaterials() {
    this.meshMaterials.clear();

    this.scene.traverse((object) => {
      if (object.isMesh && object.material) {
        const materials = Array.isArray(object.material) ? object.material : [object.material];

        materials.forEach((material, index) => {
          if (material.isMeshStandardMaterial || material.isMeshPhysicalMaterial) {
            const key = `${object.name || object.uuid}_${index}`;
            const meshPath = this._getMeshHierarchyPath(object);

            this.meshMaterials.set(key, {
              mesh: object,
              material: material,
              name: object.name || `Mesh_${object.uuid.slice(0, 8)}`,
              meshPath: meshPath,
              materialName: material.name || null,
              materialIndex: index,
              originalRoughness: material.roughness,
              originalMetalness: material.metalness,
              originalColor: '#' + material.color.getHexString(),
              originalEmissive: '#' + material.emissive.getHexString(),
              originalEmissiveIntensity: material.emissiveIntensity,
              originalOpacity: material.opacity,
              originalEnvMapIntensity: material.envMapIntensity,
            });
          }
        });
      }
    });

    this.notifyMaterialsUpdate();
    return this.getMaterialsList();
  }

  /**
   * Get a list of all materials for UI display
   */
  getMaterialsList() {
    const list = [];
    this.meshMaterials.forEach((info, key) => {
      list.push({
        id: key,
        name: info.name,
        meshPath: info.meshPath,
        materialName: info.materialName,
        materialIndex: info.materialIndex,
        roughness: info.material.roughness,
        metalness: info.material.metalness,
        color: '#' + info.material.color.getHexString(),
        emissive: '#' + info.material.emissive.getHexString(),
        emissiveIntensity: info.material.emissiveIntensity,
        opacity: info.material.opacity,
        transparent: info.material.transparent,
        envMapIntensity: info.material.envMapIntensity,
      });
    });
    return list;
  }

  /**
   * Update material properties
   */
  updateMaterial(materialId, property, value) {
    const info = this.meshMaterials.get(materialId);
    if (!info) return;

    switch (property) {
      case 'roughness':
        info.material.roughness = value;
        break;
      case 'metalness':
        info.material.metalness = value;
        break;
      case 'color':
        info.material.color.set(value);
        break;
      case 'emissive':
        info.material.emissive.set(value);
        break;
      case 'emissiveIntensity':
        info.material.emissiveIntensity = value;
        break;
      case 'opacity':
        info.material.opacity = value;
        // Auto-enable transparency if opacity < 1
        if (value < 1 && !info.material.transparent) {
          info.material.transparent = true;
        }
        break;
      case 'transparent':
        info.material.transparent = value;
        break;
      case 'envMapIntensity':
        info.material.envMapIntensity = value;
        break;
    }

    info.material.needsUpdate = true;
    this.notifyMaterialsUpdate();
  }

  /**
   * Reset material to original values
   */
  resetMaterial(materialId) {
    const info = this.meshMaterials.get(materialId);
    if (!info) return;

    info.material.roughness = info.originalRoughness;
    info.material.metalness = info.originalMetalness;
    info.material.color.set(info.originalColor);
    info.material.emissive.set(info.originalEmissive);
    info.material.emissiveIntensity = info.originalEmissiveIntensity;
    info.material.opacity = info.originalOpacity;
    info.material.envMapIntensity = info.originalEnvMapIntensity;
    info.material.needsUpdate = true;
    this.notifyMaterialsUpdate();
  }

  /**
   * Reset all materials
   */
  resetAllMaterials() {
    this.meshMaterials.forEach((info) => {
      info.material.roughness = info.originalRoughness;
      info.material.metalness = info.originalMetalness;
      info.material.color.set(info.originalColor);
      info.material.emissive.set(info.originalEmissive);
      info.material.emissiveIntensity = info.originalEmissiveIntensity;
      info.material.opacity = info.originalOpacity;
      info.material.envMapIntensity = info.originalEnvMapIntensity;
      info.material.needsUpdate = true;
    });
    this.notifyMaterialsUpdate();
  }

  /**
   * Add a new light to the scene
   */
  addLight(type = 'point', options = {}) {
    const id = `light_${++this.lightIdCounter}`;
    let light, helper, target;

    const color = options.color || 0xffffff;
    const intensity = options.intensity || 1;
    const position = options.position || new THREE.Vector3(0, 10, 0);

    switch (type) {
      case 'ambient':
        light = new THREE.AmbientLight(color, intensity);
        // AmbientLight has no position, no helper needed but we create a simple sphere for visual
        helper = this._createAmbientLightHelper(light);
        break;

      case 'hemisphere':
        light = new THREE.HemisphereLight(
          options.skyColor || color,
          options.groundColor || 0x444444,
          intensity
        );
        light.position.copy(position);
        helper = new THREE.HemisphereLightHelper(light, 1);
        break;

      case 'point':
        // Higher default distance for point lights (500 instead of 50)
        light = new THREE.PointLight(color, intensity, options.distance || 5000);
        // Disable physical light decay for more intuitive intensity control
        light.decay = 1;
        light.position.copy(position);
        light.castShadow = true;
        // Configure shadow map for better quality
        light.shadow.mapSize.width = 2048;
        light.shadow.mapSize.height = 2048;
        light.shadow.bias = -0.0001;
        light.shadow.normalBias = 0.02;
        light.shadow.camera.near = 10;
        light.shadow.camera.far = 15000;
        helper = new THREE.PointLightHelper(light, 0.5);
        break;

      case 'spot':
        light = new THREE.SpotLight(color, intensity);
        light.position.copy(position);
        light.angle = options.angle || Math.PI / 6;
        light.penumbra = options.penumbra || 0.2;
        // Higher default distance for spot lights (500 instead of 50)
        light.distance = options.distance || 5000;
        // Disable physical light decay for more intuitive intensity control
        light.decay = 1;
        light.castShadow = true;
        // Configure shadow map for better quality
        light.shadow.mapSize.width = 2048;
        light.shadow.mapSize.height = 2048;
        light.shadow.bias = -0.0001;
        light.shadow.normalBias = 0.02;
        light.shadow.camera.near = 10;
        light.shadow.camera.far = 15000;
        // Create a target for the spotlight that we can position
        target = new THREE.Object3D();
        target.position.set(position.x, 0, position.z); // Point down by default
        light.target = target;
        this.scene.add(target);
        helper = new THREE.SpotLightHelper(light);
        break;

      case 'directional':
        light = new THREE.DirectionalLight(color, intensity);
        light.position.copy(position);
        light.castShadow = true;
        // Configure shadow map for better quality
        light.shadow.mapSize.width = 2048;
        light.shadow.mapSize.height = 2048;
        light.shadow.camera.near = 100;
        light.shadow.camera.far = 15000;
        light.shadow.camera.left = -10000;
        light.shadow.camera.right = 10000;
        light.shadow.camera.top = 10000;
        light.shadow.camera.bottom = -10000;
        light.shadow.bias = -0.0001;
        light.shadow.normalBias = 0.02;
        light.shadow.camera.near = 10;
        light.shadow.camera.far = 15000;
        // Create a target for the directional light that we can position
        target = new THREE.Object3D();
        target.position.set(0, 0, 0); // Point to center by default
        light.target = target;
        this.scene.add(target);
        helper = new THREE.DirectionalLightHelper(light, 2);
        break;

      default:
        console.warn(`Unknown light type: ${type}`);
        return null;
    }

    light.name = id;
    this.scene.add(light);

    // Configure helper to not cast shadows
    this._configureHelper(helper);
    this.scene.add(helper);

    // Create direction helpers for spot and directional lights (source + target markers + connecting line)
    let directionHelpers = null;
    if (target && (type === 'spot' || type === 'directional')) {
      const helperColor = type === 'spot' ? 0xffa500 : 0x00ffff;
      directionHelpers = this._createLightDirectionHelpers(light, target, helperColor);
      this._configureHelper(directionHelpers);
      this.scene.add(directionHelpers);
      // Initial update
      this._updateLightDirectionHelpers(directionHelpers);
    }

    this.customLights.set(id, {
      light,
      helper,
      target, // Store target for cleanup
      directionHelpers, // Store direction helpers for cleanup and updates
      type,
    });

    this.notifyLightsUpdate();
    return id;
  }

  /**
   * Remove a light from the scene
   */
  removeLight(lightId) {
    const lightData = this.customLights.get(lightId);
    if (!lightData) return;

    // Deselect if this light is selected
    if (this.selectedLight === lightId) {
      this.deselectLight();
    }

    this.scene.remove(lightData.light);
    this.scene.remove(lightData.helper);

    // Remove target if exists (for spot/directional lights)
    if (lightData.target) {
      this.scene.remove(lightData.target);
    }

    // Remove direction helpers if exists
    if (lightData.directionHelpers) {
      this.scene.remove(lightData.directionHelpers);
    }

    if (lightData.light.dispose) lightData.light.dispose();
    if (lightData.helper.dispose) lightData.helper.dispose();

    this.customLights.delete(lightId);
    this.notifyLightsUpdate();
  }

  /**
   * Update light properties
   */
  updateLight(lightId, property, value) {
    const lightData = this.customLights.get(lightId);
    if (!lightData) return;

    const light = lightData.light;

    switch (property) {
      case 'intensity':
        light.intensity = value;
        break;
      case 'color':
        light.color.set(value);
        break;
      case 'skyColor':
        if (light.isHemisphereLight) {
          light.color.set(value);
        }
        break;
      case 'groundColor':
        if (light.isHemisphereLight) {
          light.groundColor.set(value);
        }
        break;
      case 'distance':
        if (light.isPointLight || light.isSpotLight) {
          light.distance = value;
        }
        break;
      case 'angle':
        if (light.isSpotLight) {
          light.angle = value;
        }
        break;
      case 'penumbra':
        if (light.isSpotLight) {
          light.penumbra = value;
        }
        break;
      case 'position':
        if (light.position) {
          light.position.copy(value);
        }
        break;
      // Shadow properties (for directional and spot lights)
      case 'castShadow':
        if (light.isDirectionalLight || light.isSpotLight) {
          light.castShadow = value;
        }
        break;
      case 'shadowMapSize':
        if (light.shadow) {
          light.shadow.mapSize.width = value;
          light.shadow.mapSize.height = value;
          // Need to dispose and recreate shadow map
          light.shadow.map?.dispose();
          light.shadow.map = null;
        }
        break;
      case 'shadowBias':
        if (light.shadow) {
          light.shadow.bias = value;
        }
        break;
      case 'shadowNormalBias':
        if (light.shadow) {
          light.shadow.normalBias = value;
        }
        break;
      case 'shadowCameraNear':
        if (light.shadow) {
          light.shadow.camera.near = value;
          light.shadow.camera.updateProjectionMatrix();
        }
        break;
      case 'shadowCameraFar':
        if (light.shadow) {
          light.shadow.camera.far = value;
          light.shadow.camera.updateProjectionMatrix();
        }
        break;
      case 'shadowCameraLeft':
        if (light.shadow && light.isDirectionalLight) {
          light.shadow.camera.left = value;
          light.shadow.camera.updateProjectionMatrix();
        }
        break;
      case 'shadowCameraRight':
        if (light.shadow && light.isDirectionalLight) {
          light.shadow.camera.right = value;
          light.shadow.camera.updateProjectionMatrix();
        }
        break;
      case 'shadowCameraTop':
        if (light.shadow && light.isDirectionalLight) {
          light.shadow.camera.top = value;
          light.shadow.camera.updateProjectionMatrix();
        }
        break;
      case 'shadowCameraBottom':
        if (light.shadow && light.isDirectionalLight) {
          light.shadow.camera.bottom = value;
          light.shadow.camera.updateProjectionMatrix();
        }
        break;
    }

    // Update helper
    if (lightData.helper && lightData.helper.update) {
      lightData.helper.update();
    }

    this.notifyLightsUpdate();
  }

  /**
   * Get list of all custom lights
   */
  getLightsList() {
    const list = [];
    this.customLights.forEach((data, id) => {
      const light = data.light;
      const info = {
        id,
        type: data.type,
        intensity: light.intensity,
        color: '#' + light.color.getHexString(),
        position: light.position ? {
          x: light.position.x,
          y: light.position.y,
          z: light.position.z,
        } : null,
        distance: light.distance ?? null,
        angle: light.angle ?? null,
        penumbra: light.penumbra ?? null,
        // HemisphereLight specific
        groundColor: light.groundColor ? '#' + light.groundColor.getHexString() : null,
        // Target position for spot/directional lights (for direction control)
        target: data.target ? {
          x: data.target.position.x,
          y: data.target.position.y,
          z: data.target.position.z,
        } : null,
        hasTarget: !!data.target,
        // Shadow properties (for directional and spot lights)
        shadow: (light.isDirectionalLight || light.isSpotLight) ? {
          enabled: light.castShadow ?? false,
          mapSize: light.shadow?.mapSize.width ?? 2048,
          bias: light.shadow?.bias ?? -0.0001,
          normalBias: light.shadow?.normalBias ?? 0,
          cameraNear: light.shadow?.camera.near ?? 100,
          cameraFar: light.shadow?.camera.far ?? 10000,
          // Orthographic camera bounds (directional only)
          cameraLeft: light.isDirectionalLight ? (light.shadow?.camera.left ?? -5000) : null,
          cameraRight: light.isDirectionalLight ? (light.shadow?.camera.right ?? 5000) : null,
          cameraTop: light.isDirectionalLight ? (light.shadow?.camera.top ?? 5000) : null,
          cameraBottom: light.isDirectionalLight ? (light.shadow?.camera.bottom ?? -5000) : null,
        } : null,
      };
      list.push(info);
    });
    return list;
  }

  /**
   * Select a light for positioning
   * @param {string} lightId - The light ID to select
   * @param {string} mode - The transform mode: 'translate' or 'rotate' (default: 'translate')
   */
  selectLight(lightId, mode = 'translate') {
    const lightData = this.customLights.get(lightId);
    if (!lightData) return;

    // Deselect any placed mesh first (avoid gizmo conflicts)
    if (this.selectedPlacedMesh) {
      this.deselectPlacedMesh();
    }

    this.selectedLight = lightId;
    this._currentTransformMode = mode;

    // Only attach transform controls if light has a position (not AmbientLight)
    if (lightData.light.position && lightData.type !== 'ambient') {
      this._ensureTransformControls();

      // For spot/directional lights, we can control the target for rotation-like behavior
      // In 'rotate' mode, attach to the target instead of the light
      if (mode === 'rotate' && lightData.target) {
        this.transformControls.attach(lightData.target);
        this.transformControls.setMode('translate'); // Move target = rotate light direction
      } else {
        this.transformControls.attach(lightData.light);
        this.transformControls.setMode('translate');
      }
      if (this._transformControlsHelper) {
        this._transformControlsHelper.visible = true;
      }
    }

    // Show the helper more prominently
    if (lightData.helper) {
      lightData.helper.visible = true;
    }
    // Show direction helpers too
    if (lightData.directionHelpers) {
      lightData.directionHelpers.visible = true;
    }

    if (this.onSelectionChange) {
      this.onSelectionChange({ type: 'light', id: lightId, mode });
    }
  }

  /**
   * Set the transform mode for the currently selected light
   * @param {string} mode - 'translate' or 'rotate'
   */
  setLightTransformMode(mode) {
    if (!this.selectedLight) return;

    const lightData = this.customLights.get(this.selectedLight);
    if (!lightData) return;

    this._currentTransformMode = mode;

    if (this.transformControls) {
      if (mode === 'rotate' && lightData.target) {
        // Attach to target for "rotation" (moving target changes light direction)
        this.transformControls.attach(lightData.target);
        this.transformControls.setMode('translate');
      } else {
        // Attach to light for translation
        this.transformControls.attach(lightData.light);
        this.transformControls.setMode('translate');
      }
    }

    if (this.onSelectionChange) {
      this.onSelectionChange({ type: 'light', id: this.selectedLight, mode });
    }
  }

  /**
   * Get the current transform mode
   */
  getLightTransformMode() {
    return this._currentTransformMode || 'translate';
  }

  /**
   * Deselect current light
   */
  deselectLight() {
    if (this.selectedLight) {
      const lightData = this.customLights.get(this.selectedLight);
      if (lightData) {
        if (lightData.helper) {
          lightData.helper.visible = true;
        }
        if (lightData.directionHelpers) {
          lightData.directionHelpers.visible = true;
        }
      }
    }

    this.selectedLight = null;
    this._currentTransformMode = 'translate';

    if (this.transformControls) {
      this.transformControls.detach();
      if (this._transformControlsHelper) {
        this._transformControlsHelper.visible = false;
      }
    }

    if (this.onSelectionChange) {
      this.onSelectionChange({ type: null, id: null });
    }
  }

  /**
   * Enable mesh selection mode
   */
  enableMeshSelection() {
    this.selectionMode = 'mesh';
    this.renderer.domElement.style.cursor = 'crosshair';
    console.log('[DevTools] Mesh selection mode enabled');
  }

  /**
   * Disable mesh selection mode
   */
  disableMeshSelection() {
    this.selectionMode = 'none';
    this.renderer.domElement.style.cursor = 'default';
    this.clearHoverHighlight();
    // Clear raycast hits when exiting selection mode
    this.raycastHits = [];
    this.currentHitIndex = 0;
    this.hideTooltip();
    console.log('[DevTools] Mesh selection mode disabled');
  }

  /**
   * Enable light positioning mode (click to place light)
   */
  enableLightPositioning(lightId) {
    this.selectionMode = 'light-position';
    this.positioningLightId = lightId;
    console.log('[DevTools] Light positioning enabled - click to place light');
  }

  /**
   * Disable light positioning mode
   */
  disableLightPositioning() {
    // Don't auto-enable mesh selection - let user explicitly enable it
    this.selectionMode = 'none';
    this.renderer.domElement.style.cursor = 'default';
    this.positioningLightId = null;
  }

  /**
   * Check if an object should be excluded from selection (helpers, outlines, etc)
   */
  _isExcludedFromSelection(object) {
    // Exclude our own outline mesh
    if (object === this.outlineMesh) {
      return true;
    }

    // Check if it's one of our registered light helpers
    for (const data of this.customLights.values()) {
      if (data.helper === object || object.parent === data.helper) {
        return true;
      }
      // Check for nested objects in helpers (like sphere geometry in ambient helper)
      let parent = object.parent;
      while (parent) {
        if (parent === data.helper) return true;
        parent = parent.parent;
      }
    }
    // Check common helper types
    if (object.type?.includes('Helper') || object.parent?.type?.includes('Helper')) {
      return true;
    }
    return false;
  }

  /**
   * Check if an object is a light helper (should be excluded from selection)
   * @deprecated Use _isExcludedFromSelection instead
   */
  _isLightHelper(object) {
    return this._isExcludedFromSelection(object);
  }

  /**
   * Handle click events for selection in dev mode
   */
  handlePointerDown(event) {
    // Only handle left click
    if (event.button !== 0) return;

    // Skip if clicking on UI
    if (event.target !== this.renderer.domElement) return;

    this.updateMousePosition(event);
    this.raycaster.setFromCamera(this.mouse, this.camera);

    console.log('[DevTools] Click, mode:', this.selectionMode, 'mouse:', this.mouse);

    if (this.selectionMode === 'mesh') {
      const intersects = this.raycaster.intersectObjects(this.scene.children, true);
      console.log('[DevTools] Intersections found:', intersects.length);

      // Collect all valid meshes along the raycast path
      const validHits = [];
      for (const intersect of intersects) {
        // Skip helpers and transform controls
        if (intersect.object.type === 'Line' ||
            intersect.object.type === 'LineSegments' ||
            intersect.object.type === 'Points' ||
            intersect.object.parent?.type === 'TransformControlsPlane' ||
            intersect.object.name?.startsWith('light_') ||
            this._isLightHelper(intersect.object)) {
          continue;
        }

        // Check if it's a placed mesh first
        const placedMeshId = this._isPlacedMesh(intersect.object);
        if (placedMeshId) {
          validHits.push({ type: 'placed', id: placedMeshId, object: intersect.object });
          continue;
        }

        if (intersect.object.isMesh && intersect.object.material) {
          // Avoid duplicates (same mesh can be hit multiple times)
          if (!validHits.some(h => h.object === intersect.object)) {
            validHits.push({ type: 'mesh', object: intersect.object });
          }
        }
      }

      if (validHits.length > 0) {
        // Store all hits for scroll cycling
        this.raycastHits = validHits;
        this.currentHitIndex = 0;

        // Select the first hit
        this._selectHitAtIndex(0);
        console.log('[DevTools] Found', validHits.length, 'meshes on raycast path. Use mouse wheel to cycle.');
      } else {
        this.raycastHits = [];
        this.currentHitIndex = 0;
        console.log('[DevTools] No valid mesh found in intersections');
      }
    } else if (this.selectionMode === 'light-position') {
      const intersects = this.raycaster.intersectObjects(this.scene.children, true);

      if (intersects.length > 0) {
        const point = intersects[0].point;
        // Place light slightly above the intersection point
        const position = new THREE.Vector3(point.x, point.y + 2, point.z);
        this.updateLight(this.positioningLightId, 'position', position);
        console.log('[DevTools] Light positioned at:', position);
      }

      this.disableLightPositioning();
    }
  }

  /**
   * Handle mouse move for hover highlighting
   */
  handleMouseMove(event) {
    if (this.selectionMode !== 'mesh') return;

    // Skip if pointer is locked
    if (document.pointerLockElement === this.renderer.domElement) return;

    this.updateMousePosition(event);
    this.raycaster.setFromCamera(this.mouse, this.camera);

    const intersects = this.raycaster.intersectObjects(this.scene.children, true);

    let foundMesh = null;
    for (const intersect of intersects) {
      // Skip helpers, transform controls, and light helpers
      if (intersect.object.type === 'Line' ||
          intersect.object.type === 'LineSegments' ||
          intersect.object.type === 'Points' ||
          intersect.object.parent?.type === 'TransformControlsPlane' ||
          intersect.object.name?.startsWith('light_') ||
          this._isLightHelper(intersect.object)) {
        continue;
      }

      if (intersect.object.isMesh && intersect.object.material) {
        foundMesh = intersect.object;
        break;
      }
    }

    if (foundMesh !== this.hoveredMesh) {
      this.clearHoverHighlight();

      if (foundMesh) {
        this.applyHoverHighlight(foundMesh);
        this.showTooltip(event, foundMesh);
      } else {
        this.hideTooltip();
      }

      this.hoveredMesh = foundMesh;
    } else if (foundMesh) {
      // Update tooltip position while hovering same mesh
      this.updateTooltipPosition(event);
    }
  }

  /**
   * Create and show tooltip with mesh and material info
   */
  showTooltip(event, mesh) {
    if (!this.tooltip) {
      this.tooltip = document.createElement('div');
      this.tooltip.style.cssText = `
        position: fixed;
        background: rgba(0, 0, 0, 0.85);
        color: #fff;
        padding: 8px 12px;
        border-radius: 4px;
        font-size: 12px;
        font-family: monospace;
        pointer-events: none;
        z-index: 10000;
        white-space: pre-line;
        border: 1px solid #444;
      `;
      document.body.appendChild(this.tooltip);
    }

    const meshName = mesh.name || `Mesh_${mesh.uuid.slice(0, 8)}`;
    const materialName = mesh.material?.name || 'unnamed';

    this.tooltip.innerHTML = `<b>Mesh:</b> ${meshName}<br><b>Material:</b> ${materialName}`;
    this.tooltip.style.display = 'block';
    this.updateTooltipPosition(event);
  }

  /**
   * Update tooltip position
   */
  updateTooltipPosition(event) {
    if (!this.tooltip) return;
    this.tooltip.style.left = (event.clientX + 15) + 'px';
    this.tooltip.style.top = (event.clientY + 15) + 'px';
  }

  /**
   * Hide tooltip
   */
  hideTooltip() {
    if (this.tooltip) {
      this.tooltip.style.display = 'none';
    }
  }

  /**
   * Update mouse position from event
   */
  updateMousePosition(event) {
    const rect = this.renderer.domElement.getBoundingClientRect();
    this.mouse.x = ((event.clientX - rect.left) / rect.width) * 2 - 1;
    this.mouse.y = -((event.clientY - rect.top) / rect.height) * 2 + 1;
  }

  /**
   * Apply hover highlight to mesh using an outline effect
   */
  applyHoverHighlight(mesh) {
    // Remove previous outline if any
    this.clearHoverHighlight();

    // Create outline mesh (scaled up wireframe)
    const outlineMaterial = new THREE.MeshBasicMaterial({
      color: 0x00ff00, // Bright green
      wireframe: true,
      transparent: true,
      opacity: 0.8,
      depthTest: true,
      depthWrite: false,
    });

    // Clone the geometry
    this.outlineMesh = new THREE.Mesh(mesh.geometry.clone(), outlineMaterial);

    // Copy transform
    this.outlineMesh.position.copy(mesh.position);
    this.outlineMesh.rotation.copy(mesh.rotation);
    this.outlineMesh.scale.copy(mesh.scale).multiplyScalar(1.01); // Slightly larger
    this.outlineMesh.quaternion.copy(mesh.quaternion);

    // If mesh has a parent, apply world transform
    if (mesh.parent) {
      mesh.parent.localToWorld(this.outlineMesh.position.copy(mesh.position));
      this.outlineMesh.quaternion.premultiply(mesh.parent.quaternion);
    }

    // Match world matrix
    this.outlineMesh.matrixAutoUpdate = false;
    this.outlineMesh.matrix.copy(mesh.matrixWorld);
    this.outlineMesh.matrix.scale(new THREE.Vector3(1.01, 1.01, 1.01));

    this.outlineMesh.renderOrder = 999;
    this.scene.add(this.outlineMesh);

    // Also add subtle emissive to original mesh for extra visibility
    const materials = Array.isArray(mesh.material) ? mesh.material : [mesh.material];
    materials.forEach((material, index) => {
      if (material.emissive) {
        this.originalMaterials.set(`${mesh.uuid}_${index}`, {
          emissive: material.emissive.clone(),
          emissiveIntensity: material.emissiveIntensity,
        });
        material.emissive.setHex(0x115511);
        material.emissiveIntensity = 0.5;
      }
    });
  }

  /**
   * Clear hover highlight
   */
  clearHoverHighlight() {
    // Hide tooltip
    this.hideTooltip();

    // Remove outline mesh
    if (this.outlineMesh) {
      this.scene.remove(this.outlineMesh);
      if (this.outlineMesh.geometry) this.outlineMesh.geometry.dispose();
      if (this.outlineMesh.material) this.outlineMesh.material.dispose();
      this.outlineMesh = null;
    }

    // Restore original material properties
    if (this.hoveredMesh) {
      const materials = Array.isArray(this.hoveredMesh.material)
        ? this.hoveredMesh.material
        : [this.hoveredMesh.material];

      materials.forEach((material, index) => {
        const original = this.originalMaterials.get(`${this.hoveredMesh.uuid}_${index}`);
        if (original && material.emissive) {
          material.emissive.copy(original.emissive);
          material.emissiveIntensity = original.emissiveIntensity;
        }
      });

      this.originalMaterials.clear();
    }
  }

  /**
   * Select a mesh and find its material entry
   */
  selectMesh(mesh) {
    this.selectedMesh = mesh;

    // Find the material entry for this mesh - need to scan first if not done
    if (this.meshMaterials.size === 0) {
      this.scanMaterials();
    }

    // Find the material entry for this mesh
    let foundEntry = null;
    this.meshMaterials.forEach((info, key) => {
      if (info.mesh === mesh) {
        foundEntry = { id: key, ...info };
      }
    });

    // If not found, add it now
    if (!foundEntry && mesh.material) {
      const materials = Array.isArray(mesh.material) ? mesh.material : [mesh.material];
      materials.forEach((material, index) => {
        if (material.isMeshStandardMaterial || material.isMeshPhysicalMaterial) {
          const key = `${mesh.name || mesh.uuid}_${index}`;
          const info = {
            mesh: mesh,
            material: material,
            name: mesh.name || `Mesh_${mesh.uuid.slice(0, 8)}`,
            materialIndex: index,
            originalRoughness: material.roughness,
            originalMetalness: material.metalness,
          };
          this.meshMaterials.set(key, info);
          if (!foundEntry) {
            foundEntry = { id: key, ...info };
          }
        }
      });
      this.notifyMaterialsUpdate();
    }

    console.log('[DevTools] Mesh selected:', foundEntry?.name || mesh.name || mesh.uuid);

    if (this.onSelectionChange) {
      this.onSelectionChange({
        type: 'mesh',
        mesh,
        materialEntry: foundEntry,
      });
    }
    // Note: In dev mode, selection stays active (no call to disableMeshSelection)
  }

  /**
   * Notify listeners of materials update
   */
  notifyMaterialsUpdate() {
    if (this.onMaterialsUpdate) {
      this.onMaterialsUpdate(this.getMaterialsList());
    }
  }

  /**
   * Notify listeners of lights update
   */
  notifyLightsUpdate() {
    if (this.onLightsUpdate) {
      this.onLightsUpdate(this.getLightsList());
    }
  }

  /**
   * Set helper visibility for all lights
   */
  setLightHelpersVisible(visible) {
    this.customLights.forEach((data) => {
      if (data.helper) {
        data.helper.visible = visible;
      }
      // Also toggle direction helpers visibility
      if (data.directionHelpers) {
        data.directionHelpers.visible = visible;
      }
    });
  }

  // ==========================================
  // PLACED MESHES MANAGEMENT
  // ==========================================

  /**
   * Load a mesh from an asset and add it to the scene
   * @param {string} assetId - The asset ID to load
   * @param {string} assetName - Display name for the mesh
   * @returns {Promise<string>} The placed mesh ID
   */
  async addMeshFromAsset(assetId, assetName) {
    const id = `mesh_${++this.meshIdCounter}`;

    try {
      let object;

      // Check cache first
      if (this.meshCache.has(assetId)) {
        object = this.meshCache.get(assetId).clone();
      } else {
        // Load the GLTF
        const url = assetApi.getDownloadUrl(assetId);
        console.log('[DevTools] Loading mesh from URL:', url);
        const gltf = await new Promise((resolve, reject) => {
          this.gltfLoader.load(
            url,
            resolve,
            (progress) => console.log('[DevTools] Loading progress:', progress.loaded, '/', progress.total),
            (error) => {
              console.error('[DevTools] GLTF load error:', error);
              reject(error);
            }
          );
        });

        // Cache the original
        this.meshCache.set(assetId, gltf.scene);
        object = gltf.scene.clone();
      }

      // Calculate bounding box to determine appropriate scale
      // Unreal Engine units: 1 unit = 1 cm, so a car is ~120 units long
      // GLB files are typically in meters (1 unit = 1 meter)
      // Target size: ~150 units (roughly car-sized) for the largest dimension
      const TARGET_SIZE = 150; // Unreal units

      const boundingBox = new THREE.Box3().setFromObject(object);
      const size = new THREE.Vector3();
      boundingBox.getSize(size);

      // Get the largest dimension
      const maxDimension = Math.max(size.x, size.y, size.z);

      // Calculate scale factor to reach target size
      // If mesh is already in Unreal-like units (>10), don't scale up too much
      let scaleFactor = 1;
      if (maxDimension > 0) {
        if (maxDimension < 10) {
          // Mesh is probably in meters, scale up to Unreal units (x100)
          scaleFactor = TARGET_SIZE / maxDimension;
        } else if (maxDimension < 50) {
          // Mesh is in some intermediate scale, adjust proportionally
          scaleFactor = TARGET_SIZE / maxDimension;
        }
        // If maxDimension >= 50, mesh is probably already in appropriate units
      }

      // Apply scale (uniform)
      object.scale.setScalar(scaleFactor);

      console.log(`[DevTools] Mesh size: ${size.x.toFixed(2)} x ${size.y.toFixed(2)} x ${size.z.toFixed(2)}, applied scale: ${scaleFactor.toFixed(2)}`);

      // Set initial position (in front of camera)
      const forward = new THREE.Vector3();
      this.camera.getWorldDirection(forward);
      const position = this.camera.position.clone().add(forward.multiplyScalar(500)); // Place 500 units in front
      position.y = 0; // Place at ground level

      object.position.copy(position);
      object.name = id;
      object.userData.placedMeshId = id;
      object.userData.assetId = assetId;
      object.userData.originalScale = scaleFactor; // Store for reference

      // Enable shadows for all children
      object.traverse((child) => {
        if (child.isMesh) {
          child.castShadow = true;
          child.receiveShadow = true;
        }
      });

      this.scene.add(object);

      this.placedMeshes.set(id, {
        id,
        object,
        assetId,
        name: assetName || `Mesh ${this.meshIdCounter}`,
        visible: true,
      });

      console.log('[DevTools] Mesh added:', id, assetName);
      this.notifyPlacedMeshesUpdate();

      // Auto-select the newly placed mesh after a frame to let React update the list first
      requestAnimationFrame(() => {
        this.selectPlacedMesh(id);
      });

      return id;
    } catch (error) {
      console.error('[DevTools] Failed to load mesh:', error);
      throw error;
    }
  }

  /**
   * Get list of placed meshes for UI
   */
  getPlacedMeshesList() {
    const list = [];
    this.placedMeshes.forEach((data, id) => {
      list.push({
        id,
        name: data.name,
        assetId: data.assetId,
        displayName: data.displayName || null,
        visible: data.object.visible,
        position: {
          x: data.object.position.x,
          y: data.object.position.y,
          z: data.object.position.z,
        },
        rotation: {
          x: data.object.rotation.x,
          y: data.object.rotation.y,
          z: data.object.rotation.z,
        },
        scale: {
          x: data.object.scale.x,
          y: data.object.scale.y,
          z: data.object.scale.z,
        },
      });
    });
    return list;
  }

  /**
   * Select a placed mesh for manipulation
   */
  selectPlacedMesh(meshId) {
    // Deselect any light first
    if (this.selectedLight) {
      this.deselectLight();
    }

    const meshData = this.placedMeshes.get(meshId);
    if (!meshData) return;

    this.selectedPlacedMesh = meshId;

    // Attach transform controls
    this._ensureTransformControls();
    this.transformControls.attach(meshData.object);
    this.transformControls.setMode(this._meshTransformMode);
    if (this._transformControlsHelper) {
      this._transformControlsHelper.visible = true;
    }

    // Store current scale for uniform scaling detection
    if (this._scaleAtDragStart) {
      this._scaleAtDragStart.copy(meshData.object.scale);
    }

    console.log('[DevTools] Placed mesh selected:', meshId);

    if (this.onSelectionChange) {
      this.onSelectionChange({
        type: 'placed-mesh',
        id: meshId,
        mode: this._meshTransformMode,
      });
    }
  }

  /**
   * Deselect placed mesh
   */
  deselectPlacedMesh() {
    const wasSelected = this.selectedPlacedMesh !== null;
    this.selectedPlacedMesh = null;

    // Always hide transform controls when deselecting mesh
    // (even if no mesh was selected, ensure gizmo is hidden)
    if (this.transformControls) {
      this.transformControls.detach();
    }
    if (this._transformControlsHelper) {
      this._transformControlsHelper.visible = false;
    }

    if (wasSelected && this.onSelectionChange) {
      this.onSelectionChange({ type: null, id: null });
    }
  }

  /**
   * Set transform mode for placed meshes
   * @param {'translate' | 'rotate' | 'scale'} mode
   */
  setMeshTransformMode(mode) {
    this._meshTransformMode = mode;
    if (this.transformControls && this.selectedPlacedMesh) {
      this.transformControls.setMode(mode);
    }

    if (this.onSelectionChange && this.selectedPlacedMesh) {
      this.onSelectionChange({
        type: 'placed-mesh',
        id: this.selectedPlacedMesh,
        mode,
      });
    }
  }

  /**
   * Get current mesh transform mode
   */
  getMeshTransformMode() {
    return this._meshTransformMode;
  }

  /**
   * Remove a placed mesh
   */
  removePlacedMesh(meshId) {
    const meshData = this.placedMeshes.get(meshId);
    if (!meshData) return;

    // Deselect if selected
    if (this.selectedPlacedMesh === meshId) {
      this.deselectPlacedMesh();
    }

    // Remove from scene
    this.scene.remove(meshData.object);

    // Dispose geometry and materials
    meshData.object.traverse((child) => {
      if (child.isMesh) {
        child.geometry?.dispose();
        if (Array.isArray(child.material)) {
          child.material.forEach(m => m.dispose());
        } else if (child.material) {
          child.material.dispose();
        }
      }
    });

    this.placedMeshes.delete(meshId);
    console.log('[DevTools] Mesh removed:', meshId);
    this.notifyPlacedMeshesUpdate();
  }

  /**
   * Duplicate a placed mesh
   */
  duplicatePlacedMesh(meshId) {
    const meshData = this.placedMeshes.get(meshId);
    if (!meshData) return null;

    const newId = `mesh_${++this.meshIdCounter}`;
    const newObject = meshData.object.clone();

    // Offset the position slightly
    newObject.position.x += 2;
    newObject.name = newId;
    newObject.userData.placedMeshId = newId;

    this.scene.add(newObject);

    this.placedMeshes.set(newId, {
      id: newId,
      object: newObject,
      assetId: meshData.assetId,
      name: `${meshData.name} (copy)`,
      visible: true,
    });

    console.log('[DevTools] Mesh duplicated:', meshId, '->', newId);
    this.notifyPlacedMeshesUpdate();

    // Select the duplicate
    this.selectPlacedMesh(newId);

    return newId;
  }

  /**
   * Toggle mesh visibility
   */
  togglePlacedMeshVisibility(meshId) {
    const meshData = this.placedMeshes.get(meshId);
    if (!meshData) return;

    meshData.object.visible = !meshData.object.visible;
    this.notifyPlacedMeshesUpdate();
  }

  /**
   * Update placed mesh name
   */
  updatePlacedMeshName(meshId, name) {
    const meshData = this.placedMeshes.get(meshId);
    if (!meshData) return;

    meshData.name = name;
    this.notifyPlacedMeshesUpdate();
  }

  /**
   * Update placed mesh display name (custom instance name)
   */
  updatePlacedMeshDisplayName(meshId, displayName) {
    const meshData = this.placedMeshes.get(meshId);
    if (!meshData) return;

    meshData.displayName = displayName;
    this.notifyPlacedMeshesUpdate();
  }

  /**
   * Notify listeners of placed meshes update
   */
  notifyPlacedMeshesUpdate() {
    if (this.onPlacedMeshesUpdate) {
      this.onPlacedMeshesUpdate(this.getPlacedMeshesList());
    }
  }

  /**
   * Check if an object is one of our placed meshes
   */
  _isPlacedMesh(object) {
    // Check if the object or any parent has a placedMeshId
    let current = object;
    while (current) {
      if (current.userData?.placedMeshId) {
        return current.userData.placedMeshId;
      }
      current = current.parent;
    }
    return null;
  }

  /**
   * Collect environment data for saving
   * Returns all placed meshes and lights in a format suitable for the API
   */
  collectEnvironmentData() {
    const meshes = [];
    this.placedMeshes.forEach((data) => {
      meshes.push({
        assetId: data.assetId,
        displayName: data.displayName || null,
        position: {
          x: data.object.position.x,
          y: data.object.position.y,
          z: data.object.position.z,
        },
        rotation: {
          x: data.object.rotation.x,
          y: data.object.rotation.y,
          z: data.object.rotation.z,
        },
        scale: {
          x: data.object.scale.x,
          y: data.object.scale.y,
          z: data.object.scale.z,
        },
        visible: data.object.visible,
      });
    });

    const lights = [];
    this.customLights.forEach((data) => {
      const light = data.light;
      const lightData = {
        lightType: data.type,
        color: '#' + light.color.getHexString(),
        intensity: light.intensity,
        enabled: true,
        params: {},
      };

      // Add type-specific params
      if (light.position && data.type !== 'ambient') {
        lightData.params.position = {
          x: light.position.x,
          y: light.position.y,
          z: light.position.z,
        };
      }

      if (data.target) {
        lightData.params.target = {
          x: data.target.position.x,
          y: data.target.position.y,
          z: data.target.position.z,
        };
      }

      if (light.distance !== undefined) {
        lightData.params.distance = light.distance;
      }

      if (light.angle !== undefined) {
        lightData.params.angle = light.angle;
      }

      if (light.penumbra !== undefined) {
        lightData.params.penumbra = light.penumbra;
      }

      if (light.groundColor) {
        lightData.params.groundColor = '#' + light.groundColor.getHexString();
      }

      // Collect shadow parameters for directional and spot lights
      if ((data.type === 'directional' || data.type === 'spot') && light.castShadow !== undefined) {
        lightData.params.shadow = {
          enabled: light.castShadow,
          mapSize: light.shadow.mapSize.width,
          bias: light.shadow.bias,
          normalBias: light.shadow.normalBias,
          cameraNear: light.shadow.camera.near,
          cameraFar: light.shadow.camera.far,
        };
        // Add orthographic camera bounds for directional lights
        if (data.type === 'directional') {
          lightData.params.shadow.cameraLeft = light.shadow.camera.left;
          lightData.params.shadow.cameraRight = light.shadow.camera.right;
          lightData.params.shadow.cameraTop = light.shadow.camera.top;
          lightData.params.shadow.cameraBottom = light.shadow.camera.bottom;
        }
      }

      lights.push(lightData);
    });

    return { meshes, lights };
  }

  /**
   * Load environment data into the editor
   * @param {Object} environmentData - Data from the API (full Environment object)
   */
  async loadEnvironmentInEditor(environmentData) {
    // Clear existing placed meshes from DevTools
    const meshIds = [...this.placedMeshes.keys()];
    meshIds.forEach(id => this.removePlacedMesh(id));

    // Clear existing lights from DevTools
    const lightIds = [...this.customLights.keys()];
    lightIds.forEach(id => this.removeLight(id));

    // Clear the replay environment (meshes, lights, ground loaded by EnvironmentManager)
    // This prevents having two environments coexisting
    if (this.environmentManager) {
      await this.environmentManager.clearEnvironment();
    }

    // Load skybox via EnvironmentManager if available
    if (this.environmentManager) {
      // Load skybox
      if (environmentData.skyboxAsset?.id || environmentData.skyboxAssetId) {
        const skyboxId = environmentData.skyboxAsset?.id || environmentData.skyboxAssetId;
        const exposure = environmentData.skyboxExposure ?? 1.0;
        await this.environmentManager.loadSkybox(skyboxId, exposure);
      }

      // Apply skybox rotation on all 3 axes (Dev Mode Lot 3 - US1/US2)
      this.environmentManager.setSkyboxRotationXYZ(
        environmentData.skyboxRotationX ?? 0,
        environmentData.skyboxRotationY ?? 0,
        environmentData.skyboxRotationZ ?? 0
      );
      if (environmentData.skyboxAnimationEnabled) {
        this.environmentManager.startSkyboxAnimation(environmentData.skyboxAnimationSpeed ?? 1.0);
      } else {
        this.environmentManager.stopSkyboxAnimation();
      }

      // Load ground (Dev Mode Lot 3 - US3/US4)
      // First remove any existing ground
      this.environmentManager.removeGround();

      if (environmentData.groundEnabled && (environmentData.groundTexture?.id || environmentData.groundTextureId)) {
        const textureId = environmentData.groundTexture?.id || environmentData.groundTextureId;
        await this.environmentManager.loadGround(
          textureId,
          environmentData.groundRepeatX ?? 10,
          environmentData.groundRepeatY ?? 10,
          environmentData.groundHeight ?? 0,
          environmentData.groundHeightmap ?? null
        );
      }
    }

    // Load meshes
    if (environmentData.meshes) {
      for (const mesh of environmentData.meshes) {
        try {
          const id = await this.addMeshFromAsset(mesh.assetId, mesh.asset?.name || 'Mesh');
          const meshData = this.placedMeshes.get(id);
          if (meshData) {
            meshData.object.position.set(mesh.position.x, mesh.position.y, mesh.position.z);
            meshData.object.rotation.set(mesh.rotation.x, mesh.rotation.y, mesh.rotation.z);
            meshData.object.scale.set(mesh.scale.x, mesh.scale.y, mesh.scale.z);
            meshData.object.visible = mesh.visible;
            // Apply custom display name if present
            if (mesh.displayName) {
              meshData.displayName = mesh.displayName;
            }
          }
        } catch (error) {
          console.error('[DevTools] Failed to load mesh:', mesh.assetId, error);
        }
      }
    }

    // Load lights
    if (environmentData.lights) {
      for (const light of environmentData.lights) {
        const options = {
          color: light.color,
          intensity: light.intensity,
          position: light.params?.position ? new THREE.Vector3(
            light.params.position.x,
            light.params.position.y,
            light.params.position.z
          ) : undefined,
          distance: light.params?.distance,
          angle: light.params?.angle,
          penumbra: light.params?.penumbra,
          groundColor: light.params?.groundColor,
          skyColor: light.color,
        };

        const id = this.addLight(light.lightType, options);

        // Set target position if exists
        if (light.params?.target && id) {
          const lightData = this.customLights.get(id);
          if (lightData?.target) {
            lightData.target.position.set(
              light.params.target.x,
              light.params.target.y,
              light.params.target.z
            );
          }
        }

        // Apply shadow parameters if exists
        if (light.params?.shadow && id) {
          const lightData = this.customLights.get(id);
          if (lightData?.light) {
            const l = lightData.light;
            const shadow = light.params.shadow;
            l.castShadow = shadow.enabled ?? false;
            if (l.shadow) {
              l.shadow.mapSize.width = shadow.mapSize ?? 2048;
              l.shadow.mapSize.height = shadow.mapSize ?? 2048;
              l.shadow.bias = shadow.bias ?? -0.0001;
              l.shadow.normalBias = shadow.normalBias ?? 0;
              l.shadow.camera.near = shadow.cameraNear ?? 100;
              l.shadow.camera.far = shadow.cameraFar ?? 10000;
              // Orthographic camera bounds for directional lights
              if (light.lightType === 'directional') {
                l.shadow.camera.left = shadow.cameraLeft ?? -5000;
                l.shadow.camera.right = shadow.cameraRight ?? 5000;
                l.shadow.camera.top = shadow.cameraTop ?? 5000;
                l.shadow.camera.bottom = shadow.cameraBottom ?? -5000;
                l.shadow.camera.updateProjectionMatrix();
              }
            }
          }
        }
      }
    }

    this.notifyPlacedMeshesUpdate();
    this.notifyLightsUpdate();

    console.log('[DevTools] Environment loaded into editor');
  }

  /**
   * Clear all placed meshes
   */
  clearAllPlacedMeshes() {
    const meshIds = [...this.placedMeshes.keys()];
    meshIds.forEach(id => this.removePlacedMesh(id));
  }

  /**
   * Clean up resources
   */
  dispose() {
    // Exit dev mode first
    this.exitDevMode();

    // Disable terraforming mode (will dispose brush preview)
    this.disableTerraformingMode();

    // Remove all placed meshes
    this.clearAllPlacedMeshes();

    // Remove all custom lights
    const lightIds = [...this.customLights.keys()];
    lightIds.forEach((id) => {
      this.removeLight(id);
    });

    // Remove transform controls
    if (this.transformControls) {
      if (this._transformControlsHelper) {
        this.scene.remove(this._transformControlsHelper);
      }
      this.transformControls.dispose();
    }

    // Remove tooltip
    if (this.tooltip && this.tooltip.parentNode) {
      this.tooltip.parentNode.removeChild(this.tooltip);
      this.tooltip = null;
    }

    // Dispose DRACO loader
    if (this.dracoLoader) {
      this.dracoLoader.dispose();
    }

    // Clear caches
    this.meshMaterials.clear();
    this.customLights.clear();
    this.placedMeshes.clear();
    this.meshCache.clear();
  }

  // ==========================================
  // TERRAFORMING MODE (Dev Mode Lot 3 - US4)
  // ==========================================

  /**
   * Create the brush preview circle
   */
  _createBrushPreview() {
    if (this._brushPreview) return;

    // Create a ring geometry for the brush outline
    const segments = 64;
    const geometry = new THREE.RingGeometry(
      this._terraformBrushSize * 0.95, // Inner radius (slightly smaller than outer)
      this._terraformBrushSize, // Outer radius
      segments
    );

    // Use a bright material for visibility
    const material = new THREE.MeshBasicMaterial({
      color: 0x00ff00,
      side: THREE.DoubleSide,
      transparent: true,
      opacity: 0.6,
      depthTest: true,
      depthWrite: false,
    });

    this._brushPreview = new THREE.Mesh(geometry, material);
    this._brushPreview.rotation.x = -Math.PI / 2; // Lay flat on ground
    this._brushPreview.visible = false;
    this._brushPreview.renderOrder = 999;
    this._brushPreview.userData.isBrushPreview = true;

    this.scene.add(this._brushPreview);
  }

  /**
   * Update brush preview size
   */
  _updateBrushPreviewSize() {
    if (!this._brushPreview) return;

    // Dispose old geometry and create new one with updated size
    this._brushPreview.geometry.dispose();
    const segments = 64;
    this._brushPreview.geometry = new THREE.RingGeometry(
      this._terraformBrushSize * 0.95,
      this._terraformBrushSize,
      segments
    );
  }

  /**
   * Update brush preview position based on mouse
   */
  _updateBrushPreviewPosition(event) {
    if (!this._brushPreview || !this._terraformingMode) return;

    const point = this._getTerraformPoint(event);
    if (point) {
      this._brushPreview.position.set(point.x, point.y + 5, point.z); // Slightly above ground
      this._brushPreview.visible = true;

      // Change color based on Shift key (raise = green, lower = red)
      const isLowering = event.shiftKey;
      this._brushPreview.material.color.setHex(isLowering ? 0xff4444 : 0x44ff44);
    } else {
      this._brushPreview.visible = false;
    }
  }

  /**
   * Show/hide brush preview
   */
  _setBrushPreviewVisible(visible) {
    if (this._brushPreview) {
      this._brushPreview.visible = visible;
      this._brushPreviewVisible = visible;
    }
  }

  /**
   * Dispose brush preview
   */
  _disposeBrushPreview() {
    if (this._brushPreview) {
      this.scene.remove(this._brushPreview);
      this._brushPreview.geometry.dispose();
      this._brushPreview.material.dispose();
      this._brushPreview = null;
      this._brushPreviewVisible = false;
    }
  }

  /**
   * Enable terraforming mode
   */
  enableTerraformingMode() {
    if (!this.environmentManager) {
      console.warn("[DevTools] No environmentManager available for terraforming");
      return;
    }

    this._terraformingMode = true;
    this.selectionMode = "terraform";
    this.renderer.domElement.style.cursor = "crosshair";

    // Create brush preview
    this._createBrushPreview();

    // Add terraforming event listeners
    this.renderer.domElement.addEventListener("pointerdown", this._handleTerraformMouseDown);
    this.renderer.domElement.addEventListener("pointermove", this._handleTerraformMouseMove);
    this.renderer.domElement.addEventListener("pointerup", this._handleTerraformMouseUp);

    console.log("[DevTools] Terraforming mode enabled");

    if (this.onTerraformingModeChange) {
      this.onTerraformingModeChange(true);
    }
  }

  /**
   * Disable terraforming mode
   */
  disableTerraformingMode() {
    this._terraformingMode = false;
    this._isTerraforming = false;
    // Don't auto-enable mesh selection - let user explicitly enable it
    this.selectionMode = "none";
    this.renderer.domElement.style.cursor = "default";

    // Remove brush preview
    this._disposeBrushPreview();

    // Remove terraforming event listeners
    this.renderer.domElement.removeEventListener("pointerdown", this._handleTerraformMouseDown);
    this.renderer.domElement.removeEventListener("pointermove", this._handleTerraformMouseMove);
    this.renderer.domElement.removeEventListener("pointerup", this._handleTerraformMouseUp);

    // Recalculate normals after terraforming session
    if (this.environmentManager) {
      this.environmentManager.recalculateGroundNormals();
    }

    console.log("[DevTools] Terraforming mode disabled");

    if (this.onTerraformingModeChange) {
      this.onTerraformingModeChange(false);
    }
  }

  /**
   * Check if terraforming mode is active
   */
  isTerraformingModeActive() {
    return this._terraformingMode;
  }

  /**
   * Set brush size for terraforming
   */
  setTerraformBrushSize(size) {
    this._terraformBrushSize = Math.max(100, Math.min(5000, size));
    // Update brush preview size if it exists
    this._updateBrushPreviewSize();
  }

  /**
   * Get current brush size
   */
  getTerraformBrushSize() {
    return this._terraformBrushSize;
  }

  /**
   * Set brush strength for terraforming
   */
  setTerraformBrushStrength(strength) {
    this._terraformBrushStrength = Math.max(1, Math.min(100, strength));
  }

  /**
   * Get current brush strength
   */
  getTerraformBrushStrength() {
    return this._terraformBrushStrength;
  }

  /**
   * Get terrain intersection point from mouse position
   */
  _getTerraformPoint(event) {
    if (!this.environmentManager) return null;
    
    const groundMesh = this.environmentManager.getGroundMesh();
    if (!groundMesh) return null;
    
    // Update mouse coordinates
    const rect = this.renderer.domElement.getBoundingClientRect();
    this.mouse.x = ((event.clientX - rect.left) / rect.width) * 2 - 1;
    this.mouse.y = -((event.clientY - rect.top) / rect.height) * 2 + 1;
    
    // Raycast to ground
    this._terraformRaycaster.setFromCamera(this.mouse, this.camera);
    const intersects = this._terraformRaycaster.intersectObject(groundMesh);
    
    if (intersects.length > 0) {
      return intersects[0].point;
    }
    return null;
  }

  /**
   * Handle mouse down for terraforming
   */
  _handleTerraformMouseDown(event) {
    if (!this._terraformingMode) return;
    if (event.button !== 0) return; // Only left click
    
    // Skip if right mouse is held (camera rotation)
    if ((event.buttons & 2) !== 0) return;
    
    this._isTerraforming = true;
    this._applyTerraformBrush(event, event.shiftKey);
  }

  /**
   * Handle mouse move for continuous terraforming
   */
  _handleTerraformMouseMove(event) {
    if (!this._terraformingMode) return;

    // Always update brush preview position
    this._updateBrushPreviewPosition(event);

    // Only apply brush if actively terraforming (left mouse down)
    if (!this._isTerraforming) return;

    // Skip if right mouse is held (camera rotation)
    if ((event.buttons & 2) !== 0) return;

    this._applyTerraformBrush(event, event.shiftKey);
  }

  /**
   * Handle mouse up for terraforming
   */
  _handleTerraformMouseUp(event) {
    if (!this._terraformingMode) return;
    
    if (this._isTerraforming) {
      this._isTerraforming = false;
      // Recalculate normals after stroke
      if (this.environmentManager) {
        this.environmentManager.recalculateGroundNormals();
      }
    }
  }

  /**
   * Apply brush to terrain at current mouse position
   */
  _applyTerraformBrush(event, isLowering) {
    const point = this._getTerraformPoint(event);
    if (!point) return;
    
    const strength = isLowering ? -this._terraformBrushStrength : this._terraformBrushStrength;
    
    // Apply brush via EnvironmentManager
    this.environmentManager.applyBrush(
      point.x,
      point.z,
      this._terraformBrushSize,
      strength * 0.5, // Scale down for smoother painting
      "smooth"
    );
  }

  /**
   * Flatten terrain to height 0
   */
  flattenTerrain() {
    if (this.environmentManager) {
      this.environmentManager.flattenTerrain(0);
    }
  }
}
