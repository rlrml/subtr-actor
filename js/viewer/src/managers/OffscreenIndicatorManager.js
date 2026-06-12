/**
 * OffscreenIndicatorManager - Shows directional arrows for off-screen pings
 *
 * Features:
 * - Displays arrow indicators on screen edges pointing to off-screen pings
 * - Arrows colored by ping author's color
 * - Arrows fade when ping expires
 * - Calculates 3D to 2D projection for positioning
 */

import * as THREE from 'three';

// Indicator constants
const INDICATOR_SIZE = 30;        // Arrow size in pixels
const EDGE_MARGIN = 50;           // Distance from screen edge

export class OffscreenIndicatorManager {
  /**
   * @param {THREE.Camera} camera - The camera for projections
   * @param {HTMLElement} container - Container element for indicators
   */
  constructor(camera, container) {
    this.camera = camera;
    this.container = container;
    this.indicators = new Map(); // Map<pingId, HTMLElement>
    this.enabled = true;
  }

  /**
   * Set the camera reference
   * @param {THREE.Camera} camera
   */
  setCamera(camera) {
    this.camera = camera;
  }

  /**
   * Create an indicator for a ping
   * @param {string} pingId
   * @param {string} color - Hex color string
   */
  createIndicator(pingId, color) {
    if (this.indicators.has(pingId)) return;

    const halfSize = INDICATOR_SIZE / 2;
    const indicator = document.createElement('div');
    indicator.className = 'offscreen-indicator';
    indicator.style.position = 'absolute';
    indicator.style.width = '0';
    indicator.style.height = '0';
    indicator.style.borderLeft = halfSize + 'px solid transparent';
    indicator.style.borderRight = halfSize + 'px solid transparent';
    indicator.style.borderBottom = INDICATOR_SIZE + 'px solid ' + color;
    indicator.style.opacity = '0.8';
    indicator.style.pointerEvents = 'none';
    indicator.style.zIndex = '1000';
    indicator.style.filter = 'drop-shadow(0 0 4px ' + color + ')';
    indicator.style.transition = 'opacity 0.3s ease-out';
    indicator.style.display = 'none';

    this.container.appendChild(indicator);
    this.indicators.set(pingId, indicator);
  }

  /**
   * Remove an indicator
   * @param {string} pingId
   */
  removeIndicator(pingId) {
    const indicator = this.indicators.get(pingId);
    if (!indicator) return;

    indicator.remove();
    this.indicators.delete(pingId);
  }

  /**
   * Update indicator position for a target position
   * @param {string} pingId
   * @param {THREE.Vector3} targetPosition - 3D world position of the ping
   * @param {number} opacity - Current opacity (for fade-out)
   */
  updateIndicator(pingId, targetPosition, opacity) {
    if (opacity === undefined) opacity = 1;
    
    const indicator = this.indicators.get(pingId);
    if (!indicator || !this.camera) return;

    // Get screen dimensions
    const rect = this.container.getBoundingClientRect();
    const width = rect.width;
    const height = rect.height;

    // Project 3D position to 2D screen coordinates
    const screenPos = targetPosition.clone().project(this.camera);
    
    // Convert from NDC (-1 to 1) to screen coordinates
    const x = (screenPos.x + 1) / 2 * width;
    const y = (-screenPos.y + 1) / 2 * height;

    // Check if target is behind camera
    const isBehind = screenPos.z > 1;

    // Check if target is on screen
    const isOnScreen = !isBehind && 
                       x >= EDGE_MARGIN && 
                       x <= width - EDGE_MARGIN &&
                       y >= EDGE_MARGIN && 
                       y <= height - EDGE_MARGIN;

    if (isOnScreen) {
      indicator.style.display = 'none';
      return;
    }

    // Calculate direction from screen center to target
    const centerX = width / 2;
    const centerY = height / 2;

    var dirX = x - centerX;
    var dirY = y - centerY;

    // If behind camera, invert direction
    if (isBehind) {
      dirX = -dirX;
      dirY = -dirY;
    }

    // Normalize direction
    const length = Math.sqrt(dirX * dirX + dirY * dirY);
    if (length === 0) {
      indicator.style.display = 'none';
      return;
    }
    dirX /= length;
    dirY /= length;

    // Calculate edge intersection
    const edgeX = this.clamp(centerX + dirX * (width / 2 - EDGE_MARGIN), EDGE_MARGIN, width - EDGE_MARGIN);
    const edgeY = this.clamp(centerY + dirY * (height / 2 - EDGE_MARGIN), EDGE_MARGIN, height - EDGE_MARGIN);

    // Calculate rotation angle (arrow points towards target)
    const angle = Math.atan2(dirY, dirX) + Math.PI / 2; // +90 degrees because arrow points up by default

    // Position and rotate indicator
    indicator.style.display = 'block';
    indicator.style.left = (edgeX - INDICATOR_SIZE / 2) + 'px';
    indicator.style.top = (edgeY - INDICATOR_SIZE / 2) + 'px';
    indicator.style.transform = 'rotate(' + angle + 'rad)';
    indicator.style.opacity = opacity.toString();
  }

  /**
   * Clamp a value between min and max
   * @param {number} value
   * @param {number} min
   * @param {number} max
   * @returns {number}
   */
  clamp(value, min, max) {
    return Math.max(min, Math.min(max, value));
  }

  /**
   * Update all indicators (call from animation loop)
   * @param {Array<{ id: string, position: THREE.Vector3, color: string, opacity?: number }>} pings
   */
  update(pings) {
    if (!this.enabled || !this.camera) return;

    // Create/update indicators for active pings
    const activePingIds = new Set();

    for (var i = 0; i < pings.length; i++) {
      var ping = pings[i];
      activePingIds.add(ping.id);

      if (!this.indicators.has(ping.id)) {
        this.createIndicator(ping.id, ping.color);
      }

      var pingOpacity = ping.opacity !== undefined ? ping.opacity : 1;
      this.updateIndicator(ping.id, ping.position, pingOpacity);
    }

    // Remove indicators for expired pings
    for (const pingId of this.indicators.keys()) {
      if (!activePingIds.has(pingId)) {
        this.removeIndicator(pingId);
      }
    }
  }

  /**
   * Enable/disable indicators
   * @param {boolean} enabled
   */
  setEnabled(enabled) {
    this.enabled = enabled;
    if (!enabled) {
      for (const indicator of this.indicators.values()) {
        indicator.style.display = 'none';
      }
    }
  }

  /**
   * Clean up all resources
   */
  dispose() {
    for (const indicator of this.indicators.values()) {
      indicator.remove();
    }
    this.indicators.clear();
  }
}
