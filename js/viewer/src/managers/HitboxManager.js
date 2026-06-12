/**
 * HitboxManager - Manages car hitbox wireframe visualization
 *
 * Features:
 * - Creates wireframe boxes for each hitbox type (Octane, Dominus, Plank, Breakout, Hybrid, Merc)
 * - Updates wireframe positions/rotations to match car transforms
 * - Color coding by hitbox type
 */

import * as THREE from 'three';
import { HITBOX_DIMENSIONS } from '../data/hitboxes.js';

// Color coding by hitbox type
const HITBOX_COLORS = {
    Octane: 0x00ffff,   // Cyan
    Dominus: 0xff8800,  // Orange
    Plank: 0x88ff00,    // Lime green
    Breakout: 0xff0088, // Pink
    Hybrid: 0x8800ff,   // Purple
    Merc: 0xffff00,     // Yellow
};

export class HitboxManager {
    /**
     * @param {THREE.Scene} scene - The scene to add hitboxes to
     */
    constructor(scene) {
        this.scene = scene;
        this.hitboxes = new Map(); // Map<carActorId, { mesh, hitboxType }>
        this.enabled = false;
    }

    /**
     * Enable or disable hitbox display
     * @param {boolean} enabled
     */
    setEnabled(enabled) {
        this.enabled = enabled;

        // Show/hide all existing hitboxes
        this.hitboxes.forEach(({ mesh }) => {
            mesh.visible = enabled;
        });
    }

    /**
     * Create a wireframe hitbox mesh for a specific hitbox type
     * @param {string} hitboxType - One of: Octane, Dominus, Plank, Breakout, Hybrid, Merc
     * @returns {THREE.Group} - Group containing wireframe box and center pivot sphere
     */
    createHitboxWireframe(hitboxType) {
        const dims = HITBOX_DIMENSIONS[hitboxType] || HITBOX_DIMENSIONS.Octane;
        const color = HITBOX_COLORS[hitboxType] || HITBOX_COLORS.Octane;

        // Dimensions in Unreal Units
        const length = dims.length;
        const width = dims.width;
        const height = dims.height;

        // Hitbox offset from car's pivot/center of rotation
        const offsetX = dims.offsetX; // Forward offset
        const offsetY = dims.offsetZ; // Height offset (Z in RocketSim = Y in Three.js)

        console.log(`[HitboxManager] Creating hitbox for ${hitboxType}:`, {
            dims,
            length, width, height,
            offsetX, offsetY
        });

        // Create a group to hold hitbox + center pivot
        const group = new THREE.Group();

        // Create box geometry
        // Three.js BoxGeometry: (sizeX, sizeY, sizeZ)
        // In this viewer's coordinate system:
        //   X+ = front of car (forward direction)
        //   Y  = up/down
        //   Z  = left/right (car width)
        // So: length = X (front-back), height = Y, width = Z (left-right)
        const boxGeometry = new THREE.BoxGeometry(length, height, width);

        // Use EdgesGeometry to only show the 12 edges of the box (no diagonals)
        const edgesGeometry = new THREE.EdgesGeometry(boxGeometry);

        // Create line segments with glow effect
        // depthTest: false ensures hitbox is always visible (like in-game)
        const material = new THREE.LineBasicMaterial({
            color: color,
            linewidth: 2,
            transparent: true,
            opacity: 0.8,
            depthTest: false,
        });

        const wireframe = new THREE.LineSegments(edgesGeometry, material);
        wireframe.frustumCulled = false;

        // Apply hitbox offset from pivot point
        // X = forward, Y = up, Z = lateral (0)
        wireframe.position.set(offsetX, offsetY, 0);

        group.add(wireframe);

        // Create center pivot sphere (wireframe) at the car's rotation center (group origin)
        const pivotRadius = 3.33; // Small sphere for pivot visualization
        const pivotGeometry = new THREE.SphereGeometry(pivotRadius, 8, 6);
        const pivotWireframeGeometry = new THREE.WireframeGeometry(pivotGeometry);
        const pivotMaterial = new THREE.LineBasicMaterial({
            color: 0xffffff, // White for visibility
            linewidth: 1,
            transparent: true,
            opacity: 0.9,
            depthTest: false,
        });
        const pivotSphere = new THREE.LineSegments(pivotWireframeGeometry, pivotMaterial);
        pivotSphere.frustumCulled = false;

        group.add(pivotSphere);

        // Store metadata in userData
        group.userData.hitboxType = hitboxType;

        // Disable frustum culling on group
        group.frustumCulled = false;

        return group;
    }

    /**
     * Add or update a hitbox for a car
     * @param {string} carActorId - The car's actor ID
     * @param {string} hitboxType - The hitbox type
     */
    addHitbox(carActorId, hitboxType) {
        // Remove existing hitbox if different type
        if (this.hitboxes.has(carActorId)) {
            const existing = this.hitboxes.get(carActorId);
            if (existing.hitboxType === hitboxType) {
                return; // Already correct type
            }
            // Remove old hitbox
            this.scene.remove(existing.mesh);
            existing.mesh.geometry.dispose();
            existing.mesh.material.dispose();
        }

        // Create new hitbox
        const mesh = this.createHitboxWireframe(hitboxType);
        mesh.visible = this.enabled;

        this.scene.add(mesh);
        this.hitboxes.set(carActorId, { mesh, hitboxType });
    }

    /**
     * Remove a hitbox for a car
     * @param {string} carActorId - The car's actor ID
     */
    removeHitbox(carActorId) {
        if (this.hitboxes.has(carActorId)) {
            const { mesh } = this.hitboxes.get(carActorId);
            this.scene.remove(mesh);
            // Dispose all children (wireframe box + pivot sphere)
            mesh.traverse((child) => {
                if (child.geometry) child.geometry.dispose();
                if (child.material) child.material.dispose();
            });
            this.hitboxes.delete(carActorId);
        }
    }

    /**
     * Update hitbox positions and rotations to match car transforms
     * @param {Object} actors - Map of actor ID to actor mesh
     * @param {Object} playerNameToCarActorId - Map of player name to car actor ID
     * @param {Function} getHitboxType - Function that returns hitbox type for a player name
     */
    updateHitboxes(actors, playerNameToCarActorId, getHitboxType) {
        if (!this.enabled) return;

        // Update each car's hitbox
        for (const [playerName, carActorId] of Object.entries(playerNameToCarActorId)) {
            const carMesh = actors[carActorId];
            if (!carMesh || !carMesh.userData.isCar) continue;

            // Get hitbox type for this player
            const hitboxType = getHitboxType ? getHitboxType(playerName) : 'Octane';

            // Ensure hitbox exists
            if (!this.hitboxes.has(carActorId)) {
                this.addHitbox(carActorId, hitboxType);
            }

            const { mesh: hitboxMesh } = this.hitboxes.get(carActorId);

            // Update position - group is placed at car's pivot point
            // The hitbox wireframe inside the group is already offset by elevation
            hitboxMesh.position.copy(carMesh.position);

            // Match car rotation
            hitboxMesh.quaternion.copy(carMesh.quaternion);
        }

        // Remove hitboxes for cars that no longer exist
        const activeCarIds = new Set(Object.values(playerNameToCarActorId));
        for (const carActorId of this.hitboxes.keys()) {
            if (!activeCarIds.has(carActorId)) {
                this.removeHitbox(carActorId);
            }
        }
    }

    /**
     * Reset all hitboxes
     */
    reset() {
        this.hitboxes.forEach(({ mesh }) => {
            this.scene.remove(mesh);
            // Dispose all children (wireframe box + pivot sphere)
            mesh.traverse((child) => {
                if (child.geometry) child.geometry.dispose();
                if (child.material) child.material.dispose();
            });
        });
        this.hitboxes.clear();
    }

    /**
     * Dispose of all resources
     */
    dispose() {
        this.reset();
    }
}
