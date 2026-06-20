import * as THREE from "three";
import CameraControlsModule from "camera-controls";

const CameraControls = CameraControlsModule.default ?? CameraControlsModule;
const FOLLOW_TARGET_HANDOFF_SECONDS = 0.55;

// Install CameraControls with THREE
CameraControls.install({ THREE });

export class CameraManager {
  constructor(camera, domElement) {
    this.camera = camera;
    this.domElement = domElement;

    // Create camera controls
    this.controls = new CameraControls(camera, domElement);

    // Configure controls
    this.controls.dollyToCursor = false;
    this.controls.infinityDolly = false;
    // Default dollySpeed (1) is sluggish at field scale (camera distances run
    // into thousands of UU) — boost wheel zoom responsiveness in free mode.
    this.controls.dollySpeed = 2.5;

    // Smooth damping for transitions - keep it snappy
    this.controls.smoothTime = 0.05; // Very fast transitions
    this.controls.draggingSmoothTime = 0.05;

    // Limit vertical rotation (don't go below ground)
    this.controls.minPolarAngle = 0.1; // Slightly above horizontal
    this.controls.maxPolarAngle = Math.PI / 2 - 0.1; // Don't go below horizon

    // Limit zoom distance (in UU)
    this.controls.minDistance = 100;
    this.controls.maxDistance = 10000;

    // Minimum height above ground (in UU)
    this.minHeight = 50;

    // Camera mode: 'free', 'ball', 'ballOrbit', 'car'
    this.mode = "free";

    // Default freecam position: side view of field, elevated, looking at center (in UU)
    this.defaultFreecamPosition = new THREE.Vector3(0, 1000, 5000);
    this.defaultFreecamLookAt = new THREE.Vector3(0, 100, 0);

    // Pointer lock state callback for UI components
    this.onPointerLockStateChange = null;

    // Target entities
    this.targetCar = null;
    this.targetBall = null;

    // Camera settings for player follow (in UU) - matching Rocket League settings
    this.followDistance = 260; // Distance behind car (RL range: 100-400)
    this.followHeight = 90; // Height above car (RL range: 40-200)
    this.followAngle = -4; // Pitch angle in degrees (RL range: -15 to 0, negative = look down)
    this.stiffness = 0.45; // Camera stiffness (RL range: 0.0-1.0)
    this.swivelSpeed = 4.3; // Rotation speed (RL range: 1.0-10.0)

    // === STATE BLENDING SYSTEM (Rocket League style) ===
    // Instead of interpolating from a fixed start position to a target,
    // we calculate BOTH camera states every frame and blend between them.
    // This ensures smooth transitions even when the car is moving.

    // Blend factor: 0.0 = Car Cam, 1.0 = Ball Cam
    this.currentBlend = 0.0; // Current interpolated blend value
    this.targetBlend = 0.0; // Target blend (0 or 1 based on ball cam toggle)

    // Transition Speed in RL (1.0-2.0): multiplier for base duration
    // Base duration ~0.5s at speed 1.0, ~0.25s at speed 2.0
    this.transitionSpeed = 1.3; // Ball cam transition speed (RL range: 1.0-2.0)
    this.baseDuration = 0.5; // Base transition duration in seconds

    // Track previous state for yaw initialization
    this.lastIsBallCam = null;
    this.targetHandoff = null;

    // Store current interpolated camera state
    this.currentCamPos = null;
    this.currentLookTarget = null;

    // Temporary objects for SLERP calculations (avoid allocations)
    this._tempQuatCarCam = new THREE.Quaternion();
    this._tempQuatBallCam = new THREE.Quaternion();
    this._tempMatrix = new THREE.Matrix4();

    // Enable user interaction in free mode
    this.controls.enabled = true;

    // Disable right-click context menu
    domElement.addEventListener("contextmenu", (e) => e.preventDefault());

    // Following mode state (for collab - following another viewer's camera)
    this.isFollowingViewer = false;
    this.followTargetPosition = new THREE.Vector3();
    this.followTargetQuaternion = new THREE.Quaternion();
    this.followPositionLerpFactor = 0.12; // Lower = smoother but more lag
    this.followRotationSlerpFactor = 0.1; // Lower = smoother rotation
    this.hasFollowTarget = false; // Whether we have received a follow target yet

    // Right-click drag state for free camera (Roblox-style)
    this.isRightMouseDown = false;
    this.lastMouseX = null;
    this.lastMouseY = null;

    // Saved camera state for replay mode (v9 protocol)
    this.savedCameraState = null;
    this.isInReplayMode = false;
  }

  /**
   * Set camera mode
   * @param {'free' | 'ball' | 'ballOrbit' | 'car'} mode
   */
  setMode(mode) {
    this.mode = mode;

    if (mode === "ballOrbit") {
      // Ball orbit mode - orbit around the ball with mouse, zoom with scroll
      this.controls.enabled = true;

      // Reset ball tracking for smooth following
      this.lastBallOrbitPos = null;

      // Setup scroll wheel zoom handler if not already setup
      if (!this.ballOrbitScrollHandler) {
        this.ballOrbitScrollHandler = (e) => {
          // Disable scroll zoom when following another viewer
          if (this.mode === "ballOrbit" && !this.isFollowingViewer) {
            e.preventDefault();
            // Dolly in/out proportionally to current distance — a fixed UU step
            // is imperceptible at field scale (orbit distances run 1000s of UU).
            const zoomStep = Math.max(this.controls.distance * 0.2, 100);
            if (e.deltaY > 0) {
              this.controls.dolly(-zoomStep, true);
            } else {
              this.controls.dolly(zoomStep, true);
            }
          }
        };
        this.domElement.addEventListener("wheel", this.ballOrbitScrollHandler, { passive: false });
      }

      // Position camera at a reasonable distance if we have a ball target
      if (this.targetBall) {
        const ballPos = this.targetBall.position;
        // Keep current camera distance from ball or use default (in UU)
        const currentDist = this.camera.position.distanceTo(ballPos);
        const targetDist = currentDist > 100 ? currentDist : 2000;

        // Move to current position but set target to ball
        this.controls.setLookAt(
          this.camera.position.x,
          this.camera.position.y,
          this.camera.position.z,
          ballPos.x,
          ballPos.y,
          ballPos.z,
          false,
        );
      }

      return;
    }

    if (mode === "free") {
      // Free camera mode - right-click drag rotation (Roblox-style)
      // Disable camera-controls orbit, we'll handle movement manually
      this.controls.enabled = false;

      // Initialize keyboard state for free cam movement
      if (!this.freeCamKeys) {
        this.freeCamKeys = {
          forward: false,
          backward: false,
          left: false,
          right: false,
          up: false,
          down: false,
        };
        this.freeCamSpeed = 2000; // Unreal Units per second (configurable)
        this.freeCamRotation = { yaw: 0, pitch: 0 };

        // Calculate initial yaw/pitch from current camera orientation
        const dir = new THREE.Vector3();
        this.camera.getWorldDirection(dir);
        this.freeCamRotation.yaw = Math.atan2(dir.x, dir.z);
        this.freeCamRotation.pitch = Math.asin(-dir.y);

        // Setup keyboard listeners
        this.onKeyDown = (e) => this.handleFreeCamKeyDown(e);
        this.onKeyUp = (e) => this.handleFreeCamKeyUp(e);
        this.onMouseMove = (e) => this.handleFreeCamMouseMove(e);

        // Right-click drag handlers (Roblox-style with pointer lock)
        this.onMouseDown = (e) => {
          if (e.button === 2 && this.mode === "free" && !this.isFollowingViewer) {
            this.isRightMouseDown = true;
            // Request pointer lock for smooth camera rotation
            this.domElement.requestPointerLock?.();
          }
        };
        this.onMouseUp = (e) => {
          if (e.button === 2) {
            this.isRightMouseDown = false;
            // Release pointer lock
            if (document.pointerLockElement === this.domElement) {
              document.exitPointerLock?.();
            }
          }
        };
        this.onPointerLockChange = () => {
          // If pointer lock was released externally (e.g., Escape key), reset drag state
          if (document.pointerLockElement !== this.domElement) {
            this.isRightMouseDown = false;
          }
        };
        this.onMouseLeave = () => {
          // Only reset if not pointer locked (pointer lock handles edge of screen)
          if (document.pointerLockElement !== this.domElement) {
            this.isRightMouseDown = false;
          }
        };
        this.onWindowBlur = () => {
          // Reset all input state when window loses focus
          this.isRightMouseDown = false;
          // Release pointer lock if active
          if (document.pointerLockElement === this.domElement) {
            document.exitPointerLock?.();
          }
          // Reset all keyboard keys to prevent stuck movement
          if (this.freeCamKeys) {
            this.freeCamKeys.forward = false;
            this.freeCamKeys.backward = false;
            this.freeCamKeys.left = false;
            this.freeCamKeys.right = false;
            this.freeCamKeys.up = false;
            this.freeCamKeys.down = false;
          }
        };
        this.onVisibilityChange = () => {
          // Reset all input state when tab becomes hidden
          if (document.hidden) {
            this.isRightMouseDown = false;
            // Release pointer lock if active
            if (document.pointerLockElement === this.domElement) {
              document.exitPointerLock?.();
            }
            if (this.freeCamKeys) {
              this.freeCamKeys.forward = false;
              this.freeCamKeys.backward = false;
              this.freeCamKeys.left = false;
              this.freeCamKeys.right = false;
              this.freeCamKeys.up = false;
              this.freeCamKeys.down = false;
            }
          }
        };

        document.addEventListener("keydown", this.onKeyDown);
        document.addEventListener("keyup", this.onKeyUp);
        document.addEventListener("mousemove", this.onMouseMove);
        this.domElement.addEventListener("mousedown", this.onMouseDown);
        document.addEventListener("mouseup", this.onMouseUp);
        document.addEventListener("pointerlockchange", this.onPointerLockChange);
        this.domElement.addEventListener("mouseleave", this.onMouseLeave);
        window.addEventListener("blur", this.onWindowBlur);
        document.addEventListener("visibilitychange", this.onVisibilityChange);
      }

      // Reset right-click drag state when entering free mode
      this.isRightMouseDown = false;
    } else {
      // Disable user rotation in follow modes
      this.controls.enabled = false;
      // Reset state blending when switching to player mode
      this.lastIsBallCam = null;
      // Initialize blend to car cam (will transition to ball cam if needed)
      this.currentBlend = 0.0;
      this.targetBlend = 0.0;
    }
  }

  /**
   * Set target car mesh to follow
   */
  setTargetCar(carMesh) {
    // Reset ball cam angle when changing target car
    if (this.targetCar !== carMesh) {
      this.currentBallCamAngle = null;
      if (this.targetCar && carMesh) {
        this.targetHandoff = {
          elapsed: 0,
          duration: FOLLOW_TARGET_HANDOFF_SECONDS,
          startPosition: this.camera.position.clone(),
          startQuaternion: this.camera.quaternion.clone(),
        };

        const carToCamera = new THREE.Vector3().subVectors(this.camera.position, carMesh.position);
        carToCamera.y = 0;
        if (carToCamera.length() > 0.01) {
          carToCamera.normalize();
          this.smoothedCarYaw = Math.atan2(-carToCamera.x, -carToCamera.z);
        }
        if (this.lastCarPos) {
          this.lastCarPos.copy(carMesh.position);
        }
      }
    }
    this.targetCar = carMesh;
  }

  /**
   * Set target ball mesh
   */
  setTargetBall(ballMesh) {
    this.targetBall = ballMesh;
  }

  /**
   * Handle keydown for free camera
   */
  handleFreeCamKeyDown(e) {
    if (this.mode !== "free" || this.isFollowingViewer) return;

    // Ignore keyboard input when user is typing in an input field
    const target = e.target;
    if (target.tagName === "INPUT" || target.tagName === "TEXTAREA" || target.isContentEditable) {
      return;
    }

    switch (e.code) {
      case "KeyW":
      case "ArrowUp":
        this.freeCamKeys.forward = true;
        break;
      case "KeyS":
      case "ArrowDown":
        this.freeCamKeys.backward = true;
        break;
      case "KeyA":
      case "ArrowLeft":
        this.freeCamKeys.left = true;
        break;
      case "KeyD":
      case "ArrowRight":
        this.freeCamKeys.right = true;
        break;
      case "Space":
        this.freeCamKeys.up = true;
        break;
      case "ShiftLeft":
      case "ShiftRight":
        this.freeCamKeys.down = true;
        break;
    }
  }

  /**
   * Handle keyup for free camera
   */
  handleFreeCamKeyUp(e) {
    switch (e.code) {
      case "KeyW":
      case "ArrowUp":
        this.freeCamKeys.forward = false;
        break;
      case "KeyS":
      case "ArrowDown":
        this.freeCamKeys.backward = false;
        break;
      case "KeyA":
      case "ArrowLeft":
        this.freeCamKeys.left = false;
        break;
      case "KeyD":
      case "ArrowRight":
        this.freeCamKeys.right = false;
        break;
      case "Space":
        this.freeCamKeys.up = false;
        break;
      case "ShiftLeft":
      case "ShiftRight":
        this.freeCamKeys.down = false;
        break;
    }
  }

  /**
   * Handle mouse movement for free camera look (right-click drag style with pointer lock)
   */
  handleFreeCamMouseMove(e) {
    if (this.mode !== "free" || !this.isRightMouseDown || this.isFollowingViewer) return;

    // Use movementX/Y when pointer is locked (more reliable, no edge issues)
    const deltaX = e.movementX || 0;
    const deltaY = e.movementY || 0;

    const sensitivity = 0.003;
    this.freeCamRotation.yaw -= deltaX * sensitivity;
    this.freeCamRotation.pitch += deltaY * sensitivity;

    // Clamp pitch to prevent flipping
    this.freeCamRotation.pitch = Math.max(
      -Math.PI / 2 + 0.01,
      Math.min(Math.PI / 2 - 0.01, this.freeCamRotation.pitch),
    );
  }

  /**
   * Update free camera movement
   */
  updateFreeCam(delta) {
    if (!this.freeCamKeys) return;

    // Calculate look direction (where camera is pointing) - for rendering
    const lookDir = new THREE.Vector3(
      Math.sin(this.freeCamRotation.yaw) * Math.cos(this.freeCamRotation.pitch),
      -Math.sin(this.freeCamRotation.pitch),
      Math.cos(this.freeCamRotation.yaw) * Math.cos(this.freeCamRotation.pitch),
    );
    lookDir.normalize();

    // For movement, use separate vectors at full speed
    // Forward direction (3D, follows camera pitch)
    const forward = new THREE.Vector3(
      Math.sin(this.freeCamRotation.yaw) * Math.cos(this.freeCamRotation.pitch),
      -Math.sin(this.freeCamRotation.pitch),
      Math.cos(this.freeCamRotation.yaw) * Math.cos(this.freeCamRotation.pitch),
    );
    forward.normalize();

    // Right vector (always horizontal)
    const right = new THREE.Vector3(
      Math.sin(this.freeCamRotation.yaw - Math.PI / 2),
      0,
      Math.cos(this.freeCamRotation.yaw - Math.PI / 2),
    );
    // Already normalized (sin² + cos² = 1)

    // Up vector (always vertical)
    const up = new THREE.Vector3(0, 1, 0);

    // Build velocity from inputs
    const velocity = new THREE.Vector3();
    const speed = this.freeCamSpeed * delta;

    if (this.freeCamKeys.forward) velocity.add(forward.clone().multiplyScalar(speed));
    if (this.freeCamKeys.backward) velocity.add(forward.clone().multiplyScalar(-speed));
    if (this.freeCamKeys.right) velocity.add(right.clone().multiplyScalar(speed));
    if (this.freeCamKeys.left) velocity.add(right.clone().multiplyScalar(-speed));
    if (this.freeCamKeys.up) velocity.add(up.clone().multiplyScalar(speed));
    if (this.freeCamKeys.down) velocity.add(up.clone().multiplyScalar(-speed));

    // Normalize combined velocity to prevent faster diagonal movement, then apply speed
    if (velocity.length() > 0) {
      velocity.normalize().multiplyScalar(speed);
    }

    // Apply movement
    this.camera.position.add(velocity);

    // Apply rotation (look direction)
    const lookTarget = this.camera.position.clone().add(lookDir);
    this.camera.lookAt(lookTarget);

    // Update controls internal state to match
    this.controls.setLookAt(
      this.camera.position.x,
      this.camera.position.y,
      this.camera.position.z,
      lookTarget.x,
      lookTarget.y,
      lookTarget.z,
      false,
    );
  }

  /**
   * Update camera - call every frame
   * @param {number} delta - Time since last frame in seconds
   * @param {boolean} isBallCam - Whether to use ball cam or car cam
   */
  update(delta, isBallCam = true) {
    // If following another viewer, only update interpolation (no controls update)
    if (this.isFollowingViewer) {
      this.updateFollowInterpolation(delta);
      // Don't call controls.update() - we handle camera position/rotation directly
      return;
    }

    if (this.mode === "free") {
      // Free camera - FPS-style movement
      this.updateFreeCam(delta);
      this.controls.update(delta);
      return;
    }

    if (this.mode === "ballOrbit") {
      // Ball orbit mode - camera orbits around ball and follows it
      if (this.targetBall) {
        const ballPos = this.targetBall.position;

        // Track last ball position to calculate movement delta
        if (!this.lastBallOrbitPos) {
          this.lastBallOrbitPos = ballPos.clone();
        }

        // Calculate how much the ball moved since last frame
        const ballMovement = new THREE.Vector3().subVectors(ballPos, this.lastBallOrbitPos);

        // Always update the target to follow the ball
        // This preserves the user's orbit angle while moving with the ball
        this.controls.setTarget(ballPos.x, ballPos.y, ballPos.z, false);

        // If ball moved significantly, translate camera position by the same amount
        // This keeps the camera at the same relative position to the ball
        if (ballMovement.lengthSq() > 0.01) {
          // Get current camera position from controls (preserves orbit state)
          const currentPos = new THREE.Vector3();
          this.controls.getPosition(currentPos);

          // Move camera by ball movement
          const newX = currentPos.x + ballMovement.x;
          const newY = currentPos.y + ballMovement.y;
          const newZ = currentPos.z + ballMovement.z;

          // Update camera position in controls (preserves orbit state)
          this.controls.setPosition(newX, newY, newZ, false);

          // Update last ball position
          this.lastBallOrbitPos.copy(ballPos);
        }
      }
      this.controls.update(delta);
      return;
    }

    if (!this.targetCar) {
      this.controls.update(delta);
      return;
    }

    const carPos = this.targetCar.position.clone();
    const carQuaternion = this.targetCar.quaternion;

    // === STATE BLENDING SYSTEM (Rocket League style) ===
    // We calculate BOTH camera states every frame and blend between them.
    // This ensures smooth transitions even when the car is moving.

    // When switching TO car cam, initialize smoothedCarYaw from current camera position
    // to avoid camera jump
    if (this.lastIsBallCam !== null && this.lastIsBallCam !== isBallCam && !isBallCam) {
      const carToCamera = new THREE.Vector3().subVectors(this.camera.position, carPos);
      carToCamera.y = 0;
      if (carToCamera.length() > 0.01) {
        carToCamera.normalize();
        this.smoothedCarYaw = Math.atan2(-carToCamera.x, -carToCamera.z);
      }
    }
    this.lastIsBallCam = isBallCam;

    // Calculate BOTH camera states every frame
    const carCamState = this.calculateCarCamPosition(carPos, carQuaternion, delta);
    const ballCamState = this.calculateBallCamPosition(carPos, carQuaternion, delta);

    // Update target blend based on ball cam toggle
    // This doesn't reset - it just changes direction (handles rapid toggling)
    this.targetBlend = isBallCam ? 1.0 : 0.0;

    // Calculate transition step based on transitionSpeed
    // At speed 1.0: duration = 0.5s, at speed 2.0: duration = 0.25s
    const transitionDuration = Math.max(
      0.15,
      Math.min(0.6, this.baseDuration / this.transitionSpeed),
    );
    const step = delta / transitionDuration;

    // Move currentBlend towards targetBlend
    if (this.currentBlend < this.targetBlend) {
      this.currentBlend = Math.min(this.currentBlend + step, this.targetBlend);
    } else if (this.currentBlend > this.targetBlend) {
      this.currentBlend = Math.max(this.currentBlend - step, this.targetBlend);
    }

    // Apply SmoothStep easing (Hermite interpolation) for natural feel
    // Formula: t² × (3 - 2t) - smooth acceleration and deceleration
    const t = this.currentBlend;
    const alpha = t * t * (3 - 2 * t);

    // === POSITION: Linear interpolation (LERP) ===
    // The arc-like visual is emergent from SLERP rotation + car movement
    const finalPos = new THREE.Vector3().lerpVectors(
      carCamState.cameraPos,
      ballCamState.cameraPos,
      alpha,
    );

    // === ROTATION: Spherical interpolation (SLERP) using quaternions ===
    // This is critical for smooth rotation without gimbal lock

    // Calculate quaternion for car cam orientation
    this._tempMatrix.lookAt(
      carCamState.cameraPos,
      carCamState.lookTarget,
      new THREE.Vector3(0, 1, 0),
    );
    this._tempQuatCarCam.setFromRotationMatrix(this._tempMatrix);

    // Calculate quaternion for ball cam orientation
    this._tempMatrix.lookAt(
      ballCamState.cameraPos,
      ballCamState.lookTarget,
      new THREE.Vector3(0, 1, 0),
    );
    this._tempQuatBallCam.setFromRotationMatrix(this._tempMatrix);

    // Check dot product - if negative, negate one quaternion to take shortest path
    if (this._tempQuatCarCam.dot(this._tempQuatBallCam) < 0) {
      this._tempQuatBallCam.set(
        -this._tempQuatBallCam.x,
        -this._tempQuatBallCam.y,
        -this._tempQuatBallCam.z,
        -this._tempQuatBallCam.w,
      );
    }

    // SLERP between the two orientations
    const finalQuat = new THREE.Quaternion().slerpQuaternions(
      this._tempQuatCarCam,
      this._tempQuatBallCam,
      alpha,
    );

    if (this.targetHandoff) {
      this.targetHandoff.elapsed += delta;
      const handoffT = Math.min(1, this.targetHandoff.elapsed / this.targetHandoff.duration);
      const handoffAlpha = handoffT * handoffT * (3 - 2 * handoffT);
      const targetPos = finalPos.clone();
      const targetQuat = finalQuat.clone();
      finalPos.lerpVectors(this.targetHandoff.startPosition, targetPos, handoffAlpha);
      finalQuat.slerpQuaternions(this.targetHandoff.startQuaternion, targetQuat, handoffAlpha);
      if (handoffT >= 1) {
        this.targetHandoff = null;
      }
    }

    // Match Ballcam: apply the blended pose directly. The car/ball meshes have
    // already been interpolated for this render tick, so another camera low-pass
    // makes ball cam visibly chase the target.
    this.camera.position.copy(finalPos);
    this.camera.quaternion.copy(finalQuat);

    // Apply camera angle (pitch adjustment) to the target orientation.
    // followAngle is negative (e.g., -4 degrees) = look down.
    if (this.followAngle !== 0) {
      const angleRad = (this.followAngle * Math.PI) / 180;
      this.camera.rotateX(-angleRad);
    }

    // Store current state for reference
    if (!this.currentCamPos) this.currentCamPos = new THREE.Vector3();
    if (!this.currentLookTarget) this.currentLookTarget = new THREE.Vector3();
    this.currentCamPos.copy(finalPos);
    // Calculate look target from quaternion for compatibility
    const lookDir = new THREE.Vector3(0, 0, -1).applyQuaternion(this.camera.quaternion);
    this.currentLookTarget.copy(finalPos).add(lookDir.multiplyScalar(100));

    this.enforceMinHeight();
  }

  /**
   * Calculate ball cam position and look target
   * Camera positioned so that both car and ball are visible
   * When ball is higher than car, camera goes lower to keep both in frame
   */
  calculateBallCamPosition(carPos, carQuaternion, delta = 1 / 60) {
    if (!this.targetBall) {
      return this.calculateCarCamPosition(carPos, carQuaternion, delta);
    }

    const ballPos = this.targetBall.position.clone();

    // Calculate direction FROM ball TO car (camera goes on opposite side of ball)
    const ballToCar = new THREE.Vector3().subVectors(carPos, ballPos);
    ballToCar.y = 0;
    ballToCar.normalize();

    // Position camera behind car relative to ball (use followDistance setting)
    const cameraPos = carPos.clone().add(ballToCar.multiplyScalar(this.followDistance));

    // Calculate blend factor for aerial adjustments (in UU)
    const ballHeightDiff = ballPos.y - carPos.y;
    const maxHeightDiff = 800; // ~8m in UU
    const blendFactor = Math.min(1, Math.max(0, ballHeightDiff / maxHeightDiff));

    // Camera height: slightly lower during aerials to see the car better
    cameraPos.y = carPos.y + this.followHeight - blendFactor * 100;

    if (cameraPos.y < this.minHeight) {
      cameraPos.y = this.minHeight;
    }

    // lookTarget blends from ball (when close) to a point closer to car (when ball is high)
    const lookTarget = new THREE.Vector3().lerpVectors(
      ballPos,
      new THREE.Vector3(carPos.x, carPos.y + 100, carPos.z),
      blendFactor * 0.6,
    );

    return { cameraPos, lookTarget };
  }

  /**
   * Calculate car cam position and look target
   * Uses velocity-based direction when car is airborne/flipping
   */
  calculateCarCamPosition(carPos, carQuaternion, delta = 1 / 60) {
    // Track car position to calculate velocity direction
    if (!this.lastCarPos) {
      this.lastCarPos = carPos.clone();
    }

    // Calculate car's movement direction (velocity-based)
    const movement = new THREE.Vector3().subVectors(carPos, this.lastCarPos);
    movement.y = 0; // Only horizontal movement
    const movementSpeed = movement.length();

    // Get car's orientation-based forward direction
    const forward = new THREE.Vector3(1, 0, 0);
    forward.applyQuaternion(carQuaternion);
    const orientationYaw = Math.atan2(forward.x, forward.z);

    // Check if car is likely flipping (up vector is not pointing up)
    const up = new THREE.Vector3(0, 1, 0);
    up.applyQuaternion(carQuaternion);
    const isFlipping = up.y < 0.5; // Car is tilted more than ~60 degrees

    // Determine target yaw
    let targetYaw;
    if (isFlipping && movementSpeed > 0.01) {
      // Car is flipping - use movement direction instead of orientation
      movement.normalize();
      targetYaw = Math.atan2(movement.x, movement.z);
    } else if (movementSpeed > 0.05) {
      // Car is moving normally - blend between orientation and movement direction
      // This helps when car is drifting or reversing
      movement.normalize();
      const movementYaw = Math.atan2(movement.x, movement.z);

      // Check if car is going roughly forward or backward relative to its orientation
      let orientationToMovement = movementYaw - orientationYaw;
      while (orientationToMovement > Math.PI) orientationToMovement -= Math.PI * 2;
      while (orientationToMovement < -Math.PI) orientationToMovement += Math.PI * 2;

      // If moving backward, flip the orientation yaw
      if (Math.abs(orientationToMovement) > Math.PI / 2) {
        targetYaw = orientationYaw + Math.PI;
      } else {
        targetYaw = orientationYaw;
      }
    } else {
      // Car is stationary or very slow - use orientation
      targetYaw = orientationYaw;
    }

    // Update last car position for next frame
    this.lastCarPos.copy(carPos);

    // Initialize smoothed yaw if needed
    if (this.smoothedCarYaw === undefined) {
      this.smoothedCarYaw = targetYaw;
    }

    // Smoothly interpolate yaw - slower when flipping to avoid jitter
    // swivelSpeed controls how fast the camera rotates around the car
    let yawDiff = targetYaw - this.smoothedCarYaw;
    while (yawDiff > Math.PI) yawDiff -= Math.PI * 2;
    while (yawDiff < -Math.PI) yawDiff += Math.PI * 2;

    // Use Ballcam's fixed 60 Hz yaw easing. This keeps the follow camera's yaw
    // behavior independent from render-FPS spikes, matching the production
    // viewer this implementation was derived from.
    const yawLerpSpeed = isFlipping ? this.swivelSpeed * 0.4 : this.swivelSpeed;
    this.smoothedCarYaw += yawDiff * Math.min(1, yawLerpSpeed * (1 / 60));

    // Calculate camera position using smoothed yaw
    const backwardX = -Math.sin(this.smoothedCarYaw);
    const backwardZ = -Math.cos(this.smoothedCarYaw);

    const cameraPos = new THREE.Vector3(
      carPos.x + backwardX * this.followDistance,
      carPos.y + this.followHeight,
      carPos.z + backwardZ * this.followDistance,
    );

    if (cameraPos.y < this.minHeight) {
      cameraPos.y = this.minHeight;
    }

    // Look target is slightly in front of the car (using smoothed yaw)
    // The angle adjustment is applied separately via camera.rotateX() in update()
    const lookAheadDistance = 50;
    const lookTarget = new THREE.Vector3(
      carPos.x + Math.sin(this.smoothedCarYaw) * lookAheadDistance,
      carPos.y,
      carPos.z + Math.cos(this.smoothedCarYaw) * lookAheadDistance,
    );

    return { cameraPos, lookTarget };
  }

  /**
   * Enforce minimum camera height
   */
  enforceMinHeight() {
    const pos = this.camera.position;
    if (pos.y < this.minHeight) {
      pos.y = this.minHeight;
      // Also update controls internal state
      this.controls.setPosition(pos.x, this.minHeight, pos.z, false);
    }
  }

  /**
   * Instantly move camera to position (no transition)
   */
  setPosition(x, y, z) {
    this.controls.setPosition(x, y, z, false);
  }

  /**
   * Instantly set camera look target (no transition)
   */
  setTarget(x, y, z) {
    this.controls.setTarget(x, y, z, false);
  }

  /**
   * Smoothly move camera to position and target
   */
  moveTo(posX, posY, posZ, targetX, targetY, targetZ, smooth = true) {
    this.controls.setLookAt(posX, posY, posZ, targetX, targetY, targetZ, smooth);
  }

  /**
   * Set transition smoothness
   * @param {number} time - Smooth time in seconds (lower = faster)
   */
  setSmoothTime(time) {
    this.controls.smoothTime = time;
  }

  /**
   * Set all camera follow settings (matching Rocket League camera options)
   * @param {Object} settings - Camera settings object
   * @param {number} settings.distance - Distance behind car (100-400 UU)
   * @param {number} settings.height - Height above car (40-200 UU)
   * @param {number} settings.angle - Pitch angle in degrees (-15 to 0)
   * @param {number} settings.stiffness - Camera stiffness (0.0-1.0)
   * @param {number} settings.swivelSpeed - Rotation speed (1.0-10.0)
   * @param {number} settings.transitionSpeed - Ball cam transition speed (1.0-2.0)
   */
  setFollowSettings(settings) {
    if (settings.distance !== undefined) this.followDistance = settings.distance;
    if (settings.height !== undefined) this.followHeight = settings.height;
    if (settings.angle !== undefined) this.followAngle = settings.angle;
    if (settings.stiffness !== undefined) this.stiffness = settings.stiffness;
    if (settings.swivelSpeed !== undefined) this.swivelSpeed = settings.swivelSpeed;
    if (settings.transitionSpeed !== undefined) this.transitionSpeed = settings.transitionSpeed;
  }

  // ============================================
  // Replay Mode Camera Management (v9 protocol)
  // ============================================

  /**
   * Save current camera state before entering replay mode
   * This captures everything needed to restore the exact camera position
   */
  saveCameraState() {
    this.savedCameraState = {
      mode: this.mode,
      position: this.camera.position.clone(),
      quaternion: this.camera.quaternion.clone(),
      targetCarIndex: this.targetCar ? this.targetCar.userData?.index : null,
      currentBlend: this.currentBlend,
      targetBlend: this.targetBlend,
      smoothedCarYaw: this.smoothedCarYaw,
      freeCamRotation: this.freeCamRotation ? { ...this.freeCamRotation } : null,
      // Ball orbit specific state
      lastBallOrbitPos: this.lastBallOrbitPos ? this.lastBallOrbitPos.clone() : null,
    };
    console.log("[CameraManager] Camera state saved:", this.savedCameraState.mode);
  }

  /**
   * Restore camera state after exiting replay mode
   * Returns the saved mode and target car index so the caller can restore
   */
  restoreCameraState() {
    if (!this.savedCameraState) {
      console.warn("[CameraManager] No saved camera state to restore");
      return null;
    }

    const saved = this.savedCameraState;
    console.log("[CameraManager] Restoring camera state:", saved.mode);

    // Restore position and rotation
    this.camera.position.copy(saved.position);
    this.camera.quaternion.copy(saved.quaternion);

    // Restore blend state (for car cam <-> ball cam transitions)
    this.currentBlend = saved.currentBlend;
    this.targetBlend = saved.targetBlend;
    this.smoothedCarYaw = saved.smoothedCarYaw;

    // Restore freecam rotation state
    if (saved.freeCamRotation && this.freeCamRotation) {
      this.freeCamRotation.yaw = saved.freeCamRotation.yaw;
      this.freeCamRotation.pitch = saved.freeCamRotation.pitch;
    }

    // Restore ball orbit state
    if (saved.lastBallOrbitPos) {
      this.lastBallOrbitPos = saved.lastBallOrbitPos;
    }

    // Update camera-controls internal state
    const lookAt = new THREE.Vector3(0, 0, -1).applyQuaternion(saved.quaternion);
    lookAt.multiplyScalar(100).add(saved.position);
    this.controls.setLookAt(
      saved.position.x,
      saved.position.y,
      saved.position.z,
      lookAt.x,
      lookAt.y,
      lookAt.z,
      false,
    );

    // Return info for caller to restore mode and target
    const result = {
      mode: saved.mode,
      targetCarIndex: saved.targetCarIndex,
    };

    this.savedCameraState = null;
    this.isInReplayMode = false;

    return result;
  }

  /**
   * Enter replay mode - saves camera state before switching
   */
  enterReplayMode() {
    if (!this.isInReplayMode) {
      this.saveCameraState();
      this.isInReplayMode = true;
    }
  }

  /**
   * Exit replay mode - restores the previous camera state
   * @returns {Object|null} The saved camera state info
   */
  exitReplayMode() {
    return this.restoreCameraState();
  }

  /**
   * Check if currently in replay mode
   * @returns {boolean}
   */
  getIsInReplayMode() {
    return this.isInReplayMode;
  }

  /**
   * Set up podium camera - positions camera to look at field center
   */
  setupPodiumCamera() {
    // Save current state if not already in replay mode
    if (!this.isInReplayMode) {
      this.saveCameraState();
      this.isInReplayMode = true;
    }

    // Position camera elevated, looking at center of field
    const podiumPosition = new THREE.Vector3(0, 1500, 3000);
    const podiumLookAt = new THREE.Vector3(0, 200, 0);

    this.camera.position.copy(podiumPosition);
    this.camera.lookAt(podiumLookAt);

    // Update controls
    this.controls.setLookAt(
      podiumPosition.x,
      podiumPosition.y,
      podiumPosition.z,
      podiumLookAt.x,
      podiumLookAt.y,
      podiumLookAt.z,
      false,
    );

    console.log("[CameraManager] Podium camera setup");
  }

  /**
   * Set freecam state from position and quaternion (for following another viewer)
   * Uses interpolation for smooth movement when following
   * @param {Object} position - { x, y, z }
   * @param {Object} rotation - { x, y, z, w } quaternion
   */
  setFreecamState(position, rotation) {
    if (!position) return;

    // Set target for interpolation (applied smoothly in update loop)
    this.followTargetPosition.set(position.x, position.y, position.z);

    if (rotation) {
      this.followTargetQuaternion.set(rotation.x, rotation.y, rotation.z, rotation.w);
    }

    // On first target, snap directly to avoid initial lag
    if (!this.hasFollowTarget) {
      this.camera.position.copy(this.followTargetPosition);
      this.camera.quaternion.copy(this.followTargetQuaternion);

      // Update freeCamRotation state to match
      if (this.freeCamRotation) {
        const dir = new THREE.Vector3();
        this.camera.getWorldDirection(dir);
        this.freeCamRotation.yaw = Math.atan2(dir.x, dir.z);
        this.freeCamRotation.pitch = Math.asin(-dir.y);
      }

      // Update camera-controls internal state
      const lookAt = new THREE.Vector3();
      this.camera.getWorldDirection(lookAt);
      lookAt.multiplyScalar(100).add(this.camera.position);
      this.controls.setLookAt(
        position.x,
        position.y,
        position.z,
        lookAt.x,
        lookAt.y,
        lookAt.z,
        false,
      );

      this.hasFollowTarget = true;
    }
  }

  /**
   * Set ball orbit camera state when following another viewer
   * Uses orbit parameters to orbit around LOCAL ball - prevents desync/stuttering
   * @param {Object} orbitParams - { distance, azimuth, polar } orbit parameters from followed viewer
   */
  setBallOrbitState(orbitParams) {
    if (!orbitParams) return;

    // Store target orbit params for smooth interpolation
    if (!this.followTargetOrbitParams) {
      this.followTargetOrbitParams = { ...orbitParams };
    } else {
      this.followTargetOrbitParams.distance = orbitParams.distance;
      this.followTargetOrbitParams.azimuth = orbitParams.azimuth;
      this.followTargetOrbitParams.polar = orbitParams.polar;
    }

    // Initialize current orbit params if first time
    if (!this.followCurrentOrbitParams) {
      this.followCurrentOrbitParams = { ...orbitParams };
    }

    // On first target, snap directly to avoid initial lag
    if (!this.hasFollowTarget) {
      const localBallPos = this.targetBall?.position || new THREE.Vector3(0, 100, 0);

      // Set target to local ball and apply orbit params directly
      this.controls.setTarget(localBallPos.x, localBallPos.y, localBallPos.z, false);
      this.controls.dollyTo(orbitParams.distance, false);
      this.controls.rotateTo(orbitParams.azimuth, orbitParams.polar, false);

      // Copy to current
      this.followCurrentOrbitParams = { ...orbitParams };

      this.hasFollowTarget = true;
    }
  }

  /**
   * Set whether we're following another viewer's camera
   * When following, inputs are disabled and camera interpolates to target
   * @param {boolean} isFollowing
   */
  setFollowingViewer(isFollowing) {
    this.isFollowingViewer = isFollowing;

    if (isFollowing) {
      // Disable camera controls (orbit, drag, etc.) when following
      this.controls.enabled = false;
    } else {
      // Reset follow state when stopping
      this.hasFollowTarget = false;
      this.followCurrentOrbitParams = null;
      this.followTargetOrbitParams = null;

      // Re-enable controls if in ballOrbit mode
      if (this.mode === "ballOrbit") {
        this.controls.enabled = true;
      }

      // Exit pointer lock if we were in free mode
      if (this.mode === "free" && document.pointerLockElement === this.domElement) {
        document.exitPointerLock();
      }
    }
  }

  /**
   * Update interpolation for following mode (called from update loop)
   * @param {number} delta - Time since last frame
   */
  updateFollowInterpolation(delta) {
    if (!this.isFollowingViewer || !this.hasFollowTarget) return;

    // In ballOrbit mode, use orbit parameters around LOCAL ball
    // This eliminates stuttering because the camera follows the ball at 60fps
    // Only the orbit angles/distance are synchronized at 20Hz
    if (this.mode === "ballOrbit") {
      const localBallPos = this.targetBall?.position;

      if (localBallPos && this.followCurrentOrbitParams && this.followTargetOrbitParams) {
        // Smoothly interpolate orbit params towards target
        // This creates smooth transitions between network updates
        const lerpFactor = 0.15;

        // Interpolate distance linearly
        this.followCurrentOrbitParams.distance +=
          (this.followTargetOrbitParams.distance - this.followCurrentOrbitParams.distance) *
          lerpFactor;

        // Interpolate angles - handle wrap-around for azimuth
        let azimuthDiff =
          this.followTargetOrbitParams.azimuth - this.followCurrentOrbitParams.azimuth;
        while (azimuthDiff > Math.PI) azimuthDiff -= Math.PI * 2;
        while (azimuthDiff < -Math.PI) azimuthDiff += Math.PI * 2;
        this.followCurrentOrbitParams.azimuth += azimuthDiff * lerpFactor;

        // Polar angle (no wrap-around needed, clamped by CameraControls)
        this.followCurrentOrbitParams.polar +=
          (this.followTargetOrbitParams.polar - this.followCurrentOrbitParams.polar) * lerpFactor;

        // Update target to follow local ball (this happens every frame at 60fps)
        this.controls.setTarget(localBallPos.x, localBallPos.y, localBallPos.z, false);

        // Apply interpolated orbit parameters
        this.controls.dollyTo(this.followCurrentOrbitParams.distance, false);
        this.controls.rotateTo(
          this.followCurrentOrbitParams.azimuth,
          this.followCurrentOrbitParams.polar,
          false,
        );

        // Update controls to apply changes
        this.controls.update(delta);
      }
    } else {
      // For free cam, use interpolation for smooth movement
      this.camera.position.lerp(this.followTargetPosition, this.followPositionLerpFactor);
      this.camera.quaternion.slerp(this.followTargetQuaternion, this.followRotationSlerpFactor);

      // Update freeCamRotation to match (for consistency if we stop following)
      if (this.freeCamRotation) {
        const dir = new THREE.Vector3();
        this.camera.getWorldDirection(dir);
        this.freeCamRotation.yaw = Math.atan2(dir.x, dir.z);
        this.freeCamRotation.pitch = Math.asin(-dir.y);
      }

      // Update camera-controls internal state to match
      const lookAt = new THREE.Vector3();
      this.camera.getWorldDirection(lookAt);
      lookAt.multiplyScalar(100).add(this.camera.position);
      this.controls.setLookAt(
        this.camera.position.x,
        this.camera.position.y,
        this.camera.position.z,
        lookAt.x,
        lookAt.y,
        lookAt.z,
        false,
      );
    }
  }

  /**
   * Set camera to default freecam position (side view of field)
   * Call this when initializing the player or when switching to freecam
   */
  setDefaultFreecamPosition() {
    this.camera.position.copy(this.defaultFreecamPosition);
    this.camera.lookAt(this.defaultFreecamLookAt);

    // Also update the freecam rotation state to match
    if (this.freeCamRotation) {
      const dir = new THREE.Vector3();
      this.camera.getWorldDirection(dir);
      this.freeCamRotation.yaw = Math.atan2(dir.x, dir.z);
      this.freeCamRotation.pitch = Math.asin(-dir.y);
    }

    // Update controls internal state
    this.controls.setLookAt(
      this.defaultFreecamPosition.x,
      this.defaultFreecamPosition.y,
      this.defaultFreecamPosition.z,
      this.defaultFreecamLookAt.x,
      this.defaultFreecamLookAt.y,
      this.defaultFreecamLookAt.z,
      false,
    );
  }

  /**
   * Get current pointer lock state
   * @returns {boolean} True if pointer is currently locked
   */
  getIsPointerLocked() {
    return this.isPointerLocked || false;
  }

  /**
   * Set callback for pointer lock state changes
   * @param {(isLocked: boolean) => void} callback
   */
  setPointerLockCallback(callback) {
    this.onPointerLockStateChange = callback;
  }

  /**
   * Dispose resources
   */
  dispose() {
    this.controls.dispose();

    // Clean up event listeners
    if (this.ballOrbitScrollHandler) {
      this.domElement.removeEventListener("wheel", this.ballOrbitScrollHandler);
    }
    if (this.onKeyDown) {
      document.removeEventListener("keydown", this.onKeyDown);
    }
    if (this.onKeyUp) {
      document.removeEventListener("keyup", this.onKeyUp);
    }
    if (this.onMouseMove) {
      document.removeEventListener("mousemove", this.onMouseMove);
    }
    // Right-click drag event listeners
    if (this.onMouseDown) {
      this.domElement.removeEventListener("mousedown", this.onMouseDown);
    }
    if (this.onMouseUp) {
      document.removeEventListener("mouseup", this.onMouseUp);
    }
    if (this.onPointerLockChange) {
      document.removeEventListener("pointerlockchange", this.onPointerLockChange);
    }
    if (this.onMouseLeave) {
      this.domElement.removeEventListener("mouseleave", this.onMouseLeave);
    }
    if (this.onWindowBlur) {
      window.removeEventListener("blur", this.onWindowBlur);
    }
    if (this.onVisibilityChange) {
      document.removeEventListener("visibilitychange", this.onVisibilityChange);
    }
  }
}
