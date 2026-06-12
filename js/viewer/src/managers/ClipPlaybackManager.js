import * as THREE from 'three';

/**
 * ClipPlaybackManager
 *
 * Manages camera playback during clip viewing.
 * Supports both capture mode (frame-by-frame) and cinematic mode (keyframe interpolation).
 *
 * Feature: 024-clip-system (US1 - Capture Mode, US2 - Cinematic Mode)
 */

export class ClipPlaybackManager {
  constructor(camera) {
    this.camera = camera;

    // Playback state
    this.isPlaying = false;
    this.clipData = null;
    this.clipStartTime = 0; // Game time when clip starts in replay

    // Current playback time within clip (ms)
    this.currentTime = 0;

    // Current camera mode from clip frames
    // 'f' = freecam, 'b' = ballcam, 'p' = playercam
    this.currentCameraMode = 'f';
    // Target player index when in playercam mode
    this.targetPlayerIndex = null;

    // Temporary objects to avoid allocations
    this._tempPosition = new THREE.Vector3();
    this._tempQuaternion = new THREE.Quaternion();
    this._tempQuatA = new THREE.Quaternion();
    this._tempQuatB = new THREE.Quaternion();

    // Catmull-Rom curve for cinematic mode (cached)
    this._positionCurve = null;
  }

  /**
   * Load clip data and prepare for playback
   *
   * NOTE (026-clip-editor-redesign): Both capture frames and cinematic keyframes
   * use RELATIVE timestamps (relative to clip start). The clipStartTime parameter
   * is used to map game time to clip time during playback.
   *
   * @param {Object} cameraData - Camera data (CameraRecording or CameraKeyframes)
   * @param {number} clipStartTime - Game time when clip starts (seconds)
   */
  load(cameraData, clipStartTime) {
    this.clipData = cameraData;
    this.clipStartTime = clipStartTime;
    this.currentTime = 0;
    this._positionCurve = null;
    // Reset debug flags
    this._debugLogged = false;
    this._emptyFramesLogged = false;
    this._noFrameLogged = false;
    this._firstFrameLogged = false;
    this._cinematicDebugLogged = false;
    this._posDebugLogged = false;
    this._notPlayingLogged = false;

    // Pre-build Catmull-Rom curve for cinematic mode
    if (cameraData.type === 'cinematic') {
      this._buildPositionCurve();
    }

    // Debug logging
    if (cameraData.type === 'capture') {
      console.log('[ClipPlaybackManager] Loaded capture data:', {
        type: cameraData.type,
        framesCount: cameraData.frames?.length ?? 0,
        duration: cameraData.duration,
        startTime: clipStartTime,
        firstFrame: cameraData.frames?.[0],
      });
    } else {
      console.log('[ClipPlaybackManager] Loaded cinematic data:', {
        type: cameraData.type,
        keyframesCount: cameraData.keyframes?.length ?? 0,
        startTime: clipStartTime,
        keyframes: cameraData.keyframes?.map(kf => ({ t: kf.t })),
      });
    }
  }

  /**
   * Start playback
   */
  play() {
    if (!this.clipData) {
      console.warn('[ClipPlaybackManager] No clip data loaded');
      return;
    }
    this.isPlaying = true;
    console.log('[ClipPlaybackManager] Playback started at', this.clipStartTime);
  }

  /**
   * Pause playback
   */
  pause() {
    this.isPlaying = false;
  }

  /**
   * Stop playback and reset
   */
  stop() {
    this.isPlaying = false;
    this.currentTime = 0;
  }

  /**
   * Check if playback is active
   * @returns {boolean}
   */
  isActive() {
    return this.isPlaying && this.clipData !== null;
  }

  /**
   * Check if clip data is loaded
   * @returns {boolean}
   */
  isLoaded() {
    return this.clipData !== null;
  }

  /**
   * Get clip duration in milliseconds
   * @returns {number}
   */
  getDuration() {
    if (!this.clipData) return 0;

    if (this.clipData.type === 'capture') {
      return this.clipData.duration;
    } else if (this.clipData.type === 'cinematic') {
      const keyframes = this.clipData.keyframes;
      if (keyframes.length === 0) return 0;
      return keyframes[keyframes.length - 1].t - keyframes[0].t;
    }

    return 0;
  }

  /**
   * Seek to a specific time in the clip
   * @param {number} time - Time in milliseconds from clip start
   */
  seek(time) {
    this.currentTime = Math.max(0, Math.min(time, this.getDuration()));
  }

  /**
   * Seek to a specific time and immediately apply camera
   * (Works even when not playing - useful for scrubbing)
   * @param {number} time - Time in milliseconds from clip start
   */
  seekAndApply(time) {
    this.currentTime = Math.max(0, Math.min(time, this.getDuration()));
    if (this.clipData) {
      this._applyCameraAtTime(this.currentTime);
    }
  }

  /**
   * Update camera position based on current game time
   * Called from GameEngine.animate() loop when in clip playback mode
   *
   * @param {number} gameTime - Current game time in seconds
   * @returns {boolean} True if still within clip bounds, false if clip ended
   */
  updateCamera(gameTime) {
    if (!this.clipData || !this.isPlaying) {
      if (!this._notPlayingLogged) {
        this._notPlayingLogged = true;
        console.log('[ClipPlaybackManager] updateCamera skipped:', {
          hasClipData: !!this.clipData,
          isPlaying: this.isPlaying,
        });
      }
      return false;
    }

    // Calculate time within clip (in ms)
    const clipTimeMs = (gameTime - this.clipStartTime) * 1000;
    this.currentTime = clipTimeMs;

    // Debug log (only first call)
    if (!this._debugLogged) {
      this._debugLogged = true;
      console.log('[ClipPlaybackManager] updateCamera first call:', {
        gameTime,
        clipStartTime: this.clipStartTime,
        clipTimeMs,
        duration: this.getDuration(),
        cameraPos: this.camera ? `${this.camera.position.x.toFixed(2)}, ${this.camera.position.y.toFixed(2)}, ${this.camera.position.z.toFixed(2)}` : 'no camera',
      });
    }

    // Check if clip has ended
    const duration = this.getDuration();
    if (clipTimeMs < 0 || clipTimeMs > duration) {
      // Clamp to bounds
      if (clipTimeMs > duration) {
        this._applyCameraAtTime(duration);
        return false; // Clip ended
      }
      return true;
    }

    // Apply camera based on mode
    this._applyCameraAtTime(clipTimeMs);
    return true;
  }

  /**
   * Apply camera position/rotation for a specific time
   * @private
   * @param {number} timeMs - Time in milliseconds from clip start
   */
  _applyCameraAtTime(timeMs) {
    if (this.clipData.type === 'capture') {
      this._applyCaptureFrame(timeMs);
    } else if (this.clipData.type === 'cinematic') {
      this._applyCinematicFrame(timeMs);
    }
  }

  /**
   * Apply camera from capture mode (frame lookup with interpolation)
   * @private
   * @param {number} timeMs - Time in milliseconds
   */
  _applyCaptureFrame(timeMs) {
    const frames = this.clipData.frames;
    if (!frames || frames.length === 0) {
      if (!this._emptyFramesLogged) {
        this._emptyFramesLogged = true;
        console.warn('[ClipPlaybackManager] No frames in capture data!');
      }
      return;
    }

    // Find the two frames to interpolate between
    const { frameA, frameB, t } = this._findFramesForInterpolation(frames, timeMs);

    if (!frameA) {
      if (!this._noFrameLogged) {
        this._noFrameLogged = true;
        console.warn('[ClipPlaybackManager] No frame found at time:', timeMs);
      }
      return;
    }

    // Debug log first frame application
    if (!this._firstFrameLogged) {
      this._firstFrameLogged = true;
      console.log('[ClipPlaybackManager] Applying interpolated frame:', {
        timeMs,
        frameA: { t: frameA.t, px: frameA.px, py: frameA.py, pz: frameA.pz, m: frameA.m, tp: frameA.tp },
        frameB: frameB ? { t: frameB.t, px: frameB.px, py: frameB.py, pz: frameB.pz } : null,
        interpolationT: t,
      });
    }

    // Update current camera mode and target player from frameA
    this.currentCameraMode = frameA.m || 'f';
    this.targetPlayerIndex = frameA.m === 'p' ? (frameA.tp ?? null) : null;

    // If no second frame or t is 0, just use frameA directly
    if (!frameB || t === 0) {
      this.camera.position.set(frameA.px, frameA.py, frameA.pz);
      this.camera.quaternion.set(frameA.qx, frameA.qy, frameA.qz, frameA.qw);
      return;
    }

    // Interpolate position (lerp)
    this.camera.position.set(
      frameA.px + (frameB.px - frameA.px) * t,
      frameA.py + (frameB.py - frameA.py) * t,
      frameA.pz + (frameB.pz - frameA.pz) * t
    );

    // Interpolate rotation (slerp)
    this._tempQuatA.set(frameA.qx, frameA.qy, frameA.qz, frameA.qw);
    this._tempQuatB.set(frameB.qx, frameB.qy, frameB.qz, frameB.qw);
    this.camera.quaternion.slerpQuaternions(this._tempQuatA, this._tempQuatB, t);
  }

  /**
   * Find the two frames surrounding the target time for interpolation
   * @private
   * @param {Array} frames - Array of frames sorted by time
   * @param {number} timeMs - Target time in ms
   * @returns {Object} { frameA, frameB, t } where t is interpolation factor 0-1
   */
  _findFramesForInterpolation(frames, timeMs) {
    if (frames.length === 0) return { frameA: null, frameB: null, t: 0 };

    // Edge cases
    if (timeMs <= frames[0].t) {
      return { frameA: frames[0], frameB: null, t: 0 };
    }
    if (timeMs >= frames[frames.length - 1].t) {
      return { frameA: frames[frames.length - 1], frameB: null, t: 0 };
    }

    // Binary search to find frame at or before timeMs
    let left = 0;
    let right = frames.length - 1;

    while (left < right) {
      const mid = Math.floor((left + right + 1) / 2);
      if (frames[mid].t <= timeMs) {
        left = mid;
      } else {
        right = mid - 1;
      }
    }

    const frameA = frames[left];
    const frameB = frames[left + 1]; // Safe because we handled edge case above

    // Calculate interpolation factor
    const segmentDuration = frameB.t - frameA.t;
    const t = segmentDuration > 0 ? (timeMs - frameA.t) / segmentDuration : 0;

    return { frameA, frameB, t };
  }

  /**
   * Binary search to find frame at or before specified time
   * @private
   * @param {Array} frames - Array of frames sorted by time
   * @param {number} timeMs - Target time in ms
   * @returns {Object|null} Frame object
   */
  _findFrameAtTime(frames, timeMs) {
    if (frames.length === 0) return null;

    let left = 0;
    let right = frames.length - 1;

    // Handle edge cases
    if (timeMs <= frames[0].t) return frames[0];
    if (timeMs >= frames[right].t) return frames[right];

    // Binary search
    while (left < right) {
      const mid = Math.floor((left + right + 1) / 2);
      if (frames[mid].t <= timeMs) {
        left = mid;
      } else {
        right = mid - 1;
      }
    }

    return frames[left];
  }

  /**
   * Apply camera from cinematic mode (Catmull-Rom interpolation)
   * Uses segment-by-segment TIME-BASED interpolation to match KeyframeVisualizer behavior
   * (Arc-length parameterization causes speed variations when keyframes are unevenly spaced)
   * @private
   * @param {number} timeMs - Time in milliseconds
   */
  _applyCinematicFrame(timeMs) {
    const keyframes = this.clipData.keyframes;
    if (keyframes.length < 2) {
      console.warn('[ClipPlaybackManager] _applyCinematicFrame: less than 2 keyframes!', keyframes.length);
      return;
    }

    const startTime = keyframes[0].t;
    const endTime = keyframes[keyframes.length - 1].t;

    // Clamp time to keyframe range
    const clampedTime = Math.max(startTime, Math.min(endTime, timeMs));

    // Debug log first few applications
    if (!this._cinematicDebugLogged) {
      this._cinematicDebugLogged = true;
      console.log('[ClipPlaybackManager] _applyCinematicFrame:', {
        timeMs,
        startTime,
        endTime,
        clampedTime,
        keyframes: keyframes.map(kf => ({
          t: kf.t,
          pos: `${kf.px?.toFixed(0)},${kf.py?.toFixed(0)},${kf.pz?.toFixed(0)}`,
        })),
      });
    }

    // Find which segment we're in based on time
    let segmentIndex = 0;
    while (
      segmentIndex < keyframes.length - 1 &&
      keyframes[segmentIndex + 1].t < clampedTime
    ) {
      segmentIndex++;
    }

    // Edge case: at or past the last keyframe
    if (segmentIndex >= keyframes.length - 1) {
      const kf = keyframes[keyframes.length - 1];
      this.camera.position.set(kf.px, kf.py, kf.pz);
      this.camera.quaternion.set(kf.qx, kf.qy, kf.qz, kf.qw);
      return;
    }

    const kfA = keyframes[segmentIndex];
    const kfB = keyframes[segmentIndex + 1];

    // Calculate local t within this segment (0-1)
    const segmentDuration = kfB.t - kfA.t;
    const localT = segmentDuration > 0 ? (clampedTime - kfA.t) / segmentDuration : 0;

    // At exact keyframe positions, use the exact keyframe data
    if (localT <= 0.001) {
      this.camera.position.set(kfA.px, kfA.py, kfA.pz);
      this.camera.quaternion.set(kfA.qx, kfA.qy, kfA.qz, kfA.qw);
      return;
    }
    if (localT >= 0.999) {
      this.camera.position.set(kfB.px, kfB.py, kfB.pz);
      this.camera.quaternion.set(kfB.qx, kfB.qy, kfB.qz, kfB.qw);
      return;
    }

    // Use Catmull-Rom interpolation with 4 control points for smooth curves
    // Get the 4 points: p0, p1, p2, p3 where we interpolate between p1 and p2
    const p0 = segmentIndex > 0 ? keyframes[segmentIndex - 1] : kfA;
    const p1 = kfA;
    const p2 = kfB;
    const p3 = segmentIndex < keyframes.length - 2 ? keyframes[segmentIndex + 2] : kfB;

    // Catmull-Rom interpolation for position
    const pos = this._catmullRomInterpolate(
      new THREE.Vector3(p0.px, p0.py, p0.pz),
      new THREE.Vector3(p1.px, p1.py, p1.pz),
      new THREE.Vector3(p2.px, p2.py, p2.pz),
      new THREE.Vector3(p3.px, p3.py, p3.pz),
      localT
    );
    this.camera.position.copy(pos);

    // Debug log position on first application
    if (!this._posDebugLogged) {
      this._posDebugLogged = true;
      console.log('[ClipPlaybackManager] Applied camera position:', {
        segmentIndex,
        localT,
        position: `${pos.x.toFixed(2)}, ${pos.y.toFixed(2)}, ${pos.z.toFixed(2)}`,
      });
    }

    // Interpolate rotation (SLERP between the two keyframes)
    this._tempQuatA.set(kfA.qx, kfA.qy, kfA.qz, kfA.qw);
    this._tempQuatB.set(kfB.qx, kfB.qy, kfB.qz, kfB.qw);
    this.camera.quaternion.slerpQuaternions(this._tempQuatA, this._tempQuatB, localT);
  }

  /**
   * Build Catmull-Rom curve from keyframes
   * NOTE: This is kept for potential future use but _applyCinematicFrame now uses
   * segment-by-segment interpolation for consistent timing with KeyframeVisualizer
   * @private
   */
  _buildPositionCurve() {
    if (!this.clipData || this.clipData.type !== 'cinematic') return;

    const keyframes = this.clipData.keyframes;
    if (keyframes.length < 2) return;

    // Extract positions
    const points = keyframes.map((kf) => new THREE.Vector3(kf.px, kf.py, kf.pz));

    // Create Catmull-Rom curve
    this._positionCurve = new THREE.CatmullRomCurve3(points);
    this._positionCurve.curveType = 'catmullrom';
    this._positionCurve.tension = this.clipData.tension ?? 0.5;
  }

  /**
   * Catmull-Rom spline interpolation between p1 and p2
   * Uses the same algorithm as KeyframeVisualizer for consistent results
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
   * Interpolate rotation between keyframes using SLERP
   * @private
   * @param {number} t - Normalized time 0-1
   * @param {Array} keyframes - Array of keyframes
   */
  _interpolateRotation(t, keyframes) {
    if (keyframes.length < 2) return;

    // Find the two keyframes to interpolate between
    const startTime = keyframes[0].t;
    const endTime = keyframes[keyframes.length - 1].t;
    const duration = endTime - startTime;
    const currentTime = startTime + t * duration;

    // Find keyframe indices
    let i = 0;
    while (i < keyframes.length - 1 && keyframes[i + 1].t < currentTime) {
      i++;
    }

    // Edge case: at or past the last keyframe
    if (i >= keyframes.length - 1) {
      const kf = keyframes[keyframes.length - 1];
      this.camera.quaternion.set(kf.qx, kf.qy, kf.qz, kf.qw);
      return;
    }

    const kfA = keyframes[i];
    const kfB = keyframes[i + 1];

    // Calculate local t between these two keyframes
    const segmentDuration = kfB.t - kfA.t;
    const localT = segmentDuration > 0 ? (currentTime - kfA.t) / segmentDuration : 0;

    // Apply easing if specified
    const easedT = this._applyEasing(localT, kfB.easing);

    // SLERP between quaternions
    this._tempQuatA.set(kfA.qx, kfA.qy, kfA.qz, kfA.qw);
    this._tempQuatB.set(kfB.qx, kfB.qy, kfB.qz, kfB.qw);
    this.camera.quaternion.slerpQuaternions(this._tempQuatA, this._tempQuatB, easedT);
  }

  /**
   * Apply easing function to t value
   * @private
   * @param {number} t - Input value 0-1
   * @param {string} easing - Easing type
   * @returns {number} Eased value 0-1
   */
  _applyEasing(t, easing) {
    switch (easing) {
      case 'ease-in':
        return t * t;
      case 'ease-out':
        return 1 - (1 - t) * (1 - t);
      case 'ease-in-out':
        return t < 0.5 ? 2 * t * t : 1 - Math.pow(-2 * t + 2, 2) / 2;
      case 'linear':
      default:
        return t;
    }
  }

  /**
   * Get current camera state info (mode and target player)
   * @returns {Object} { mode: 'f'|'b'|'p', targetPlayerIndex: number|null }
   */
  getCameraState() {
    return {
      mode: this.currentCameraMode,
      targetPlayerIndex: this.targetPlayerIndex,
    };
  }

  /**
   * Dispose resources
   */
  dispose() {
    this.clipData = null;
    this._positionCurve = null;
    this.isPlaying = false;
    this.currentCameraMode = 'f';
    this.targetPlayerIndex = null;
  }
}
