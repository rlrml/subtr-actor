import * as THREE from 'three';
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js';
import { DRACOLoader } from 'three/examples/jsm/loaders/DRACOLoader.js';
import { clone as skeletonClone } from 'three/examples/jsm/utils/SkeletonUtils.js';

/**
 * KeyframeVisualizer
 *
 * Visualizes camera keyframes in 3D space for cinematic mode editing.
 * Shows keyframe markers, path curve, and handles click selection.
 * Uses the camera_drone.glb model for realistic camera representation.
 *
 * Feature: 024-clip-system (US2 - Cinematic Mode)
 */

export class KeyframeVisualizer {
  constructor(scene, camera, renderer) {
    this.scene = scene;
    this.camera = camera;
    this.renderer = renderer;

    // Keyframe data
    this.keyframes = [];

    // 3D objects
    this.markerGroup = new THREE.Group();
    this.markerGroup.name = 'KeyframeMarkers';
    this.pathLine = null;
    this.pathTube = null;
    this.pathDash = null;

    // Marker meshes for raycasting
    this.markerMeshes = [];

    // Selection state
    this.selectedKeyframeId = null;

    // Active keyframe state (for "View" mode - hides marker to prevent obstruction)
    // 026-clip-editor-redesign: T040
    this.activeKeyframeId = null;

    // Marker offset - set to 0 since ghost camera shows real position
    // 026-clip-editor-redesign: Diamond markers at exact position
    this.markerVerticalOffset = 0;

    // Preview camera (animated ghost that follows trajectory)
    // 026-clip-editor-redesign: Animated preview camera
    this.previewCamera = null;
    this.previewCameraVisible = false;
    this.previewCameraColor = 0xf97316; // Orange for distinction
    this._previewCurve = null; // Cached Catmull-Rom curve for preview
    this._previewQuatA = new THREE.Quaternion();
    this._previewQuatB = new THREE.Quaternion();

    // Raycaster for click detection
    this.raycaster = new THREE.Raycaster();
    this.raycaster.params.Points = { threshold: 10 };

    // Visual settings
    // 026-clip-editor-redesign: Orange diamond markers to match 2D UI
    this.markerColor = 0xf97316; // Orange (matches 2D timeline keyframes)
    this.selectedMarkerColor = 0x22c55e; // Green for selected
    this.pathColor = 0xa855f7; // Brighter purple
    this.pathWidth = 8; // Thicker path for better visibility
    this.diamondSize = 25; // Size of diamond marker
    this.modelScale = 200; // Scale for ghost camera model

    // Model loading
    this.gltfLoader = new GLTFLoader();
    const dracoLoader = new DRACOLoader();
    dracoLoader.setDecoderPath('https://www.gstatic.com/draco/versioned/decoders/1.5.6/');
    this.gltfLoader.setDRACOLoader(dracoLoader);
    this.modelTemplate = null;
    this.modelLoading = null;

    // Temporary vectors
    this._tempVec = new THREE.Vector3();
    this._tempMouse = new THREE.Vector2();

    // Add to scene
    this.scene.add(this.markerGroup);

    // Load camera model
    this._loadCameraModel();
  }

  /**
   * Load the camera drone model
   * @private
   */
  async _loadCameraModel() {
    if (this.modelLoading) return this.modelLoading;

    this.modelLoading = new Promise((resolve) => {
      this.gltfLoader.load(
        '/models/camera/camera_drone.glb',
        (gltf) => {
          console.log('[KeyframeVisualizer] Loaded camera model');
          this.modelTemplate = gltf.scene;
          this.modelTemplate.scale.setScalar(this.modelScale);

          // Setup materials
          this.modelTemplate.traverse((child) => {
            if (child.isMesh) {
              child.frustumCulled = false;
              if (child.material) {
                child.userData.originalMaterial = child.material.clone();
              }
            }
          });

          // Rebuild visualization if keyframes exist
          if (this.keyframes.length > 0) {
            this._rebuildVisualization();
          }

          resolve(this.modelTemplate);
        },
        undefined,
        (error) => {
          console.warn('[KeyframeVisualizer] Failed to load camera model, using fallback:', error);
          resolve(null);
        }
      );
    });

    return this.modelLoading;
  }

  /**
   * Update keyframes and rebuild visualization
   * @param {Array} keyframes - Array of CameraKeyframe objects
   */
  setKeyframes(keyframes) {
    this.keyframes = keyframes || [];
    this._rebuildVisualization();
  }

  /**
   * Get keyframes
   * @returns {Array}
   */
  getKeyframes() {
    return this.keyframes;
  }

  /**
   * Add a new keyframe
   * @param {Object} keyframe - CameraKeyframe object
   */
  addKeyframe(keyframe) {
    this.keyframes.push(keyframe);
    // Sort by time
    this.keyframes.sort((a, b) => a.t - b.t);
    this._rebuildVisualization();
    return keyframe;
  }

  /**
   * Remove a keyframe by ID
   * @param {string} id - Keyframe ID
   */
  removeKeyframe(id) {
    const index = this.keyframes.findIndex((kf) => kf.id === id);
    if (index !== -1) {
      this.keyframes.splice(index, 1);
      this._rebuildVisualization();

      // Clear selection if removed keyframe was selected
      if (this.selectedKeyframeId === id) {
        this.selectedKeyframeId = null;
      }
    }
  }

  /**
   * Clear all keyframes, markers, path, and hide preview camera
   * Used when user clicks "Clear all" button
   */
  clearAllKeyframes() {
    // Clear keyframes array
    this.keyframes = [];
    this.selectedKeyframeId = null;
    this.activeKeyframeId = null; // Reset active keyframe to prevent stale references

    // Clear all visual elements
    this._clearMarkers();
    this._clearPath();

    // Hide and dispose preview camera
    this.hidePreviewCamera();

    // Reset markerGroup visibility for next use
    this.markerGroup.visible = true;
  }

  /**
   * Update a keyframe
   * @param {string} id - Keyframe ID
   * @param {Object} updates - Partial keyframe data
   */
  updateKeyframe(id, updates) {
    const keyframe = this.keyframes.find((kf) => kf.id === id);
    if (keyframe) {
      Object.assign(keyframe, updates);
      // Re-sort if time changed
      if (updates.t !== undefined) {
        this.keyframes.sort((a, b) => a.t - b.t);
      }
      this._rebuildVisualization();
    }
  }

  /**
   * Select a keyframe
   * @param {string} id - Keyframe ID or null to deselect
   */
  selectKeyframe(id) {
    this.selectedKeyframeId = id;
    this._updateMarkerColors();
  }

  /**
   * Set the active keyframe (for "View" mode)
   * The active keyframe's marker is hidden to prevent camera obstruction
   * 026-clip-editor-redesign: T041
   * @param {string|null} id - Keyframe ID to set as active, or null to clear
   */
  setActiveKeyframe(id) {
    this.activeKeyframeId = id;
    this._updateMarkerVisibility();
  }

  /**
   * Get the currently active keyframe ID
   * @returns {string|null}
   */
  getActiveKeyframeId() {
    return this.activeKeyframeId;
  }

  /**
   * Show the animated preview camera
   * 026-clip-editor-redesign: Animated preview camera
   */
  showPreviewCamera() {
    if (this.keyframes.length < 2) return;

    if (!this.previewCamera) {
      this._createPreviewCamera();
    }

    if (this.previewCamera) {
      this.previewCamera.visible = true;
      this.previewCameraVisible = true;
    }
  }

  /**
   * Hide the animated preview camera
   */
  hidePreviewCamera() {
    if (this.previewCamera) {
      this.previewCamera.visible = false;
    }
    this.previewCameraVisible = false;
  }

  /**
   * Update preview camera visibility based on distance to user camera
   * Hides the ghost camera when user is too close (to avoid being inside the mesh)
   * @param {THREE.Camera} userCamera - The user's camera
   * @param {number} minDistance - Minimum distance before hiding (default 150 units)
   */
  updatePreviewCameraVisibility(userCamera, minDistance = 150) {
    if (!this.previewCamera || !this.previewCameraVisible || !userCamera) return;

    const distance = userCamera.position.distanceTo(this.previewCamera.position);
    // Hide if user is too close, show otherwise
    this.previewCamera.visible = distance > minDistance;
  }

  /**
   * Update the preview camera position based on current time
   * Uses Catmull-Rom spline interpolation segment by segment
   * @param {number} timeMs - Current time in ms (absolute)
   */
  updatePreviewCamera(timeMs) {
    if (!this.previewCamera || !this.previewCameraVisible || this.keyframes.length < 2) {
      return;
    }

    // Get keyframe time range
    const startTime = this.keyframes[0].t;
    const endTime = this.keyframes[this.keyframes.length - 1].t;

    // Clamp time to keyframe range
    const clampedTime = Math.max(startTime, Math.min(endTime, timeMs));

    // Find which segment we're in based on time (use < not <= to stay on current segment at exact keyframe time)
    let segmentIndex = 0;
    while (
      segmentIndex < this.keyframes.length - 1 &&
      this.keyframes[segmentIndex + 1].t < clampedTime
    ) {
      segmentIndex++;
    }

    // Edge case: at or past the last keyframe
    if (segmentIndex >= this.keyframes.length - 1) {
      const kf = this.keyframes[this.keyframes.length - 1];
      this.previewCamera.position.set(kf.px, kf.py, kf.pz);
      this.previewCamera.quaternion.set(kf.qx, kf.qy, kf.qz, kf.qw);
      return;
    }

    const kfA = this.keyframes[segmentIndex];
    const kfB = this.keyframes[segmentIndex + 1];

    // Calculate local t within this segment (0-1)
    const segmentDuration = kfB.t - kfA.t;
    const localT = segmentDuration > 0 ? (clampedTime - kfA.t) / segmentDuration : 0;

    // At exact keyframe positions, use the exact keyframe data
    if (localT <= 0.001) {
      this.previewCamera.position.set(kfA.px, kfA.py, kfA.pz);
      this.previewCamera.quaternion.set(kfA.qx, kfA.qy, kfA.qz, kfA.qw);
      return;
    }
    if (localT >= 0.999) {
      this.previewCamera.position.set(kfB.px, kfB.py, kfB.pz);
      this.previewCamera.quaternion.set(kfB.qx, kfB.qy, kfB.qz, kfB.qw);
      return;
    }

    // Use Catmull-Rom interpolation with 4 control points for smooth curves
    // Get the 4 points: p0, p1, p2, p3 where we interpolate between p1 and p2
    const p0 = segmentIndex > 0 ? this.keyframes[segmentIndex - 1] : kfA;
    const p1 = kfA;
    const p2 = kfB;
    const p3 = segmentIndex < this.keyframes.length - 2 ? this.keyframes[segmentIndex + 2] : kfB;

    // Catmull-Rom interpolation for position
    const pos = this._catmullRomInterpolate(
      new THREE.Vector3(p0.px, p0.py, p0.pz),
      new THREE.Vector3(p1.px, p1.py, p1.pz),
      new THREE.Vector3(p2.px, p2.py, p2.pz),
      new THREE.Vector3(p3.px, p3.py, p3.pz),
      localT
    );
    this.previewCamera.position.copy(pos);

    // Interpolate rotation (SLERP between the two keyframes)
    this._previewQuatA.set(kfA.qx, kfA.qy, kfA.qz, kfA.qw);
    this._previewQuatB.set(kfB.qx, kfB.qy, kfB.qz, kfB.qw);
    this.previewCamera.quaternion.slerpQuaternions(this._previewQuatA, this._previewQuatB, localT);
  }

  /**
   * Catmull-Rom spline interpolation between p1 and p2
   * @private
   * @param {THREE.Vector3} p0 - Control point before p1
   * @param {THREE.Vector3} p1 - Start point
   * @param {THREE.Vector3} p2 - End point
   * @param {THREE.Vector3} p3 - Control point after p2
   * @param {number} t - Interpolation factor (0-1)
   * @returns {THREE.Vector3}
   */
  _catmullRomInterpolate(p0, p1, p2, p3, t) {
    const t2 = t * t;
    const t3 = t2 * t;

    // Catmull-Rom basis functions (tension = 0.5)
    const v = new THREE.Vector3();
    v.x =
      0.5 *
      (2 * p1.x +
        (-p0.x + p2.x) * t +
        (2 * p0.x - 5 * p1.x + 4 * p2.x - p3.x) * t2 +
        (-p0.x + 3 * p1.x - 3 * p2.x + p3.x) * t3);
    v.y =
      0.5 *
      (2 * p1.y +
        (-p0.y + p2.y) * t +
        (2 * p0.y - 5 * p1.y + 4 * p2.y - p3.y) * t2 +
        (-p0.y + 3 * p1.y - 3 * p2.y + p3.y) * t3);
    v.z =
      0.5 *
      (2 * p1.z +
        (-p0.z + p2.z) * t +
        (2 * p0.z - 5 * p1.z + 4 * p2.z - p3.z) * t2 +
        (-p0.z + 3 * p1.z - 3 * p2.z + p3.z) * t3);

    return v;
  }

  /**
   * Build Catmull-Rom curve for preview camera
   * @private
   */
  _buildPreviewCurve() {
    if (this.keyframes.length < 2) {
      this._previewCurve = null;
      return;
    }

    const points = this.keyframes.map((kf) => new THREE.Vector3(kf.px, kf.py, kf.pz));
    this._previewCurve = new THREE.CatmullRomCurve3(points);
    this._previewCurve.curveType = 'catmullrom';
    this._previewCurve.tension = 0.5;
  }

  /**
   * Create the preview camera mesh
   * @private
   */
  _createPreviewCamera() {
    if (this.previewCamera) {
      this.scene.remove(this.previewCamera);
      this._disposePreviewCamera();
    }

    const container = new THREE.Group();
    container.name = 'PreviewCamera';

    // Create camera mesh (use loaded model or fallback)
    let cameraMesh;
    if (this.modelTemplate) {
      cameraMesh = skeletonClone(this.modelTemplate);
      cameraMesh.scale.setScalar(this.modelScale * 1.2); // Slightly larger
      cameraMesh.rotation.y = Math.PI;

      // Apply orange tint for distinction
      cameraMesh.traverse((child) => {
        if (child.isMesh && child.material) {
          child.material = child.material.clone();
          if (child.material.color) {
            child.material.color.setHex(this.previewCameraColor);
          }
          if (child.material.emissive) {
            child.material.emissive.setHex(this.previewCameraColor);
            child.material.emissiveIntensity = 0.5;
          }
          child.material.transparent = true;
          child.material.opacity = 0.9;
          child.frustumCulled = false;
        }
      });
    } else {
      cameraMesh = this._createFallbackCamera(this.previewCameraColor);
      cameraMesh.scale.setScalar(1.2);
    }

    container.add(cameraMesh);

    // Add glowing sphere behind it for visibility
    const glowGeom = new THREE.SphereGeometry(25, 16, 16);
    const glowMat = new THREE.MeshBasicMaterial({
      color: this.previewCameraColor,
      transparent: true,
      opacity: 0.3,
    });
    const glow = new THREE.Mesh(glowGeom, glowMat);
    container.add(glow);

    // View direction indicator (cone) - opens toward where camera looks
    const coneGeom = new THREE.ConeGeometry(40, 80, 4);
    const coneMat = new THREE.MeshBasicMaterial({
      color: this.previewCameraColor,
      transparent: true,
      opacity: 0.25,
      side: THREE.DoubleSide,
    });
    const cone = new THREE.Mesh(coneGeom, coneMat);
    cone.rotation.x = Math.PI / 2; // Tip at camera, opens toward view direction
    cone.position.z = -40;
    container.add(cone);

    this.previewCamera = container;
    this.previewCamera.visible = false;
    this.scene.add(this.previewCamera);
  }

  /**
   * Dispose preview camera resources
   * @private
   */
  _disposePreviewCamera() {
    if (!this.previewCamera) return;

    this.previewCamera.traverse((child) => {
      if (child.geometry) child.geometry.dispose();
      if (child.material) {
        if (Array.isArray(child.material)) {
          child.material.forEach((m) => m.dispose());
        } else {
          child.material.dispose();
        }
      }
    });

    this.previewCamera = null;
  }

  /**
   * Toggle visibility of all markers
   * 026-clip-editor-redesign: T043
   * @param {boolean} visible - Whether markers should be visible
   */
  toggleAllMarkers(visible) {
    // First, ensure the markerGroup itself is visible/hidden
    this.markerGroup.visible = visible;

    // Then toggle individual marker visibility
    this.markerGroup.children.forEach((container) => {
      container.visible = visible;
    });
    // Also toggle path visibility
    if (this.pathTube) this.pathTube.visible = visible;
    if (this.pathLine) this.pathLine.visible = visible;
    if (this.pathDash) this.pathDash.visible = visible;
  }

  /**
   * Update marker visibility based on activeKeyframeId
   * @private
   */
  _updateMarkerVisibility() {
    this.markerGroup.children.forEach((container) => {
      const keyframeId = container.userData.keyframeId;
      // Hide the marker if it's the active keyframe (camera is viewing from this position)
      container.visible = keyframeId !== this.activeKeyframeId;
    });
  }

  /**
   * Handle click event and check for keyframe hit
   * @param {number} x - Normalized device X (-1 to 1)
   * @param {number} y - Normalized device Y (-1 to 1)
   * @returns {Object|null} Clicked keyframe or null
   */
  handleClick(x, y) {
    if (this.markerMeshes.length === 0) return null;

    this._tempMouse.set(x, y);
    this.raycaster.setFromCamera(this._tempMouse, this.camera);

    const intersects = this.raycaster.intersectObjects(this.markerMeshes, false);

    if (intersects.length > 0) {
      const clickedMesh = intersects[0].object;
      const keyframeId = clickedMesh.userData.keyframeId;
      return this.keyframes.find((kf) => kf.id === keyframeId) || null;
    }

    return null;
  }

  /**
   * Rebuild all 3D visualization
   * @private
   */
  _rebuildVisualization() {
    // Remember current visibility state before clearing
    const wasMarkerGroupVisible = this.markerGroup.visible;

    // Clear existing
    this._clearMarkers();
    this._clearPath();

    // Invalidate preview curve cache (will rebuild on next updatePreviewCamera call)
    this._previewCurve = null;

    if (this.keyframes.length === 0) return;

    // Create markers
    this._createMarkers();

    // Create path if enough keyframes
    if (this.keyframes.length >= 2) {
      this._createPath();
    }

    // Restore markerGroup visibility (it gets reset when children are cleared)
    this.markerGroup.visible = wasMarkerGroupVisible;

    // Also set path visibility to match markerGroup
    if (this.pathTube) this.pathTube.visible = wasMarkerGroupVisible;
    if (this.pathLine) this.pathLine.visible = wasMarkerGroupVisible;
    if (this.pathDash) this.pathDash.visible = wasMarkerGroupVisible;

    // Update preview camera if visible
    if (this.previewCameraVisible && this.keyframes.length >= 2) {
      if (!this.previewCamera) {
        this._createPreviewCamera();
        this.previewCamera.visible = true;
      }
    }
  }

  /**
   * Create a fallback camera mesh when GLB model is not loaded
   * @private
   * @param {number} color - Hex color
   * @returns {THREE.Group}
   */
  _createFallbackCamera(color) {
    const group = new THREE.Group();

    // Camera body
    const bodyGeom = new THREE.BoxGeometry(20, 14, 12);
    const bodyMat = new THREE.MeshBasicMaterial({ color, transparent: true, opacity: 0.9 });
    const body = new THREE.Mesh(bodyGeom, bodyMat);
    group.add(body);

    // Lens
    const lensGeom = new THREE.CylinderGeometry(5, 7, 8, 8);
    const lensMat = new THREE.MeshBasicMaterial({ color: 0x333333 });
    const lens = new THREE.Mesh(lensGeom, lensMat);
    lens.rotation.x = Math.PI / 2;
    lens.position.z = -10;
    group.add(lens);

    return group;
  }

  /**
   * Create marker meshes for each keyframe
   * 026-clip-editor-redesign: Orange diamond markers with view cone
   * @private
   */
  _createMarkers() {
    this.keyframes.forEach((kf, index) => {
      const isSelected = kf.id === this.selectedKeyframeId;
      const isActive = kf.id === this.activeKeyframeId;
      const color = isSelected ? this.selectedMarkerColor : this.markerColor;

      // Create container group at exact keyframe position
      const container = new THREE.Group();
      container.position.set(kf.px, kf.py + this.markerVerticalOffset, kf.pz);
      container.quaternion.set(kf.qx, kf.qy, kf.qz, kf.qw);
      container.userData.keyframeId = kf.id;
      container.userData.keyframeIndex = index;

      // Hide marker if it's the active keyframe (camera viewing from this position)
      container.visible = !isActive;

      // Create diamond marker (octahedron = 3D rhombus/losange)
      const diamondGeom = new THREE.OctahedronGeometry(this.diamondSize);
      const diamondMat = new THREE.MeshBasicMaterial({
        color,
        transparent: true,
        opacity: 0.9,
      });
      const diamond = new THREE.Mesh(diamondGeom, diamondMat);
      diamond.userData.keyframeId = kf.id;
      diamond.frustumCulled = false;
      this.markerMeshes.push(diamond);
      container.add(diamond);

      // Diamond wireframe edges for better visibility
      const diamondWireGeom = new THREE.OctahedronGeometry(this.diamondSize * 1.05);
      const diamondWireMat = new THREE.MeshBasicMaterial({
        color: 0xffffff,
        wireframe: true,
        transparent: true,
        opacity: 0.6,
      });
      const diamondWire = new THREE.Mesh(diamondWireGeom, diamondWireMat);
      container.add(diamondWire);

      // View direction cone (frustum showing where camera points)
      // Tip at camera position, opens toward view direction
      const coneGeom = new THREE.ConeGeometry(50, 120, 4);
      const coneMat = new THREE.MeshBasicMaterial({
        color,
        transparent: true,
        opacity: 0.15,
        side: THREE.DoubleSide,
      });
      const cone = new THREE.Mesh(coneGeom, coneMat);
      cone.rotation.x = Math.PI / 2; // Tip points backward (+Z), base opens forward (-Z)
      cone.position.z = -60; // Position so tip is near diamond, base opens forward
      container.add(cone);

      // Wireframe edges for the view cone
      const wireGeom = new THREE.ConeGeometry(50, 120, 4);
      const wireMat = new THREE.MeshBasicMaterial({
        color,
        wireframe: true,
        transparent: true,
        opacity: 0.4,
      });
      const wire = new THREE.Mesh(wireGeom, wireMat);
      wire.rotation.x = Math.PI / 2;
      wire.position.z = -60;
      container.add(wire);

      // Keyframe number label (small sphere with index color)
      const labelColors = [0xff6b6b, 0x4ecdc4, 0xffe66d, 0x95e1d3, 0xf38181, 0xaa96da];
      const labelGeom = new THREE.SphereGeometry(10, 12, 12);
      const labelMat = new THREE.MeshBasicMaterial({
        color: labelColors[index % labelColors.length],
      });
      const label = new THREE.Mesh(labelGeom, labelMat);
      label.position.set(0, this.diamondSize + 15, 0); // Just above diamond
      container.add(label);

      this.markerGroup.add(container);
    });
  }

  /**
   * Create path visualization between keyframes
   * @private
   */
  _createPath() {
    if (this.keyframes.length < 2) return;

    // Extract positions
    const points = this.keyframes.map((kf) => new THREE.Vector3(kf.px, kf.py, kf.pz));

    // Create Catmull-Rom curve
    const curve = new THREE.CatmullRomCurve3(points);
    curve.curveType = 'catmullrom';
    curve.tension = 0.5;

    // Create tube geometry along the curve (outer glow)
    const tubeGeometry = new THREE.TubeGeometry(curve, points.length * 20, this.pathWidth, 8, false);

    const tubeMaterial = new THREE.MeshBasicMaterial({
      color: this.pathColor,
      transparent: true,
      opacity: 0.6,
      side: THREE.DoubleSide,
    });

    this.pathTube = new THREE.Mesh(tubeGeometry, tubeMaterial);
    this.pathTube.name = 'KeyframePath';
    this.scene.add(this.pathTube);

    // Inner bright core line for better visibility
    const linePoints = curve.getPoints(points.length * 30);
    const lineGeometry = new THREE.BufferGeometry().setFromPoints(linePoints);
    const lineMaterial = new THREE.LineBasicMaterial({
      color: 0xffffff, // White core
      transparent: true,
      opacity: 0.9,
      linewidth: 2, // Note: linewidth only works in some renderers
    });

    this.pathLine = new THREE.Line(lineGeometry, lineMaterial);
    this.pathLine.name = 'KeyframePathLine';
    this.scene.add(this.pathLine);

    // Add dashed direction indicators along path
    const dashPoints = curve.getPoints(Math.max(10, points.length * 4));
    const dashGeometry = new THREE.BufferGeometry().setFromPoints(dashPoints);
    const dashMaterial = new THREE.LineDashedMaterial({
      color: 0xffffff,
      transparent: true,
      opacity: 0.5,
      dashSize: 30,
      gapSize: 20,
    });

    this.pathDash = new THREE.Line(dashGeometry, dashMaterial);
    this.pathDash.computeLineDistances(); // Required for dashed lines
    this.pathDash.name = 'KeyframePathDash';
    this.scene.add(this.pathDash);
  }

  /**
   * Update marker colors based on selection
   * @private
   */
  _updateMarkerColors() {
    // For now, rebuild visualization to update colors properly
    // This handles both GLB models and geometric shapes
    this._rebuildVisualization();
  }

  /**
   * Clear all markers
   * @private
   */
  _clearMarkers() {
    // Recursively dispose all children
    const disposeObject = (obj) => {
      if (obj.geometry) obj.geometry.dispose();
      if (obj.material) {
        if (Array.isArray(obj.material)) {
          obj.material.forEach((m) => {
            if (m.map) m.map.dispose();
            m.dispose();
          });
        } else {
          if (obj.material.map) obj.material.map.dispose();
          obj.material.dispose();
        }
      }
      // Dispose children recursively
      while (obj.children && obj.children.length > 0) {
        const child = obj.children[0];
        disposeObject(child);
        obj.remove(child);
      }
    };

    // Dispose and remove all children
    while (this.markerGroup.children.length > 0) {
      const child = this.markerGroup.children[0];
      disposeObject(child);
      this.markerGroup.remove(child);
    }
    this.markerMeshes = [];
  }

  /**
   * Clear path visualization
   * @private
   */
  _clearPath() {
    if (this.pathTube) {
      this.scene.remove(this.pathTube);
      this.pathTube.geometry.dispose();
      this.pathTube.material.dispose();
      this.pathTube = null;
    }

    if (this.pathLine) {
      this.scene.remove(this.pathLine);
      this.pathLine.geometry.dispose();
      this.pathLine.material.dispose();
      this.pathLine = null;
    }

    if (this.pathDash) {
      this.scene.remove(this.pathDash);
      this.pathDash.geometry.dispose();
      this.pathDash.material.dispose();
      this.pathDash = null;
    }
  }

  /**
   * Set visibility
   * @param {boolean} visible
   */
  setVisible(visible) {
    this.markerGroup.visible = visible;
    if (this.pathTube) this.pathTube.visible = visible;
    if (this.pathLine) this.pathLine.visible = visible;
    if (this.pathDash) this.pathDash.visible = visible;
  }

  /**
   * Check if visible
   * @returns {boolean}
   */
  isVisible() {
    return this.markerGroup.visible;
  }

  /**
   * Show the visualizer
   */
  show() {
    this.setVisible(true);
  }

  /**
   * Hide the visualizer
   */
  hide() {
    this.setVisible(false);
  }

  /**
   * Dispose all resources
   */
  dispose() {
    this._clearMarkers();
    this._clearPath();
    this._disposePreviewCamera();

    if (this.previewCamera) {
      this.scene.remove(this.previewCamera);
    }

    this.scene.remove(this.markerGroup);
    this.keyframes = [];
    this.selectedKeyframeId = null;
    this._previewCurve = null;
  }
}
