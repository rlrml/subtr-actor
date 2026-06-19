import * as THREE from "three";

/**
 * SpeedLabelManager - Renders speed labels near ball and cars using Three.js Sprites
 * Uses canvas textures to draw speed values with unit conversion
 */
export class SpeedLabelManager {
  constructor(scene, camera) {
    this.scene = scene;
    this.camera = camera;
    this.labels = new Map(); // id -> { sprite, canvas, ctx, texture, lastSpeed }
    this.playerTeams = {}; // playerName -> team (0 or 1)

    // Settings
    this.showBallSpeed = false;
    this.showCarSpeed = false;
    this.speedUnit = "kmh"; // 'kmh' or 'mph'

    // Conversion factor: Unreal Units/s to km/h
    // In Rocket League: 1 UU = 1 cm, so 100 UU = 1 meter
    // 2778 uu/s = 100 km/h, therefore factor = 100/2778 = 0.036
    this.UU_TO_KMH = 0.036;
    this.KMH_TO_MPH = 0.621371;

    // Canvas dimensions for speed label texture
    this.canvasWidth = 128;
    this.canvasHeight = 48;

    // Sprite scale
    this.spriteScale = 0.04;
    this.ballLabelOffset = 1.5; // World units above ball
    this.carLabelOffset = 0.3; // World units below name tag (offset from car)
  }

  setPlayerTeams(teams) {
    this.playerTeams = teams;
  }

  setSettings(showBallSpeed, showCarSpeed, speedUnit) {
    this.showBallSpeed = showBallSpeed;
    this.showCarSpeed = showCarSpeed;
    this.speedUnit = speedUnit;
  }

  /**
   * Convert Unreal units/s velocity to display speed
   * @param {THREE.Vector3} velocity - Velocity in Unreal units/s
   * @returns {number} Speed in current unit (km/h or mph)
   */
  calculateSpeed(velocity) {
    const speedUU = velocity.length();
    const speedKmh = speedUU * this.UU_TO_KMH;

    if (this.speedUnit === "mph") {
      return speedKmh * this.KMH_TO_MPH;
    }
    return speedKmh;
  }

  /**
   * Get unit suffix
   */
  getUnitSuffix() {
    return this.speedUnit === "mph" ? "mph" : "km/h";
  }

  /**
   * Create or update a speed label
   */
  createOrUpdateLabel(id, speed, position, isBall = false, team = 0) {
    let labelData = this.labels.get(id);

    if (!labelData) {
      labelData = this._createLabel(id);
      this.labels.set(id, labelData);
    }

    // Update texture if speed changed significantly (> 1 unit)
    const speedDiff = Math.abs((labelData.lastSpeed || 0) - speed);
    if (
      speedDiff > 1 ||
      labelData.lastSpeed === undefined ||
      labelData.lastUnit !== this.speedUnit
    ) {
      this._updateTexture(labelData, speed, isBall, team);
      labelData.lastSpeed = speed;
      labelData.lastUnit = this.speedUnit;
    }

    // Update position
    if (position && this.camera) {
      const offset = isBall ? this.ballLabelOffset : this.carLabelOffset;
      labelData.sprite.position.set(position.x, position.y + offset, position.z);
      labelData.sprite.visible = true;
    }
  }

  /**
   * Create a new label sprite with canvas
   */
  _createLabel(id) {
    const canvas = document.createElement("canvas");
    canvas.width = this.canvasWidth;
    canvas.height = this.canvasHeight;
    const ctx = canvas.getContext("2d");

    const texture = new THREE.CanvasTexture(canvas);
    texture.minFilter = THREE.LinearFilter;
    texture.magFilter = THREE.LinearFilter;

    const material = new THREE.SpriteMaterial({
      map: texture,
      transparent: true,
      depthTest: false,
      depthWrite: false,
      sizeAttenuation: false,
    });

    const sprite = new THREE.Sprite(material);
    const aspectRatio = this.canvasWidth / this.canvasHeight;
    sprite.scale.set(this.spriteScale * aspectRatio, this.spriteScale, 1);
    sprite.renderOrder = 998; // Below name tags (999)

    this.scene.add(sprite);

    return { sprite, canvas, ctx, texture };
  }

  /**
   * Update the canvas texture with current speed
   */
  _updateTexture(labelData, speed, isBall, team) {
    const { canvas, ctx, texture } = labelData;

    const width = canvas.width;
    const height = canvas.height;

    // Clear canvas
    ctx.clearRect(0, 0, width, height);

    // Format speed
    const speedText = `${Math.round(speed)}`;
    const unitText = this.getUnitSuffix();

    // Background color based on type
    let bgColor, textColor;
    if (isBall) {
      bgColor = "rgba(255, 255, 255, 0.85)";
      textColor = "#333333";
    } else {
      // Team colors
      bgColor = team === 0 ? "rgba(25, 118, 210, 0.85)" : "rgba(230, 81, 0, 0.85)";
      textColor = "#FFFFFF";
    }

    // Measure text
    ctx.font = "bold 24px Arial, sans-serif";
    const speedWidth = ctx.measureText(speedText).width;
    ctx.font = "14px Arial, sans-serif";
    const unitWidth = ctx.measureText(unitText).width;

    const totalWidth = speedWidth + unitWidth + 8;
    const padding = 8;
    const tagWidth = totalWidth + padding * 2;
    const tagHeight = 32;
    const tagX = (width - tagWidth) / 2;
    const tagY = (height - tagHeight) / 2;

    // Draw pill background
    const radius = tagHeight / 2;
    ctx.beginPath();
    ctx.roundRect(tagX, tagY, tagWidth, tagHeight, radius);
    ctx.fillStyle = bgColor;
    ctx.fill();

    // Draw border
    ctx.strokeStyle = "rgba(255, 255, 255, 0.5)";
    ctx.lineWidth = 1;
    ctx.stroke();

    // Draw speed text
    ctx.font = "bold 24px Arial, sans-serif";
    ctx.fillStyle = textColor;
    ctx.textAlign = "left";
    ctx.textBaseline = "middle";
    ctx.shadowColor = "rgba(0, 0, 0, 0.3)";
    ctx.shadowBlur = 2;

    const textX = tagX + padding;
    ctx.fillText(speedText, textX, height / 2);

    // Draw unit text (smaller)
    ctx.font = "14px Arial, sans-serif";
    ctx.fillText(unitText, textX + speedWidth + 4, height / 2 + 2);

    ctx.shadowBlur = 0;

    texture.needsUpdate = true;
  }

  /**
   * Hide a label
   */
  hideLabel(id) {
    const labelData = this.labels.get(id);
    if (labelData) {
      labelData.sprite.visible = false;
    }
  }

  /**
   * Update speed labels for ball and cars
   * @param {Object} actors - All actors in the scene
   * @param {Object} ballActor - Ball actor with position
   * @param {THREE.Vector3} ballVelocity - Ball velocity
   * @param {Object} playerNameToCarActorId - Mapping of player names to car actor IDs
   * @param {Object} playerVelocities - Player velocities { playerName: THREE.Vector3 }
   */
  update(actors, ballActor, ballVelocity, playerNameToCarActorId, playerVelocities) {
    const updatedLabels = new Set();

    // Update ball speed label
    if (this.showBallSpeed && ballActor && ballVelocity) {
      const speed = this.calculateSpeed(ballVelocity);
      this.createOrUpdateLabel("ball", speed, ballActor.position, true);
      updatedLabels.add("ball");
    }

    // Update car speed labels
    if (this.showCarSpeed) {
      Object.entries(playerNameToCarActorId).forEach(([playerName, carActorId]) => {
        const car = actors[carActorId];
        const velocity = playerVelocities?.[playerName];

        if (!car || !car.visible || !velocity) {
          this.hideLabel(`car_${playerName}`);
          return;
        }

        const team = this.playerTeams[playerName] ?? 0;
        const speed = this.calculateSpeed(velocity);

        // Position below the name tag
        const labelPosition = new THREE.Vector3(
          car.position.x,
          car.position.y - 0.3,
          car.position.z,
        );

        this.createOrUpdateLabel(`car_${playerName}`, speed, labelPosition, false, team);
        updatedLabels.add(`car_${playerName}`);
      });
    }

    // Hide labels not updated
    this.labels.forEach((labelData, id) => {
      if (!updatedLabels.has(id)) {
        labelData.sprite.visible = false;
      }
    });
  }

  /**
   * Reset all labels
   */
  reset() {
    this.labels.forEach((labelData) => {
      this.scene.remove(labelData.sprite);
      labelData.sprite.material.map.dispose();
      labelData.sprite.material.dispose();
    });
    this.labels.clear();
  }

  /**
   * Dispose of all resources
   */
  dispose() {
    this.reset();
  }
}
