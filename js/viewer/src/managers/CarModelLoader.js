import * as THREE from 'three';
import { FBXLoader } from 'three/examples/jsm/loaders/FBXLoader.js';
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js';
import { DRACOLoader } from 'three/examples/jsm/loaders/DRACOLoader.js';
import { clone as skeletonClone } from 'three/examples/jsm/utils/SkeletonUtils.js';

/**
 * CarModelLoader - Loads and manages car models (GLB format) with team-colored materials
 *
 * Supports 7 car models: octane, fennec, dominus, breakout, merc, mantis, x-devil
 * Each hitbox type maps to its representative car model.
 *
 * Mapping:
 * - Octane hitbox -> octane (or fennec if car name is Fennec)
 * - Dominus hitbox -> dominus
 * - Breakout hitbox -> breakout
 * - Plank hitbox -> mantis
 * - Hybrid hitbox -> x-devil
 * - Merc hitbox -> merc
 */
export class CarModelLoader {
    constructor() {
        this.fbxLoader = new FBXLoader();
        this.gltfLoader = new GLTFLoader();
        this.textureLoader = new THREE.TextureLoader();

        // Setup DRACO loader for compressed GLB files (local copy for better caching)
        const dracoLoader = new DRACOLoader();
        dracoLoader.setDecoderPath('/draco/');
        this.gltfLoader.setDRACOLoader(dracoLoader);

        // Cache for loaded models (we clone them for each car)
        this.modelCache = new Map(); // modelType -> { model, chassisTexture }
        this.loadingPromises = new Map(); // modelType -> Promise

        // Model configuration: all models are GLB format
        // format: 'glb'
        // file: filename with extension
        // scale: 100.0 (arena is scaled 100x)
        // wheelSockets: if true, model uses empty objects as wheel attachment points
        // wheelModel: path to separate wheel model (relative to /models/wheels/)
        this.modelConfig = {
            'octane': {
                format: 'glb',
                file: 'octane.glb',
                scale: 100.0,
                wheelSockets: true,
                wheelModel: 'Wheel_Boog.glb'
            },
            'fennec': {
                format: 'glb',
                file: 'fennec.glb',
                scale: 100.0,
                wheelSockets: true,
                wheelModel: 'Wheel_Boog.glb'
            },
            'dominus': {
                format: 'glb',
                file: 'dominus.glb',
                scale: 100.0,
                wheelSockets: true,
                wheelModel: 'Wheel_Boog.glb'
            },
            'breakout': {
                format: 'glb',
                file: 'breakout.glb',
                scale: 100.0,
                wheelSockets: true,
                wheelModel: 'Wheel_Boog.glb'
            },
            'merc': {
                format: 'glb',
                file: 'merc.glb',
                scale: 100.0,
                wheelSockets: true,
                wheelModel: 'Wheel_Boog.glb'
            },
            'mantis': {
                format: 'glb',
                file: 'mantis.glb',
                scale: 100.0,
                wheelSockets: true,
                wheelModel: 'Wheel_Boog.glb'
            },
            'x-devil': {
                format: 'glb',
                file: 'x-devil.glb',
                scale: 100.0,
                wheelSockets: true,
                wheelModel: 'Wheel_Boog.glb'
            },
        };

        // Cache for wheel models
        this.wheelModelCache = new Map();
        this.wheelLoadingPromises = new Map();

        // Deferred preload - will be set when preloadModelsForReplay() is called
        this.preloadReady = Promise.resolve();
        this._preloadStarted = false;

        // Map car names to model folders (exact matches)
        this.carNameToModel = {
            'Octane': 'octane',
            'Octane ZSR': 'octane',
            'Fennec': 'fennec',
            'Dominus': 'dominus',
            'Dominus GT': 'dominus',
            'Breakout': 'breakout',
            'Breakout Type-S': 'breakout',
            'Merc': 'merc',
            'Mantis': 'mantis',
            'X-Devil': 'x-devil',
            'X-Devil Mk2': 'x-devil',
        };

        // Fallback: Map hitbox types to model folders (for cars we don't have exact models for)
        this.hitboxToModel = {
            'Octane': 'octane',
            'Dominus': 'dominus',
            'Breakout': 'breakout',
            'Plank': 'mantis',
            'Hybrid': 'x-devil',
            'Merc': 'merc',
        };

        // Team colors
        this.TEAM_COLORS = {
            blue: new THREE.Color(0x0066ff),
            orange: new THREE.Color(0xff6600)
        };

        // Hitbox dimensions in Unreal Units (matching framework/src/data/hitbox_dimensions.ts)
        // offsetX = forward offset from pivot point
        // offsetZ = height of hitbox CENTER above the pivot point
        // The pivot is where the replay position is reported
        this.HITBOX_DIMENSIONS = {
            'octane': { length: 118.0074, width: 84.1994, height: 36.1591, offsetX: 13.87566, offsetZ: 20.75499 },
            'fennec': { length: 118.0074, width: 84.1994, height: 36.1591, offsetX: 13.87566, offsetZ: 20.75499 }, // Same as Octane hitbox
            'dominus': { length: 127.9268, width: 83.2800, height: 31.3000, offsetX: 9.0, offsetZ: 15.75 },
            'breakout': { length: 131.4924, width: 80.521, height: 30.3, offsetX: 12.5, offsetZ: 11.75 },
            'mantis': { length: 128.8198, width: 84.6704, height: 29.3944, offsetX: 9.00857, offsetZ: 12.0942 }, // Plank hitbox
            'x-devil': { length: 127.0192, width: 82.1879, height: 34.1591, offsetX: 13.87566, offsetZ: 20.75499 }, // Hybrid hitbox
            'merc': { length: 120.72, width: 76.71, height: 41.66, offsetX: 11.37566, offsetZ: 21.504988 },
        };

        // NOTE: Preload is now deferred - call preloadModelsForReplay() after knowing which cars are needed
    }

    /**
     * Preload car models for a specific replay based on players' cars.
     * Only loads models actually used in the replay.
     * @param {Array<{carName: string, hitboxType: string}>} players - Array of player entities with carName and hitboxType
     * @returns {Promise<void>}
     */
    async preloadModelsForReplay(players) {
        if (this._preloadStarted) {
            console.log('[CarModelLoader] Preload already started, returning existing promise');
            return this.preloadReady;
        }
        this._preloadStarted = true;

        // Determine unique model types needed for this replay
        const neededModelTypes = new Set();
        for (const player of players) {
            const modelType = this.getModelTypeForCar(player.carName, player.hitboxType);
            neededModelTypes.add(modelType);
        }

        const modelTypesArray = [...neededModelTypes];
        console.log(`[CarModelLoader] Preloading ${modelTypesArray.length} car models for replay: ${modelTypesArray.join(', ')}`);

        this.preloadReady = this._preloadModels(modelTypesArray);
        return this.preloadReady;
    }

    /**
     * Preload specific car models. Returns a promise that resolves when all models are loaded.
     * @param {string[]} modelTypes - Array of model types to preload (e.g., ['octane', 'fennec'])
     * @returns {Promise<void>}
     */
    async _preloadModels(modelTypes) {
        for (const modelType of modelTypes) {
            try {
                await this.loadModel(modelType);
                console.log(`✓ Preloaded car model: ${modelType}`);
            } catch (error) {
                console.warn(`⚠️ Failed to preload ${modelType}:`, error.message);
            }
        }
    }

    /**
     * Preload ALL car models (legacy method for backwards compatibility).
     * Prefer preloadModelsForReplay() for better performance.
     * @returns {Promise<void>}
     */
    async preloadAllModels() {
        const allModelTypes = ['octane', 'fennec', 'dominus', 'breakout', 'merc', 'mantis', 'x-devil'];
        console.log('[CarModelLoader] Preloading ALL car models (legacy mode)');
        this._preloadStarted = true;
        this.preloadReady = this._preloadModels(allModelTypes);
        return this.preloadReady;
    }

    /**
     * Wait for all preloaded models to be ready
     * @returns {Promise<void>}
     */
    async waitForPreload() {
        return this.preloadReady;
    }

    /**
     * Load a car model and its chassis texture
     * @param {string} modelType - 'octane', 'dominus', or 'fennec'
     * @returns {Promise<{model: THREE.Group, chassisTexture: THREE.Texture}>}
     */
    async loadModel(modelType) {
        // Return cached if available
        if (this.modelCache.has(modelType)) {
            return this.modelCache.get(modelType);
        }

        // Return existing loading promise if in progress
        if (this.loadingPromises.has(modelType)) {
            return this.loadingPromises.get(modelType);
        }

        // Start loading
        const loadPromise = this._loadModelInternal(modelType);
        this.loadingPromises.set(modelType, loadPromise);

        try {
            const result = await loadPromise;
            this.modelCache.set(modelType, result);
            return result;
        } finally {
            this.loadingPromises.delete(modelType);
        }
    }

    async _loadModelInternal(modelType) {
        const basePath = `/models/cars/${modelType}`;
        const config = this.modelConfig[modelType] || { format: 'glb', file: `${modelType}.glb`, scale: 100.0, wheelSockets: true, wheelModel: 'Wheel_Boog.glb' };

        let model;
        let chassisTexture = null;
        let wheelModel = null;

        if (config.format === 'glb') {
            // Load GLB file (textures are embedded)
            const glbPath = `${basePath}/${config.file}`;

            // If model uses wheel sockets, load wheel model in parallel
            if (config.wheelSockets && config.wheelModel) {
                [model, wheelModel] = await Promise.all([
                    this._loadGLB(glbPath),
                    this.loadWheelModel(config.wheelModel)
                ]);
                console.log(`✓ Loaded GLB model with separate wheels: ${glbPath}`);
            } else {
                model = await this._loadGLB(glbPath);
                console.log(`✓ Loaded GLB model: ${glbPath}`);
            }
        } else {
            // Load FBX file with external texture
            const fbxPath = `${basePath}/${config.file}.fbx`;
            const modelName = config.file;
            const texturePath = `${basePath}/${modelName}_engine.png`;

            // Load FBX and texture in parallel
            [model, chassisTexture] = await Promise.all([
                this._loadFBX(fbxPath),
                this._loadTexture(texturePath).catch((err) => {
                    console.warn(`⚠️ Could not load texture ${texturePath}:`, err.message);
                    return null;
                })
            ]);
        }

        // Process the model materials
        this._processModelMaterials(model, chassisTexture, modelType, config.format);

        // Calculate proper scale and offset based on bounding box
        const scaleInfo = this._calculateModelScale(model, modelType, config.scale);
        model.userData.scaleInfo = scaleInfo;
        model.userData.format = config.format;

        // Store wheel socket configuration
        if (config.wheelSockets) {
            model.userData.wheelSockets = true;
            model.userData.wheelModelName = config.wheelModel;

            // Find and store wheel socket objects
            const wheelSockets = this._findWheelSockets(model);
            model.userData.wheelSocketObjects = wheelSockets;
            console.log(`🔌 Found ${Object.keys(wheelSockets).length} wheel sockets`);
        }

        return { model, chassisTexture, wheelModel };
    }

    /**
     * Find wheel socket empty objects in the model
     * Expected names: Wheel_BL, Wheel_BR, Wheel_FL, Wheel_FR
     * (BackLeft, BackRight, FrontLeft, FrontRight)
     * @param {THREE.Group} model - The loaded model
     * @returns {Object} Map of socket name to socket object
     */
    _findWheelSockets(model) {
        const sockets = {};
        const expectedSockets = ['Wheel_BL', 'Wheel_BR', 'Wheel_FL', 'Wheel_FR'];

        console.log('🔍 Searching for wheel sockets...');

        model.traverse((child) => {
            // Check both exact match and case-insensitive
            const name = child.name;
            const nameLower = name.toLowerCase();

            for (const socketName of expectedSockets) {
                if (name === socketName || nameLower === socketName.toLowerCase()) {
                    sockets[socketName] = child;
                    console.log(`   Found socket: "${name}" at position (${child.position.x.toFixed(2)}, ${child.position.y.toFixed(2)}, ${child.position.z.toFixed(2)})`);
                }
            }
        });

        // Log any missing sockets
        for (const socketName of expectedSockets) {
            if (!sockets[socketName]) {
                console.warn(`   ⚠️ Missing wheel socket: ${socketName}`);
            }
        }

        // If no sockets found, log all objects in model for debugging
        if (Object.keys(sockets).length === 0) {
            console.warn('⚠️ No wheel sockets found! Listing all objects:');
            model.traverse((child) => {
                console.log(`   - "${child.name}" (${child.type})`);
            });
        }

        return sockets;
    }

    /**
     * Calculate the proper scale factor to match Rocket League hitbox dimensions
     * @param {THREE.Group} model - The loaded model
     * @param {string} modelType - 'octane', 'dominus', or 'fennec'
     * @param {number|null} overrideScale - If provided, use this scale directly (skip auto-calc)
     * @returns {{ scale: number, offsetX: number, offsetY: number }}
     */
    _calculateModelScale(model, modelType, overrideScale = null) {
        const box = new THREE.Box3().setFromObject(model);
        const size = new THREE.Vector3();
        box.getSize(size);

        console.log(`📐 ${modelType.toUpperCase()} model dimensions (raw):`);
        console.log(`   Size: X=${size.x.toFixed(2)}, Y=${size.y.toFixed(2)}, Z=${size.z.toFixed(2)}`);
        console.log(`   Min Y: ${box.min.y.toFixed(2)}, Max Y: ${box.max.y.toFixed(2)}`);

        const hitbox = this.HITBOX_DIMENSIONS[modelType] || this.HITBOX_DIMENSIONS['octane'];
        let finalScale;

        if (overrideScale !== null) {
            // Use override scale directly (model is already at correct RL scale)
            finalScale = overrideScale;
            console.log(`   Using override scale: ${finalScale}`);
        } else {
            // Auto-calculate scale based on hitbox dimensions
            // Assuming Z is length (forward), X is width (sideways), Y is height
            const fbxLength = size.z;

            // Calculate scale to match RL dimensions (in Unreal Units)
            const targetVisualLength = hitbox.length * 1.0;
            const scaleToRL = targetVisualLength / fbxLength;

            // Apply correction factor for FBX models
            finalScale = scaleToRL * 0.55;

            console.log(`   Target RL: ${hitbox.length} x ${hitbox.width} x ${hitbox.height} uu`);
            console.log(`   Scale to RL: ${scaleToRL.toFixed(4)}, Final scale: ${finalScale.toFixed(6)}`);
        }

        return { scale: finalScale };
    }

    _loadFBX(path) {
        return new Promise((resolve, reject) => {
            this.fbxLoader.load(
                path,
                (fbx) => resolve(fbx),
                undefined,
                (error) => reject(new Error(`Failed to load FBX: ${path} - ${error.message}`))
            );
        });
    }

    _loadGLB(path) {
        return new Promise((resolve, reject) => {
            this.gltfLoader.load(
                path,
                (gltf) => {
                    // GLTFLoader returns { scene, animations, ... }
                    // We need the scene which contains all the meshes
                    resolve(gltf.scene);
                },
                undefined,
                (error) => reject(new Error(`Failed to load GLB: ${path} - ${error.message}`))
            );
        });
    }

    /**
     * Load a wheel model for cars with separate wheels
     * @param {string} wheelModelName - The wheel model filename (e.g., 'Wheel_Boog.glb')
     * @returns {Promise<THREE.Group>}
     */
    async loadWheelModel(wheelModelName) {
        // Return cached if available
        if (this.wheelModelCache.has(wheelModelName)) {
            return this.wheelModelCache.get(wheelModelName);
        }

        // Return existing loading promise if in progress
        if (this.wheelLoadingPromises.has(wheelModelName)) {
            return this.wheelLoadingPromises.get(wheelModelName);
        }

        // Start loading
        const loadPromise = this._loadGLB(`/models/wheels/${wheelModelName}`);
        this.wheelLoadingPromises.set(wheelModelName, loadPromise);

        try {
            const wheelModel = await loadPromise;
            console.log(`✓ Loaded wheel model: ${wheelModelName}`);

            // Process wheel materials
            wheelModel.traverse((child) => {
                if (child.isMesh) {
                    child.castShadow = true;
                    child.receiveShadow = true;
                }
            });

            this.wheelModelCache.set(wheelModelName, wheelModel);
            return wheelModel;
        } catch (error) {
            console.error(`Failed to load wheel model ${wheelModelName}:`, error);
            throw error;
        } finally {
            this.wheelLoadingPromises.delete(wheelModelName);
        }
    }

    _loadTexture(path) {
        return new Promise((resolve, reject) => {
            this.textureLoader.load(
                path,
                (texture) => {
                    texture.flipY = false; // FBX typically doesn't need Y flip
                    texture.colorSpace = THREE.SRGBColorSpace;
                    resolve(texture);
                },
                undefined,
                (error) => reject(new Error(`Failed to load texture: ${path}`))
            );
        });
    }

    _processModelMaterials(model, chassisTexture, modelType, format = 'fbx') {
        console.log(`📦 Processing materials for ${modelType} (${format}):`);

        // Material names that are body/paint (should be shiny metallic and receive team color)
        // Only match explicit body/paint suffixes, NOT car model names (to avoid matching chassis)
        const bodyMaterialNames = ['body', 'paint'];

        // Remove any lights that were imported with the model
        const lightsToRemove = [];
        model.traverse((child) => {
            if (child.isLight) {
                lightsToRemove.push(child);
                console.log(`  🔦 Removing imported light: "${child.name || child.type}"`);
            }
        });
        lightsToRemove.forEach(light => {
            if (light.parent) {
                light.parent.remove(light);
            }
        });

        // Traverse the model and process materials
        model.traverse((child) => {
            if (child.isMesh) {
                console.log(`  Mesh: "${child.name}"`);
                const materials = Array.isArray(child.material) ? child.material : [child.material];

                materials.forEach((mat, index) => {
                    console.log(`    [${index}] Material: "${mat.name}" - Color: #${mat.color?.getHexString() || 'none'}`);

                    const matName = (mat.name || '').toLowerCase();
                    const meshName = (child.name || '').toLowerCase();
                    const isBodyMaterial = bodyMaterialNames.some(name => matName.includes(name) || meshName.includes(name));

                    if (format === 'glb') {
                        // GLB models already have MeshStandardMaterial from GLTF
                        // Just log info and mark body materials for team coloring
                        if (mat.isMeshStandardMaterial || mat.isMeshPhysicalMaterial) {
                            console.log(`      → GLB material (keeping as-is): metalness=${mat.metalness?.toFixed(2)}, roughness=${mat.roughness?.toFixed(2)}`);
                            // Store original color for reference
                            mat.userData.originalColor = mat.color?.clone();
                            mat.userData.isBodyMaterial = isBodyMaterial;
                        }
                    } else {
                        // FBX: Convert to MeshStandardMaterial for better lighting
                        if (mat.isMeshPhongMaterial || mat.isMeshLambertMaterial || mat.isMeshBasicMaterial) {
                            let stdMat;

                            if (isBodyMaterial) {
                                // Body/chassis: shiny metallic
                                stdMat = new THREE.MeshStandardMaterial({
                                    color: mat.color,
                                    map: mat.map,
                                    metalness: 0.8,
                                    roughness: 0.15
                                });
                                console.log(`      → Body material: shiny metallic`);
                            } else {
                                // Other parts (wheels, engine, etc.): keep original look
                                stdMat = new THREE.MeshStandardMaterial({
                                    color: mat.color,
                                    map: mat.map,
                                    metalness: 0.1,
                                    roughness: 0.6
                                });
                                console.log(`      → Non-body material: matte`);
                            }

                            stdMat.name = mat.name; // Preserve the name!

                            if (Array.isArray(child.material)) {
                                child.material[index] = stdMat;
                            } else {
                                child.material = stdMat;
                            }
                        }
                    }
                });
            }
        });

        // Apply chassis texture if available (FBX only)
        if (chassisTexture) {
            console.log(`  ✓ Chassis texture loaded for ${modelType}`);
        } else if (format === 'fbx') {
            console.log(`  ⚠️ No chassis texture for ${modelType}`);
        }
    }

    /**
     * Get the model type for a given car name and hitbox type
     * @param {string} carName - The car name (e.g., "Fennec", "Octane")
     * @param {string} hitboxType - The hitbox type as fallback
     * @returns {string} The model folder name
     */
    getModelTypeForCar(carName, hitboxType) {
        // First try exact car name match
        if (carName && this.carNameToModel[carName]) {
            return this.carNameToModel[carName];
        }
        // Fallback to hitbox type mapping
        return this.hitboxToModel[hitboxType] || 'octane';
    }

    /**
     * @deprecated Use getModelTypeForCar instead
     */
    getModelTypeForHitbox(hitboxType) {
        return this.hitboxToModel[hitboxType] || 'octane';
    }

    /**
     * Create a car mesh for a specific hitbox type and team
     * @param {string} hitboxType - 'Octane', 'Dominus', etc.
     * @param {number} team - 0 for blue, 1 for orange
     * @returns {Promise<THREE.Group|null>} The car mesh or null if not loaded
     */
    async createCarMesh(hitboxType, team = 0) {
        const modelType = this.getModelTypeForHitbox(hitboxType);

        try {
            const cached = await this.loadModel(modelType);
            if (!cached || !cached.model) {
                console.warn(`No cached model for ${modelType}`);
                return null;
            }

            const format = cached.model.userData.format || 'fbx';

            // Clone the model - use SkeletonUtils for GLB to properly clone materials/textures
            let modelClone;
            if (format === 'glb') {
                modelClone = skeletonClone(cached.model);
                // Deep clone materials for GLB to allow independent team colors
                modelClone.traverse((child) => {
                    if (child.isMesh) {
                        if (Array.isArray(child.material)) {
                            child.material = child.material.map(mat => mat.clone());
                        } else if (child.material) {
                            child.material = child.material.clone();
                        }
                    }
                });
            } else {
                modelClone = cached.model.clone();
            }

            this.applyTeamColor(modelClone, team);

            // Create a container Group that will receive position/rotation from replay
            const carContainer = new THREE.Group();

            // Get scale info (always present from _calculateModelScale)
            const scaleInfo = cached.model.userData.scaleInfo;
            if (scaleInfo) {
                modelClone.scale.setScalar(scaleInfo.scale);
            }

            // Add model to container
            carContainer.add(modelClone);

            // Enable shadow casting on all meshes
            modelClone.traverse((child) => {
                if (child.isMesh) {
                    child.castShadow = true;
                }
            });

            // Store metadata on container
            carContainer.userData.modelType = modelType;
            carContainer.userData.hitboxType = hitboxType;
            carContainer.userData.team = team;
            carContainer.userData.isFBXModel = format === 'fbx';
            carContainer.userData.isGLBModel = format === 'glb';

            return carContainer;
        } catch (error) {
            console.error(`Failed to create car mesh for ${hitboxType}:`, error);
            return null;
        }
    }

    /**
     * Apply team color to the car body material
     * @param {THREE.Group} carMesh - The car mesh group
     * @param {number} team - 0 for blue, 1 for orange
     */
    applyTeamColor(carMesh, team) {
        const color = team === 0 ? this.TEAM_COLORS.blue : this.TEAM_COLORS.orange;

        // Material/mesh names that indicate body parts (should receive team color)
        // Only match explicit body/paint suffixes, NOT car model names (to avoid matching chassis)
        const bodyMaterialNames = ['body', 'paint'];

        let bodyMaterialFound = false;

        // Log all meshes for debugging
        console.log('🔍 Analyzing car meshes for team coloring:');
        carMesh.traverse((child) => {
            if (child.isMesh) {
                const materials = Array.isArray(child.material) ? child.material : [child.material];
                console.log(`  Mesh: "${child.name}" with ${materials.length} material(s)`);
                materials.forEach((mat, idx) => {
                    console.log(`    [${idx}] Material: "${mat.name}", isBodyMaterial: ${mat.userData?.isBodyMaterial}`);
                });
            }
        });

        carMesh.traverse((child) => {
            if (child.isMesh) {
                const materials = Array.isArray(child.material) ? child.material : [child.material];

                materials.forEach((mat, index) => {
                    const matName = (mat.name || '').toLowerCase();
                    const meshName = (child.name || '').toLowerCase();

                    // Check if this material/mesh name matches body parts
                    // Also check userData.isBodyMaterial (set during GLB processing)
                    const isBodyMaterial = mat.userData?.isBodyMaterial ||
                        bodyMaterialNames.some(name => matName.includes(name) || meshName.includes(name));

                    console.log(`    Checking "${mat.name}" on "${child.name}": isBody=${isBodyMaterial}`);

                    if (isBodyMaterial) {
                        bodyMaterialFound = true;
                        const clonedMat = mat.clone();
                        clonedMat.color = color.clone();
                        clonedMat.metalness = 0.39;
                        clonedMat.roughness = 0.47;
                        // Preserve userData
                        clonedMat.userData = { ...mat.userData };

                        if (Array.isArray(child.material)) {
                            child.material[index] = clonedMat;
                        } else {
                            child.material = clonedMat;
                        }

                        console.log(`🎨 Applied team color to: "${mat.name}" on mesh "${child.name}" (index ${index})`);
                    }
                });
            }
        });

        if (!bodyMaterialFound) {
            console.warn(`⚠️ No body material found for team coloring! Check material names.`);
        }
    }

    /**
     * Update the team color of an existing car mesh
     * @param {THREE.Group} carMesh - The car mesh group
     * @param {number} team - 0 for blue, 1 for orange
     */
    updateTeamColor(carMesh, team) {
        this.applyTeamColor(carMesh, team);
    }

    /**
     * Check if a model is loaded and ready
     * @param {string} carName - The car name
     * @param {string} hitboxType - The hitbox type as fallback
     * @returns {boolean}
     */
    isModelReady(carName, hitboxType) {
        const modelType = this.getModelTypeForCar(carName, hitboxType);
        return this.modelCache.has(modelType);
    }

    /**
     * Get a synchronous car mesh if available (returns null if not loaded)
     * @param {string} carName - The car name
     * @param {string} hitboxType - The hitbox type as fallback
     * @param {number} team - 0 for blue, 1 for orange
     * @returns {THREE.Group|null}
     */
    getCarMeshSync(carName, hitboxType, team = 0) {
        const modelType = this.getModelTypeForCar(carName, hitboxType);
        const cached = this.modelCache.get(modelType);

        if (!cached || !cached.model) {
            return null;
        }

        const format = cached.model.userData.format || 'fbx';
        const hasWheelSockets = cached.model.userData.wheelSockets;

        // Clone the model - use SkeletonUtils for GLB to properly clone materials/textures
        let modelClone;
        if (format === 'glb') {
            modelClone = skeletonClone(cached.model);
            // Deep clone materials for GLB to allow independent team colors
            modelClone.traverse((child) => {
                if (child.isMesh) {
                    if (Array.isArray(child.material)) {
                        child.material = child.material.map(mat => mat.clone());
                    } else if (child.material) {
                        child.material = child.material.clone();
                    }
                }
            });
        } else {
            modelClone = cached.model.clone();
        }

        this.applyTeamColor(modelClone, team);

        // Create a container Group that will receive position/rotation from replay
        const carContainer = new THREE.Group();

        // Get scale info (always present from _calculateModelScale)
        const scaleInfo = cached.model.userData.scaleInfo;
        if (scaleInfo) {
            modelClone.scale.setScalar(scaleInfo.scale);
        }

        // Add model to container
        carContainer.add(modelClone);

        // Enable shadow casting on all meshes
        modelClone.traverse((child) => {
            if (child.isMesh) {
                child.castShadow = true;
            }
        });

        // Store metadata on container
        carContainer.userData.modelType = modelType;
        carContainer.userData.carName = carName;
        carContainer.userData.hitboxType = hitboxType;
        carContainer.userData.team = team;
        carContainer.userData.isFBXModel = format === 'fbx';
        carContainer.userData.isGLBModel = format === 'glb';
        carContainer.userData.hasWheelSockets = hasWheelSockets;

        // Handle wheel attachment based on model type
        if (hasWheelSockets) {
            // New method: attach separate wheel models to socket empty objects
            carContainer.userData.wheels = this._attachWheelsToSockets(modelClone, cached.wheelModel);
        } else {
            // Old method: find embedded wheel meshes
            carContainer.userData.wheels = this._findWheelMeshes(modelClone);
        }

        return carContainer;
    }

    /**
     * Attach wheel models to socket empty objects in the car model
     * Socket naming: Wheel_BL (BackLeft), Wheel_BR (BackRight), Wheel_FL (FrontLeft), Wheel_FR (FrontRight)
     * @param {THREE.Group} carModel - The cloned car model
     * @param {THREE.Group} wheelModelTemplate - The wheel model to clone and attach
     * @returns {Array<{mesh: THREE.Object3D, steeringPivot: THREE.Object3D|null, side: string, position: string}>}
     */
    _attachWheelsToSockets(carModel, wheelModelTemplate) {
        const wheels = [];

        // Socket to wheel info mapping
        const socketMapping = {
            'Wheel_FL': { side: 'left', position: 'front' },
            'Wheel_FR': { side: 'right', position: 'front' },
            'Wheel_BL': { side: 'left', position: 'rear' },
            'Wheel_BR': { side: 'right', position: 'rear' }
        };

        if (!wheelModelTemplate) {
            console.warn('⚠️ No wheel model template available for socket attachment');
            return wheels;
        }

        console.log('🔧 Attaching wheels to sockets...');

        // Find sockets in the cloned model
        const sockets = {};
        carModel.traverse((child) => {
            const name = child.name;
            if (socketMapping[name]) {
                sockets[name] = child;
            }
        });

        // Attach wheels to each socket
        for (const [socketName, socketInfo] of Object.entries(socketMapping)) {
            const socket = sockets[socketName];
            if (!socket) {
                console.warn(`   ⚠️ Socket not found: ${socketName}`);
                continue;
            }

            // Clone the wheel model
            const wheelClone = skeletonClone(wheelModelTemplate);

            // Deep clone materials
            wheelClone.traverse((child) => {
                if (child.isMesh) {
                    if (Array.isArray(child.material)) {
                        child.material = child.material.map(mat => mat.clone());
                    } else if (child.material) {
                        child.material = child.material.clone();
                    }
                    child.castShadow = true;
                }
            });

            // Reset wheel position/rotation (socket already has correct transform)
            wheelClone.position.set(0, 0, 0);
            wheelClone.rotation.set(0, 0, 0);

            // Attach wheel to socket
            socket.add(wheelClone);

            console.log(`   ✓ Attached wheel to ${socketName} (${socketInfo.position} ${socketInfo.side})`);

            // Store wheel info for animation
            // The socket itself acts as the steering pivot for front wheels
            wheels.push({
                mesh: wheelClone,
                steeringPivot: socketInfo.position === 'front' ? socket : null,
                side: socketInfo.side,
                position: socketInfo.position,
                socket: socket  // Keep reference to socket for debugging
            });
        }

        console.log(`✓ Attached ${wheels.length} wheels to sockets`);
        return wheels;
    }

    /**
     * Find wheel meshes in the car model
     * New naming convention with pivot hierarchy:
     * - Wheel_XX_Z = steering pivot (rotates around Z for steering)
     * - Wheel_XX_Y = wheel mesh (rotates around Y for rolling)
     * @param {THREE.Group} model - The FBX model
     * @returns {Array<{mesh: THREE.Object3D, steeringPivot: THREE.Object3D|null, side: string, position: string}>}
     */
    _findWheelMeshes(model) {
        const wheels = [];

        // Wheel positions mapping
        const wheelPositions = {
            'fl': { side: 'left', position: 'front' },
            'fr': { side: 'right', position: 'front' },
            'rl': { side: 'left', position: 'rear' },
            'rr': { side: 'right', position: 'rear' },
        };

        console.log('🔍 Searching for wheels in model...');

        // Find all wheel components
        const wheelParts = {};

        model.traverse((child) => {
            const name = child.name.toLowerCase();

            // Match pattern: wheel_XX_Y or wheel_XX_Z
            const match = name.match(/^wheel_(fl|fr|rl|rr)_(y|z)$/);
            if (match) {
                const pos = match[1]; // fl, fr, rl, rr
                const axis = match[2]; // y or z

                if (!wheelParts[pos]) {
                    wheelParts[pos] = {};
                }
                wheelParts[pos][axis] = child;

                console.log(`   Found: "${child.name}" (${axis === 'y' ? 'wheel mesh' : 'steering pivot'})`);
            }
        });

        // Build wheel objects
        for (const [pos, parts] of Object.entries(wheelParts)) {
            const info = wheelPositions[pos];
            if (!info) continue;

            const wheelMesh = parts.y; // _Y is the wheel that spins
            const steeringPivot = parts.z; // _Z is the steering pivot (front wheels only)

            if (wheelMesh) {
                // Fix front-right wheel orientation (rotate 180° on Z axis)
                if (pos === 'fr') {
                    wheelMesh.rotation.z += Math.PI;
                    console.log(`   Fixed FR wheel orientation (rotation.z += PI)`);
                }

                wheels.push({
                    mesh: wheelMesh,
                    steeringPivot: info.position === 'front' ? steeringPivot : null,
                    side: info.side,
                    position: info.position
                });
                console.log(`🛞 Wheel ${pos.toUpperCase()}: mesh="${wheelMesh.name}"${steeringPivot && info.position === 'front' ? `, steering="${steeringPivot.name}"` : ''}`);
            }
        }

        if (wheels.length === 0) {
            console.warn('⚠️ No wheel meshes found. Expected: Wheel_FL_Y, Wheel_FR_Y, etc.');
            console.warn('   Listing all objects in model:');
            model.traverse((child) => {
                console.log(`   - "${child.name}" (${child.type})`);
            });
        } else {
            console.log(`✓ Found ${wheels.length} wheels`);
        }

        return wheels;
    }

    dispose() {
        // Dispose all cached car models and textures
        this.modelCache.forEach(({ model, chassisTexture }) => {
            model.traverse((child) => {
                if (child.isMesh) {
                    if (child.geometry) child.geometry.dispose();
                    const materials = Array.isArray(child.material) ? child.material : [child.material];
                    materials.forEach(mat => mat.dispose());
                }
            });
            if (chassisTexture) chassisTexture.dispose();
        });
        this.modelCache.clear();

        // Dispose all cached wheel models
        this.wheelModelCache.forEach((wheelModel) => {
            wheelModel.traverse((child) => {
                if (child.isMesh) {
                    if (child.geometry) child.geometry.dispose();
                    const materials = Array.isArray(child.material) ? child.material : [child.material];
                    materials.forEach(mat => mat.dispose());
                }
            });
        });
        this.wheelModelCache.clear();
    }
}
