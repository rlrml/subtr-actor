import * as THREE from 'three';
import { TrailRenderer } from '../lib/TrailRenderer.js';
import { SpiralBallTrail } from '../lib/BallTrailRenderer.js';

// ============================================================================
// Cached textures for explosion effects (created once, reused for all explosions)
// ============================================================================

let _particleTexture = null;
let _glowTexture = null;
let _smokeTexture = null;
let _texturesInitialized = false;

/**
 * Pre-initialize all textures to avoid freeze on first explosion
 * Call this early in app startup
 */
export function initExplosionTextures() {
    if (_texturesInitialized) return;
    getParticleTexture();
    getGlowTexture();
    getSmokeTexture();
    _texturesInitialized = true;
}

// Shared materials for explosions (created once, reused by all explosions)
// This avoids WebGL shader recompilation on each explosion
let _sharedExplosionMaterials = null;

// Material pools - pre-created materials to avoid cloning (which causes shader recompilation)
let _materialPools = null;

// Object pools - pre-created THREE objects (geometry + material) to avoid shader recompilation
// WebGL compiles a unique shader program for each geometry+material combination
let _objectPools = null;

/**
 * Get shared explosion materials (creates them on first call)
 */
function getSharedExplosionMaterials() {
    if (_sharedExplosionMaterials) return _sharedExplosionMaterials;

    _sharedExplosionMaterials = {
        // Sparks particle material
        particle: new THREE.PointsMaterial({
            size: 15, // Size in UU
            map: getParticleTexture(),
            vertexColors: true,
            transparent: true,
            opacity: 1.0,
            blending: THREE.AdditiveBlending,
            depthWrite: false,
            sizeAttenuation: true
        }),
        // Horizontal shockwave - orange color like explosion.test
        ringHorizontal: new THREE.MeshBasicMaterial({
            color: 0xffaa44,
            transparent: true,
            opacity: 0.8,
            side: THREE.DoubleSide,
            blending: THREE.AdditiveBlending,
            depthWrite: false
        }),
        // Vertical shockwave - darker orange like explosion.test
        ringVertical: new THREE.MeshBasicMaterial({
            color: 0xff6622,
            transparent: true,
            opacity: 0.6,
            side: THREE.DoubleSide,
            blending: THREE.AdditiveBlending,
            depthWrite: false
        }),
        // Debris material
        debris: new THREE.MeshBasicMaterial({
            color: 0x664422
        })
    };

    return _sharedExplosionMaterials;
}

/**
 * Create material pools with pre-compiled materials
 * These are allocated once at startup and reused for all explosions
 */
function createMaterialPools() {
    if (_materialPools) return _materialPools;

    const glowTexture = getGlowTexture();
    const smokeTexture = getSmokeTexture();

    // Pool sizes - enough for concurrent explosions
    const FIREBALL_POOL_SIZE = 50;  // 40 per explosion + buffer
    const SMOKE_POOL_SIZE = 20;     // 15 per explosion + buffer
    const SHOCKWAVE_POOL_SIZE = 4;  // 2 per explosion x 2 concurrent
    const COREFLASH_POOL_SIZE = 3;  // 1 per explosion x 3 concurrent
    const DEBRIS_POOL_SIZE = 25;    // 20 per explosion + buffer

    _materialPools = {
        fireball: [],
        smoke: [],
        shockwaveH: [],
        shockwaveV: [],
        coreFlash: [],
        debris: [],
        // Track usage
        fireballIndex: 0,
        smokeIndex: 0,
        shockwaveHIndex: 0,
        shockwaveVIndex: 0,
        coreFlashIndex: 0,
        debrisIndex: 0
    };

    // Pre-create fireball materials with varied colors
    for (let i = 0; i < FIREBALL_POOL_SIZE; i++) {
        const mat = new THREE.SpriteMaterial({
            map: glowTexture,
            color: new THREE.Color().setHSL(0.08 - (i / FIREBALL_POOL_SIZE) * 0.08, 1, 0.5 + (i / FIREBALL_POOL_SIZE) * 0.3),
            transparent: true,
            blending: THREE.AdditiveBlending,
            depthWrite: false
        });
        _materialPools.fireball.push(mat);
    }

    // Pre-create smoke materials
    for (let i = 0; i < SMOKE_POOL_SIZE; i++) {
        const mat = new THREE.SpriteMaterial({
            map: smokeTexture,
            color: 0x222222,
            transparent: true,
            opacity: 0,
            depthWrite: false
        });
        _materialPools.smoke.push(mat);
    }

    // Pre-create shockwave materials (horizontal)
    for (let i = 0; i < SHOCKWAVE_POOL_SIZE; i++) {
        const mat = new THREE.MeshBasicMaterial({
            color: 0xffaa44,
            transparent: true,
            opacity: 0.8,
            side: THREE.DoubleSide,
            blending: THREE.AdditiveBlending,
            depthWrite: false
        });
        _materialPools.shockwaveH.push(mat);
    }

    // Pre-create shockwave materials (vertical)
    for (let i = 0; i < SHOCKWAVE_POOL_SIZE; i++) {
        const mat = new THREE.MeshBasicMaterial({
            color: 0xff6622,
            transparent: true,
            opacity: 0.6,
            side: THREE.DoubleSide,
            blending: THREE.AdditiveBlending,
            depthWrite: false
        });
        _materialPools.shockwaveV.push(mat);
    }

    // Pre-create core flash materials
    for (let i = 0; i < COREFLASH_POOL_SIZE; i++) {
        const mat = new THREE.SpriteMaterial({
            map: glowTexture,
            color: 0xffffaa,
            transparent: true,
            blending: THREE.AdditiveBlending,
            depthWrite: false
        });
        _materialPools.coreFlash.push(mat);
    }

    // Pre-create debris materials with varied colors
    for (let i = 0; i < DEBRIS_POOL_SIZE; i++) {
        const mat = new THREE.MeshBasicMaterial({
            color: new THREE.Color().setHSL(0.08, 0.8, 0.2 + (i / DEBRIS_POOL_SIZE) * 0.3)
        });
        _materialPools.debris.push(mat);
    }

    return _materialPools;
}

/**
 * Get a fireball material from the pool (round-robin)
 */
function getFireballMaterial() {
    const pools = createMaterialPools();
    const mat = pools.fireball[pools.fireballIndex];
    pools.fireballIndex = (pools.fireballIndex + 1) % pools.fireball.length;
    return mat;
}

/**
 * Get a smoke material from the pool (round-robin)
 */
function getSmokeMaterial() {
    const pools = createMaterialPools();
    const mat = pools.smoke[pools.smokeIndex];
    pools.smokeIndex = (pools.smokeIndex + 1) % pools.smoke.length;
    return mat;
}

/**
 * Get shockwave materials from the pool (round-robin)
 */
function getShockwaveMaterials() {
    const pools = createMaterialPools();
    const matH = pools.shockwaveH[pools.shockwaveHIndex];
    const matV = pools.shockwaveV[pools.shockwaveVIndex];
    pools.shockwaveHIndex = (pools.shockwaveHIndex + 1) % pools.shockwaveH.length;
    pools.shockwaveVIndex = (pools.shockwaveVIndex + 1) % pools.shockwaveV.length;
    return { horizontal: matH, vertical: matV };
}

/**
 * Get a core flash material from the pool (round-robin)
 */
function getCoreFlashMaterial() {
    const pools = createMaterialPools();
    const mat = pools.coreFlash[pools.coreFlashIndex];
    pools.coreFlashIndex = (pools.coreFlashIndex + 1) % pools.coreFlash.length;
    return mat;
}

/**
 * Get a debris material from the pool (round-robin)
 */
function getDebrisMaterial() {
    const pools = createMaterialPools();
    const mat = pools.debris[pools.debrisIndex];
    pools.debrisIndex = (pools.debrisIndex + 1) % pools.debris.length;
    return mat;
}

// =============================================================================
// OBJECT POOLS - Pre-created Three.js objects to avoid shader recompilation
// =============================================================================

/**
 * Create object pools with pre-compiled geometry+material combinations
 * These are actual Three.js objects that are hidden/shown and repositioned
 */
function createObjectPools() {
    if (_objectPools) return _objectPools;

    const pools = createMaterialPools();
    const glowTexture = getGlowTexture();
    const smokeTexture = getSmokeTexture();

    // Pool sizes: support up to 3 concurrent explosions
    // Each explosion uses: 40 fireballs, 15 smoke, 1 coreFlash, 2 shockwaves, 20 debris, 1 sparks, 1 light
    const MAX_CONCURRENT = 3;

    _objectPools = {
        // Sprites for fireballs (40 per explosion x 3 = 120)
        fireballs: [],
        fireballIndex: 0,

        // Sprites for smoke (15 per explosion x 3 = 45)
        smoke: [],
        smokeIndex: 0,

        // Sprites for core flash (1 per explosion x 3 = 3)
        coreFlash: [],
        coreFlashIndex: 0,

        // Meshes for shockwaves (1H + 1V per explosion x 3 = 6 each)
        shockwaveH: [],
        shockwaveHIndex: 0,
        shockwaveV: [],
        shockwaveVIndex: 0,

        // Meshes for debris (20 per explosion x 3 = 60)
        debris: [],
        debrisIndex: 0,

        // Points for sparks (1 system per explosion x 3 = 3)
        sparks: [],
        sparksIndex: 0,

        // Lights (1 per explosion x 3 = 3)
        lights: [],
        lightIndex: 0,

        // Shared geometries (created once, in UU)
        geometries: {
            ringH: new THREE.RingGeometry(10, 50, 64),
            ringV: new THREE.RingGeometry(10, 40, 64),
            tetra: new THREE.TetrahedronGeometry(8),
            box: new THREE.BoxGeometry(10, 10, 10),
            octa: new THREE.OctahedronGeometry(6)
        }
    };

    // Pre-create 120 fireball sprites (40 per explosion x 3)
    const fireballCount = 40 * MAX_CONCURRENT;
    for (let i = 0; i < fireballCount; i++) {
        // Cycle through color variations within each set of 40
        const colorIndex = i % 40;
        const mat = new THREE.SpriteMaterial({
            map: glowTexture,
            color: new THREE.Color().setHSL(0.08 - (colorIndex / 40) * 0.08, 1, 0.5 + (colorIndex / 40) * 0.3),
            transparent: true,
            blending: THREE.AdditiveBlending,
            depthWrite: false
        });
        const sprite = new THREE.Sprite(mat);
        sprite.visible = false;
        _objectPools.fireballs.push(sprite);
    }

    // Pre-create 45 smoke sprites (15 per explosion x 3)
    for (let i = 0; i < 15 * MAX_CONCURRENT; i++) {
        const mat = new THREE.SpriteMaterial({
            map: smokeTexture,
            color: 0x222222,
            transparent: true,
            opacity: 0,
            depthWrite: false
        });
        const sprite = new THREE.Sprite(mat);
        sprite.visible = false;
        _objectPools.smoke.push(sprite);
    }

    // Pre-create 3 core flash sprites (1 per explosion x 3)
    for (let i = 0; i < MAX_CONCURRENT; i++) {
        const mat = new THREE.SpriteMaterial({
            map: glowTexture,
            color: 0xffffaa,
            transparent: true,
            blending: THREE.AdditiveBlending,
            depthWrite: false
        });
        const sprite = new THREE.Sprite(mat);
        sprite.visible = false;
        _objectPools.coreFlash.push(sprite);
    }

    // Pre-create 3 horizontal shockwave meshes (1 per explosion x 3)
    for (let i = 0; i < MAX_CONCURRENT; i++) {
        const mat = new THREE.MeshBasicMaterial({
            color: 0xffaa44,
            transparent: true,
            opacity: 0.8,
            side: THREE.DoubleSide,
            blending: THREE.AdditiveBlending,
            depthWrite: false
        });
        const mesh = new THREE.Mesh(_objectPools.geometries.ringH, mat);
        mesh.visible = false;
        _objectPools.shockwaveH.push(mesh);
    }

    // Pre-create 3 vertical shockwave meshes (1 per explosion x 3)
    for (let i = 0; i < MAX_CONCURRENT; i++) {
        const mat = new THREE.MeshBasicMaterial({
            color: 0xff6622,
            transparent: true,
            opacity: 0.6,
            side: THREE.DoubleSide,
            blending: THREE.AdditiveBlending,
            depthWrite: false
        });
        const mesh = new THREE.Mesh(_objectPools.geometries.ringV, mat);
        mesh.visible = false;
        _objectPools.shockwaveV.push(mesh);
    }

    // Pre-create 60 debris meshes (20 per explosion x 3)
    const debrisGeos = [_objectPools.geometries.tetra, _objectPools.geometries.box, _objectPools.geometries.octa];
    for (let i = 0; i < 20 * MAX_CONCURRENT; i++) {
        const mat = new THREE.MeshBasicMaterial({
            color: new THREE.Color().setHSL(0.08, 0.8, 0.2 + (i / 60) * 0.3)
        });
        const mesh = new THREE.Mesh(debrisGeos[i % 3], mat);
        mesh.visible = false;
        _objectPools.debris.push(mesh);
    }

    // Pre-create 3 spark systems (1 per explosion x 3)
    for (let i = 0; i < MAX_CONCURRENT; i++) {
        const count = 80;
        const geometry = new THREE.BufferGeometry();
        const positions = new Float32Array(count * 3);
        const colors = new Float32Array(count * 3);

        // Initialize with dummy data
        for (let j = 0; j < count; j++) {
            positions[j * 3] = 0;
            positions[j * 3 + 1] = 0;
            positions[j * 3 + 2] = 0;
            colors[j * 3] = 1;
            colors[j * 3 + 1] = 0.5;
            colors[j * 3 + 2] = 0;
        }

        geometry.setAttribute('position', new THREE.BufferAttribute(positions, 3));
        geometry.setAttribute('color', new THREE.BufferAttribute(colors, 3));

        const mat = new THREE.PointsMaterial({
            size: 15, // Size in UU
            map: getParticleTexture(),
            vertexColors: true,
            transparent: true,
            opacity: 1.0,
            blending: THREE.AdditiveBlending,
            depthWrite: false,
            sizeAttenuation: true
        });

        const points = new THREE.Points(geometry, mat);
        points.visible = false;
        _objectPools.sparks.push(points);
    }

    // Pre-create 3 point lights (1 per explosion x 3)
    for (let i = 0; i < MAX_CONCURRENT; i++) {
        const light = new THREE.PointLight(0xff6600, 0, 1500); // Distance in UU
        light.visible = false;
        _objectPools.lights.push(light);
    }

    return _objectPools;
}

/**
 * Get objects from pool for an explosion
 */
function getExplosionObjects() {
    const pools = createObjectPools();

    const result = {
        fireballs: [],
        smoke: [],
        coreFlash: null,
        shockwaveH: null,
        shockwaveV: null,
        debris: [],
        sparks: null,
        light: null
    };

    // Get 40 fireballs
    for (let i = 0; i < 40; i++) {
        result.fireballs.push(pools.fireballs[pools.fireballIndex]);
        pools.fireballIndex = (pools.fireballIndex + 1) % pools.fireballs.length;
    }

    // Get 15 smoke sprites
    for (let i = 0; i < 15; i++) {
        result.smoke.push(pools.smoke[pools.smokeIndex]);
        pools.smokeIndex = (pools.smokeIndex + 1) % pools.smoke.length;
    }

    // Get 1 core flash
    result.coreFlash = pools.coreFlash[pools.coreFlashIndex];
    pools.coreFlashIndex = (pools.coreFlashIndex + 1) % pools.coreFlash.length;

    // Get shockwaves
    result.shockwaveH = pools.shockwaveH[pools.shockwaveHIndex];
    pools.shockwaveHIndex = (pools.shockwaveHIndex + 1) % pools.shockwaveH.length;
    result.shockwaveV = pools.shockwaveV[pools.shockwaveVIndex];
    pools.shockwaveVIndex = (pools.shockwaveVIndex + 1) % pools.shockwaveV.length;

    // Get 20 debris
    for (let i = 0; i < 20; i++) {
        result.debris.push(pools.debris[pools.debrisIndex]);
        pools.debrisIndex = (pools.debrisIndex + 1) % pools.debris.length;
    }

    // Get sparks system
    result.sparks = pools.sparks[pools.sparksIndex];
    pools.sparksIndex = (pools.sparksIndex + 1) % pools.sparks.length;

    // Get light
    result.light = pools.lights[pools.lightIndex];
    pools.lightIndex = (pools.lightIndex + 1) % pools.lights.length;

    return result;
}

// ===========================================
// SIMPLIFIED EXPLOSION POOL - Performance optimized
// Uses MeshBasicMaterial only (no shader compilation issues)
// ===========================================

let _simplifiedExplosionPool = null;

class SimplifiedExplosionPool {
    constructor(scene, renderer, camera, maxExplosions = 2) {
        this.scene = scene;
        this.renderer = renderer;
        this.camera = camera;
        this.maxExplosions = maxExplosions;
        this.explosions = [];
        this.warmedUp = false;
        this.initPool();
    }

    initPool() {
        // Shared geometries (created once)
        this.sphereGeo = new THREE.SphereGeometry(1, 16, 12);
        this.coreGeo = new THREE.SphereGeometry(1, 12, 8); // Inner bright core
        this.ringGeo = new THREE.RingGeometry(0.5, 1, 32);
        this.particleGeo = new THREE.PlaneGeometry(1, 1);

        // Shared materials with AdditiveBlending for glow effect (no dynamic lights!)
        // IMPORTANT: DoubleSide is required so we can see the explosion from inside when it expands

        // Bright white/yellow core (hottest part)
        this.coreMaterial = new THREE.MeshBasicMaterial({
            color: 0xffffaa,
            transparent: true,
            opacity: 0.9,
            blending: THREE.AdditiveBlending,
            side: THREE.DoubleSide,
            depthWrite: false
        });

        // Orange outer sphere
        this.sphereMaterial = new THREE.MeshBasicMaterial({
            color: 0xff6600,
            transparent: true,
            opacity: 0.5,
            blending: THREE.AdditiveBlending,
            side: THREE.DoubleSide,
            depthWrite: false
        });

        // Shockwave ring - brighter orange
        this.ringMaterial = new THREE.MeshBasicMaterial({
            color: 0xff8800,
            transparent: true,
            opacity: 0.7,
            blending: THREE.AdditiveBlending,
            side: THREE.DoubleSide,
            depthWrite: false
        });

        // Debris particles - yellow/orange
        this.particleMaterial = new THREE.MeshBasicMaterial({
            color: 0xffcc00,
            transparent: true,
            opacity: 0.8,
            blending: THREE.AdditiveBlending,
            side: THREE.DoubleSide,
            depthWrite: false
        });

        // Pre-create explosion instances
        for (let i = 0; i < this.maxExplosions; i++) {
            const explosion = this.createExplosion();
            this.explosions.push(explosion);
        }
    }

    // Pre-compile shaders by doing an ACTUAL render (not just compile)
    // renderer.compile() only does CPU-side prep, GPU compilation happens on first render
    warmup() {
        if (this.warmedUp || !this.renderer || !this.camera) return;

        console.log('[SimplifiedExplosionPool] Starting GPU shader warmup...');

        // Save current renderer state
        const currentClearColor = this.renderer.getClearColor(new THREE.Color());
        const currentClearAlpha = this.renderer.getClearAlpha();
        const currentAutoClear = this.renderer.autoClear;

        // Position explosions in view but nearly invisible
        for (const explosion of this.explosions) {
            // Position at camera location (guaranteed to be in frustum)
            explosion.container.position.copy(this.camera.position);
            explosion.container.visible = true;
            // Set tiny scale so they're effectively invisible
            explosion.core.scale.set(0.001, 0.001, 0.001);
            explosion.sphere.scale.set(0.001, 0.001, 0.001);
            explosion.ring.scale.set(0.001, 0.001, 0.001);
            for (const p of explosion.particles) {
                p.mesh.scale.set(0.001, 0.001, 0.001);
            }
        }

        // Do an ACTUAL render to force GPU shader compilation
        // This is the only way to truly compile shaders - compile() is not enough
        this.renderer.autoClear = false;
        this.renderer.render(this.scene, this.camera);

        // Immediately hide and restore
        for (const explosion of this.explosions) {
            explosion.container.visible = false;
            explosion.core.scale.set(0.1, 0.1, 0.1);
            explosion.sphere.scale.set(0.1, 0.1, 0.1);
            explosion.ring.scale.set(0.1, 0.1, 0.1);
            for (const p of explosion.particles) {
                p.mesh.scale.set(12, 12, 12);
            }
        }

        // Restore renderer state
        this.renderer.autoClear = currentAutoClear;
        this.renderer.setClearColor(currentClearColor, currentClearAlpha);

        this.warmedUp = true;
        console.log('[SimplifiedExplosionPool] GPU shader warmup complete');
    }

    createExplosion() {
        const container = new THREE.Group();
        container.visible = false;
        // Render explosions LAST so they appear on top (after all opaque objects)
        container.renderOrder = 999;
        this.scene.add(container);

        // Clone materials per explosion (not per mesh) for independent opacity animation
        const coreMat = this.coreMaterial.clone();
        const sphereMat = this.sphereMaterial.clone();
        const ringMat = this.ringMaterial.clone();
        const particleMat = this.particleMaterial.clone();

        // Bright inner core (white/yellow hot center)
        const core = new THREE.Mesh(this.coreGeo, coreMat);
        core.scale.set(0.1, 0.1, 0.1);
        core.renderOrder = 999;
        container.add(core);

        // Outer orange fireball sphere
        const sphere = new THREE.Mesh(this.sphereGeo, sphereMat);
        sphere.scale.set(0.1, 0.1, 0.1);
        sphere.renderOrder = 999;
        container.add(sphere);

        // Shockwave ring (horizontal expanding)
        const ring = new THREE.Mesh(this.ringGeo, ringMat);
        ring.rotation.x = -Math.PI / 2;
        ring.scale.set(0.1, 0.1, 0.1);
        ring.renderOrder = 999;
        container.add(ring);

        // Flying debris particles (12 for more density)
        const particles = [];
        for (let j = 0; j < 12; j++) {
            const particle = new THREE.Mesh(this.particleGeo, particleMat);
            particle.scale.set(12, 12, 12);
            particle.renderOrder = 999;
            container.add(particle);

            const angle = (j / 12) * Math.PI * 2;
            const elevation = (Math.random() - 0.3) * Math.PI; // Mostly upward
            const speed = 350 + Math.random() * 250;
            particles.push({
                mesh: particle,
                velocity: new THREE.Vector3(
                    Math.cos(angle) * Math.cos(elevation) * speed,
                    Math.sin(elevation) * speed + 100, // Upward bias
                    Math.sin(angle) * Math.cos(elevation) * speed
                )
            });
        }

        // NO PointLight - causes 500ms+ freeze due to shader recompilation

        return {
            container,
            core,
            coreMat,
            sphere,
            ring,
            particles,
            particleMat,
            active: false,
            elapsed: 0,
            duration: 0.4,
            position: new THREE.Vector3()
        };
    }

    trigger(position) {
        // Find available explosion
        let explosion = this.explosions.find(e => !e.active);
        if (!explosion) {
            // Recycle oldest
            explosion = this.explosions[0];
            this.resetExplosion(explosion);
        }

        explosion.active = true;
        explosion.elapsed = 0;
        explosion.position.copy(position);
        explosion.container.position.copy(position);
        explosion.container.visible = true;

        // Reset core (bright center)
        explosion.core.scale.set(0.1, 0.1, 0.1);
        explosion.coreMat.opacity = 1.0;

        // Reset sphere (outer fireball)
        explosion.sphere.scale.set(0.1, 0.1, 0.1);
        explosion.sphere.material.opacity = 0.6;

        // Reset ring (shockwave)
        explosion.ring.scale.set(0.1, 0.1, 0.1);
        explosion.ring.material.opacity = 0.8;

        // Reset particles
        explosion.particleMat.opacity = 0.9;
        explosion.particles.forEach((p, i) => {
            p.mesh.position.set(0, 0, 0);
            const angle = (i / 12) * Math.PI * 2;
            const elevation = (Math.random() - 0.3) * Math.PI;
            const speed = 350 + Math.random() * 250;
            p.velocity.set(
                Math.cos(angle) * Math.cos(elevation) * speed,
                Math.sin(elevation) * speed + 100,
                Math.sin(angle) * Math.cos(elevation) * speed
            );
        });
    }

    resetExplosion(explosion) {
        explosion.active = false;
        explosion.container.visible = false;
    }

    update(deltaTime) {
        for (const explosion of this.explosions) {
            if (!explosion.active) continue;

            explosion.elapsed += deltaTime;
            const progress = explosion.elapsed / explosion.duration;

            if (progress >= 1) {
                this.resetExplosion(explosion);
                continue;
            }

            // Core: expands fast then fades quickly (brightest at start)
            const coreScale = 30 + progress * 80;
            explosion.core.scale.set(coreScale, coreScale, coreScale);
            explosion.coreMat.opacity = 1.0 * Math.pow(1 - progress, 2); // Fast fade

            // Outer sphere: larger, fades slower
            const sphereScale = 50 + progress * 200;
            explosion.sphere.scale.set(sphereScale, sphereScale, sphereScale);
            explosion.sphere.material.opacity = 0.6 * (1 - progress);

            // Ring: expands outward as shockwave
            const ringScale = 80 + progress * 350;
            explosion.ring.scale.set(ringScale, ringScale, ringScale);
            explosion.ring.material.opacity = 0.8 * (1 - progress * progress);

            // Particles: fly outward with gravity
            explosion.particleMat.opacity = 0.9 * (1 - progress);
            for (const p of explosion.particles) {
                p.mesh.position.add(p.velocity.clone().multiplyScalar(deltaTime));
                p.velocity.y -= 300 * deltaTime; // Gravity
                // Billboard effect: face camera
                if (this.camera) {
                    p.mesh.lookAt(this.camera.position);
                }
            }
        }
    }

    dispose() {
        for (const explosion of this.explosions) {
            this.scene.remove(explosion.container);
            explosion.coreMat.dispose();
            explosion.sphere.material.dispose();
            explosion.ring.material.dispose();
            explosion.particleMat.dispose();
        }
        // Dispose shared geometries and base materials
        this.coreGeo.dispose();
        this.sphereGeo.dispose();
        this.ringGeo.dispose();
        this.particleGeo.dispose();
        this.coreMaterial.dispose();
        this.sphereMaterial.dispose();
        this.ringMaterial.dispose();
        this.particleMaterial.dispose();
    }
}

// Helper to get/create simplified explosion pool
function getSimplifiedExplosionPool(scene, renderer = null, camera = null) {
    if (!_simplifiedExplosionPool) {
        _simplifiedExplosionPool = new SimplifiedExplosionPool(scene, renderer, camera);
    }
    // Warmup if renderer/camera are provided and not already warmed up
    if (renderer && camera && !_simplifiedExplosionPool.warmedUp) {
        _simplifiedExplosionPool.renderer = renderer;
        _simplifiedExplosionPool.camera = camera;
        _simplifiedExplosionPool.warmup();
    }
    return _simplifiedExplosionPool;
}

// Call this from GameEngine to pre-warm explosion shaders
export function warmupExplosionPool(scene, renderer, camera) {
    getSimplifiedExplosionPool(scene, renderer, camera);
    getGoalExplosionPool(scene, renderer, camera);
}

// Reset explosion pools - call this when navigating away or before re-initialization
// This fixes the issue where pools retain stale scene/renderer references after React Router navigation
export function resetExplosionPools() {
    if (_simplifiedExplosionPool) {
        _simplifiedExplosionPool.dispose?.();
        _simplifiedExplosionPool = null;
    }
    if (_goalExplosionPool) {
        _goalExplosionPool.dispose?.();
        _goalExplosionPool = null;
    }
}

// ===========================================
// GOAL EXPLOSION POOL - Team-colored spectacular goal celebrations
// Uses same optimizations as SimplifiedExplosionPool (no PointLight, AdditiveBlending)
// ===========================================

let _goalExplosionPool = null;

// Team colors
const TEAM_COLORS = {
    // Team 0 = Blue
    0: {
        core: 0x66ccff,    // Bright cyan-blue
        sphere: 0x0088ff,  // Blue
        ring: 0x00aaff,    // Light blue
        particles: 0x88ddff // Pale blue
    },
    // Team 1 = Orange
    1: {
        core: 0xffdd66,    // Bright yellow-orange
        sphere: 0xff6600,  // Orange
        ring: 0xff8800,    // Light orange
        particles: 0xffaa44 // Pale orange
    }
};

class GoalExplosionPool {
    constructor(scene, renderer, camera, maxExplosions = 2) {
        this.scene = scene;
        this.renderer = renderer;
        this.camera = camera;
        this.maxExplosions = maxExplosions;
        this.explosions = [];
        this.warmedUp = false;
        this.initPool();
    }

    initPool() {
        // Shared geometries
        this.coreGeo = new THREE.SphereGeometry(1, 16, 12);
        this.sphereGeo = new THREE.SphereGeometry(1, 20, 14);
        this.ringGeo = new THREE.RingGeometry(0.3, 1, 48);
        this.particleGeo = new THREE.PlaneGeometry(1, 1);
        this.rayGeo = new THREE.PlaneGeometry(1, 1); // For light rays

        // Pre-create explosion instances for both teams
        for (let i = 0; i < this.maxExplosions; i++) {
            const explosion = this.createExplosion();
            this.explosions.push(explosion);
        }
    }

    createMaterialsForTeam(team) {
        const colors = TEAM_COLORS[team] || TEAM_COLORS[0];

        return {
            core: new THREE.MeshBasicMaterial({
                color: colors.core,
                transparent: true,
                opacity: 1.0,
                blending: THREE.AdditiveBlending,
                side: THREE.DoubleSide,
                depthWrite: false
            }),
            sphere: new THREE.MeshBasicMaterial({
                color: colors.sphere,
                transparent: true,
                opacity: 0.6,
                blending: THREE.AdditiveBlending,
                side: THREE.DoubleSide,
                depthWrite: false
            }),
            ring: new THREE.MeshBasicMaterial({
                color: colors.ring,
                transparent: true,
                opacity: 0.8,
                blending: THREE.AdditiveBlending,
                side: THREE.DoubleSide,
                depthWrite: false
            }),
            particles: new THREE.MeshBasicMaterial({
                color: colors.particles,
                transparent: true,
                opacity: 0.9,
                blending: THREE.AdditiveBlending,
                side: THREE.DoubleSide,
                depthWrite: false
            }),
            rays: new THREE.MeshBasicMaterial({
                color: colors.core,
                transparent: true,
                opacity: 0.7,
                blending: THREE.AdditiveBlending,
                side: THREE.DoubleSide,
                depthWrite: false
            })
        };
    }

    createExplosion() {
        const container = new THREE.Group();
        container.visible = false;
        container.renderOrder = 999;
        this.scene.add(container);

        // Create materials for both teams (we'll swap colors on trigger)
        const materials = {
            0: this.createMaterialsForTeam(0),
            1: this.createMaterialsForTeam(1)
        };

        // Bright inner core (will pulse)
        const core = new THREE.Mesh(this.coreGeo, materials[0].core);
        core.scale.set(0.1, 0.1, 0.1);
        core.renderOrder = 999;
        container.add(core);

        // Second core layer for extra glow
        const core2 = new THREE.Mesh(this.coreGeo, materials[0].core.clone());
        core2.scale.set(0.1, 0.1, 0.1);
        core2.renderOrder = 999;
        container.add(core2);

        // Third core - largest glow
        const core3 = new THREE.Mesh(this.coreGeo, materials[0].sphere.clone());
        core3.scale.set(0.1, 0.1, 0.1);
        core3.renderOrder = 999;
        container.add(core3);

        // Outer sphere
        const sphere = new THREE.Mesh(this.sphereGeo, materials[0].sphere);
        sphere.scale.set(0.1, 0.1, 0.1);
        sphere.renderOrder = 999;
        container.add(sphere);

        // Rings disabled for cleaner look
        const rings = [];

        // Light rays (12 rays emanating outward, rotating)
        const rays = [];
        for (let i = 0; i < 12; i++) {
            const ray = new THREE.Mesh(this.rayGeo, materials[0].rays.clone());
            ray.scale.set(20, 300, 1);
            ray.renderOrder = 999;
            const angle = (i / 12) * Math.PI * 2;
            ray.rotation.z = angle;
            ray.position.set(0, 0, 0);
            container.add(ray);
            rays.push({ mesh: ray, baseAngle: angle });
        }

        // PARTICLE JETS - 12 jets with 8 particles each = 96 particles
        // Each jet is a stream of particles going in the same direction
        const particles = [];
        const JET_COUNT = 12;
        const PARTICLES_PER_JET = 8;

        for (let jet = 0; jet < JET_COUNT; jet++) {
            // Each jet has a base direction
            const jetAngle = (jet / JET_COUNT) * Math.PI * 2;
            // Alternate between upward and horizontal jets
            const jetElevation = jet % 3 === 0 ? 0.6 : jet % 3 === 1 ? 0.2 : -0.1;

            for (let p = 0; p < PARTICLES_PER_JET; p++) {
                const particle = new THREE.Mesh(this.particleGeo, materials[0].particles.clone());
                particle.scale.set(40, 40, 40);
                particle.renderOrder = 999;
                container.add(particle);

                // Particles in same jet have similar direction but varying speeds
                // Creates a "streaming" effect
                const angleSpread = jetAngle + (Math.random() - 0.5) * 0.3;
                const elevSpread = jetElevation + (Math.random() - 0.5) * 0.2;

                // Leading particles are faster, trailing are slower
                const speedMultiplier = 1.0 - (p / PARTICLES_PER_JET) * 0.5;
                const baseSpeed = 1800 * speedMultiplier;
                const speed = baseSpeed + Math.random() * 300;

                // Staggered delay creates trailing effect
                const delay = p * 0.02;

                // Size varies - leading particles bigger
                const sizeMultiplier = 1.0 - (p / PARTICLES_PER_JET) * 0.4;
                const particleSize = (35 + Math.random() * 25) * sizeMultiplier;

                particles.push({
                    mesh: particle,
                    velocity: new THREE.Vector3(
                        Math.cos(angleSpread) * Math.cos(elevSpread) * speed,
                        Math.sin(elevSpread) * speed + 300,
                        Math.sin(angleSpread) * Math.cos(elevSpread) * speed
                    ),
                    initialScale: particleSize,
                    delay: delay,
                    jetIndex: jet
                });
            }
        }

        // Add LOTS of small scatter particles that spray everywhere
        const SCATTER_COUNT = 100;
        for (let s = 0; s < SCATTER_COUNT; s++) {
            const particle = new THREE.Mesh(this.particleGeo, materials[0].particles.clone());
            // Smaller particles for spray effect
            const baseSize = 8 + Math.random() * 18;
            particle.scale.set(baseSize, baseSize, baseSize);
            particle.renderOrder = 999;
            container.add(particle);

            // Random directions - true 360° spray
            const angle = Math.random() * Math.PI * 2;
            const elevation = (Math.random() - 0.5) * Math.PI; // Full sphere coverage

            // Mix of fast and slow particles
            const speedType = Math.random();
            let speed;
            if (speedType < 0.3) {
                speed = 1500 + Math.random() * 800; // Fast spray
            } else if (speedType < 0.6) {
                speed = 800 + Math.random() * 600; // Medium
            } else {
                speed = 300 + Math.random() * 500; // Slow floaters
            }

            particles.push({
                mesh: particle,
                velocity: new THREE.Vector3(
                    Math.cos(angle) * Math.cos(elevation) * speed,
                    Math.sin(elevation) * speed + 150,
                    Math.sin(angle) * Math.cos(elevation) * speed
                ),
                initialScale: baseSize,
                delay: Math.random() * 0.2, // Staggered launch
                jetIndex: -1 // scatter particle
            });
        }

        return {
            container,
            core,
            core2,
            core3,
            sphere,
            rings,
            rays,
            particles,
            materials,
            currentTeam: 0,
            active: false,
            elapsed: 0,
            duration: 1.8, // Longer for dramatic jets
            position: new THREE.Vector3(),
            rotationOffset: 0
        };
    }

    warmup() {
        if (this.warmedUp || !this.renderer || !this.camera) return;

        console.log('[GoalExplosionPool] Starting GPU shader warmup...');

        for (const explosion of this.explosions) {
            explosion.container.position.copy(this.camera.position);
            explosion.container.visible = true;
            explosion.core.scale.set(0.001, 0.001, 0.001);
            explosion.core2.scale.set(0.001, 0.001, 0.001);
            explosion.core3.scale.set(0.001, 0.001, 0.001);
            explosion.sphere.scale.set(0.001, 0.001, 0.001);
            for (const ring of explosion.rings) {
                ring.mesh.scale.set(0.001, 0.001, 0.001);
            }
            for (const ray of explosion.rays) {
                ray.mesh.scale.set(0.001, 0.001, 0.001);
            }
            for (const p of explosion.particles) {
                p.mesh.scale.set(0.001, 0.001, 0.001);
            }
        }

        this.renderer.render(this.scene, this.camera);

        for (const explosion of this.explosions) {
            explosion.container.visible = false;
            explosion.core.scale.set(0.1, 0.1, 0.1);
            explosion.core2.scale.set(0.1, 0.1, 0.1);
            explosion.core3.scale.set(0.1, 0.1, 0.1);
            explosion.sphere.scale.set(0.1, 0.1, 0.1);
            for (const ring of explosion.rings) {
                ring.mesh.scale.set(0.1, 0.1, 0.1);
            }
            for (const ray of explosion.rays) {
                ray.mesh.scale.set(20, 300, 1);
            }
            for (const p of explosion.particles) {
                p.mesh.scale.set(30, 30, 30);
            }
        }

        this.warmedUp = true;
        console.log('[GoalExplosionPool] GPU shader warmup complete');
    }

    // Easing functions for non-linear animation
    easeOutElastic(x) {
        const c4 = (2 * Math.PI) / 3;
        return x === 0 ? 0 : x === 1 ? 1 : Math.pow(2, -10 * x) * Math.sin((x * 10 - 0.75) * c4) + 1;
    }

    easeOutExpo(x) {
        return x === 1 ? 1 : 1 - Math.pow(2, -10 * x);
    }

    easeOutBack(x) {
        const c1 = 1.70158;
        const c3 = c1 + 1;
        return 1 + c3 * Math.pow(x - 1, 3) + c1 * Math.pow(x - 1, 2);
    }

    trigger(position, team = 0) {
        let explosion = this.explosions.find(e => !e.active);
        if (!explosion) {
            explosion = this.explosions[0];
            this.resetExplosion(explosion);
        }

        // Swap materials to team color
        const mats = explosion.materials[team] || explosion.materials[0];
        explosion.core.material = mats.core;
        explosion.core2.material = mats.core.clone();
        explosion.core3.material = mats.sphere.clone();
        explosion.sphere.material = mats.sphere;
        for (const ring of explosion.rings) {
            ring.mesh.material = mats.ring.clone();
        }
        for (const ray of explosion.rays) {
            ray.mesh.material = mats.rays.clone();
        }
        for (const p of explosion.particles) {
            p.mesh.material = mats.particles.clone();
        }

        explosion.active = true;
        explosion.elapsed = 0;
        explosion.currentTeam = team;
        explosion.position.copy(position);
        explosion.container.position.copy(position);
        explosion.container.visible = true;
        explosion.rotationOffset = 0;

        // Reset scales and opacities
        explosion.core.scale.set(0.1, 0.1, 0.1);
        explosion.core.material.opacity = 1.0;
        explosion.core2.scale.set(0.1, 0.1, 0.1);
        explosion.core2.material.opacity = 0.8;
        explosion.core3.scale.set(0.1, 0.1, 0.1);
        explosion.core3.material.opacity = 0.5;
        explosion.sphere.scale.set(0.1, 0.1, 0.1);
        explosion.sphere.material.opacity = 0.4;

        // Reset rings
        for (const ring of explosion.rings) {
            ring.mesh.scale.set(0.1, 0.1, 0.1);
            ring.mesh.material.opacity = 0.9;
        }

        // Reset rays
        for (let i = 0; i < explosion.rays.length; i++) {
            const ray = explosion.rays[i];
            ray.mesh.material.opacity = 0.8;
            ray.mesh.scale.set(40, 0.1, 1);
        }

        // Reset particles - Jets + Scatter
        const JET_COUNT = 12;
        const PARTICLES_PER_JET = 8;
        let particleIndex = 0;

        // Reset jet particles
        for (let jet = 0; jet < JET_COUNT; jet++) {
            const jetAngle = (jet / JET_COUNT) * Math.PI * 2;
            const jetElevation = jet % 3 === 0 ? 0.6 : jet % 3 === 1 ? 0.2 : -0.1;

            for (let p = 0; p < PARTICLES_PER_JET; p++) {
                if (particleIndex >= explosion.particles.length) break;
                const particle = explosion.particles[particleIndex];

                particle.mesh.position.set(0, 0, 0);
                particle.mesh.material.opacity = 1.0;
                const scale = particle.initialScale;
                particle.mesh.scale.set(scale, scale, scale);

                const angleSpread = jetAngle + (Math.random() - 0.5) * 0.3;
                const elevSpread = jetElevation + (Math.random() - 0.5) * 0.2;
                const speedMultiplier = 1.0 - (p / PARTICLES_PER_JET) * 0.5;
                const baseSpeed = 1800 * speedMultiplier;
                const speed = baseSpeed + Math.random() * 300;

                particle.velocity.set(
                    Math.cos(angleSpread) * Math.cos(elevSpread) * speed,
                    Math.sin(elevSpread) * speed + 300,
                    Math.sin(angleSpread) * Math.cos(elevSpread) * speed
                );
                particle.delay = p * 0.02;

                particleIndex++;
            }
        }

        // Reset scatter particles
        while (particleIndex < explosion.particles.length) {
            const particle = explosion.particles[particleIndex];
            particle.mesh.position.set(0, 0, 0);
            particle.mesh.material.opacity = 1.0;
            const scale = particle.initialScale;
            particle.mesh.scale.set(scale, scale, scale);

            const angle = Math.random() * Math.PI * 2;
            const elevation = (Math.random() - 0.4) * Math.PI;
            const speed = 600 + Math.random() * 800;

            particle.velocity.set(
                Math.cos(angle) * Math.cos(elevation) * speed,
                Math.sin(elevation) * speed + 200,
                Math.sin(angle) * Math.cos(elevation) * speed
            );
            particle.delay = Math.random() * 0.15;

            particleIndex++;
        }
    }

    resetExplosion(explosion) {
        explosion.active = false;
        explosion.container.visible = false;
    }

    update(deltaTime) {
        for (const explosion of this.explosions) {
            if (!explosion.active) continue;

            explosion.elapsed += deltaTime;
            const progress = explosion.elapsed / explosion.duration;

            if (progress >= 1) {
                this.resetExplosion(explosion);
                continue;
            }

            // Update rotation for dynamic feel
            explosion.rotationOffset += deltaTime * 2;

            // Non-linear progress for more punch
            const elasticProgress = this.easeOutElastic(Math.min(progress * 2, 1));
            const expoProgress = this.easeOutExpo(progress);
            const backProgress = this.easeOutBack(Math.min(progress * 1.5, 1));

            // Core 1: Bright center with elastic expansion and pulse
            const pulse = 1 + Math.sin(explosion.elapsed * 15) * 0.15 * (1 - progress);
            const coreScale = (150 + elasticProgress * 300) * pulse;
            explosion.core.scale.set(coreScale, coreScale, coreScale);
            explosion.core.material.opacity = 1.0 * Math.pow(1 - progress, 1.2);

            // Core 2: Slightly larger, different phase
            const pulse2 = 1 + Math.sin(explosion.elapsed * 12 + 1) * 0.12 * (1 - progress);
            const core2Scale = (200 + backProgress * 400) * pulse2;
            explosion.core2.scale.set(core2Scale, core2Scale, core2Scale);
            explosion.core2.material.opacity = 0.7 * Math.pow(1 - progress, 1.5);

            // Core 3: Largest glow layer
            const core3Scale = 300 + expoProgress * 600;
            explosion.core3.scale.set(core3Scale, core3Scale, core3Scale);
            explosion.core3.material.opacity = 0.4 * Math.pow(1 - progress, 2);

            // Outer sphere: expands fast then slow
            const sphereScale = 400 + expoProgress * 1200;
            explosion.sphere.scale.set(sphereScale, sphereScale, sphereScale);
            explosion.sphere.material.opacity = 0.3 * (1 - progress * progress);

            // Rings: each expands at different rates with rotation
            for (let i = 0; i < explosion.rings.length; i++) {
                const ring = explosion.rings[i];
                const ringProgress = this.easeOutExpo(Math.min(progress * (1.2 + i * 0.1), 1));
                const ringScale = (300 + i * 100) + ringProgress * (1500 + i * 200);
                ring.mesh.scale.set(ringScale, ringScale, ringScale);
                ring.mesh.material.opacity = 0.8 * Math.pow(1 - progress, 1.5);

                // Add rotation to rings for dynamic effect
                if (ring.axis === 'horizontal') {
                    ring.mesh.rotation.z = explosion.rotationOffset * 0.3;
                } else if (ring.axis === 'verticalX') {
                    ring.mesh.rotation.x += deltaTime * 1.5;
                } else if (ring.axis === 'verticalZ') {
                    ring.mesh.rotation.y += deltaTime * 1.2;
                } else {
                    ring.mesh.rotation.z += deltaTime * 0.8;
                }
            }

            // Light rays: extend outward with rotation and pulsing
            const rayBaseLength = 200 + expoProgress * 2000;
            const rayPulse = 1 + Math.sin(explosion.elapsed * 20) * 0.1 * (1 - progress);
            for (let i = 0; i < explosion.rays.length; i++) {
                const ray = explosion.rays[i];
                const rayLength = rayBaseLength * rayPulse * (0.8 + Math.sin(i * 0.5 + explosion.elapsed * 8) * 0.2);
                const rayWidth = 60 * (1 - progress * 0.4);
                ray.mesh.scale.set(rayWidth, rayLength, 1);
                ray.mesh.rotation.z = ray.baseAngle + explosion.rotationOffset * 0.5;
                ray.mesh.material.opacity = 0.8 * Math.pow(1 - progress, 1.3);
            }

            // Particles: fly outward with gravity, trails, and billboard
            for (const p of explosion.particles) {
                // Only start moving after delay
                const particleProgress = Math.max(0, explosion.elapsed - (p.delay || 0));
                if (particleProgress > 0) {
                    p.mesh.position.add(p.velocity.clone().multiplyScalar(deltaTime));
                    p.velocity.y -= 600 * deltaTime; // Stronger gravity
                    p.velocity.multiplyScalar(0.995); // Air resistance

                    // Scale down as they travel
                    const scaleFade = Math.max(0.3, 1 - progress * 0.7);
                    const currentScale = p.initialScale * scaleFade;
                    p.mesh.scale.set(currentScale, currentScale, currentScale);
                }

                p.mesh.material.opacity = 1.0 * Math.pow(1 - progress, 1.2);

                // Billboard towards camera
                if (this.camera) {
                    p.mesh.lookAt(this.camera.position);
                }
            }
        }
    }

    dispose() {
        for (const explosion of this.explosions) {
            this.scene.remove(explosion.container);
            // Dispose all materials for both teams
            for (const teamMats of Object.values(explosion.materials)) {
                teamMats.core.dispose();
                teamMats.sphere.dispose();
                teamMats.ring.dispose();
                teamMats.particles.dispose();
                teamMats.rays.dispose();
            }
        }
        this.coreGeo.dispose();
        this.sphereGeo.dispose();
        this.ringGeo.dispose();
        this.particleGeo.dispose();
        this.rayGeo.dispose();
    }
}

function getGoalExplosionPool(scene, renderer = null, camera = null) {
    if (!_goalExplosionPool) {
        _goalExplosionPool = new GoalExplosionPool(scene, renderer, camera);
    }
    if (renderer && camera && !_goalExplosionPool.warmedUp) {
        _goalExplosionPool.renderer = renderer;
        _goalExplosionPool.camera = camera;
        _goalExplosionPool.warmup();
    }
    return _goalExplosionPool;
}

// Export for use in EffectsManager
export function triggerGoalExplosionFromPool(scene, position, team) {
    const pool = getGoalExplosionPool(scene);
    if (pool) {
        pool.trigger(position, team);
    }
}

// ===========================================
// EXPLOSION POOL - Zero allocation at runtime (DEPRECATED)
// All objects created once at init
// NOTE: Use SimplifiedExplosionPool instead for better performance
// ===========================================

let _explosionPool = null;

class ExplosionPool {
    constructor(scene, maxExplosions = 3) {
        this.scene = scene;
        this.maxExplosions = maxExplosions;

        // Config per explosion
        this.fireballCount = 30;
        this.sparkCount = 60;
        this.debrisCount = 15;
        this.smokeCount = 10;

        // Textures
        this.glowTex = getGlowTexture();
        this.particleTex = getParticleTexture();
        this.smokeTex = getSmokeTexture();

        // Pools
        this.explosions = [];
        this.activeExplosions = [];

        this.initPools();
    }

    initPools() {
        for (let e = 0; e < this.maxExplosions; e++) {
            const explosion = {
                active: false,
                elapsed: 0,
                duration: 0.5,
                position: new THREE.Vector3(),

                // Core flash
                flash: this.createFlash(),

                // Fireballs
                fireballs: [],

                // Sparks (single Points with buffer)
                sparks: this.createSparksSystem(),
                sparkVelocities: [],

                // Debris (InstancedMesh)
                debris: this.createDebrisSystem(),
                debrisData: [],

                // Smoke
                smoke: [],

                // Light
                light: new THREE.PointLight(0xff6600, 0, 15),

                // Shockwave
                shockwave: this.createShockwave()
            };

            // Fireballs pool
            for (let i = 0; i < this.fireballCount; i++) {
                const mat = new THREE.SpriteMaterial({
                    map: this.glowTex,
                    transparent: true,
                    blending: THREE.AdditiveBlending,
                    depthWrite: false,
                    toneMapped: false // Bypass HDR tone mapping for bright explosions
                });
                const sprite = new THREE.Sprite(mat);
                sprite.visible = false;
                this.scene.add(sprite);
                explosion.fireballs.push({ sprite, velocity: new THREE.Vector3(), life: 0, startScale: 0 });
            }

            // Smoke pool
            for (let i = 0; i < this.smokeCount; i++) {
                const mat = new THREE.SpriteMaterial({
                    map: this.smokeTex,
                    color: 0x333333,
                    transparent: true,
                    depthWrite: false,
                    toneMapped: false // Bypass HDR tone mapping
                });
                const sprite = new THREE.Sprite(mat);
                sprite.visible = false;
                this.scene.add(sprite);
                explosion.smoke.push({ sprite, velocity: new THREE.Vector3(), delay: 0, life: 0, startScale: 0 });
            }

            // Spark velocities
            for (let i = 0; i < this.sparkCount; i++) {
                explosion.sparkVelocities.push(new THREE.Vector3());
            }

            // Debris data
            for (let i = 0; i < this.debrisCount; i++) {
                explosion.debrisData.push({
                    velocity: new THREE.Vector3(),
                    rotSpeed: new THREE.Vector3(),
                    position: new THREE.Vector3(),
                    rotation: new THREE.Euler(),
                    life: 0
                });
            }

            this.scene.add(explosion.flash);
            this.scene.add(explosion.sparks);
            this.scene.add(explosion.debris);
            this.scene.add(explosion.light);
            this.scene.add(explosion.shockwave);

            explosion.flash.visible = false;
            explosion.sparks.visible = false;
            explosion.debris.visible = false;
            explosion.shockwave.visible = false;

            this.explosions.push(explosion);
        }
    }

    createFlash() {
        const mat = new THREE.SpriteMaterial({
            map: this.glowTex,
            color: 0xffffaa,
            transparent: true,
            blending: THREE.AdditiveBlending,
            depthWrite: false,
            toneMapped: false // Bypass HDR tone mapping for bright explosions
        });
        return new THREE.Sprite(mat);
    }

    createSparksSystem() {
        const positions = new Float32Array(this.sparkCount * 3);
        const colors = new Float32Array(this.sparkCount * 3);

        const geo = new THREE.BufferGeometry();
        geo.setAttribute('position', new THREE.BufferAttribute(positions, 3));
        geo.setAttribute('color', new THREE.BufferAttribute(colors, 3));

        const mat = new THREE.PointsMaterial({
            size: 0.12,
            map: this.particleTex,
            vertexColors: true,
            transparent: true,
            blending: THREE.AdditiveBlending,
            depthWrite: false,
            sizeAttenuation: true,
            toneMapped: false // Bypass HDR tone mapping for bright sparks
        });

        return new THREE.Points(geo, mat);
    }

    createDebrisSystem() {
        const geo = new THREE.TetrahedronGeometry(0.08);
        const mat = new THREE.MeshBasicMaterial({
            color: 0x553311,
            toneMapped: false
        });
        const mesh = new THREE.InstancedMesh(geo, mat, this.debrisCount);
        mesh.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
        return mesh;
    }

    createShockwave() {
        const geo = new THREE.RingGeometry(0.1, 0.3, 32);
        const mat = new THREE.MeshBasicMaterial({
            color: 0xffaa44,
            transparent: true,
            side: THREE.DoubleSide,
            blending: THREE.AdditiveBlending,
            depthWrite: false,
            toneMapped: false // Bypass HDR tone mapping for bright shockwave
        });
        const mesh = new THREE.Mesh(geo, mat);
        mesh.rotation.x = -Math.PI / 2;
        return mesh;
    }

    trigger(position) {
        // Find inactive explosion
        let exp = this.explosions.find(e => !e.active);
        if (!exp) {
            // Recycle oldest
            exp = this.activeExplosions.shift();
            if (!exp) return;
            this.deactivate(exp);
        }

        this.resetExplosion(exp, position);
        exp.active = true;
        this.activeExplosions.push(exp);
    }

    resetExplosion(exp, pos) {
        exp.elapsed = 0;
        exp.position.copy(pos);

        // Flash
        exp.flash.position.copy(pos);
        exp.flash.scale.setScalar(10); // Initial scale in UU
        exp.flash.material.opacity = 1;
        exp.flash.visible = true;

        // Fireballs
        for (let i = 0; i < this.fireballCount; i++) {
            const fb = exp.fireballs[i];
            fb.sprite.position.copy(pos);

            const theta = Math.random() * Math.PI * 2;
            const phi = Math.acos(2 * Math.random() - 1);
            const speed = 300 + Math.random() * 600; // Speed in UU

            fb.velocity.set(
                Math.sin(phi) * Math.cos(theta) * speed,
                Math.sin(phi) * Math.sin(theta) * speed + 200,
                Math.cos(phi) * speed
            );
            fb.startScale = 40 + Math.random() * 120; // Scale in UU
            fb.life = 0.15 + Math.random() * 0.2;
            fb.sprite.material.color.setHSL(0.08 - Math.random() * 0.06, 1, 0.5 + Math.random() * 0.3);
            fb.sprite.visible = true;
        }

        // Sparks
        const sparkPos = exp.sparks.geometry.attributes.position.array;
        const sparkCol = exp.sparks.geometry.attributes.color.array;
        for (let i = 0; i < this.sparkCount; i++) {
            sparkPos[i * 3] = pos.x;
            sparkPos[i * 3 + 1] = pos.y;
            sparkPos[i * 3 + 2] = pos.z;

            const col = new THREE.Color().setHSL(0.1 + Math.random() * 0.05, 1, 0.7 + Math.random() * 0.3);
            sparkCol[i * 3] = col.r;
            sparkCol[i * 3 + 1] = col.g;
            sparkCol[i * 3 + 2] = col.b;

            const theta = Math.random() * Math.PI * 2;
            const phi = Math.acos(2 * Math.random() - 1);
            const speed = 800 + Math.random() * 1500; // Speed in UU
            exp.sparkVelocities[i].set(
                Math.sin(phi) * Math.cos(theta) * speed,
                Math.sin(phi) * Math.sin(theta) * speed + 400,
                Math.cos(phi) * speed
            );
        }
        exp.sparks.geometry.attributes.position.needsUpdate = true;
        exp.sparks.geometry.attributes.color.needsUpdate = true;
        exp.sparks.material.opacity = 1;
        exp.sparks.visible = true;

        // Debris
        const dummy = new THREE.Object3D();
        for (let i = 0; i < this.debrisCount; i++) {
            const d = exp.debrisData[i];
            d.position.copy(pos);
            d.rotation.set(Math.random() * 6, Math.random() * 6, Math.random() * 6);

            dummy.position.copy(d.position);
            dummy.rotation.copy(d.rotation);
            dummy.scale.setScalar(1);
            dummy.updateMatrix();
            exp.debris.setMatrixAt(i, dummy.matrix);

            const theta = Math.random() * Math.PI * 2;
            const phi = Math.acos(2 * Math.random() - 1);
            const speed = 400 + Math.random() * 1200; // Speed in UU
            d.velocity.set(
                Math.sin(phi) * Math.cos(theta) * speed,
                Math.sin(phi) * Math.sin(theta) * speed + 600,
                Math.cos(phi) * speed
            );
            d.rotSpeed.set((Math.random() - 0.5) * 15, (Math.random() - 0.5) * 15, (Math.random() - 0.5) * 15);
            d.life = 0.4 + Math.random() * 0.4;
        }
        exp.debris.instanceMatrix.needsUpdate = true;
        exp.debris.visible = true;

        // Smoke
        for (let i = 0; i < this.smokeCount; i++) {
            const s = exp.smoke[i];
            s.sprite.position.copy(pos);
            s.sprite.position.x += (Math.random() - 0.5) * 30; // Spread in UU
            s.sprite.position.y += Math.random() * 30;
            s.sprite.position.z += (Math.random() - 0.5) * 30;
            s.velocity.set((Math.random() - 0.5) * 150, 100 + Math.random() * 150, (Math.random() - 0.5) * 150); // Velocity in UU
            s.delay = i * 0.015;
            s.life = 0.6 + Math.random() * 0.3;
            s.startScale = 40 + Math.random() * 40; // Scale in UU
            s.sprite.material.opacity = 0;
            s.sprite.visible = true;
        }

        // Light
        exp.light.position.copy(pos);
        exp.light.intensity = 12;

        // Shockwave
        exp.shockwave.position.copy(pos);
        exp.shockwave.position.y = pos.y + 1; // Small offset in UU
        exp.shockwave.scale.setScalar(10); // Initial scale in UU
        exp.shockwave.material.opacity = 0.9;
        exp.shockwave.visible = true;
    }

    update(dt) {
        const dummy = new THREE.Object3D();

        for (const exp of this.activeExplosions) {
            if (!exp.active) continue;

            exp.elapsed += dt;
            const progress = exp.elapsed / exp.duration;

            // Flash
            const flashP = Math.min(exp.elapsed / 0.06, 1);
            exp.flash.scale.setScalar((1 - Math.pow(2, -10 * flashP)) * 800); // Scale in UU
            exp.flash.material.opacity = Math.max(0, 1 - flashP * 2.5);

            // Fireballs
            for (const fb of exp.fireballs) {
                const lp = exp.elapsed / fb.life;
                fb.sprite.position.addScaledVector(fb.velocity, dt);
                fb.velocity.multiplyScalar(0.94);
                fb.velocity.y -= 800 * dt; // Gravity in UU
                fb.sprite.scale.setScalar(fb.startScale * (1 + lp * 1.5) * Math.max(0, 1 - lp));
                fb.sprite.material.opacity = Math.max(0, 1 - lp * lp);
            }

            // Sparks
            const sparkPos = exp.sparks.geometry.attributes.position.array;
            for (let i = 0; i < this.sparkCount; i++) {
                const v = exp.sparkVelocities[i];
                sparkPos[i * 3] += v.x * dt;
                sparkPos[i * 3 + 1] += v.y * dt;
                sparkPos[i * 3 + 2] += v.z * dt;
                v.y -= 2500 * dt; // Gravity in UU
                v.multiplyScalar(0.97);
            }
            exp.sparks.geometry.attributes.position.needsUpdate = true;
            exp.sparks.material.opacity = Math.max(0, 1 - progress * 1.5);

            // Debris
            for (let i = 0; i < this.debrisCount; i++) {
                const d = exp.debrisData[i];
                const lp = exp.elapsed / d.life;

                d.position.addScaledVector(d.velocity, dt);
                d.velocity.y -= 2000 * dt; // Gravity in UU
                d.velocity.multiplyScalar(0.97);
                d.rotation.x += d.rotSpeed.x * dt;
                d.rotation.y += d.rotSpeed.y * dt;
                d.rotation.z += d.rotSpeed.z * dt;

                dummy.position.copy(d.position);
                dummy.rotation.copy(d.rotation);
                dummy.scale.setScalar(Math.max(0.01, 1 - lp));
                dummy.updateMatrix();
                exp.debris.setMatrixAt(i, dummy.matrix);
            }
            exp.debris.instanceMatrix.needsUpdate = true;

            // Smoke
            for (const s of exp.smoke) {
                const le = Math.max(0, exp.elapsed - s.delay);
                const lp = le / s.life;
                if (le > 0) {
                    s.sprite.position.addScaledVector(s.velocity, dt);
                    s.velocity.multiplyScalar(0.96);
                    s.sprite.scale.setScalar(s.startScale * (1 + lp * 3));
                    s.sprite.material.opacity = Math.min(lp * 5, 1) * Math.max(0, 1 - (lp - 0.2) * 1.5) * 0.4;
                }
            }

            // Light
            exp.light.intensity = Math.max(0, 12 * (1 - Math.min(exp.elapsed / 0.08, 1)));

            // Shockwave
            const swP = Math.min(exp.elapsed / 0.15, 1);
            exp.shockwave.scale.setScalar((1 - Math.pow(2, -10 * swP)) * 1200); // Scale in UU
            exp.shockwave.material.opacity = Math.max(0, 0.9 - swP * 1.2);

            // End
            if (exp.elapsed > exp.duration + 0.5) {
                this.deactivate(exp);
            }
        }

        this.activeExplosions = this.activeExplosions.filter(e => e.active);
    }

    deactivate(exp) {
        exp.active = false;
        exp.flash.visible = false;
        exp.sparks.visible = false;
        exp.debris.visible = false;
        exp.shockwave.visible = false;
        exp.fireballs.forEach(fb => fb.sprite.visible = false);
        exp.smoke.forEach(s => s.sprite.visible = false);
        exp.light.intensity = 0;
    }

    reset() {
        for (const exp of this.explosions) {
            this.deactivate(exp);
        }
        this.activeExplosions = [];
    }
}

// Store scene reference for lazy pool initialization
let _explosionScene = null;

/**
 * Initialize the explosion pool and do a warmup render to compile shaders.
 * This is now a no-op - pool is created lazily on first explosion.
 * Kept for backward compatibility.
 */
export async function precompileExplosionMaterials(renderer, scene, camera) {
    // Store scene reference for lazy initialization
    _explosionScene = scene;
    // Pool will be created on first explosion trigger
    console.log('[precompileExplosionMaterials] Deferred - pool will be created on first explosion');
}

/**
 * Get or create the explosion pool (lazy initialization)
 */
function getExplosionPool() {
    if (!_explosionPool && _explosionScene) {
        console.log('[ExplosionPool] Creating pool on first use...');
        console.time('[ExplosionPool] Init');

        // Create textures first
        getGlowTexture();
        getParticleTexture();
        getSmokeTexture();

        // Create the pool
        _explosionPool = new ExplosionPool(_explosionScene, 3);

        console.timeEnd('[ExplosionPool] Init');
    }
    return _explosionPool;
}

/**
 * Get cached soft particle texture with radial gradient
 */
function getParticleTexture() {
    if (_particleTexture) return _particleTexture;

    const canvas = document.createElement('canvas');
    canvas.width = 64;
    canvas.height = 64;
    const ctx = canvas.getContext('2d');

    const gradient = ctx.createRadialGradient(32, 32, 0, 32, 32, 32);
    gradient.addColorStop(0, 'rgba(255,255,255,1)');
    gradient.addColorStop(0.2, 'rgba(255,255,255,0.8)');
    gradient.addColorStop(0.5, 'rgba(255,255,255,0.3)');
    gradient.addColorStop(1, 'rgba(255,255,255,0)');

    ctx.fillStyle = gradient;
    ctx.fillRect(0, 0, 64, 64);

    _particleTexture = new THREE.CanvasTexture(canvas);
    return _particleTexture;
}

/**
 * Get cached glow texture for fire effects
 */
function getGlowTexture() {
    if (_glowTexture) return _glowTexture;

    const canvas = document.createElement('canvas');
    canvas.width = 128;
    canvas.height = 128;
    const ctx = canvas.getContext('2d');

    const gradient = ctx.createRadialGradient(64, 64, 0, 64, 64, 64);
    gradient.addColorStop(0, 'rgba(255,255,255,1)');
    gradient.addColorStop(0.1, 'rgba(255,200,100,0.9)');
    gradient.addColorStop(0.4, 'rgba(255,100,50,0.4)');
    gradient.addColorStop(0.7, 'rgba(255,50,0,0.1)');
    gradient.addColorStop(1, 'rgba(0,0,0,0)');

    ctx.fillStyle = gradient;
    ctx.fillRect(0, 0, 128, 128);

    _glowTexture = new THREE.CanvasTexture(canvas);
    return _glowTexture;
}

/**
 * Get cached smoke texture with noise
 */
function getSmokeTexture() {
    if (_smokeTexture) return _smokeTexture;

    const canvas = document.createElement('canvas');
    canvas.width = 64;
    canvas.height = 64;
    const ctx = canvas.getContext('2d');

    const imageData = ctx.createImageData(64, 64);
    for (let i = 0; i < imageData.data.length; i += 4) {
        const x = (i / 4) % 64;
        const y = Math.floor((i / 4) / 64);
        const dx = x - 32;
        const dy = y - 32;
        const dist = Math.sqrt(dx * dx + dy * dy) / 32;
        const noise = Math.random() * 0.3 + 0.7;
        const alpha = Math.max(0, (1 - dist * dist)) * noise * 255;

        imageData.data[i] = 255;
        imageData.data[i + 1] = 255;
        imageData.data[i + 2] = 255;
        imageData.data[i + 3] = alpha;
    }
    ctx.putImageData(imageData, 0, 0);

    _smokeTexture = new THREE.CanvasTexture(canvas);
    return _smokeTexture;
}

class BoostTrail {
    constructor(carMesh) {
        this.carMesh = carMesh;
        this.active = false;
        this.particleCount = 200;
        this.particles = [];

        // Create geometry for boost particles
        const geometry = new THREE.BufferGeometry();
        const positions = new Float32Array(this.particleCount * 3);
        const colors = new Float32Array(this.particleCount * 3);
        const sizes = new Float32Array(this.particleCount);
        const alphas = new Float32Array(this.particleCount);

        // Initialize all particles as inactive
        for (let i = 0; i < this.particleCount; i++) {
            positions[i * 3] = 0;
            positions[i * 3 + 1] = 0;
            positions[i * 3 + 2] = 0;

            // Orange boost colors (more orange, less yellow)
            colors[i * 3] = 1.0;     // R
            colors[i * 3 + 1] = 0.5; // G (reduced for more orange)
            colors[i * 3 + 2] = 0.1; // B (reduced for deeper orange)

            sizes[i] = 2; // Initial size (in UU)
            alphas[i] = 0;

            this.particles.push({
                life: 0,
                maxLife: 0.5,
                velocity: new THREE.Vector3(),
                active: false
            });
        }

        geometry.setAttribute('position', new THREE.BufferAttribute(positions, 3));
        geometry.setAttribute('color', new THREE.BufferAttribute(colors, 3));
        geometry.setAttribute('size', new THREE.BufferAttribute(sizes, 1));
        geometry.setAttribute('alpha', new THREE.BufferAttribute(alphas, 1));

        this.geometry = geometry;

        // Custom shader material for rocket flame effect
        const material = new THREE.ShaderMaterial({
            uniforms: {},
            vertexShader: `
                attribute float size;
                attribute float alpha;
                attribute vec3 color;
                varying vec3 vColor;
                varying float vAlpha;

                void main() {
                    vColor = color;
                    vAlpha = alpha;
                    vec4 mvPosition = modelViewMatrix * vec4(position, 1.0);
                    gl_PointSize = size * (2500.0 / -mvPosition.z); // Slightly larger for flame effect
                    gl_Position = projectionMatrix * mvPosition;
                }
            `,
            fragmentShader: `
                varying vec3 vColor;
                varying float vAlpha;

                void main() {
                    float dist = length(gl_PointCoord - vec2(0.5));
                    if (dist > 0.5) discard;

                    // Softer glow with hot center
                    float glow = 1.0 - (dist * 2.0);
                    glow = pow(glow, 0.6); // Softer falloff for more glow

                    // Brighter center (white-hot core)
                    vec3 flameColor = vColor;
                    if (dist < 0.15) {
                        flameColor = mix(vec3(1.0, 1.0, 0.9), vColor, dist / 0.15); // White-yellow core
                    }

                    gl_FragColor = vec4(flameColor * glow * 1.5, vAlpha * glow);
                }
            `,
            transparent: true,
            depthWrite: false,
            blending: THREE.AdditiveBlending
        });

        this.points = new THREE.Points(geometry, material);
        this.points.frustumCulled = false; // Disable frustum culling since particles move dynamically
        this.nextParticleIndex = 0;
    }

    setActive(active) {
        this.active = active;
    }

    emit(position, rotation, velocity, playbackSpeed = 1.0) {
        if (!this.active) return;

        // Emit particles proportional to playbackSpeed
        // At 1.0x: 3-5 particles, at 0.5x: 1-2 particles, at 2.0x: 6-10 particles
        const baseEmit = Math.floor(Math.random() * 3) + 3;
        const emitCount = Math.max(1, Math.round(baseEmit * playbackSpeed));

        for (let i = 0; i < emitCount; i++) {
            const particle = this.particles[this.nextParticleIndex];
            const positions = this.geometry.attributes.position.array;
            const alphas = this.geometry.attributes.alpha.array;
            const sizes = this.geometry.attributes.size.array;
            const colors = this.geometry.attributes.color.array;
            const idx = this.nextParticleIndex;

            // Position particles at rear of car (in UU)
            const rearOffset = new THREE.Vector3(-55, 0, 0); // Rear of car in local space
            rearOffset.applyQuaternion(rotation);

            // Tighter spread for more focused flame (in UU)
            const spread = new THREE.Vector3(
                (Math.random() - 0.5) * 10,
                (Math.random() - 0.5) * 15,
                (Math.random() - 0.5) * 15
            );

            const startPos = position.clone().add(rearOffset).add(spread);
            positions[idx * 3] = startPos.x;
            positions[idx * 3 + 1] = startPos.y;
            positions[idx * 3 + 2] = startPos.z;

            // Velocity: opposite to car direction + inherit some car velocity (in UU)
            const backwardDir = new THREE.Vector3(-1, 0, 0);
            backwardDir.applyQuaternion(rotation);
            backwardDir.multiplyScalar(150 + Math.random() * 80); // Slightly slower for denser look

            particle.velocity.copy(backwardDir);
            particle.velocity.add(velocity.clone().multiplyScalar(0.2)); // Inherit 20% of car velocity

            // Less turbulence for more cohesive flame (in UU)
            particle.velocity.add(new THREE.Vector3(
                (Math.random() - 0.5) * 30,
                (Math.random() - 0.5) * 30,
                (Math.random() - 0.5) * 30
            ));

            particle.life = 0;
            particle.maxLife = 0.3 + Math.random() * 0.3; // Shorter life (0.3-0.6s) for tighter flame
            particle.active = true;

            // Start with high alpha and larger size (intense at exhaust)
            alphas[idx] = 1.0;
            sizes[idx] = 3.0 + Math.random() * 2.0; // Larger initial size (3-5 UU)
            particle.initialSize = sizes[idx];

            // Color gradient: white/yellow at start, orange/red at end
            colors[idx * 3] = 1.0;     // R
            colors[idx * 3 + 1] = 0.8 + Math.random() * 0.2; // G (0.8-1.0 for yellow-white)
            colors[idx * 3 + 2] = 0.3 + Math.random() * 0.3; // B (0.3-0.6 for warm tint)

            this.nextParticleIndex = (this.nextParticleIndex + 1) % this.particleCount;
        }

        this.geometry.attributes.position.needsUpdate = true;
        this.geometry.attributes.alpha.needsUpdate = true;
        this.geometry.attributes.size.needsUpdate = true;
        this.geometry.attributes.color.needsUpdate = true;
    }

    update(delta) {
        const positions = this.geometry.attributes.position.array;
        const alphas = this.geometry.attributes.alpha.array;
        const sizes = this.geometry.attributes.size.array;
        const colors = this.geometry.attributes.color.array;

        for (let i = 0; i < this.particleCount; i++) {
            const particle = this.particles[i];
            if (!particle.active) continue;

            particle.life += delta;

            if (particle.life >= particle.maxLife) {
                particle.active = false;
                alphas[i] = 0;
                sizes[i] = 0;
                continue;
            }

            // Update position
            positions[i * 3] += particle.velocity.x * delta;
            positions[i * 3 + 1] += particle.velocity.y * delta;
            positions[i * 3 + 2] += particle.velocity.z * delta;

            // Life factor (0 = just born, 1 = about to die)
            const lifeFactor = particle.life / particle.maxLife;

            // Fade out with smooth curve
            alphas[i] = Math.pow(1.0 - lifeFactor, 0.5);

            // Shrink particles as they age (rocket flame tapers off)
            const initialSize = particle.initialSize || 3.0;
            sizes[i] = initialSize * (1.0 - lifeFactor * 0.7); // Shrink to 30% of original

            // Color transition: yellow-white → orange → red as particle ages
            colors[i * 3] = 1.0; // R stays at 1.0
            colors[i * 3 + 1] = Math.max(0.2, 0.9 - lifeFactor * 0.7); // G: 0.9 → 0.2
            colors[i * 3 + 2] = Math.max(0.0, 0.4 - lifeFactor * 0.4); // B: 0.4 → 0.0

            // Very light gravity (flames rise slightly)
            particle.velocity.y += 20 * delta;
        }

        this.geometry.attributes.position.needsUpdate = true;
        this.geometry.attributes.alpha.needsUpdate = true;
        this.geometry.attributes.size.needsUpdate = true;
        this.geometry.attributes.color.needsUpdate = true;
    }

    addToScene(scene) {
        scene.add(this.points);
    }

    removeFromScene(scene) {
        scene.remove(this.points);
    }

    dispose() {
        this.geometry.dispose();
        this.points.material.dispose();
    }
}

class BallTrail {
    constructor() {
        this.active = false;
        this.team = 0; // 0 = blue, 1 = orange
        this.velocity = 0; // Current ball velocity magnitude
        this.MIN_VELOCITY = 15; // Start showing trail at lower speeds

        // Internal game time (updated by delta, not real time)
        this.gameTime = 0;

        // Trail ribbon/line settings
        this.maxPoints = 100; // Number of points in the trail
        this.trailPoints = []; // Array of {position: Vector3, time: number}
        this.trailLifetime = 0.8; // Trail fades after 0.8 seconds

        // Create geometry for ribbon effect with variable size
        const positions = new Float32Array(this.maxPoints * 3);
        const colors = new Float32Array(this.maxPoints * 3);
        const sizes = new Float32Array(this.maxPoints);

        this.geometry = new THREE.BufferGeometry();
        this.geometry.setAttribute('position', new THREE.BufferAttribute(positions, 3));
        this.geometry.setAttribute('color', new THREE.BufferAttribute(colors, 3));
        this.geometry.setAttribute('size', new THREE.BufferAttribute(sizes, 1));

        // Custom shader material for glowing trail with variable size
        const material = new THREE.ShaderMaterial({
            uniforms: {},
            vertexShader: `
                attribute vec3 color;
                attribute float size;
                varying vec3 vColor;

                void main() {
                    vColor = color;
                    vec4 mvPosition = modelViewMatrix * vec4(position, 1.0);
                    gl_PointSize = size * (300.0 / -mvPosition.z); // Size scales with distance
                    gl_Position = projectionMatrix * mvPosition;
                }
            `,
            fragmentShader: `
                varying vec3 vColor;

                void main() {
                    // Circular shape
                    float dist = length(gl_PointCoord - vec2(0.5));
                    if (dist > 0.5) discard;

                    // Soft edges
                    float alpha = 1.0 - (dist * 2.0);
                    alpha = pow(alpha, 1.5);

                    gl_FragColor = vec4(vColor, alpha);
                }
            `,
            transparent: true,
            depthWrite: false,
            blending: THREE.AdditiveBlending
        });

        this.points = new THREE.Points(this.geometry, material);
        this.points.frustumCulled = false;
    }

    setTeam(team) {
        this.team = team;
        // Colors will be updated in the next emit() call
    }

    emit(position, velocity) {
        const speed = velocity.length();

        // Only emit if velocity is above threshold
        if (speed < this.MIN_VELOCITY) {
            this.active = false;
            return;
        }

        // Log when trail becomes active for the first time
        if (!this.active) {
            console.log(`✨ Ball trail activated! Speed: ${speed.toFixed(2)}, Team: ${this.team === 0 ? 'Blue' : 'Orange'}`);
        }

        this.active = true;
        this.velocity = speed;

        // Add new point to trail using internal game time
        this.trailPoints.push({
            position: position.clone(),
            time: this.gameTime
        });

        // Limit number of points
        if (this.trailPoints.length > this.maxPoints) {
            this.trailPoints.shift();
        }
    }

    update(delta) {
        // Advance internal game time
        this.gameTime += delta;

        const positions = this.geometry.attributes.position.array;
        const colors = this.geometry.attributes.color.array;
        const sizes = this.geometry.attributes.size.array;

        // Remove old points based on game time
        this.trailPoints = this.trailPoints.filter(point => (this.gameTime - point.time) < this.trailLifetime);

        // Team colors - more subtle and desaturated
        const teamColor = this.team === 1
            ? { r: 0.8, g: 0.35, b: 0.1 }  // Orange - more subtle
            : { r: 0.25, g: 0.45, b: 0.8 }; // Blue - more subtle

        // Update geometry with current points
        const numPoints = this.trailPoints.length;
        for (let i = 0; i < this.maxPoints; i++) {
            if (i < numPoints) {
                const point = this.trailPoints[i];
                const idx = i * 3;

                // Set position
                positions[idx] = point.position.x;
                positions[idx + 1] = point.position.y;
                positions[idx + 2] = point.position.z;

                // Calculate alpha based on age (fade out older points)
                const age = this.gameTime - point.time;
                const ageFactor = 1.0 - (age / this.trailLifetime);
                const alpha = Math.max(0, ageFactor) * 0.5; // Reduced overall opacity to 50%

                // Set color with fade - apply color intensity reduction
                colors[idx] = teamColor.r * alpha;
                colors[idx + 1] = teamColor.g * alpha;
                colors[idx + 2] = teamColor.b * alpha;

                // Calculate size - larger near the ball (newer points), smaller for older points
                // ageFactor = 1.0 (newest) to 0.0 (oldest)
                const sizeFactor = ageFactor; // Linear decrease
                const baseSize = 0.3; // Base size (much smaller)
                const maxSize = 1.2; // Max size near ball (much smaller)
                sizes[i] = baseSize + (maxSize - baseSize) * sizeFactor;
            } else {
                // Hide unused points
                positions[i * 3] = 0;
                positions[i * 3 + 1] = 0;
                positions[i * 3 + 2] = 0;
                colors[i * 3] = 0;
                colors[i * 3 + 1] = 0;
                colors[i * 3 + 2] = 0;
                sizes[i] = 0;
            }
        }

        this.geometry.attributes.position.needsUpdate = true;
        this.geometry.attributes.color.needsUpdate = true;
        this.geometry.attributes.size.needsUpdate = true;
    }

    addToScene(scene) {
        scene.add(this.points);
    }

    removeFromScene(scene) {
        scene.remove(this.points);
    }

    dispose() {
        this.geometry.dispose();
        this.points.material.dispose();
    }
}

class SupersonicTrail {
    constructor(team = 0) {
        this.team = team;
        this.active = false;

        // Trail ribbon settings - two trails (left and right wheel)
        this.maxPoints = 60; // Points per trail
        this.trailLifetime = 0.4; // Trail fades after 0.4 seconds

        // Store points for both trails
        this.leftTrailPoints = [];
        this.rightTrailPoints = [];

        // Team colors
        this.teamColors = {
            0: { r: 0.2, g: 0.5, b: 1.0 },   // Blue
            1: { r: 1.0, g: 0.4, b: 0.1 }    // Orange
        };

        // Arena dimensions in Unreal Units
        // Floor at y=0, ceiling at y~2044 UU, walls at x=±4096 UU, z=±5120 UU
        this.arenaBounds = {
            floor: 0,
            ceiling: 2044,
            wallX: 4096,    // Side walls
            wallZ: 5120     // Back walls (goals)
        };
        // Distance threshold to consider "grounded" on a surface (in UU)
        this.groundedThreshold = 50;

        // Create geometry for left trail
        this.leftGeometry = this.createTrailGeometry();
        this.rightGeometry = this.createTrailGeometry();

        // Custom shader material for glowing trail
        const material = new THREE.ShaderMaterial({
            uniforms: {},
            vertexShader: `
                attribute vec3 color;
                attribute float size;
                varying vec3 vColor;

                void main() {
                    vColor = color;
                    vec4 mvPosition = modelViewMatrix * vec4(position, 1.0);
                    gl_PointSize = size * (300.0 / -mvPosition.z);
                    gl_Position = projectionMatrix * mvPosition;
                }
            `,
            fragmentShader: `
                varying vec3 vColor;

                void main() {
                    float dist = length(gl_PointCoord - vec2(0.5));
                    if (dist > 0.5) discard;

                    // Soft edges with glow
                    float alpha = 1.0 - (dist * 2.0);
                    alpha = pow(alpha, 1.2);

                    gl_FragColor = vec4(vColor, alpha * 0.8);
                }
            `,
            transparent: true,
            depthWrite: false,
            blending: THREE.AdditiveBlending
        });

        this.leftPoints = new THREE.Points(this.leftGeometry, material);
        this.rightPoints = new THREE.Points(this.rightGeometry, material.clone());

        this.leftPoints.frustumCulled = false;
        this.rightPoints.frustumCulled = false;
    }

    createTrailGeometry() {
        const positions = new Float32Array(this.maxPoints * 3);
        const colors = new Float32Array(this.maxPoints * 3);
        const sizes = new Float32Array(this.maxPoints);

        const geometry = new THREE.BufferGeometry();
        geometry.setAttribute('position', new THREE.BufferAttribute(positions, 3));
        geometry.setAttribute('color', new THREE.BufferAttribute(colors, 3));
        geometry.setAttribute('size', new THREE.BufferAttribute(sizes, 1));

        return geometry;
    }

    setTeam(team) {
        this.team = team;
    }

    setActive(active) {
        if (this.active && !active) {
            // When deactivating, let existing trail points fade naturally
        }
        this.active = active;
    }

    /**
     * Check if the car is grounded on any surface (floor, walls, ceiling)
     * Returns { grounded: boolean, surface: 'floor'|'wall'|'ceiling'|null, normal: Vector3 }
     */
    isGrounded(position) {
        const threshold = this.groundedThreshold;
        const bounds = this.arenaBounds;

        // Check floor (y close to 0)
        if (position.y < threshold) {
            return { grounded: true, surface: 'floor', normal: new THREE.Vector3(0, 1, 0) };
        }

        // Check ceiling (y close to ceiling height)
        if (position.y > bounds.ceiling - threshold) {
            return { grounded: true, surface: 'ceiling', normal: new THREE.Vector3(0, -1, 0) };
        }

        // Check side walls (|x| close to wallX)
        if (Math.abs(position.x) > bounds.wallX - threshold) {
            const normalX = position.x > 0 ? -1 : 1;
            return { grounded: true, surface: 'wall', normal: new THREE.Vector3(normalX, 0, 0) };
        }

        // Check back walls (|z| close to wallZ)
        if (Math.abs(position.z) > bounds.wallZ - threshold) {
            const normalZ = position.z > 0 ? -1 : 1;
            return { grounded: true, surface: 'wall', normal: new THREE.Vector3(0, 0, normalZ) };
        }

        return { grounded: false, surface: null, normal: null };
    }

    emit(position, rotation, velocity) {
        if (!this.active) return;

        // Check if car is grounded on any surface
        const groundCheck = this.isGrounded(position);
        if (!groundCheck.grounded) return;

        const currentTime = Date.now() / 1000;

        // Calculate left and right wheel positions relative to car
        // Car dimensions roughly: length ~118 UU, width ~84 UU
        const leftOffset = new THREE.Vector3(-50, 0, 35);  // Rear left
        const rightOffset = new THREE.Vector3(-50, 0, -35); // Rear right

        // Apply car rotation to offsets
        leftOffset.applyQuaternion(rotation);
        rightOffset.applyQuaternion(rotation);

        // Calculate world positions
        const leftPos = position.clone().add(leftOffset);
        const rightPos = position.clone().add(rightOffset);

        // Snap trail positions to the surface the car is on
        const snapOffset = 2; // Slightly above surface to avoid z-fighting (in UU)
        if (groundCheck.surface === 'floor') {
            leftPos.y = snapOffset;
            rightPos.y = snapOffset;
        } else if (groundCheck.surface === 'ceiling') {
            leftPos.y = this.arenaBounds.ceiling - snapOffset;
            rightPos.y = this.arenaBounds.ceiling - snapOffset;
        } else if (groundCheck.surface === 'wall') {
            // For walls, snap to the wall surface
            if (groundCheck.normal.x !== 0) {
                // Side wall (X axis)
                const wallX = groundCheck.normal.x > 0 ? -this.arenaBounds.wallX + snapOffset : this.arenaBounds.wallX - snapOffset;
                leftPos.x = wallX;
                rightPos.x = wallX;
            } else if (groundCheck.normal.z !== 0) {
                // Back wall (Z axis)
                const wallZ = groundCheck.normal.z > 0 ? -this.arenaBounds.wallZ + snapOffset : this.arenaBounds.wallZ - snapOffset;
                leftPos.z = wallZ;
                rightPos.z = wallZ;
            }
        }

        // Add points to trails
        this.leftTrailPoints.push({
            position: leftPos.clone(),
            time: currentTime
        });

        this.rightTrailPoints.push({
            position: rightPos.clone(),
            time: currentTime
        });

        // Limit points
        if (this.leftTrailPoints.length > this.maxPoints) {
            this.leftTrailPoints.shift();
        }
        if (this.rightTrailPoints.length > this.maxPoints) {
            this.rightTrailPoints.shift();
        }
    }

    update(delta) {
        const currentTime = Date.now() / 1000;

        // Remove old points
        this.leftTrailPoints = this.leftTrailPoints.filter(
            point => (currentTime - point.time) < this.trailLifetime
        );
        this.rightTrailPoints = this.rightTrailPoints.filter(
            point => (currentTime - point.time) < this.trailLifetime
        );

        // Update geometries
        this.updateTrailGeometry(this.leftGeometry, this.leftTrailPoints, currentTime);
        this.updateTrailGeometry(this.rightGeometry, this.rightTrailPoints, currentTime);
    }

    updateTrailGeometry(geometry, trailPoints, currentTime) {
        const positions = geometry.attributes.position.array;
        const colors = geometry.attributes.color.array;
        const sizes = geometry.attributes.size.array;

        const teamColor = this.teamColors[this.team] || this.teamColors[0];
        const numPoints = trailPoints.length;

        for (let i = 0; i < this.maxPoints; i++) {
            if (i < numPoints) {
                const point = trailPoints[i];
                const idx = i * 3;

                // Set position
                positions[idx] = point.position.x;
                positions[idx + 1] = point.position.y;
                positions[idx + 2] = point.position.z;

                // Calculate fade based on age
                const age = currentTime - point.time;
                const ageFactor = 1.0 - (age / this.trailLifetime);
                const alpha = Math.max(0, ageFactor);

                // Set color with fade
                colors[idx] = teamColor.r * alpha;
                colors[idx + 1] = teamColor.g * alpha;
                colors[idx + 2] = teamColor.b * alpha;

                // Size: larger near car (newer points), smaller for older
                const baseSize = 0.15;
                const maxSize = 0.5;
                sizes[i] = baseSize + (maxSize - baseSize) * ageFactor;
            } else {
                // Hide unused points
                positions[i * 3] = 0;
                positions[i * 3 + 1] = 0;
                positions[i * 3 + 2] = 0;
                colors[i * 3] = 0;
                colors[i * 3 + 1] = 0;
                colors[i * 3 + 2] = 0;
                sizes[i] = 0;
            }
        }

        geometry.attributes.position.needsUpdate = true;
        geometry.attributes.color.needsUpdate = true;
        geometry.attributes.size.needsUpdate = true;
    }

    addToScene(scene) {
        scene.add(this.leftPoints);
        scene.add(this.rightPoints);
    }

    removeFromScene(scene) {
        scene.remove(this.leftPoints);
        scene.remove(this.rightPoints);
    }

    dispose() {
        this.leftGeometry.dispose();
        this.rightGeometry.dispose();
        this.leftPoints.material.dispose();
        this.rightPoints.material.dispose();
    }
}

/**
 * TrailSegment - A single trail segment (pair of left/right ribbons)
 * Each segment lives independently and fades out over time
 */
class TrailSegment {
    constructor(scene, team, trailWidth, trailLength) {
        this.scene = scene;
        this.team = team;
        this.trailWidth = trailWidth;
        this.trailLength = trailLength;
        this.active = true;
        this.dying = false;
        this.deathTime = 0;
        this.maxDeathTime = 1.5; // Time to fully fade out (seconds)

        // Team colors with alpha
        this.teamColors = {
            0: new THREE.Vector4(0.3, 0.6, 1.0, 0.9),   // Blue
            1: new THREE.Vector4(1.0, 0.5, 0.15, 0.9)   // Orange
        };

        // Create target objects
        this.leftTarget = new THREE.Object3D();
        this.rightTarget = new THREE.Object3D();
        scene.add(this.leftTarget);
        scene.add(this.rightTarget);

        // Create trail renderers
        this.leftTrail = this.createTrail(this.leftTarget);
        this.rightTrail = this.createTrail(this.rightTarget);

        // Set colors
        this.updateColors();

        // Activate immediately
        this.leftTrail.activate();
        this.rightTrail.activate();
    }

    createTrail(targetObject) {
        const trail = new TrailRenderer(this.scene, false);

        const material = TrailRenderer.createBaseMaterial();
        material.blending = THREE.AdditiveBlending;
        material.depthWrite = false;
        material.side = THREE.DoubleSide; // Visible from both sides

        // Create a cross-shaped head geometry so the trail is visible from all angles
        // This creates an "X" shape that's always visible regardless of camera angle
        const w = this.trailWidth;
        const headGeometry = [
            // Vertical ribbon
            new THREE.Vector3(0, 0, 0),
            new THREE.Vector3(0, w, 0),
            // Horizontal ribbon (perpendicular)
            new THREE.Vector3(-w/2, w/2, 0),
            new THREE.Vector3(w/2, w/2, 0),
        ];

        trail.initialize(
            material,
            this.trailLength,
            false,
            0,
            headGeometry,
            targetObject
        );

        trail.setAdvanceFrequency(60);

        // Disable frustum culling - the trail geometry is dynamic and
        // Three.js often miscalculates its bounding box, causing it to
        // disappear at certain camera angles
        if (trail.mesh) {
            trail.mesh.frustumCulled = false;
        }

        return trail;
    }

    updateColors() {
        const color = this.teamColors[this.team] || this.teamColors[0];
        const tailColor = new THREE.Vector4(color.x * 0.3, color.y * 0.3, color.z * 0.3, 0);

        if (this.leftTrail?.material) {
            this.leftTrail.material.uniforms.headColor.value.copy(color);
            this.leftTrail.material.uniforms.tailColor.value.copy(tailColor);
        }
        if (this.rightTrail?.material) {
            this.rightTrail.material.uniforms.headColor.value.copy(color);
            this.rightTrail.material.uniforms.tailColor.value.copy(tailColor);
        }
    }

    // Start the death process - trail will fade out
    startDying() {
        if (!this.dying) {
            this.dying = true;
            this.deathTime = 0;
            // Pause the trails so they stop growing
            this.leftTrail.pause();
            this.rightTrail.pause();
        }
    }

    updatePosition(leftPos, rightPos, rotation) {
        if (this.dying) return;

        this.leftTarget.position.copy(leftPos);
        this.rightTarget.position.copy(rightPos);
        this.leftTarget.quaternion.copy(rotation);
        this.rightTarget.quaternion.copy(rotation);
        this.leftTarget.updateMatrixWorld();
        this.rightTarget.updateMatrixWorld();
    }

    update(delta) {
        if (this.dying) {
            this.deathTime += delta;

            // Fade out the trails by reducing alpha
            const fadeProgress = Math.min(1, this.deathTime / this.maxDeathTime);
            const alpha = 1 - fadeProgress;

            const color = this.teamColors[this.team] || this.teamColors[0];
            const fadedColor = new THREE.Vector4(color.x, color.y, color.z, color.w * alpha);
            const fadedTail = new THREE.Vector4(color.x * 0.3, color.y * 0.3, color.z * 0.3, 0);

            if (this.leftTrail?.material) {
                this.leftTrail.material.uniforms.headColor.value.copy(fadedColor);
                this.leftTrail.material.uniforms.tailColor.value.copy(fadedTail);
            }
            if (this.rightTrail?.material) {
                this.rightTrail.material.uniforms.headColor.value.copy(fadedColor);
                this.rightTrail.material.uniforms.tailColor.value.copy(fadedTail);
            }

            // Mark as dead when fully faded
            if (this.deathTime >= this.maxDeathTime) {
                this.active = false;
            }
        }

        // Always update the trail renderer with delta for playback sync
        if (this.leftTrail.isActive) {
            this.leftTrail.update(delta);
        }
        if (this.rightTrail.isActive) {
            this.rightTrail.update(delta);
        }
    }

    dispose() {
        this.leftTrail.deactivate();
        this.rightTrail.deactivate();

        if (this.leftTrail.geometry) this.leftTrail.geometry.dispose();
        if (this.rightTrail.geometry) this.rightTrail.geometry.dispose();
        if (this.leftTrail.material) this.leftTrail.material.dispose();
        if (this.rightTrail.material) this.rightTrail.material.dispose();

        this.scene.remove(this.leftTarget);
        this.scene.remove(this.rightTarget);
    }
}

/**
 * SupersonicTrailV2 - Uses TrailRendererJS for proper ribbon trails
 * Manages multiple independent trail segments that can overlap
 */
class SupersonicTrailV2 {
    constructor(scene, team = 0) {
        this.scene = scene;
        this.team = team;
        this.active = false;

        // Trail settings
        this.trailLength = 80;
        this.trailWidth = 15; // Width in UU (scaled up from 0.15)

        // Arena bounds for grounded check (in Unreal Units, arena is 100x scale)
        this.arenaBounds = {
            floor: 0,
            ceiling: 2044,    // ~20.44 meters in UU
            wallX: 4096,      // Side walls
            wallZ: 5120       // Back walls (goal ends)
        };
        this.groundedThreshold = 50; // Threshold in UU to account for car height

        // Store all trail segments (active and dying)
        this.segments = [];
        this.currentSegment = null;

        // Track if we were grounded last frame (to detect landing)
        this.wasGrounded = true;
    }

    setTeam(team) {
        if (this.team !== team) {
            this.team = team;
            // Update current segment color if exists
            if (this.currentSegment && !this.currentSegment.dying) {
                this.currentSegment.team = team;
                this.currentSegment.updateColors();
            }
        }
    }

    setActive(active) {
        if (active && !this.active) {
            // Activating - don't create segment here, let emit() handle it
            // This ensures the segment starts at the correct position
            this.currentSegment = null;
            this.wasGrounded = true; // Assume grounded so first emit creates segment
        } else if (!active && this.active) {
            // Deactivating - let current segment die naturally
            if (this.currentSegment) {
                this.currentSegment.startDying();
                this.currentSegment = null;
            }
        }
        this.active = active;
    }

    isGrounded(position) {
        const threshold = this.groundedThreshold;
        const bounds = this.arenaBounds;

        if (position.y < threshold) {
            return { grounded: true, surface: 'floor', normal: new THREE.Vector3(0, 1, 0) };
        }
        if (position.y > bounds.ceiling - threshold) {
            return { grounded: true, surface: 'ceiling', normal: new THREE.Vector3(0, -1, 0) };
        }
        if (Math.abs(position.x) > bounds.wallX - threshold) {
            const normalX = position.x > 0 ? -1 : 1;
            return { grounded: true, surface: 'wall', normal: new THREE.Vector3(normalX, 0, 0) };
        }
        if (Math.abs(position.z) > bounds.wallZ - threshold) {
            const normalZ = position.z > 0 ? -1 : 1;
            return { grounded: true, surface: 'wall', normal: new THREE.Vector3(0, 0, normalZ) };
        }
        return { grounded: false, surface: null, normal: null };
    }

    emit(position, rotation, velocity) {
        if (!this.active) return;

        // Check if car is grounded
        const groundCheck = this.isGrounded(position);
        const isGrounded = groundCheck.grounded;

        if (!isGrounded) {
            // Car is airborne - kill current segment and start dying
            if (this.currentSegment && !this.currentSegment.dying) {
                this.currentSegment.startDying();
                this.currentSegment = null;
            }
            this.wasGrounded = false;
            return;
        }

        // Car is grounded - check if we just landed (need new segment)
        if (!this.wasGrounded || !this.currentSegment) {
            // Just landed or no current segment - create a new one
            if (this.currentSegment && !this.currentSegment.dying) {
                this.currentSegment.startDying();
            }
            this.currentSegment = new TrailSegment(
                this.scene, this.team, this.trailWidth, this.trailLength
            );
            this.segments.push(this.currentSegment);
        }

        this.wasGrounded = true;

        // Calculate wheel positions (closer to the rear wheels) in Unreal Units
        // X: -30 = behind car center (toward rear wheels)
        // Y: 5 = slightly above ground
        // Z: ±40 = lateral offset (positive = left, negative = right)
        const leftOffset = new THREE.Vector3(-30, 5, 40);
        const rightOffset = new THREE.Vector3(-30, 5, -40);

        leftOffset.applyQuaternion(rotation);
        rightOffset.applyQuaternion(rotation);

        const leftPos = position.clone().add(leftOffset);
        const rightPos = position.clone().add(rightOffset);

        // Snap to surface (in UU)
        const snapOffset = 2;
        if (groundCheck.surface === 'floor') {
            leftPos.y = snapOffset;
            rightPos.y = snapOffset;
        } else if (groundCheck.surface === 'ceiling') {
            leftPos.y = this.arenaBounds.ceiling - snapOffset;
            rightPos.y = this.arenaBounds.ceiling - snapOffset;
        } else if (groundCheck.surface === 'wall') {
            if (groundCheck.normal.x !== 0) {
                const wallX = groundCheck.normal.x > 0
                    ? -this.arenaBounds.wallX + snapOffset
                    : this.arenaBounds.wallX - snapOffset;
                leftPos.x = wallX;
                rightPos.x = wallX;
            } else if (groundCheck.normal.z !== 0) {
                const wallZ = groundCheck.normal.z > 0
                    ? -this.arenaBounds.wallZ + snapOffset
                    : this.arenaBounds.wallZ - snapOffset;
                leftPos.z = wallZ;
                rightPos.z = wallZ;
            }
        }

        // Update current segment position
        this.currentSegment.updatePosition(leftPos, rightPos, rotation);
    }

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

    addToScene(scene) {
        // Segments add themselves to the scene
    }

    removeFromScene(scene) {
        // Kill all segments
        for (const segment of this.segments) {
            segment.startDying();
        }
        this.currentSegment = null;
    }

    dispose() {
        for (const segment of this.segments) {
            segment.dispose();
        }
        this.segments = [];
        this.currentSegment = null;
    }
}

class ParticleExplosion {
    constructor(position, team = 0) {
        this.position = position.clone();
        this.team = team;
        this.age = 0;
        this.maxAge = 3.0; // 3 seconds duration

        const particleCount = 1000;
        const movementSpeed = 6000; // Speed in UU

        // Create geometry
        const geometry = new THREE.BufferGeometry();
        const positions = new Float32Array(particleCount * 3);
        const colors = new Float32Array(particleCount * 3);
        const sizes = new Float32Array(particleCount);

        // Store velocities for each particle
        this.velocities = [];

        // Team colors: Blue for team 0, Orange for team 1
        const baseColor = team === 0 ? new THREE.Color(0x3399ff) : new THREE.Color(0xff6600);

        for (let i = 0; i < particleCount; i++) {
            // Start at explosion position
            positions[i * 3] = position.x;
            positions[i * 3 + 1] = position.y;
            positions[i * 3 + 2] = position.z;

            // Random direction for explosion (spherical distribution)
            // Use spherical coordinates for uniform sphere distribution
            const theta = Math.random() * Math.PI * 2; // Angle around vertical axis
            const phi = Math.acos(2 * Math.random() - 1); // Angle from vertical axis
            const speed = movementSpeed * (0.3 + Math.random() * 0.7); // Random speed variation

            this.velocities.push({
                x: speed * Math.sin(phi) * Math.cos(theta),
                y: speed * Math.sin(phi) * Math.sin(theta),
                z: speed * Math.cos(phi)
            });

            // Color variation
            const colorVar = 0.2;
            colors[i * 3] = Math.min(1, Math.max(0, baseColor.r + (Math.random() - 0.5) * colorVar));
            colors[i * 3 + 1] = Math.min(1, Math.max(0, baseColor.g + (Math.random() - 0.5) * colorVar));
            colors[i * 3 + 2] = Math.min(1, Math.max(0, baseColor.b + (Math.random() - 0.5) * colorVar));

            // Size variation (in UU)
            sizes[i] = (Math.random() * 20 + 20);
        }

        geometry.setAttribute('position', new THREE.BufferAttribute(positions, 3));
        geometry.setAttribute('color', new THREE.BufferAttribute(colors, 3));
        geometry.setAttribute('size', new THREE.BufferAttribute(sizes, 1));

        this.geometry = geometry;

        // Create material
        const material = new THREE.PointsMaterial({
            size: 30, // Size in UU
            vertexColors: true,
            transparent: true,
            opacity: 1.0,
            sizeAttenuation: true,
            depthWrite: false,
            blending: THREE.AdditiveBlending
        });

        this.points = new THREE.Points(geometry, material);
    }

    update(delta) {
        this.age += delta;
        const ageFactor = this.age / this.maxAge;

        if (ageFactor >= 1.0) return true; // Signal for removal

        const positions = this.geometry.attributes.position.array;

        // Update particle positions
        for (let i = 0; i < this.velocities.length; i++) {
            positions[i * 3] += this.velocities[i].x * delta;
            positions[i * 3 + 1] += this.velocities[i].y * delta;
            positions[i * 3 + 2] += this.velocities[i].z * delta;
        }

        this.geometry.attributes.position.needsUpdate = true;

        // Fade out
        this.points.material.opacity = 1.0 - ageFactor;

        return false; // Keep alive
    }

    addToScene(scene) {
        scene.add(this.points);
    }

    removeFromScene(scene) {
        scene.remove(this.points);
        this.geometry.dispose();
        this.points.material.dispose();
    }
}

/**
 * DemolitionExplosion - Full explosion effect (EXACTLY like explosion.test)
 *
 * Creates FRESH materials each explosion but shares TEXTURES and GEOMETRIES.
 * This is the approach that works in explosion.test without freezing.
 *
 * Why this works: WebGL shader programs are keyed by material TYPE + parameters.
 * Once compiled via precompileExplosionMaterials(), new materials with same
 * parameters reuse the compiled shader programs.
 */
class DemolitionExplosion {
    constructor(scene, position, rotation, team = 0) {
        this.scene = scene;
        this.position = position.clone();
        this.rotation = rotation ? rotation.clone() : new THREE.Quaternion();
        this.team = team;

        this.elapsed = 0;
        this.duration = 0.6;
        this.isActive = true;

        // All objects created for this explosion (for cleanup)
        this.objects = [];
        this.fireballs = [];
        this.debris = [];
        this.smokeParticles = [];
        this.sparkData = [];

        this.init();
    }

    init() {
        const glowTex = getGlowTexture();
        const smokeTex = getSmokeTexture();
        const particleTex = getParticleTexture();
        const geoCache = getGeometryCache();

        // Core flash - NEW material, shared texture
        const flashMat = new THREE.SpriteMaterial({
            map: glowTex,
            color: 0xffffaa,
            transparent: true,
            blending: THREE.AdditiveBlending,
            depthWrite: false
        });
        this.coreFlash = new THREE.Sprite(flashMat);
        this.coreFlash.position.copy(this.position);
        this.coreFlash.scale.setScalar(0.1);
        this.scene.add(this.coreFlash);
        this.objects.push(this.coreFlash);

        // Fireballs (40) - NEW materials, shared texture
        for (let i = 0; i < 40; i++) {
            const mat = new THREE.SpriteMaterial({
                map: glowTex,
                color: new THREE.Color().setHSL(0.08 - Math.random() * 0.08, 1, 0.5 + Math.random() * 0.3),
                transparent: true,
                blending: THREE.AdditiveBlending,
                depthWrite: false
            });
            const sprite = new THREE.Sprite(mat);
            sprite.position.copy(this.position);

            const theta = Math.random() * Math.PI * 2;
            const phi = Math.acos(2 * Math.random() - 1);
            const speed = 3 + Math.random() * 8;

            sprite.userData = {
                velocity: new THREE.Vector3(
                    Math.sin(phi) * Math.cos(theta) * speed,
                    Math.sin(phi) * Math.sin(theta) * speed + 2,
                    Math.cos(phi) * speed
                ),
                startScale: 0.5 + Math.random() * 1.5,
                life: 0.15 + Math.random() * 0.25
            };

            this.scene.add(sprite);
            this.fireballs.push(sprite);
            this.objects.push(sprite);
        }

        // Sparks (80) - NEW geometry, NEW material, shared texture
        const sparkCount = 80;
        const positions = new Float32Array(sparkCount * 3);
        const colors = new Float32Array(sparkCount * 3);

        for (let i = 0; i < sparkCount; i++) {
            positions[i * 3] = this.position.x;
            positions[i * 3 + 1] = this.position.y;
            positions[i * 3 + 2] = this.position.z;

            const color = new THREE.Color().setHSL(0.1 + Math.random() * 0.05, 1, 0.7 + Math.random() * 0.3);
            colors[i * 3] = color.r;
            colors[i * 3 + 1] = color.g;
            colors[i * 3 + 2] = color.b;

            const theta = Math.random() * Math.PI * 2;
            const phi = Math.acos(2 * Math.random() - 1);
            const speed = 10 + Math.random() * 20;

            this.sparkData.push({
                velocity: new THREE.Vector3(
                    Math.sin(phi) * Math.cos(theta) * speed,
                    Math.sin(phi) * Math.sin(theta) * speed + 5,
                    Math.cos(phi) * speed
                )
            });
        }

        const sparkGeo = new THREE.BufferGeometry();
        sparkGeo.setAttribute('position', new THREE.BufferAttribute(positions, 3));
        sparkGeo.setAttribute('color', new THREE.BufferAttribute(colors, 3));
        this._sparkGeo = sparkGeo; // Keep reference for disposal

        const sparkMat = new THREE.PointsMaterial({
            size: 0.15,
            map: particleTex,
            vertexColors: true,
            transparent: true,
            blending: THREE.AdditiveBlending,
            depthWrite: false,
            sizeAttenuation: true
        });

        this.sparks = new THREE.Points(sparkGeo, sparkMat);
        this.scene.add(this.sparks);
        this.objects.push(this.sparks);

        // Debris (20) - NEW materials, SHARED geometries
        for (let i = 0; i < 20; i++) {
            const geo = geoCache.debris[i % 3];
            const mat = new THREE.MeshBasicMaterial({
                color: new THREE.Color().setHSL(0.08, 0.8, 0.2 + Math.random() * 0.3)
            });
            const mesh = new THREE.Mesh(geo, mat);
            mesh.position.copy(this.position);

            const theta = Math.random() * Math.PI * 2;
            const phi = Math.acos(2 * Math.random() - 1);
            const speed = 5 + Math.random() * 15;

            mesh.userData = {
                velocity: new THREE.Vector3(
                    Math.sin(phi) * Math.cos(theta) * speed,
                    Math.sin(phi) * Math.sin(theta) * speed + 8,
                    Math.cos(phi) * speed
                ),
                rotationSpeed: new THREE.Vector3(
                    (Math.random() - 0.5) * 20,
                    (Math.random() - 0.5) * 20,
                    (Math.random() - 0.5) * 20
                ),
                life: 0.5 + Math.random() * 0.5
            };

            this.scene.add(mesh);
            this.debris.push(mesh);
            this.objects.push(mesh);
        }

        // Shockwave horizontal - NEW material, SHARED geometry
        const swMat = new THREE.MeshBasicMaterial({
            color: 0xffaa44,
            transparent: true,
            opacity: 0.8,
            side: THREE.DoubleSide,
            blending: THREE.AdditiveBlending,
            depthWrite: false
        });
        this.shockwave = new THREE.Mesh(geoCache.ringH, swMat);
        this.shockwave.position.copy(this.position);
        this.shockwave.rotation.x = -Math.PI / 2;
        this.scene.add(this.shockwave);
        this.objects.push(this.shockwave);

        // Shockwave vertical - NEW material, SHARED geometry
        const sw2Mat = new THREE.MeshBasicMaterial({
            color: 0xff6622,
            transparent: true,
            opacity: 0.6,
            side: THREE.DoubleSide,
            blending: THREE.AdditiveBlending,
            depthWrite: false
        });
        this.shockwave2 = new THREE.Mesh(geoCache.ringV, sw2Mat);
        this.shockwave2.position.copy(this.position);
        this.scene.add(this.shockwave2);
        this.objects.push(this.shockwave2);

        // Smoke (15) - NEW materials, shared texture
        for (let i = 0; i < 15; i++) {
            const mat = new THREE.SpriteMaterial({
                map: smokeTex,
                color: 0x222222,
                transparent: true,
                opacity: 0,
                depthWrite: false
            });
            const sprite = new THREE.Sprite(mat);
            sprite.position.copy(this.position);
            sprite.position.add(new THREE.Vector3(
                (Math.random() - 0.5) * 0.5,
                Math.random() * 0.5,
                (Math.random() - 0.5) * 0.5
            ));

            sprite.userData = {
                velocity: new THREE.Vector3(
                    (Math.random() - 0.5) * 2,
                    1 + Math.random() * 2,
                    (Math.random() - 0.5) * 2
                ),
                startScale: 0.5 + Math.random() * 0.5,
                delay: i * 0.02,
                life: 0.8 + Math.random() * 0.4
            };

            this.scene.add(sprite);
            this.smokeParticles.push(sprite);
            this.objects.push(sprite);
        }

        // Light
        this.light = new THREE.PointLight(0xff6600, 10, 15);
        this.light.position.copy(this.position);
        this.scene.add(this.light);
        this.objects.push(this.light);
    }

    easeOutExpo(t) {
        return t === 1 ? 1 : 1 - Math.pow(2, -10 * t);
    }

    update(dt) {
        if (!this.isActive) return true;

        this.elapsed += dt;
        const progress = this.elapsed / this.duration;

        // Core flash
        const flashP = Math.min(this.elapsed / 0.08, 1);
        this.coreFlash.scale.setScalar(this.easeOutExpo(flashP) * 10);
        this.coreFlash.material.opacity = Math.max(0, 1 - flashP * 2);

        // Fireballs
        for (const fb of this.fireballs) {
            const lp = this.elapsed / fb.userData.life;
            fb.position.add(fb.userData.velocity.clone().multiplyScalar(dt));
            fb.userData.velocity.multiplyScalar(0.94);
            fb.userData.velocity.y -= 10 * dt;
            fb.scale.setScalar(fb.userData.startScale * (1 + lp * 2) * Math.max(0, 1 - lp));
            fb.material.opacity = Math.max(0, 1 - lp * lp);
        }

        // Sparks
        const pos = this.sparks.geometry.attributes.position.array;
        for (let i = 0; i < this.sparkData.length; i++) {
            const d = this.sparkData[i];
            pos[i * 3] += d.velocity.x * dt;
            pos[i * 3 + 1] += d.velocity.y * dt;
            pos[i * 3 + 2] += d.velocity.z * dt;
            d.velocity.y -= 30 * dt;
            d.velocity.multiplyScalar(0.97);
        }
        this.sparks.geometry.attributes.position.needsUpdate = true;
        this.sparks.material.opacity = Math.max(0, 1 - progress * 1.5);

        // Debris
        for (const d of this.debris) {
            const lp = this.elapsed / d.userData.life;
            d.position.add(d.userData.velocity.clone().multiplyScalar(dt));
            d.userData.velocity.y -= 25 * dt;
            d.userData.velocity.multiplyScalar(0.97);
            d.rotation.x += d.userData.rotationSpeed.x * dt;
            d.rotation.y += d.userData.rotationSpeed.y * dt;
            d.rotation.z += d.userData.rotationSpeed.z * dt;
            d.scale.setScalar(Math.max(0, 1 - lp));
        }

        // Shockwaves
        const swP = Math.min(this.elapsed / 0.2, 1);
        this.shockwave.scale.setScalar(this.easeOutExpo(swP) * 8);
        this.shockwave.material.opacity = Math.max(0, 0.9 - swP * 1.2);
        this.shockwave2.scale.setScalar(this.easeOutExpo(swP) * 5);
        this.shockwave2.material.opacity = Math.max(0, 0.7 - swP * 1.5);

        // Smoke
        for (const s of this.smokeParticles) {
            const le = Math.max(0, this.elapsed - s.userData.delay);
            const lp = le / s.userData.life;
            if (le > 0) {
                s.position.add(s.userData.velocity.clone().multiplyScalar(dt));
                s.userData.velocity.multiplyScalar(0.96);
                s.scale.setScalar(s.userData.startScale * (1 + lp * 4));
                s.material.opacity = Math.min(lp * 6, 1) * Math.max(0, 1 - (lp - 0.2) * 1.5) * 0.5;
            }
        }

        // Light
        this.light.intensity = Math.max(0, 15 * (1 - Math.min(this.elapsed / 0.1, 1)));

        if (this.elapsed > this.duration + 0.6) {
            this.isActive = false;
            return true;
        }

        return false;
    }

    addToScene(scene) {
        // Objects are already added in init()
    }

    removeFromScene(scene) {
        // Remove all objects and dispose materials (NOT shared textures/geometries)
        for (const obj of this.objects) {
            scene.remove(obj);
            // Dispose material but NOT the texture (it's shared)
            if (obj.material && !obj.isLight) {
                obj.material.dispose();
            }
        }
        // Dispose spark geometry (it's not shared)
        if (this._sparkGeo) {
            this._sparkGeo.dispose();
        }
        this.isActive = false;
    }
}

export class EffectsManager {
    constructor(scene) {
        this.scene = scene;
        this.renderer = null;
        this.camera = null;
        this.explosions = {
            active: [],
        };
        this.boostTrails = new Map(); // carActorId -> BoostTrail
        this.supersonicTrails = new Map(); // carActorId -> SupersonicTrail
        this.ballTrail = null; // Single ball trail

        // Pre-initialize explosion textures to avoid freeze on first explosion
        initExplosionTextures();
    }

    /**
     * Set renderer and camera references for explosion pools
     * Should be called from GameEngine after initialization
     */
    setRenderContext(renderer, camera) {
        this.renderer = renderer;
        this.camera = camera;
        // Pre-warm explosion pools now that we have renderer/camera
        warmupExplosionPool(this.scene, renderer, camera);
    }

    reset() {
        this.explosions.active.forEach(explosion => explosion.removeFromScene(this.scene));
        this.explosions.active = [];

        // Clear boost trails
        this.boostTrails.forEach(trail => {
            trail.removeFromScene(this.scene);
            trail.dispose();
        });
        this.boostTrails.clear();

        // Clear supersonic trails
        this.supersonicTrails.forEach(trail => {
            trail.removeFromScene(this.scene);
            trail.dispose();
        });
        this.supersonicTrails.clear();

        // Clear ball trail
        if (this.ballTrail) {
            this.ballTrail.removeFromScene(this.scene);
            this.ballTrail.dispose();
            this.ballTrail = null;
        }
    }

    clearEvents() {
        this.explosions.goalEvents.clear();
        this.explosions.demoEvents.clear();
    }

    /**
     * Reset ball trail (call when seeking to avoid stale segments)
     */
    resetBallTrail() {
        if (this.ballTrail) {
            this.ballTrail.reset();
        }
    }

    createBoostTrail(carMesh, carActorId) {
        // Remove old trail if exists
        if (this.boostTrails.has(carActorId)) {
            const oldTrail = this.boostTrails.get(carActorId);
            oldTrail.removeFromScene(this.scene);
            oldTrail.dispose();
        }

        const trail = new BoostTrail(carMesh);
        trail.addToScene(this.scene);
        this.boostTrails.set(carActorId, trail);
        return trail;
    }

    removeBoostTrail(carActorId) {
        const trail = this.boostTrails.get(carActorId);
        if (trail) {
            trail.removeFromScene(this.scene);
            trail.dispose();
            this.boostTrails.delete(carActorId);
        }
    }

    updateBoostTrail(carActorId, isBoosting, position, rotation, velocity) {
        const trail = this.boostTrails.get(carActorId);
        if (!trail) return;

        trail.setActive(isBoosting);
        if (isBoosting) {
            // Pass playbackSpeed so emit can adjust particle count
            trail.emit(position, rotation, velocity, this._playbackSpeed || 1.0);
        }
    }

    createSupersonicTrail(carActorId, team) {
        // Remove old trail if exists
        if (this.supersonicTrails.has(carActorId)) {
            const oldTrail = this.supersonicTrails.get(carActorId);
            oldTrail.removeFromScene(this.scene);
            oldTrail.dispose();
        }

        // Use SupersonicTrailV2 with TrailRendererJS for proper ribbon trails
        const trail = new SupersonicTrailV2(this.scene, team);
        trail.addToScene(this.scene);
        this.supersonicTrails.set(carActorId, trail);
        return trail;
    }

    removeSupersonicTrail(carActorId) {
        const trail = this.supersonicTrails.get(carActorId);
        if (trail) {
            trail.removeFromScene(this.scene);
            trail.dispose();
            this.supersonicTrails.delete(carActorId);
        }
    }

    updateSupersonicTrail(carActorId, isSupersonic, position, rotation, velocity, team) {
        let trail = this.supersonicTrails.get(carActorId);

        // Create trail if needed and supersonic
        if (!trail && isSupersonic) {
            trail = this.createSupersonicTrail(carActorId, team);
        }

        if (!trail) return;

        // Update team if changed
        if (team !== undefined && trail.team !== team) {
            trail.setTeam(team);
        }

        trail.setActive(isSupersonic);
        if (isSupersonic) {
            trail.emit(position, rotation, velocity);
        }
    }

    createBallTrail() {
        if (this.ballTrail) {
            this.ballTrail.removeFromScene(this.scene);
            this.ballTrail.dispose();
        }

        // Use new SpiralBallTrail for enhanced spiral effect
        this.ballTrail = new SpiralBallTrail(this.scene, 0);
        this.ballTrail.addToScene(this.scene);
        console.log('✓ Spiral ball trail created and added to scene');
        return this.ballTrail;
    }

    /**
     * Update ball trail with position and velocity
     * @param {THREE.Vector3} position - Ball position
     * @param {THREE.Vector3} velocity - Ball velocity
     * @param {number} team - Team (0 = blue, 1 = orange)
     */
    updateBallTrail(position, velocity, team) {
        if (!this.ballTrail) {
            this.createBallTrail();
        }

        // Update team color if changed
        if (team !== undefined && this.ballTrail.team !== team) {
            this.ballTrail.setTeam(team);
        }

        // SpiralBallTrail.emit requires delta for rotation calculation
        // Use scaled delta based on playbackSpeed (1/60 * playbackSpeed)
        const scaledDelta = (1/60) * (this._playbackSpeed || 1.0);
        this.ballTrail.emit(position, velocity, scaledDelta);
    }

    triggerGoalExplosion(position, team) {
        // Use the goal explosion pool with team colors (blue = 0, orange = 1)
        // Pass renderer/camera to ensure warmup happens if not already done
        const pool = getGoalExplosionPool(this.scene, this.renderer, this.camera);
        if (pool) {
            // Update camera reference for billboarding
            if (this.camera) {
                pool.camera = this.camera;
            }
            pool.trigger(position, team);
        }
    }

    /**
     * Trigger a demolition explosion with car orientation
     * @param {THREE.Vector3} position - Explosion position
     * @param {THREE.Quaternion} rotation - Car rotation (optional, defaults to identity)
     * @param {number} team - Team (0 = blue, 1 = orange)
     */
    triggerDemoExplosion(position, rotation, team) {
        // Support legacy calls with (position, team) signature
        if (typeof rotation === 'number') {
            team = rotation;
            rotation = new THREE.Quaternion();
        }

        // Use the simplified explosion pool (better performance, no shader issues)
        const pool = getSimplifiedExplosionPool(this.scene);
        if (pool) {
            pool.trigger(position);
        }
    }

    update(delta, isPlaying = true, playbackSpeed = 1.0) {
        // Store playbackSpeed for use by emit functions
        this._playbackSpeed = playbackSpeed;

        // Scale delta by playbackSpeed so effects are synchronized with game time
        const scaledDelta = delta * playbackSpeed;

        // Update simplified explosion pool (new performant system)
        // Explosions should always animate even when paused
        if (_simplifiedExplosionPool) {
            _simplifiedExplosionPool.update(scaledDelta);
        }

        // Update goal explosion pool (team-colored goal explosions)
        if (_goalExplosionPool) {
            _goalExplosionPool.update(scaledDelta);
        }

        // NOTE: Legacy _explosionPool is deprecated and no longer updated
        // The old ExplosionPool class is kept in the codebase for reference only

        // Update active explosions (legacy particle explosions for goals)
        for (let i = this.explosions.active.length - 1; i >= 0; i--) {
            const explosion = this.explosions.active[i];
            const isDead = explosion.update(scaledDelta);
            if (isDead) {
                explosion.removeFromScene(this.scene);
                this.explosions.active.splice(i, 1);
            }
        }

        // Trail updates should only happen when playing
        // When paused, trails should freeze in place
        if (!isPlaying) {
            return;
        }

        // Update boost trails with scaled delta
        this.boostTrails.forEach(trail => {
            trail.update(scaledDelta);
        });

        // Update supersonic trails with scaled delta
        this.supersonicTrails.forEach(trail => {
            trail.update(scaledDelta);
        });

        // Update ball trail with scaled delta
        if (this.ballTrail) {
            this.ballTrail.update(scaledDelta);
        }
    }
}
