/**
 * ViewerCameraManager - Manages 3D visualization of other viewers' cameras in collaborative sessions
 *
 * Features:
 * - Loads and displays drone camera models for each participant
 * - Updates positions/rotations based on camera updates from collab
 * - Shows viewer nickname labels with their assigned color
 * - Excludes the local viewer's own camera
 */

import * as THREE from "three";
import { GLTFLoader } from "three/examples/jsm/loaders/GLTFLoader.js";
import { DRACOLoader } from "three/examples/jsm/loaders/DRACOLoader.js";
import { clone as skeletonClone } from "three/examples/jsm/utils/SkeletonUtils.js";

export class ViewerCameraManager {
  /**
   * @param {THREE.Scene} scene - The scene to add camera meshes to
   * @param {THREE.Camera} camera - The main camera (for label orientation)
   */
  constructor(scene, camera) {
    this.scene = scene;
    this.camera = camera;

    // Map of participantId -> { mesh, label, color, nickname }
    this.viewerCameras = new Map();

    // ID of local viewer (to exclude from rendering)
    this.selfId = null;

    // Model loading
    this.gltfLoader = new GLTFLoader();
    const dracoLoader = new DRACOLoader();
    dracoLoader.setDecoderPath("https://www.gstatic.com/draco/versioned/decoders/1.5.6/");
    this.gltfLoader.setDRACOLoader(dracoLoader);

    // Cached model template
    this.modelTemplate = null;
    this.modelLoading = null;
    this.modelLoadError = false;

    // Camera model scale (adjust based on the model)
    this.modelScale = 300; // 6x larger for better visibility

    // Label settings (same approach as NameTagManager)
    this.labelScale = 0.06; // Screen-space scale (sizeAttenuation: false)
    this.canvasWidth = 256;
    this.canvasHeight = 80;

    // Interpolation settings
    this.positionLerpFactor = 0.15; // Higher = faster catch-up, lower = smoother
    this.rotationSlerpFactor = 0.12; // Slightly slower for rotation to feel natural

    // Visibility tracking for follow feature
    this.hiddenViewerId = null; // ID of viewer we're following (their mesh is hidden for us)
    this.hiddenFollowers = new Set(); // IDs of viewers who are following someone (hidden for everyone)

    // Track pending viewer creations to avoid duplicate meshes from async race conditions
    this.pendingCreations = new Set();

    // Load the model immediately
    this.loadModel();
  }

  /**
   * Load the drone camera model
   */
  async loadModel() {
    if (this.modelLoading) return this.modelLoading;
    if (this.modelLoadError) return null;

    this.modelLoading = new Promise((resolve, reject) => {
      console.log("[ViewerCameraManager] Loading camera drone model...");
      this.gltfLoader.load(
        "/models/camera/camera_drone.glb",
        (gltf) => {
          console.log("[ViewerCameraManager] Loaded camera drone model successfully");
          this.modelTemplate = gltf.scene;

          // Apply scale to template
          this.modelTemplate.scale.setScalar(this.modelScale);

          // Make materials use standard lighting and ensure visibility
          this.modelTemplate.traverse((child) => {
            if (child.isMesh) {
              child.castShadow = false;
              child.receiveShadow = false;
              child.frustumCulled = false; // Always render
              // Store original material for cloning
              if (child.material) {
                child.userData.originalMaterial = child.material.clone();
              }
            }
          });

          resolve(this.modelTemplate);
        },
        (progress) => {
          if (progress.total > 0) {
            console.log(
              `[ViewerCameraManager] Loading: ${Math.round((progress.loaded / progress.total) * 100)}%`,
            );
          }
        },
        (error) => {
          console.error("[ViewerCameraManager] Failed to load camera model:", error);
          this.modelLoadError = true;
          reject(error);
        },
      );
    });

    return this.modelLoading;
  }

  /**
   * Set the local viewer's ID (to exclude from rendering)
   * @param {string} selfId
   */
  setSelfId(selfId) {
    this.selfId = selfId;
    // Remove own camera if it exists
    if (selfId && this.viewerCameras.has(selfId)) {
      this.removeViewer(selfId);
    }
  }

  /**
   * Create a text label sprite for a viewer
   * @param {string} nickname
   * @param {string} color - Hex color string
   * @returns {THREE.Sprite}
   */
  /**
   * Darken a hex color by a percentage
   * @param {string} hex - Hex color string
   * @param {number} percent - Percentage to darken (0-100)
   * @returns {string} Darkened hex color
   */
  darkenColor(hex, percent) {
    // Remove # if present
    hex = hex.replace("#", "");

    // Parse RGB values
    let r = parseInt(hex.substring(0, 2), 16);
    let g = parseInt(hex.substring(2, 4), 16);
    let b = parseInt(hex.substring(4, 6), 16);

    // Darken each component
    const factor = 1 - percent / 100;
    r = Math.round(r * factor);
    g = Math.round(g * factor);
    b = Math.round(b * factor);

    // Convert back to hex
    return `#${r.toString(16).padStart(2, "0")}${g.toString(16).padStart(2, "0")}${b.toString(16).padStart(2, "0")}`;
  }

  createLabel(nickname, color) {
    // Create a NEW canvas for each label (important!)
    const canvas = document.createElement("canvas");
    const ctx = canvas.getContext("2d");
    canvas.width = this.canvasWidth;
    canvas.height = this.canvasHeight;

    // Ensure color is valid hex
    const labelColor = color && color.startsWith("#") ? color : "#888888";
    // Darken the color by 50% for better contrast with white text
    const bgColor = this.darkenColor(labelColor, 50);
    console.log(
      `[ViewerCameraManager] createLabel: nickname=${nickname}, inputColor=${color}, bgColor=${bgColor}`,
    );

    const width = canvas.width;
    const height = canvas.height;

    // Clear canvas
    ctx.clearRect(0, 0, width, height);

    // Draw name tag with pill shape (similar to NameTagManager style)
    const tagHeight = 44;
    const tagY = (height - tagHeight) / 2;

    // Measure text to determine tag width
    ctx.font = "bold 20px Arial, sans-serif";
    const textWidth = ctx.measureText(nickname).width;
    const padding = 24;
    const tagWidth = textWidth + padding * 2;
    const tagX = (width - tagWidth) / 2;

    // Draw pill shape (100% rounded on X axis)
    const radius = tagHeight / 2;
    ctx.beginPath();
    ctx.roundRect(tagX, tagY, tagWidth, tagHeight, radius);

    // Colored background (darkened viewer's color for contrast)
    ctx.fillStyle = bgColor;
    ctx.fill();

    // White border
    ctx.strokeStyle = "#FFFFFF";
    ctx.lineWidth = 3;
    ctx.stroke();

    // Draw nickname text in white
    ctx.font = "bold 20px Arial, sans-serif";
    ctx.fillStyle = "#FFFFFF";
    ctx.textAlign = "center";
    ctx.textBaseline = "middle";
    ctx.shadowColor = "rgba(0, 0, 0, 0.5)";
    ctx.shadowBlur = 3;
    ctx.shadowOffsetX = 1;
    ctx.shadowOffsetY = 1;
    ctx.fillText(nickname, width / 2, height / 2);
    ctx.shadowBlur = 0;

    // Create texture from canvas
    const texture = new THREE.CanvasTexture(canvas);
    texture.minFilter = THREE.LinearFilter;
    texture.magFilter = THREE.LinearFilter;
    texture.needsUpdate = true;

    // Create sprite material (sizeAttenuation: false = constant screen size)
    const spriteMaterial = new THREE.SpriteMaterial({
      map: texture,
      transparent: true,
      depthTest: false,
      depthWrite: false,
      sizeAttenuation: false, // Constant screen size like NameTagManager
    });

    const sprite = new THREE.Sprite(spriteMaterial);

    // Scale sprite (maintain aspect ratio) - same as NameTagManager
    const aspectRatio = this.canvasWidth / this.canvasHeight;
    sprite.scale.set(this.labelScale * aspectRatio, this.labelScale, 1);
    sprite.renderOrder = 999; // Render on top

    console.log(
      `[ViewerCameraManager] Created label sprite for ${nickname} with color ${labelColor}`,
    );

    return sprite;
  }

  /**
   * Create a simple fallback mesh if model loading fails
   * @param {string} color
   * @returns {THREE.Group}
   */
  createFallbackMesh(color) {
    const group = new THREE.Group();

    // Create a simple camera-like shape
    const bodyGeometry = new THREE.BoxGeometry(30, 20, 40);
    const bodyMaterial = new THREE.MeshStandardMaterial({
      color: new THREE.Color(color || "#888888"),
      metalness: 0.5,
      roughness: 0.5,
    });
    const body = new THREE.Mesh(bodyGeometry, bodyMaterial);
    group.add(body);

    // Lens
    const lensGeometry = new THREE.CylinderGeometry(8, 10, 15, 16);
    const lensMaterial = new THREE.MeshStandardMaterial({
      color: 0x222222,
      metalness: 0.8,
      roughness: 0.2,
    });
    const lens = new THREE.Mesh(lensGeometry, lensMaterial);
    lens.rotation.x = Math.PI / 2;
    lens.position.z = 25;
    group.add(lens);

    group.frustumCulled = false;

    return group;
  }

  /**
   * Add or update a viewer's camera
   * @param {string} participantId
   * @param {object} participant - { nickname, color, camera }
   */
  async addOrUpdateViewer(participantId, participant) {
    // Don't render own camera
    if (participantId === this.selfId) return;

    const { nickname, color, camera } = participant;

    let viewerData = this.viewerCameras.get(participantId);

    if (!viewerData) {
      // Check if creation is already in progress for this participant (async race condition guard)
      if (this.pendingCreations.has(participantId)) return;

      // Mark as pending before any async operation
      this.pendingCreations.add(participantId);

      // Try to load model, use fallback if it fails
      let mesh;

      try {
        await this.loadModel();
      } catch (e) {
        // Model load failed, will use fallback
      }

      // Check again after async operation - another call might have completed first
      if (this.viewerCameras.has(participantId)) {
        this.pendingCreations.delete(participantId);
        return;
      }

      if (this.modelTemplate) {
        // Clone the model
        mesh = skeletonClone(this.modelTemplate);
        mesh.scale.setScalar(this.modelScale);

        // Apply color tint to the model
        mesh.traverse((child) => {
          if (child.isMesh && child.material) {
            // Clone material and apply color tint
            child.material = child.material.clone();
            if (child.material.color && color) {
              // Blend with viewer color
              const viewerColor = new THREE.Color(color);
              child.material.color.lerp(viewerColor, 0.5);
            }
            if (child.material.emissive && color) {
              child.material.emissive.set(color);
              child.material.emissiveIntensity = 0.4;
            }
            child.frustumCulled = false;
          }
        });
      } else {
        // Use fallback mesh
        console.log("[ViewerCameraManager] Using fallback mesh");
        mesh = this.createFallbackMesh(color);
      }

      mesh.frustumCulled = false;

      // Rotate mesh 180° on Y axis (model faces backwards by default)
      mesh.rotation.y = Math.PI;

      // Create a container group for position/rotation updates
      // The mesh inside has the 180° offset, container receives camera quaternion
      const container = new THREE.Group();
      container.add(mesh);

      // Create label and add to container (not mesh, so it doesn't inherit mesh rotation)
      const label = this.createLabel(nickname, color);
      label.position.set(0, 120, 0); // Above the drone (adjusted for larger mesh)
      container.add(label);

      // Set a default visible position before adding to scene
      container.position.set(0, 500, 0); // High above arena, visible
      container.frustumCulled = false;

      // Check if this viewer should be hidden:
      // 1. We're following them (hiddenViewerId)
      // 2. They're following someone (hiddenFollowers set)
      const shouldBeHidden =
        participantId === this.hiddenViewerId ||
        (this.hiddenFollowers && this.hiddenFollowers.has(participantId));
      container.visible = !shouldBeHidden;
      if (shouldBeHidden) {
        console.log(`[ViewerCameraManager] *** New viewer ${nickname} HIDDEN (following someone)`);
      }

      // Add to scene
      this.scene.add(container);

      viewerData = {
        mesh: container, // Store container as "mesh" for position/rotation updates
        innerMesh: mesh, // Keep reference to actual mesh
        label,
        color,
        nickname,
        // Interpolation targets
        targetPosition: new THREE.Vector3(0, 500, 0),
        targetQuaternion: new THREE.Quaternion(),
        // Flag to know if we've received first update
        hasReceivedUpdate: false,
      };
      this.viewerCameras.set(participantId, viewerData);

      // Clear pending flag now that creation is complete
      this.pendingCreations.delete(participantId);
    } else {
      // Viewer already exists - enforce visibility state
      // This handles the case where follow status changed but mesh was already created
      const shouldBeHidden =
        participantId === this.hiddenViewerId ||
        (this.hiddenFollowers && this.hiddenFollowers.has(participantId));
      if (viewerData.mesh.visible !== !shouldBeHidden) {
        viewerData.mesh.visible = !shouldBeHidden;
        console.log(
          `[ViewerCameraManager] *** Visibility change: ${nickname} -> ${!shouldBeHidden ? "VISIBLE" : "HIDDEN"}`,
        );
      }
    }

    // Update position and rotation if camera data available
    if (camera && camera.position) {
      const { position, rotation } = camera;

      // Set target position for interpolation
      viewerData.targetPosition.set(position.x, position.y, position.z);

      if (rotation) {
        viewerData.targetQuaternion.set(rotation.x, rotation.y, rotation.z, rotation.w);
      }

      // For first update, snap directly to position (no interpolation)
      if (!viewerData.hasReceivedUpdate) {
        viewerData.mesh.position.copy(viewerData.targetPosition);
        viewerData.mesh.quaternion.copy(viewerData.targetQuaternion);
        viewerData.hasReceivedUpdate = true;
      }
    }
  }

  /**
   * Update a viewer's camera position/rotation (sets interpolation targets)
   * @param {string} participantId
   * @param {object} cameraState - { position, rotation, mode, targetPlayer }
   */
  updateViewerCamera(participantId, cameraState) {
    // Don't update own camera
    if (participantId === this.selfId) return;

    const viewerData = this.viewerCameras.get(participantId);
    if (!viewerData) return;

    const { position, rotation } = cameraState;

    if (position) {
      // Update target position (will be interpolated in update())
      viewerData.targetPosition.set(position.x, position.y, position.z);

      // Ensure mesh is visible (unless it should be hidden for follow feature)
      const shouldBeHidden =
        participantId === this.hiddenViewerId || this.hiddenFollowers.has(participantId);
      if (!shouldBeHidden) {
        viewerData.mesh.visible = true;
      }

      // For first update, snap directly to position (no interpolation lag)
      if (!viewerData.hasReceivedUpdate) {
        viewerData.mesh.position.copy(viewerData.targetPosition);
        viewerData.hasReceivedUpdate = true;
      }
    }

    if (rotation) {
      // Update target rotation (will be interpolated in update())
      viewerData.targetQuaternion.set(rotation.x, rotation.y, rotation.z, rotation.w);

      // For first update, snap directly
      if (!viewerData.hasReceivedUpdate && !position) {
        viewerData.mesh.quaternion.copy(viewerData.targetQuaternion);
        viewerData.hasReceivedUpdate = true;
      }
    }
  }

  /**
   * Set visibility for a specific viewer's camera mesh
   * Used when following a viewer - hide their mesh so we can see through it
   * @param {string} participantId
   * @param {boolean} visible
   */
  setViewerVisibility(participantId, visible) {
    const viewerData = this.viewerCameras.get(participantId);
    if (!viewerData) return;

    viewerData.mesh.visible = visible;
  }

  /**
   * Get the ID of the participant being followed (if any)
   * @returns {string | null}
   */
  getHiddenViewerId() {
    return this.hiddenViewerId || null;
  }

  /**
   * Set which viewer is being followed (hides their mesh for us)
   * @param {string | null} participantId - ID to hide, or null to show all
   */
  setFollowedViewer(participantId) {
    // Show previously hidden viewer (unless they're in hiddenFollowers)
    if (this.hiddenViewerId && this.hiddenViewerId !== participantId) {
      // Only show if they're not in hiddenFollowers
      if (!this.hiddenFollowers.has(this.hiddenViewerId)) {
        this.setViewerVisibility(this.hiddenViewerId, true);
      }
    }

    // Hide new followed viewer
    if (participantId) {
      this.setViewerVisibility(participantId, false);
    }

    this.hiddenViewerId = participantId;
  }

  /**
   * Mark a viewer as following someone (hides their mesh for everyone)
   * @param {string} participantId - ID of the viewer who is following
   * @param {boolean} isFollowing - true if following, false if stopped following
   */
  setViewerIsFollowing(participantId, isFollowing) {
    if (isFollowing) {
      this.hiddenFollowers.add(participantId);
      this.setViewerVisibility(participantId, false);
    } else {
      this.hiddenFollowers.delete(participantId);
      // Only show if we're not following them
      if (participantId !== this.hiddenViewerId) {
        this.setViewerVisibility(participantId, true);
      }
    }
  }

  /**
   * Remove a viewer's camera
   * @param {string} participantId
   */
  removeViewer(participantId) {
    const viewerData = this.viewerCameras.get(participantId);
    if (!viewerData) return;

    // Remove from scene
    this.scene.remove(viewerData.mesh);

    // Dispose resources
    viewerData.mesh.traverse((child) => {
      if (child.geometry) child.geometry.dispose();
      if (child.material) {
        if (child.material.map) child.material.map.dispose();
        child.material.dispose();
      }
    });

    // Dispose label
    if (viewerData.label && viewerData.label.material) {
      if (viewerData.label.material.map) viewerData.label.material.map.dispose();
      viewerData.label.material.dispose();
    }

    this.viewerCameras.delete(participantId);
    console.log(`[ViewerCameraManager] Removed camera for ${participantId}`);
  }

  /**
   * Update all viewer cameras from participants map
   * @param {Object} participants - Map of participantId -> participant data
   * @param {string} selfId - Local viewer's ID
   */
  updateFromParticipants(participants, selfId) {
    this.setSelfId(selfId);

    // Track which participants we've seen
    const seenIds = new Set();

    // Synchronize hiddenFollowers from participants state
    // This is the authoritative source - ensures visibility is always correct
    // regardless of whether socket events were received
    const newHiddenFollowers = new Set();
    for (const [id, participant] of Object.entries(participants)) {
      if (id === selfId) continue;
      // If this participant is following someone, they should be hidden for everyone
      if (participant.followingId !== null && participant.followingId !== undefined) {
        newHiddenFollowers.add(id);
      }
    }

    // Update visibility for any changes
    // Check for newly hidden followers
    for (const id of newHiddenFollowers) {
      if (!this.hiddenFollowers.has(id)) {
        const viewerData = this.viewerCameras.get(id);
        console.log(
          `[ViewerCameraManager] *** HIDING follower ${viewerData?.nickname || id} (mesh exists: ${!!viewerData})`,
        );
        this.setViewerVisibility(id, false);
      }
    }
    // Check for followers who stopped following
    for (const id of this.hiddenFollowers) {
      if (!newHiddenFollowers.has(id) && id !== this.hiddenViewerId) {
        console.log(`[ViewerCameraManager] Participant ${id} stopped following - showing`);
        this.setViewerVisibility(id, true);
      }
    }
    this.hiddenFollowers = newHiddenFollowers;

    // Add/update cameras for all participants
    for (const [id, participant] of Object.entries(participants)) {
      if (id === selfId) continue;

      seenIds.add(id);
      this.addOrUpdateViewer(id, participant);
    }

    // Remove cameras for participants who left
    for (const id of this.viewerCameras.keys()) {
      if (!seenIds.has(id)) {
        this.removeViewer(id);
      }
    }
  }

  /**
   * Update each frame - interpolates positions/rotations and updates label sizes
   * Should be called from the render loop
   */
  update() {
    if (!this.camera) return;

    const aspectRatio = this.canvasWidth / this.canvasHeight;
    const proximityThreshold = 800; // Distance at which size starts growing

    this.viewerCameras.forEach((viewerData, participantId) => {
      if (!viewerData.mesh) return;

      // Enforce visibility state every frame (handles timing edge cases)
      const shouldBeHidden =
        participantId === this.hiddenViewerId || this.hiddenFollowers.has(participantId);
      if (shouldBeHidden && viewerData.mesh.visible) {
        viewerData.mesh.visible = false;
      }

      // Only interpolate if we've received at least one update
      if (viewerData.hasReceivedUpdate) {
        // Interpolate position (lerp)
        viewerData.mesh.position.lerp(viewerData.targetPosition, this.positionLerpFactor);

        // Interpolate rotation (slerp)
        viewerData.mesh.quaternion.slerp(viewerData.targetQuaternion, this.rotationSlerpFactor);
      }

      // Update label size based on distance
      if (viewerData.label) {
        // Calculate distance from camera to viewer drone
        const distance = this.camera.position.distanceTo(viewerData.mesh.position);

        // Hybrid size behavior (same as NameTagManager):
        // - Far away: constant screen size (base scale)
        // - Close up: grows on screen (world-space size)
        if (distance < proximityThreshold) {
          // Close: scale increases as we get closer (simulates world-space size)
          const growthFactor = proximityThreshold / Math.max(distance, 100);
          const closeScale = this.labelScale * growthFactor;
          viewerData.label.scale.set(closeScale * aspectRatio, closeScale, 1);
        } else {
          // Far: constant screen size
          viewerData.label.scale.set(this.labelScale * aspectRatio, this.labelScale, 1);
        }
      }
    });
  }

  /**
   * Reset all viewer cameras
   */
  reset() {
    for (const id of [...this.viewerCameras.keys()]) {
      this.removeViewer(id);
    }
    this.viewerCameras.clear();
    this.selfId = null;
    this.hiddenViewerId = null;
    this.hiddenFollowers.clear();
    this.pendingCreations.clear();
  }

  /**
   * Dispose of all resources
   */
  dispose() {
    this.reset();

    // Dispose model template
    if (this.modelTemplate) {
      this.modelTemplate.traverse((child) => {
        if (child.geometry) child.geometry.dispose();
        if (child.material) {
          if (child.material.map) child.material.map.dispose();
          child.material.dispose();
        }
      });
      this.modelTemplate = null;
    }
  }
}
