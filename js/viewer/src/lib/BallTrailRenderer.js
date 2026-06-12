/**
 * BallTrailRenderer - Spiral ball trail effect
 *
 * Creates a continuous trail effect with:
 * - Main trail: round head matching ball diameter, shrinking over distance
 * - 4 secondary trails: thin trails rotating at 90-degree intervals around main trail
 * - Spiral effect: secondary trails rotate around center as ball moves
 * - Gradient: white near ball head, fading to team color
 * - Intensity varies with ball speed (opacity-based)
 *
 * Uses a segment-based system to handle discontinuities (pauses, seeks, teleports)
 */

import * as THREE from "three";
import { TrailRenderer } from "./TrailRenderer.js";

/**
 * Custom shader for ball trail with non-linear gradient (white -> team color)
 * The first 20% of the trail blends from white to team color
 */
const BallTrailShader = {
  vertexShader: `
        attribute float nodeID;
        attribute float nodeVertexID;
        attribute vec3 nodeCenter;
        uniform float minID;
        uniform float maxID;
        uniform float trailLength;
        uniform float maxTrailLength;
        uniform float verticesPerNode;
        uniform vec2 textureTileFactor;
        uniform vec4 headColor;
        uniform vec4 tailColor;
        uniform float intensityMultiplier;
        varying vec4 vColor;
        varying float vFraction;

        void main() {
            float fraction = (maxID - nodeID) / (maxID - minID);
            vFraction = fraction;

            // Non-linear gradient: white at head (0-20%), then team color (20-100%)
            // Use smoothstep for smooth transition
            float whiteZone = 0.20; // First 20% is white blend zone
            float whiteFactor = 1.0 - smoothstep(0.0, whiteZone, fraction);

            // Mix between white and team color based on whiteFactor
            vec4 white = vec4(1.0, 1.0, 1.0, headColor.a);
            vec4 baseColor = mix(headColor, tailColor, fraction);
            vec4 colorWithWhite = mix(baseColor, white, whiteFactor * 0.7); // 70% white blend max

            // Apply intensity multiplier to alpha
            colorWithWhite.a *= intensityMultiplier;

            vColor = colorWithWhite;
            vec4 realPosition = vec4((1.0 - fraction) * position.xyz + fraction * nodeCenter.xyz, 1.0);
            gl_Position = projectionMatrix * viewMatrix * realPosition;
        }
    `,
  fragmentShader: `
        varying vec4 vColor;
        varying float vFraction;

        void main() {
            gl_FragColor = vColor;
        }
    `,
};

/**
 * Create custom material for ball trail with gradient and intensity control
 */
function createBallTrailMaterial() {
  const customUniforms = {
    trailLength: { type: "f", value: null },
    verticesPerNode: { type: "f", value: null },
    minID: { type: "f", value: null },
    maxID: { type: "f", value: null },
    dragTexture: { type: "f", value: null },
    maxTrailLength: { type: "f", value: null },
    textureTileFactor: { type: "v2", value: new THREE.Vector2(1.0, 1.0) },
    headColor: { type: "v4", value: new THREE.Vector4() },
    tailColor: { type: "v4", value: new THREE.Vector4() },
    intensityMultiplier: { type: "f", value: 1.0 },
  };

  return new THREE.ShaderMaterial({
    uniforms: customUniforms,
    vertexShader: BallTrailShader.vertexShader,
    fragmentShader: BallTrailShader.fragmentShader,
    transparent: true,
    blending: THREE.AdditiveBlending,
    depthTest: true,
    depthWrite: false,
    side: THREE.DoubleSide,
  });
}

/**
 * BallTrailSegment - A single trail segment that can fade out independently
 */
class BallTrailSegment {
  constructor(scene, team, config, initialIntensity = 1.0) {
    this.scene = scene;
    this.team = team;
    this.config = config;
    this.active = true;
    this.dying = false;
    this.deathTime = 0;
    this.maxDeathTime = 0.8; // Time to fully fade out (seconds)
    this.intensity = initialIntensity;

    // Team colors - more saturated blue (less cyan), vibrant orange
    // Head color is the bright version, tail fades to darker/transparent
    this.teamColors = {
      0: {
        head: new THREE.Vector4(0.2, 0.4, 1.0, 1.0), // Deep blue (less cyan)
        tail: new THREE.Vector4(0.1, 0.2, 0.8, 0.0), // Darker blue, transparent
      },
      1: {
        head: new THREE.Vector4(1.0, 0.45, 0.0, 1.0), // Vibrant orange
        tail: new THREE.Vector4(0.8, 0.25, 0.0, 0.0), // Darker orange, transparent
      },
    };

    // Target objects for TrailRenderer (emission points)
    this.mainTarget = new THREE.Object3D();
    this.secondaryTargets = [
      new THREE.Object3D(),
      new THREE.Object3D(),
      new THREE.Object3D(),
      new THREE.Object3D(),
    ];

    // Add targets to scene
    scene.add(this.mainTarget);
    this.secondaryTargets.forEach((t) => scene.add(t));

    // Create trail renderers
    this.mainTrail = this._createMainTrail();
    this.secondaryTrails = this._createSecondaryTrails();

    // Apply initial colors and intensity
    this._updateColors();
    this._updateIntensity();

    // Activate immediately
    this.mainTrail.activate();
    this.secondaryTrails.forEach((trail) => trail.activate());
  }

  _createMainTrail() {
    // orientToMovement = false - we use a star shape that looks good from all angles
    const trail = new TrailRenderer(this.scene, false);

    const material = createBallTrailMaterial();

    // Create a 6-pointed star head geometry that looks circular from any angle
    // This creates 3 perpendicular ribbons that form a star shape
    const w = this.config.mainTrailWidth;
    const headGeometry = [
      // Vertical ribbon (Y axis)
      new THREE.Vector3(0, -w, 0),
      new THREE.Vector3(0, w, 0),
      // Horizontal ribbon (X axis)
      new THREE.Vector3(-w, 0, 0),
      new THREE.Vector3(w, 0, 0),
      // Depth ribbon (Z axis)
      new THREE.Vector3(0, 0, -w),
      new THREE.Vector3(0, 0, w),
    ];

    trail.initialize(material, this.config.trailLength, false, 0, headGeometry, this.mainTarget);

    trail.setAdvanceFrequency(60);

    if (trail.mesh) {
      trail.mesh.frustumCulled = false;
      trail.mesh.renderOrder = 100;
    }

    return trail;
  }

  _createSecondaryTrails() {
    const trails = [];

    for (let i = 0; i < 4; i++) {
      // orientToMovement = false - we use a star shape that looks good from all angles
      const trail = new TrailRenderer(this.scene, false);

      const material = createBallTrailMaterial();

      // Smaller star head geometry for secondary trails (same approach as main)
      const w = this.config.secondaryTrailWidth;
      const headGeometry = [
        // Vertical ribbon (Y axis)
        new THREE.Vector3(0, -w, 0),
        new THREE.Vector3(0, w, 0),
        // Horizontal ribbon (X axis)
        new THREE.Vector3(-w, 0, 0),
        new THREE.Vector3(w, 0, 0),
        // Depth ribbon (Z axis)
        new THREE.Vector3(0, 0, -w),
        new THREE.Vector3(0, 0, w),
      ];

      trail.initialize(
        material,
        this.config.trailLength,
        false,
        0,
        headGeometry,
        this.secondaryTargets[i],
      );

      trail.setAdvanceFrequency(60);

      if (trail.mesh) {
        trail.mesh.frustumCulled = false;
        trail.mesh.renderOrder = 100;
      }

      trails.push(trail);
    }

    return trails;
  }

  _updateColors() {
    const colors = this.teamColors[this.team] || this.teamColors[0];

    // Main trail colors
    if (this.mainTrail?.material) {
      this.mainTrail.material.uniforms.headColor.value.copy(colors.head);
      this.mainTrail.material.uniforms.tailColor.value.copy(colors.tail);
    }

    // Secondary trails - same colors but slightly dimmer
    const secondaryHead = colors.head.clone();
    secondaryHead.w = colors.head.w * 0.85;
    const secondaryTail = colors.tail.clone();

    this.secondaryTrails.forEach((trail) => {
      if (trail?.material) {
        trail.material.uniforms.headColor.value.copy(secondaryHead);
        trail.material.uniforms.tailColor.value.copy(secondaryTail);
      }
    });
  }

  _updateIntensity() {
    if (this.mainTrail?.material) {
      this.mainTrail.material.uniforms.intensityMultiplier.value = this.intensity;
    }
    this.secondaryTrails.forEach((trail) => {
      if (trail?.material) {
        trail.material.uniforms.intensityMultiplier.value = this.intensity;
      }
    });
  }

  setTeam(team) {
    if (this.team !== team) {
      this.team = team;
      this._updateColors();
    }
  }

  setIntensity(intensity) {
    this.intensity = intensity;
    if (!this.dying) {
      this._updateIntensity();
    }
  }

  /**
   * Start the death process - segment will fade out
   */
  startDying() {
    if (!this.dying) {
      this.dying = true;
      this.deathTime = 0;
      // Pause the trails so they stop growing
      this.mainTrail.pause();
      this.secondaryTrails.forEach((trail) => trail.pause());
    }
  }

  /**
   * Update trail positions
   */
  updatePosition(position, velocity, currentRotation) {
    if (this.dying) return;

    // Calculate movement direction for orientation
    const direction = velocity.clone().normalize();

    // Main trail position
    this.mainTarget.position.copy(position);
    this.mainTarget.updateMatrixWorld();

    // Update secondary trail positions (rotating around main trail)
    for (let i = 0; i < 4; i++) {
      const baseAngle = (i / 4) * Math.PI * 2;
      const angle = baseAngle + currentRotation;

      const offset = new THREE.Vector3(
        Math.cos(angle) * this.config.secondaryTrailOffset,
        Math.sin(angle) * this.config.secondaryTrailOffset,
        0,
      );

      // Rotate offset to align with movement direction
      if (direction.lengthSq() > 0.001) {
        const up = new THREE.Vector3(0, 0, 1);
        const quaternion = new THREE.Quaternion();
        quaternion.setFromUnitVectors(up, direction);
        offset.applyQuaternion(quaternion);
      }

      this.secondaryTargets[i].position.copy(position).add(offset);
      this.secondaryTargets[i].updateMatrixWorld();
    }
  }

  update(delta) {
    if (this.dying) {
      this.deathTime += delta;

      // Fade out using intensity multiplier
      const fadeProgress = Math.min(1, this.deathTime / this.maxDeathTime);
      const fadeIntensity = this.intensity * (1 - fadeProgress);

      if (this.mainTrail?.material) {
        this.mainTrail.material.uniforms.intensityMultiplier.value = fadeIntensity;
      }
      this.secondaryTrails.forEach((trail) => {
        if (trail?.material) {
          trail.material.uniforms.intensityMultiplier.value = fadeIntensity;
        }
      });

      // Mark as dead when fully faded
      if (this.deathTime >= this.maxDeathTime) {
        this.active = false;
      }
    }

    // Always update the trail renderers with delta for playback sync
    if (this.mainTrail.isActive) {
      this.mainTrail.update(delta);
    }
    this.secondaryTrails.forEach((trail) => {
      if (trail.isActive) {
        trail.update(delta);
      }
    });
  }

  dispose() {
    this.mainTrail.deactivate();
    this.secondaryTrails.forEach((trail) => trail.deactivate());

    if (this.mainTrail.geometry) this.mainTrail.geometry.dispose();
    if (this.mainTrail.material) this.mainTrail.material.dispose();

    this.secondaryTrails.forEach((trail) => {
      if (trail.geometry) trail.geometry.dispose();
      if (trail.material) trail.material.dispose();
    });

    this.scene.remove(this.mainTarget);
    this.secondaryTargets.forEach((t) => this.scene.remove(t));
  }
}

/**
 * SpiralBallTrail - Main class for ball trail effect
 * Manages multiple independent trail segments that can overlap
 */
export class SpiralBallTrail {
  /**
   * @param {THREE.Scene} scene - The scene to add trails to
   * @param {number} team - Team color (0 = blue, 1 = orange)
   */
  constructor(scene, team = 0) {
    this.scene = scene;
    this.team = team;
    this.active = false;

    // Ball properties (in Unreal Units)
    this.ballRadius = 92.75; // Ball radius in Unreal Units

    // Trail settings - shared config for all segments (in Unreal Units)
    this.config = {
      trailLength: 60,
      mainTrailWidth: 15, // 15 UU
      secondaryTrailWidth: 1.5, // Thinner satellite trails
      secondaryTrailOffset: this.ballRadius * 0.7,
    };

    // Rotation settings - 60 degrees per second (slow spiral)
    this.rotationSpeed = Math.PI / 3;
    this.currentRotation = 0;

    // Velocity thresholds for trail visibility and intensity (in UU/s)
    this.minVelocity = 1500; // Minimum velocity to show trail
    this.maxVelocity = 6000; // Velocity for full intensity (approx supersonic)
    this.minIntensity = 0.3; // Minimum opacity at minVelocity
    this.maxIntensity = 1.0; // Maximum opacity at maxVelocity

    // Track if we were emitting last frame (velocity above threshold)
    this.wasEmitting = false;

    // Store all trail segments (active and dying)
    this.segments = [];
    this.currentSegment = null;
    this.currentIntensity = 1.0;
  }

  /**
   * Calculate intensity based on velocity
   * @param {number} speed - Current ball speed
   * @returns {number} Intensity value between minIntensity and maxIntensity
   */
  _calculateIntensity(speed) {
    if (speed <= this.minVelocity) return this.minIntensity;
    if (speed >= this.maxVelocity) return this.maxIntensity;

    // Linear interpolation between min and max
    const t = (speed - this.minVelocity) / (this.maxVelocity - this.minVelocity);
    return this.minIntensity + t * (this.maxIntensity - this.minIntensity);
  }

  /**
   * Set team color
   * @param {number} team - 0 = blue, 1 = orange
   */
  setTeam(team) {
    if (this.team !== team) {
      this.team = team;
      // Kill current segment and force new one with new color
      if (this.currentSegment && !this.currentSegment.dying) {
        this.currentSegment.startDying();
        this.currentSegment = null;
        // wasEmitting stays true so next emit() creates new segment immediately
      }
    }
  }

  /**
   * Activate the trail system
   */
  activate() {
    if (!this.active) {
      this.active = true;
      this.currentSegment = null;
      this.wasEmitting = false;
    }
  }

  /**
   * Deactivate the trail system
   */
  deactivate() {
    if (this.active) {
      this.active = false;
      // Let current segment die naturally
      if (this.currentSegment) {
        this.currentSegment.startDying();
        this.currentSegment = null;
      }
    }
  }

  /**
   * Update trail emission
   * @param {THREE.Vector3} position - Ball position
   * @param {THREE.Vector3} velocity - Ball velocity
   * @param {number} delta - Time delta in seconds
   */
  emit(position, velocity, delta) {
    const speed = velocity.length();
    const shouldEmit = speed >= this.minVelocity;

    if (!shouldEmit) {
      // Velocity below threshold - kill current segment and let it fade
      if (this.currentSegment && !this.currentSegment.dying) {
        this.currentSegment.startDying();
        this.currentSegment = null;
      }
      this.wasEmitting = false;
      return;
    }

    // Calculate intensity based on speed
    this.currentIntensity = this._calculateIntensity(speed);

    // Velocity is above threshold - we should emit
    if (!this.active) {
      this.activate();
    }

    // Check if we need a new segment (wasn't emitting before or no current segment)
    if (!this.wasEmitting || !this.currentSegment) {
      // Start a new segment with current intensity
      if (this.currentSegment && !this.currentSegment.dying) {
        this.currentSegment.startDying();
      }
      this.currentSegment = new BallTrailSegment(
        this.scene,
        this.team,
        this.config,
        this.currentIntensity,
      );
      this.segments.push(this.currentSegment);
    } else {
      // Update intensity of current segment
      this.currentSegment.setIntensity(this.currentIntensity);
    }

    this.wasEmitting = true;

    // Update rotation for spiral effect
    this.currentRotation += this.rotationSpeed * delta;
    if (this.currentRotation > Math.PI * 2) {
      this.currentRotation -= Math.PI * 2;
    }

    // Update current segment position
    this.currentSegment.updatePosition(position, velocity, this.currentRotation);
  }

  /**
   * Update trails (call every frame)
   * @param {number} delta - Time delta in seconds
   */
  update(delta) {
    // Update all segments
    for (let i = this.segments.length - 1; i >= 0; i--) {
      const segment = this.segments[i];
      segment.update(delta);

      // Remove dead segments
      if (!segment.active) {
        segment.dispose();
        this.segments.splice(i, 1);
      }
    }
  }

  /**
   * Reset trails (call when seeking)
   */
  reset() {
    // Kill all segments immediately
    for (const segment of this.segments) {
      segment.dispose();
    }
    this.segments = [];
    this.currentSegment = null;
    this.currentRotation = 0;
    this.wasEmitting = false;
  }

  /**
   * Add to scene (segments add themselves)
   */
  addToScene(scene) {
    // Segments are already added via constructor
  }

  /**
   * Remove from scene
   */
  removeFromScene(scene) {
    // Kill all segments
    for (const segment of this.segments) {
      segment.startDying();
    }
    this.currentSegment = null;
  }

  /**
   * Dispose of all resources
   */
  dispose() {
    for (const segment of this.segments) {
      segment.dispose();
    }
    this.segments = [];
    this.currentSegment = null;
  }
}
