/**
 * DrawingManager - Manages collaborative drawing strokes on the terrain
 *
 * Features:
 * - Renders strokes as fat lines (Line2) for consistent thickness
 * - Handles real-time incremental point updates
 * - Supports undo and erase operations
 * - Manages stroke limit with FIFO eviction
 */

import * as THREE from 'three';
import { Line2 } from 'three/addons/lines/Line2.js';
import { LineMaterial } from 'three/addons/lines/LineMaterial.js';
import { LineGeometry } from 'three/addons/lines/LineGeometry.js';

// Drawing constants
const MAX_STROKES = 100;
const LINE_HEIGHT_OFFSET = 0; // Offset now handled by raycaster via surface normal

export class DrawingManager {
  /**
   * @param {THREE.Scene} scene - The scene to add drawings to
   * @param {THREE.WebGLRenderer} renderer - The renderer (needed for resolution)
   */
  constructor(scene, renderer) {
    this.scene = scene;
    this.renderer = renderer;
    this.strokes = new Map(); // Map<strokeId, { line, material, geometry, points, authorId }>
    this.enabled = true;
  }

  /**
   * Start a new stroke
   * @param {string} strokeId - Unique stroke ID
   * @param {string} authorId - Socket ID of the author
   * @param {string} colorHex - Hex color string (#RRGGBB)
   * @param {number} thickness - Line thickness (1-10)
   * @param {{x: number, y: number, z: number}} startPoint - Initial point
   */
  startStroke(strokeId, authorId, colorHex, thickness, startPoint) {
    // Check stroke limit, remove oldest if needed
    if (this.strokes.size >= MAX_STROKES) {
      const oldestId = this.strokes.keys().next().value;
      if (oldestId) {
        this.removeStroke(oldestId);
        console.log(`[DrawingManager] FIFO eviction: removed stroke ${oldestId}`);
      }
    }

    const color = new THREE.Color(colorHex);

    // Get renderer size for LineMaterial resolution
    const size = new THREE.Vector2();
    this.renderer.getSize(size);

    // Create fat line material (supports linewidth on all platforms)
    // thickness is 1-10, we scale it to 5-50 world units for visible strokes
    const worldLineWidth = 5 + (thickness - 1) * 5; // 1->5, 5->25, 10->50
    const material = new LineMaterial({
      color: color.getHex(),
      linewidth: worldLineWidth,
      transparent: true,
      opacity: 0.9,
      depthTest: true,
      depthWrite: false, // Don't write to depth buffer to avoid z-fighting
      worldUnits: true, // Use world units for consistent thickness
    });
    material.resolution.set(size.x, size.y);

    // Create geometry with initial point (Line2 needs at least 2 points)
    const geometry = new LineGeometry();
    const positions = [
      startPoint.x, startPoint.y + LINE_HEIGHT_OFFSET, startPoint.z,
      startPoint.x, startPoint.y + LINE_HEIGHT_OFFSET, startPoint.z, // Duplicate for initial
    ];
    geometry.setPositions(positions);

    // Create Line2 object
    const line = new Line2(geometry, material);
    line.computeLineDistances();
    line.frustumCulled = false;
    line.renderOrder = 999; // Render after arena

    this.scene.add(line);
    this.strokes.set(strokeId, {
      line,
      material,
      geometry,
      points: [startPoint],
      authorId,
      color: colorHex,
      thickness,
    });

    console.log(`[DrawingManager] Started stroke ${strokeId}`);
  }

  /**
   * Add points to an existing stroke
   * @param {string} strokeId - The stroke to update
   * @param {Array<{x: number, y: number, z: number}>} newPoints - Points to add
   */
  addPoints(strokeId, newPoints) {
    const stroke = this.strokes.get(strokeId);
    if (!stroke) {
      console.warn(`[DrawingManager] Stroke ${strokeId} not found for adding points`);
      return;
    }

    // Add new points
    stroke.points.push(...newPoints);

    // Build positions array for LineGeometry
    const positions = [];
    for (const p of stroke.points) {
      positions.push(p.x, p.y + LINE_HEIGHT_OFFSET, p.z);
    }

    // Update geometry
    stroke.geometry.dispose();
    stroke.geometry = new LineGeometry();
    stroke.geometry.setPositions(positions);
    stroke.line.geometry = stroke.geometry;
    stroke.line.computeLineDistances();
  }

  /**
   * Complete a stroke (called when draw-stroke-end is received)
   * @param {string} strokeId
   */
  completeStroke(strokeId) {
    const stroke = this.strokes.get(strokeId);
    if (!stroke) return;

    // Mark as completed (for potential future use)
    stroke.completed = true;
    console.log(`[DrawingManager] Completed stroke ${strokeId} with ${stroke.points.length} points`);
  }

  /**
   * Remove a stroke by ID
   * @param {string} strokeId
   * @returns {boolean} True if stroke was removed
   */
  removeStroke(strokeId) {
    const stroke = this.strokes.get(strokeId);
    if (!stroke) return false;

    // Dispose resources
    stroke.geometry.dispose();
    stroke.material.dispose();
    this.scene.remove(stroke.line);
    this.strokes.delete(strokeId);

    console.log(`[DrawingManager] Removed stroke ${strokeId}`);
    return true;
  }

  /**
   * Get stroke data by ID
   * @param {string} strokeId
   * @returns {Object|null}
   */
  getStroke(strokeId) {
    return this.strokes.get(strokeId) || null;
  }

  /**
   * Get all stroke IDs for a specific author
   * @param {string} authorId
   * @returns {string[]}
   */
  getStrokesByAuthor(authorId) {
    const result = [];
    for (const [id, stroke] of this.strokes) {
      if (stroke.authorId === authorId) {
        result.push(id);
      }
    }
    return result;
  }

  /**
   * Clear all strokes
   */
  clearAll() {
    for (const strokeId of this.strokes.keys()) {
      this.removeStroke(strokeId);
    }
    console.log('[DrawingManager] Cleared all strokes');
  }

  /**
   * Sync all strokes from server state
   * @param {Array<Object>} strokesData - Array of stroke data objects
   */
  syncStrokes(strokesData) {
    // Clear existing strokes
    this.clearAll();

    // Add all strokes from server
    for (const strokeData of strokesData) {
      // Start with first point
      if (strokeData.points && strokeData.points.length > 0) {
        this.startStroke(
          strokeData.id,
          strokeData.authorId,
          strokeData.color,
          strokeData.thickness,
          strokeData.points[0]
        );
        // Add remaining points
        if (strokeData.points.length > 1) {
          this.addPoints(strokeData.id, strokeData.points.slice(1));
        }
        // Mark as completed since it's from sync
        this.completeStroke(strokeData.id);
      }
    }

    console.log(`[DrawingManager] Synced ${strokesData.length} strokes`);
  }

  /**
   * Check if a point is near any stroke (for eraser)
   * @param {THREE.Vector3} point - Point to check
   * @param {number} radius - Detection radius
   * @returns {string[]} Array of strokeIds that intersect
   */
  getStrokesNearPoint(point, radius = 30) {
    const results = [];

    for (const [strokeId, stroke] of this.strokes) {
      for (const p of stroke.points) {
        const dist = Math.sqrt(
          Math.pow(point.x - p.x, 2) +
          Math.pow(point.y - p.y, 2) +
          Math.pow(point.z - p.z, 2)
        );
        if (dist <= radius) {
          results.push(strokeId);
          break; // Only need to find one point per stroke
        }
      }
    }

    return results;
  }

  /**
   * Update resolution when window resizes
   */
  updateResolution() {
    if (!this.renderer) return;

    const size = new THREE.Vector2();
    this.renderer.getSize(size);

    for (const stroke of this.strokes.values()) {
      stroke.material.resolution.set(size.x, size.y);
    }
  }

  /**
   * Update method (called in animation loop)
   * @param {number} deltaTime
   */
  update(deltaTime) {
    // Currently no per-frame updates needed
    // Could add drawing animations here in the future
  }

  /**
   * Get total stroke count
   * @returns {number}
   */
  getStrokeCount() {
    return this.strokes.size;
  }

  /**
   * Clean up all resources
   */
  dispose() {
    this.clearAll();
  }
}
