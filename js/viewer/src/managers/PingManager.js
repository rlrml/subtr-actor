/**
 * PingManager - Manages ping markers on the terrain
 *
 * Features:
 * - Creates visual ping markers at 3D positions
 * - Fade-out animation on expiration (5 seconds)
 * - Colored by participant color
 * - One active ping per participant maximum
 */

import * as THREE from 'three';

// Ping visual constants
const PING_RADIUS = 150;      // Radius of the ping circle (increased for visibility)
const PING_HEIGHT = 600;      // Height of the ping arrow/pillar (increased)
const PING_FADE_DURATION = 500; // Fade-out duration in ms
const WAVE_COUNT = 3;         // Number of expanding wave spheres
const WAVE_MAX_RADIUS = 800;  // Maximum radius of wave expansion
const WAVE_SPEED = 0.35;      // Wave expansion speed (cycles per second)

export class PingManager {
  /**
   * @param {THREE.Scene} scene - The scene to add pings to
   */
  constructor(scene) {
    this.scene = scene;
    this.pings = new Map(); // Map<pingId, { group, expiresAt, authorId }>
    this.enabled = true;
  }

  /**
   * Create a ping marker at the specified position
   * @param {Object} pingData - Ping data from server
   * @param {string} pingData.id - Unique ping ID
   * @param {string} pingData.authorId - Socket ID of the author
   * @param {string} pingData.authorNickname - Nickname for display
   * @param {string} pingData.authorColor - Hex color string (#RRGGBB)
   * @param {{ x: number, y: number, z: number }} pingData.position - 3D position
   * @param {{ x: number, y: number, z: number }} [pingData.normal] - Surface normal for orientation
   * @param {number} pingData.createdAt - Creation timestamp
   * @param {number} pingData.expiresAt - Expiration timestamp
   */
  createPing(pingData) {
    // Remove existing ping from same author
    this.removeByAuthor(pingData.authorId);

    const color = new THREE.Color(pingData.authorColor);

    // Create a group to hold all ping visuals
    const group = new THREE.Group();
    group.position.set(pingData.position.x, pingData.position.y, pingData.position.z);

    // Orient the ping according to surface normal (if provided)
    if (pingData.normal) {
      const normal = new THREE.Vector3(pingData.normal.x, pingData.normal.y, pingData.normal.z).normalize();
      const up = new THREE.Vector3(0, 1, 0);
      const quaternion = new THREE.Quaternion().setFromUnitVectors(up, normal);
      group.quaternion.copy(quaternion);
    }

    // Create base circle (flat ring on the ground)
    const ringGeometry = new THREE.RingGeometry(PING_RADIUS * 0.7, PING_RADIUS, 32);
    const ringMaterial = new THREE.MeshBasicMaterial({
      color,
      transparent: true,
      opacity: 0.9,
      side: THREE.DoubleSide,
      depthTest: false,
    });
    const ring = new THREE.Mesh(ringGeometry, ringMaterial);
    ring.rotation.x = -Math.PI / 2; // Lay flat on the ground
    ring.position.y = 10; // Slightly above ground to avoid z-fighting
    ring.renderOrder = 998;
    group.add(ring);

    // Create vertical pillar/arrow
    const pillarGeometry = new THREE.CylinderGeometry(15, 15, PING_HEIGHT, 12);
    const pillarMaterial = new THREE.MeshBasicMaterial({
      color,
      transparent: true,
      opacity: 0.7,
      depthTest: false,
    });
    const pillar = new THREE.Mesh(pillarGeometry, pillarMaterial);
    pillar.position.y = PING_HEIGHT / 2;
    pillar.renderOrder = 998;
    group.add(pillar);

    // Create cone at top (larger and more visible)
    const coneGeometry = new THREE.ConeGeometry(50, 120, 12);
    const coneMaterial = new THREE.MeshBasicMaterial({
      color,
      transparent: true,
      opacity: 1.0,
      depthTest: false,
    });
    const cone = new THREE.Mesh(coneGeometry, coneMaterial);
    cone.position.y = PING_HEIGHT + 60;
    cone.rotation.x = Math.PI; // Point downward
    cone.renderOrder = 998;
    group.add(cone);

    // Create expanding wave spheres for long-distance visibility
    const waveMaterials = [];
    const waveMeshes = [];
    for (let i = 0; i < WAVE_COUNT; i++) {
      const waveGeometry = new THREE.SphereGeometry(1, 16, 8);
      const waveMaterial = new THREE.MeshBasicMaterial({
        color,
        transparent: true,
        opacity: 0.4,
        side: THREE.DoubleSide,
        depthTest: false,
      });
      const waveMesh = new THREE.Mesh(waveGeometry, waveMaterial);
      waveMesh.position.y = 0; // Center at the base of the ping
      waveMesh.renderOrder = 997;
      group.add(waveMesh);
      waveMaterials.push(waveMaterial);
      waveMeshes.push(waveMesh);
    }

    // Add pulsing animation data
    group.userData = {
      pingId: pingData.id,
      authorId: pingData.authorId,
      expiresAt: pingData.expiresAt,
      materials: [ringMaterial, pillarMaterial, coneMaterial, ...waveMaterials],
      baseOpacities: [0.9, 0.7, 1.0, ...waveMaterials.map(() => 0.6)],
      createdAt: pingData.createdAt,
      waveMeshes,
      waveMaterials,
    };

    this.scene.add(group);
    this.pings.set(pingData.id, {
      group,
      expiresAt: pingData.expiresAt,
      authorId: pingData.authorId,
      color: pingData.authorColor,
    });

    console.log(`[PingManager] Created ping ${pingData.id} at`, pingData.position);
  }

  /**
   * Remove a ping by ID
   * @param {string} pingId
   */
  removePing(pingId) {
    const ping = this.pings.get(pingId);
    if (!ping) return;

    // Dispose of geometries and materials
    ping.group.traverse((child) => {
      if (child.geometry) child.geometry.dispose();
      if (child.material) child.material.dispose();
    });

    this.scene.remove(ping.group);
    this.pings.delete(pingId);

    console.log(`[PingManager] Removed ping ${pingId}`);
  }

  /**
   * Remove ping by author ID
   * @param {string} authorId
   */
  removeByAuthor(authorId) {
    for (const [pingId, ping] of this.pings) {
      if (ping.authorId === authorId) {
        this.removePing(pingId);
        return;
      }
    }
  }

  /**
   * Start fade-out animation for a ping
   * @param {string} pingId
   */
  fadeOutPing(pingId) {
    const ping = this.pings.get(pingId);
    if (!ping) return;

    ping.fadeStart = Date.now();
    ping.fading = true;
  }

  /**
   * Update all pings (call in animation loop)
   * @param {number} deltaTime - Time since last update in seconds
   */
  update(deltaTime) {
    const now = Date.now();

    for (const [pingId, ping] of this.pings) {
      const group = ping.group;
      const { materials, baseOpacities, createdAt, expiresAt } = group.userData;

      // Check for expiration - start fade-out 500ms before
      if (!ping.fading && now >= expiresAt - PING_FADE_DURATION) {
        ping.fading = true;
        ping.fadeStart = now;
      }

      // Handle fade-out animation
      if (ping.fading) {
        const fadeProgress = Math.min(1, (now - ping.fadeStart) / PING_FADE_DURATION);

        materials.forEach((mat, i) => {
          mat.opacity = baseOpacities[i] * (1 - fadeProgress);
        });

        if (fadeProgress >= 1) {
          this.removePing(pingId);
          continue;
        }
      }

      // Pulse animation (gentle scale breathing) for main elements
      const age = (now - createdAt) / 1000;
      const pulseScale = 1 + Math.sin(age * 3) * 0.1;
      group.scale.setScalar(pulseScale);

      // Animate expanding wave spheres
      const { waveMeshes, waveMaterials } = group.userData;
      if (waveMeshes && waveMaterials) {
        for (let i = 0; i < waveMeshes.length; i++) {
          // Each wave is offset by 1/WAVE_COUNT of a cycle
          const wavePhase = (age * WAVE_SPEED + i / WAVE_COUNT) % 1;
          const waveRadius = wavePhase * WAVE_MAX_RADIUS;
          const baseWaveOpacity = (1 - wavePhase) * 0.4; // Fade out as it expands

          waveMeshes[i].scale.setScalar(waveRadius);
          waveMaterials[i].opacity = ping.fading
            ? baseWaveOpacity * (1 - (now - ping.fadeStart) / PING_FADE_DURATION)
            : baseWaveOpacity;
        }
      }
    }
  }

  /**
   * Get all active pings
   * @returns {Array<{ id: string, authorId: string, position: THREE.Vector3 }>}
   */
  getActivePings() {
    return Array.from(this.pings.entries()).map(([id, ping]) => {
      // Get current opacity from first material
      const materials = ping.group.userData.materials;
      const baseOpacities = ping.group.userData.baseOpacities;
      const currentOpacity = materials && materials[0] ? materials[0].opacity / baseOpacities[0] : 1;

      return {
        id,
        authorId: ping.authorId,
        position: ping.group.position.clone(),
        color: ping.color,
        opacity: currentOpacity,
      };
    });
  }

  /**
   * Clean up all resources
   */
  dispose() {
    for (const pingId of this.pings.keys()) {
      this.removePing(pingId);
    }
  }
}
