import * as THREE from 'three';

/**
 * NameTagManager - Renders player name tags above cars using Three.js Sprites
 * Uses canvas textures to draw name tags with boost indicators (pie chart style)
 */
export class NameTagManager {
    constructor(scene, camera) {
        this.scene = scene;
        this.camera = camera;
        this.nameTags = new Map(); // playerName -> { sprite, canvas, ctx, lastBoost }
        this.playerTeams = {}; // playerName -> team (0 or 1)

        // Team colors matching Rocket League
        this.teamColors = {
            0: { // Blue team
                bg: '#1976D2',
                border: '#FFFFFF',
                text: '#FFFFFF'
            },
            1: { // Orange team
                bg: '#E65100',
                border: '#FFFFFF',
                text: '#FFFFFF'
            }
        };

        // Canvas dimensions for name tag texture
        this.canvasWidth = 256;
        this.canvasHeight = 80;

        // Sprite scale (screen units since sizeAttenuation is false)
        this.spriteScale = 0.06;
        this.spriteWorldHeight = 1.2; // World units above car
    }

    setPlayerTeams(teams) {
        this.playerTeams = teams;
    }

    /**
     * Create or update a name tag for a player
     */
    createOrUpdateNameTag(playerName, boost, carPosition) {
        let tagData = this.nameTags.get(playerName);

        if (!tagData) {
            // Create new name tag
            tagData = this._createNameTag(playerName);
            this.nameTags.set(playerName, tagData);
        }

        // Update texture if boost changed significantly
        const boostDiff = Math.abs((tagData.lastBoost || 0) - boost);
        if (boostDiff > 1 || tagData.lastBoost === undefined) {
            this._updateTexture(tagData, playerName, boost);
            tagData.lastBoost = boost;
        }

        // Update position (above car, height adjusts based on camera distance)
        if (carPosition && this.camera) {
            // Calculate distance from camera to car
            const distance = this.camera.position.distanceTo(carPosition);

            // Adjust height based on distance (scaled for 100x arena):
            // Close (distance < 500): lower height (~80)
            // Far (distance > 5000): higher height (~200)
            const minHeight = 80;
            const maxHeight = 200;
            const minDist = 500;
            const maxDist = 5000;

            const t = Math.max(0, Math.min(1, (distance - minDist) / (maxDist - minDist)));
            const dynamicHeight = minHeight + t * (maxHeight - minHeight);

            tagData.sprite.position.set(
                carPosition.x,
                carPosition.y + dynamicHeight,
                carPosition.z
            );

            // Hybrid size behavior:
            // - Far away: constant screen size (base scale)
            // - Close up: grows on screen (world-space size)
            const proximityThreshold = 800; // Distance at which size starts growing
            const aspectRatio = this.canvasWidth / this.canvasHeight;

            if (distance < proximityThreshold) {
                // Close: scale increases as we get closer (simulates world-space size)
                const growthFactor = proximityThreshold / Math.max(distance, 100);
                const closeScale = this.spriteScale * growthFactor;
                tagData.sprite.scale.set(closeScale * aspectRatio, closeScale, 1);
            } else {
                // Far: constant screen size
                tagData.sprite.scale.set(this.spriteScale * aspectRatio, this.spriteScale, 1);
            }

            tagData.sprite.visible = true;
        }
    }

    /**
     * Create a new name tag sprite with canvas
     */
    _createNameTag(playerName) {
        // Create canvas for texture
        const canvas = document.createElement('canvas');
        canvas.width = this.canvasWidth;
        canvas.height = this.canvasHeight;
        const ctx = canvas.getContext('2d');

        // Create texture from canvas
        const texture = new THREE.CanvasTexture(canvas);
        texture.minFilter = THREE.LinearFilter;
        texture.magFilter = THREE.LinearFilter;

        // Create sprite material
        // sizeAttenuation: false = constant screen size regardless of distance
        // depthTest: false = always render on top of everything
        const material = new THREE.SpriteMaterial({
            map: texture,
            transparent: true,
            depthTest: false,
            depthWrite: false,
            sizeAttenuation: false
        });

        // Create sprite
        const sprite = new THREE.Sprite(material);

        // Scale sprite (maintain aspect ratio)
        const aspectRatio = this.canvasWidth / this.canvasHeight;
        sprite.scale.set(this.spriteScale * aspectRatio, this.spriteScale, 1);

        // Render after everything else
        sprite.renderOrder = 999;

        // Add to scene
        this.scene.add(sprite);

        return { sprite, canvas, ctx, texture };
    }

    /**
     * Update the canvas texture with current boost value
     */
    _updateTexture(tagData, playerName, boost) {
        const { canvas, ctx, texture } = tagData;
        const team = this.playerTeams[playerName] ?? 0;
        const colors = this.teamColors[team];

        const width = canvas.width;
        const height = canvas.height;

        // Clear canvas
        ctx.clearRect(0, 0, width, height);

        // Draw name tag background with pill shape (100% rounded X borders)
        const padding = 4;
        const tagHeight = 44;
        const tagY = (height - tagHeight) / 2;
        const boostCircleSize = 28;
        const boostCircleMargin = 8;

        // Measure text to determine tag width
        ctx.font = 'bold 20px Arial, sans-serif';
        const textWidth = ctx.measureText(playerName).width;
        const tagWidth = textWidth + boostCircleSize + boostCircleMargin * 3 + padding * 2;
        const tagX = (width - tagWidth) / 2;

        // Draw pill shape (100% rounded on X axis)
        const radius = tagHeight / 2;
        ctx.beginPath();
        ctx.roundRect(tagX, tagY, tagWidth, tagHeight, radius);

        // Solid background color (no gradient)
        ctx.fillStyle = colors.bg;
        ctx.fill();

        // Draw white border
        ctx.strokeStyle = colors.border;
        ctx.lineWidth = 3;
        ctx.stroke();

        // Draw boost indicator (pie chart)
        const boostCenterX = tagX + boostCircleMargin + boostCircleSize / 2 + 2;
        const boostCenterY = height / 2;
        const boostRadius = boostCircleSize / 2 - 2;

        // Background circle (same color as team background)
        ctx.beginPath();
        ctx.arc(boostCenterX, boostCenterY, boostRadius, 0, Math.PI * 2);
        ctx.fillStyle = colors.bg;
        ctx.fill();
        ctx.strokeStyle = '#FFFFFF';
        ctx.lineWidth = 2;
        ctx.stroke();

        // Boost vertical fill (fills from bottom to top inside the circle)
        if (boost > 0) {
            const boostPercentage = Math.min(100, Math.max(0, boost)) / 100;

            // Calculate the fill height
            const fillHeight = boostRadius * 2 * boostPercentage;
            const fillTop = boostCenterY + boostRadius - fillHeight;

            // Clip to circle shape
            ctx.save();
            ctx.beginPath();
            ctx.arc(boostCenterX, boostCenterY, boostRadius - 1, 0, Math.PI * 2);
            ctx.clip();

            // Draw filled rectangle from bottom up
            ctx.fillStyle = '#FFFFFF';
            ctx.fillRect(
                boostCenterX - boostRadius,
                fillTop,
                boostRadius * 2,
                fillHeight
            );

            ctx.restore();

            // Glow effect when full
            if (boost >= 100) {
                ctx.shadowColor = 'rgba(255, 255, 255, 0.8)';
                ctx.shadowBlur = 10;
                ctx.beginPath();
                ctx.arc(boostCenterX, boostCenterY, boostRadius - 1, 0, Math.PI * 2);
                ctx.fillStyle = '#FFFFFF';
                ctx.fill();
                ctx.shadowBlur = 0;
            }
        }

        // Draw player name
        ctx.font = 'bold 20px Arial, sans-serif';
        ctx.fillStyle = colors.text;
        ctx.textAlign = 'left';
        ctx.textBaseline = 'middle';
        ctx.shadowColor = 'rgba(0, 0, 0, 0.5)';
        ctx.shadowBlur = 3;
        ctx.shadowOffsetX = 1;
        ctx.shadowOffsetY = 1;

        const textX = boostCenterX + boostRadius + boostCircleMargin;
        ctx.fillText(playerName, textX, height / 2);
        ctx.shadowBlur = 0;

        // Update texture
        texture.needsUpdate = true;
    }

    /**
     * Hide a name tag (when car is not visible)
     */
    hideNameTag(playerName) {
        const tagData = this.nameTags.get(playerName);
        if (tagData) {
            tagData.sprite.visible = false;
        }
    }

    /**
     * Update all name tags based on actor data
     * Called each frame from GameEngine
     * @param {Object} actors - All actors in the scene
     * @param {Object} playerBoosts - Player boost amounts
     * @param {Object} playerNameToCarActorId - Mapping of player names to car actor IDs
     * @param {string|null} followedPlayer - Player being followed in player cam (hide their tag)
     */
    update(actors, playerBoosts, playerNameToCarActorId, followedPlayer = null) {
        // Track which players were updated
        const updatedPlayers = new Set();

        // Update each player's name tag
        Object.entries(playerNameToCarActorId).forEach(([playerName, carActorId]) => {
            // Hide name tag for the followed player in player cam
            if (playerName === followedPlayer) {
                this.hideNameTag(playerName);
                updatedPlayers.add(playerName);
                return;
            }

            const car = actors[carActorId];
            if (!car || !car.visible) {
                this.hideNameTag(playerName);
                return;
            }

            const boost = playerBoosts[playerName] ?? 0;
            this.createOrUpdateNameTag(playerName, boost, car.position);
            updatedPlayers.add(playerName);
        });

        // Hide name tags for players not updated
        this.nameTags.forEach((tagData, playerName) => {
            if (!updatedPlayers.has(playerName)) {
                tagData.sprite.visible = false;
            }
        });
    }

    /**
     * Reset all name tags
     */
    reset() {
        this.nameTags.forEach((tagData) => {
            this.scene.remove(tagData.sprite);
            tagData.sprite.material.map.dispose();
            tagData.sprite.material.dispose();
        });
        this.nameTags.clear();
    }

    /**
     * Dispose of all resources
     */
    dispose() {
        this.reset();
    }
}
