import { useRef, useEffect, useState, useMemo, useCallback } from 'react';
import * as THREE from 'three';
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import { Play, Pause, RotateCw, Mouse } from 'lucide-react';

interface OctaneHeroProps {
  className?: string;
}

// Colors for neon rays
const rayColors = [
  { color: '#a855f7', glow: '#a855f7' }, // violet
  { color: '#c084fc', glow: '#c084fc' }, // violet light
  { color: '#3b82f6', glow: '#3b82f6' }, // blue
  { color: '#60a5fa', glow: '#60a5fa' }, // blue light
  { color: '#06b6d4', glow: '#06b6d4' }, // cyan
  { color: '#22d3ee', glow: '#22d3ee' }, // cyan light
];

interface RaySegment {
  angle: number;
  distance: number;
  width: number;
  height: number;
  color: string;
  glow: string;
}

function generateRaySegments(count: number): RaySegment[] {
  const segments: RaySegment[] = [];

  for (let i = 0; i < count; i++) {
    const colorSet = rayColors[Math.floor(Math.random() * rayColors.length)];
    // Use exponential distribution to favor shorter segments
    const lengthRandom = Math.pow(Math.random(), 2); // Squared = more small values
    segments.push({
      angle: Math.random() * 360,
      distance: 140 + Math.random() * 160, // 140-300px from center
      width: 15 + lengthRandom * 140, // 15-155px, mostly short
      height: 2 + Math.random() * 4, // 2-6px thickness
      color: colorSet.color,
      glow: colorSet.glow,
    });
  }

  return segments;
}

// Easing function for smooth animations
const easeOutCubic = (t: number) => 1 - Math.pow(1 - t, 3);

// Animation duration in seconds
const ANIMATION_DURATION = 3.0;

// Playback speed options
const SPEED_OPTIONS = [0.5, 1, 2];

export function OctaneHero({ className = '' }: OctaneHeroProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Player state
  const [isPlaying, setIsPlaying] = useState(true);
  const [progress, setProgress] = useState(0);
  const [speed, setSpeed] = useState(1);
  const [isDragging, setIsDragging] = useState(false);
  const [autoRotate, setAutoRotate] = useState(true);
  const [showDragHint, setShowDragHint] = useState(true);

  // Refs for animation
  const progressRef = useRef(0);
  const isPlayingRef = useRef(true);
  const speedRef = useRef(1);
  const lastTimeRef = useRef(0);
  const wasPlayingBeforeDrag = useRef(true);
  const controlsRef = useRef<InstanceType<typeof OrbitControls> | null>(null);
  const isFirstPlayRef = useRef(true); // Track first playthrough

  // Generate random ray segments once on mount
  const raySegments = useMemo(() => generateRaySegments(24), []);

  // Sync refs with state
  useEffect(() => {
    isPlayingRef.current = isPlaying;
  }, [isPlaying]);

  useEffect(() => {
    speedRef.current = speed;
  }, [speed]);

  useEffect(() => {
    if (controlsRef.current) {
      controlsRef.current.autoRotate = autoRotate;
    }
  }, [autoRotate]);


  // Handle progress changes from slider
  const handleProgressChange = useCallback((newProgress: number) => {
    progressRef.current = newProgress;
    setProgress(newProgress);
  }, []);

  // Timeline ref for global mouse move calculations
  const timelineRef = useRef<HTMLDivElement>(null);

  // Timeline drag handlers
  const handleTimelineMouseDown = useCallback((e: React.MouseEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.stopPropagation();
    wasPlayingBeforeDrag.current = isPlaying;
    setIsDragging(true);
    setIsPlaying(false);
    const rect = e.currentTarget.getBoundingClientRect();
    const x = (e.clientX - rect.left) / rect.width;
    handleProgressChange(Math.max(0, Math.min(1, x)));
  }, [handleProgressChange, isPlaying]);

  // Global mouse move/up handlers for smooth dragging
  useEffect(() => {
    if (!isDragging) return;

    const handleGlobalMouseMove = (e: MouseEvent) => {
      e.preventDefault();
      if (!timelineRef.current) return;
      const rect = timelineRef.current.getBoundingClientRect();
      const x = (e.clientX - rect.left) / rect.width;
      handleProgressChange(Math.max(0, Math.min(1, x)));
    };

    const handleGlobalMouseUp = () => {
      setIsDragging(false);
      setIsPlaying(wasPlayingBeforeDrag.current);
    };

    window.addEventListener('mousemove', handleGlobalMouseMove);
    window.addEventListener('mouseup', handleGlobalMouseUp);

    return () => {
      window.removeEventListener('mousemove', handleGlobalMouseMove);
      window.removeEventListener('mouseup', handleGlobalMouseUp);
    };
  }, [isDragging, handleProgressChange]);

  useEffect(() => {
    if (!containerRef.current) return;

    const container = containerRef.current;
    const width = container.clientWidth;
    const height = container.clientHeight;

    // Scene setup
    const scene = new THREE.Scene();

    // Camera setup - further back to show full scene with ball and car
    const camera = new THREE.PerspectiveCamera(35, width / height, 0.1, 1000);
    camera.position.set(14, 7, 14);
    camera.lookAt(0, 2.5, 0);

    // Renderer setup
    const renderer = new THREE.WebGLRenderer({
      antialias: true,
      alpha: true,
      powerPreference: 'high-performance',
    });
    renderer.setSize(width, height);
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    renderer.outputColorSpace = THREE.SRGBColorSpace;
    renderer.toneMapping = THREE.ACESFilmicToneMapping;
    renderer.toneMappingExposure = 1.44;
    renderer.shadowMap.enabled = true;
    renderer.shadowMap.type = THREE.PCFSoftShadowMap;
    container.appendChild(renderer.domElement);

    // Orbit controls - horizontal only
    const controls = new OrbitControls(camera, renderer.domElement);
    controls.enableDamping = true;
    controls.dampingFactor = 0.05;
    controls.enableZoom = false;
    controls.enablePan = false;
    controls.minPolarAngle = Math.PI / 3;
    controls.maxPolarAngle = Math.PI / 2.2;
    controls.autoRotate = true;
    controls.autoRotateSpeed = 1.0;
    controls.target.set(0, 2.5, 0);
    controlsRef.current = controls;

    // Disable auto-rotate and hide drag hint when user manually interacts
    controls.addEventListener('start', () => {
      setAutoRotate(false);
      setShowDragHint(false);
    });

    // Create studio environment map
    const pmremGenerator = new THREE.PMREMGenerator(renderer);
    pmremGenerator.compileEquirectangularShader();

    const envScene = new THREE.Scene();
    envScene.background = new THREE.Color(0x888888);

    // Bright studio-style environment
    const envGeometry = new THREE.SphereGeometry(50, 64, 32);
    const envMaterial = new THREE.ShaderMaterial({
      side: THREE.BackSide,
      uniforms: {
        topColor: { value: new THREE.Color(0xffffff) },
        bottomColor: { value: new THREE.Color(0x666666) },
      },
      vertexShader: `
        varying vec3 vWorldPosition;
        void main() {
          vec4 worldPosition = modelMatrix * vec4(position, 1.0);
          vWorldPosition = worldPosition.xyz;
          gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
        }
      `,
      fragmentShader: `
        uniform vec3 topColor;
        uniform vec3 bottomColor;
        varying vec3 vWorldPosition;
        void main() {
          float h = normalize(vWorldPosition).y * 0.5 + 0.5;
          gl_FragColor = vec4(mix(bottomColor, topColor, h), 1.0);
        }
      `,
    });
    const envMesh = new THREE.Mesh(envGeometry, envMaterial);
    envScene.add(envMesh);

    // Add bright light spots for reflections
    const addLightSpot = (color: number, x: number, y: number, z: number, size: number) => {
      const geom = new THREE.SphereGeometry(size, 16, 16);
      const mat = new THREE.MeshBasicMaterial({ color });
      const mesh = new THREE.Mesh(geom, mat);
      mesh.position.set(x, y, z);
      envScene.add(mesh);
    };

    addLightSpot(0xffffff, 0, 30, 0, 8);
    addLightSpot(0xffffff, 20, 15, 20, 5);
    addLightSpot(0xffffff, -20, 15, -20, 5);
    addLightSpot(0x9966ff, -25, 10, 10, 4);
    addLightSpot(0x6699ff, 25, 10, -10, 4);

    const envMap = pmremGenerator.fromScene(envScene, 0.04).texture;
    scene.environment = envMap;

    // Ambient light - reduced
    const ambientLight = new THREE.AmbientLight(0xffffff, 0.25);
    scene.add(ambientLight);

    // Key light from top - reduced
    const keyLight = new THREE.DirectionalLight(0xffffff, 1.0);
    keyLight.position.set(3, 8, 5);
    keyLight.castShadow = true;
    keyLight.shadow.mapSize.width = 2048;
    keyLight.shadow.mapSize.height = 2048;
    keyLight.shadow.camera.near = 1;
    keyLight.shadow.camera.far = 20;
    keyLight.shadow.camera.left = -5;
    keyLight.shadow.camera.right = 5;
    keyLight.shadow.camera.top = 5;
    keyLight.shadow.camera.bottom = -5;
    keyLight.shadow.bias = -0.001;
    keyLight.shadow.radius = 4;
    scene.add(keyLight);

    // Fill light - reduced
    const fillLight = new THREE.DirectionalLight(0xffffff, 0.5);
    fillLight.position.set(-5, 3, 0);
    scene.add(fillLight);

    // Violet accent
    const violetLight = new THREE.DirectionalLight(0x8b5cf6, 0.4);
    violetLight.position.set(-4, 2, -3);
    scene.add(violetLight);

    // Blue accent
    const blueLight = new THREE.DirectionalLight(0x3b82f6, 0.4);
    blueLight.position.set(4, 2, 3);
    scene.add(blueLight);

    // Rim light - reduced
    const rimLight = new THREE.DirectionalLight(0xffffff, 0.6);
    rimLight.position.set(0, 3, -5);
    scene.add(rimLight);

    // Impact effect group (will be animated)
    const impactGroup = new THREE.Group();
    scene.add(impactGroup);

    // Glowing impact ring - violet (thinner)
    const ringGeometry = new THREE.TorusGeometry(0.6, 0.03, 16, 32);
    const ringMaterial = new THREE.MeshBasicMaterial({
      color: 0xa855f7,
      transparent: true,
      opacity: 0,
    });
    const impactRing = new THREE.Mesh(ringGeometry, ringMaterial);
    impactGroup.add(impactRing);

    // Outer expanding ring - blue
    const outerRingGeometry = new THREE.TorusGeometry(1.0, 0.04, 16, 32);
    const outerRingMaterial = new THREE.MeshBasicMaterial({
      color: 0x60a5fa,
      transparent: true,
      opacity: 0,
    });
    const outerRing = new THREE.Mesh(outerRingGeometry, outerRingMaterial);
    impactGroup.add(outerRing);

    // Glow shader material - creates soft radial glow with additive blending
    const glowShaderMaterial = new THREE.ShaderMaterial({
      uniforms: {
        glowColor: { value: new THREE.Color(0x67e8f9) },
        intensity: { value: 0.0 },
      },
      vertexShader: `
        varying vec3 vNormal;
        varying vec3 vPosition;
        void main() {
          vNormal = normalize(normalMatrix * normal);
          vPosition = position;
          gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
        }
      `,
      fragmentShader: `
        uniform vec3 glowColor;
        uniform float intensity;
        varying vec3 vNormal;
        varying vec3 vPosition;
        void main() {
          // Radial falloff from center
          float dist = length(vPosition);
          float falloff = 1.0 - smoothstep(0.0, 1.0, dist);
          falloff = pow(falloff, 1.5); // Sharper falloff curve

          // Edge glow based on view angle
          vec3 viewDir = normalize(cameraPosition - vPosition);
          float rim = 1.0 - abs(dot(viewDir, vNormal));
          rim = pow(rim, 2.0);

          float alpha = (falloff * 0.8 + rim * 0.4) * intensity;
          gl_FragColor = vec4(glowColor, alpha);
        }
      `,
      transparent: true,
      blending: THREE.AdditiveBlending,
      side: THREE.DoubleSide,
      depthWrite: false,
    });

    // Inner bright core glow
    const glowGeometry = new THREE.SphereGeometry(1.0, 32, 32);
    const glowSphere = new THREE.Mesh(glowGeometry, glowShaderMaterial);
    impactGroup.add(glowSphere);

    // Outer larger glow with different color
    const outerGlowMaterial = new THREE.ShaderMaterial({
      uniforms: {
        glowColor: { value: new THREE.Color(0x22d3ee) },
        intensity: { value: 0.0 },
      },
      vertexShader: glowShaderMaterial.vertexShader,
      fragmentShader: `
        uniform vec3 glowColor;
        uniform float intensity;
        varying vec3 vNormal;
        varying vec3 vPosition;
        void main() {
          float dist = length(vPosition);
          float falloff = 1.0 - smoothstep(0.0, 1.0, dist);
          falloff = pow(falloff, 2.0); // Softer for outer glow
          float alpha = falloff * intensity * 0.6;
          gl_FragColor = vec4(glowColor, alpha);
        }
      `,
      transparent: true,
      blending: THREE.AdditiveBlending,
      side: THREE.DoubleSide,
      depthWrite: false,
    });

    const outerGlowGeometry = new THREE.SphereGeometry(1.0, 32, 32);
    const outerGlowSphere = new THREE.Mesh(outerGlowGeometry, outerGlowMaterial);
    impactGroup.add(outerGlowSphere);

    // Shockwave sphere - pressure wave effect
    const shockwaveMaterial = new THREE.ShaderMaterial({
      uniforms: {
        opacity: { value: 0.0 },
      },
      vertexShader: `
        varying vec3 vNormal;
        varying vec3 vViewPosition;
        void main() {
          vNormal = normalize(normalMatrix * normal);
          vec4 mvPosition = modelViewMatrix * vec4(position, 1.0);
          vViewPosition = -mvPosition.xyz;
          gl_Position = projectionMatrix * mvPosition;
        }
      `,
      fragmentShader: `
        uniform float opacity;
        varying vec3 vNormal;
        varying vec3 vViewPosition;
        void main() {
          // Fresnel effect - visible mainly at edges like a bubble
          vec3 viewDir = normalize(vViewPosition);
          float fresnel = 1.0 - abs(dot(viewDir, vNormal));
          fresnel = pow(fresnel, 3.0); // Sharp edge falloff

          // Subtle white/cyan tint
          vec3 color = mix(vec3(1.0), vec3(0.8, 0.95, 1.0), fresnel);
          float alpha = fresnel * opacity * 0.4;

          gl_FragColor = vec4(color, alpha);
        }
      `,
      transparent: true,
      blending: THREE.AdditiveBlending,
      side: THREE.BackSide, // Render inside to see the edge effect
      depthWrite: false,
    });

    const shockwaveGeometry = new THREE.SphereGeometry(1.0, 32, 32);
    const shockwaveSphere = new THREE.Mesh(shockwaveGeometry, shockwaveMaterial);
    impactGroup.add(shockwaveSphere);

    // Impact point light for illumination - violet/blue mix (stronger)
    const impactLight = new THREE.PointLight(0x8b5cf6, 0, 12);
    impactGroup.add(impactLight);

    // Spark particles radiating from impact - cyan
    const sparkGroup = new THREE.Group();
    const sparkGeometry = new THREE.SphereGeometry(0.05, 8, 8);
    const sparkMaterials: THREE.MeshBasicMaterial[] = [];
    const sparkColors = [0x22d3ee, 0x06b6d4, 0x60a5fa, 0xa855f7]; // cyan, cyan-dark, blue, violet

    for (let i = 0; i < 12; i++) {
      const sparkMaterial = new THREE.MeshBasicMaterial({
        color: sparkColors[i % sparkColors.length],
        transparent: true,
        opacity: 0,
      });
      sparkMaterials.push(sparkMaterial);
      const spark = new THREE.Mesh(sparkGeometry, sparkMaterial);
      const angle = (i / 12) * Math.PI * 2;
      const radius = 0.8 + Math.random() * 0.4;
      const spreadY = (Math.random() - 0.5) * 0.6;
      spark.position.set(
        Math.cos(angle) * radius,
        spreadY,
        Math.sin(angle) * radius
      );
      spark.scale.setScalar(0.5 + Math.random() * 1.0);
      spark.userData = { baseRadius: radius, angle, spreadY };
      sparkGroup.add(spark);
    }
    impactGroup.add(sparkGroup);

    // Spark trails/streaks shooting outward from impact - violet/blue gradient
    const trailGroup = new THREE.Group();
    const trailMaterials: THREE.MeshBasicMaterial[] = [];
    const trails: THREE.Mesh[] = [];
    const trailColors = [0xa855f7, 0xc084fc, 0x60a5fa, 0x3b82f6, 0x22d3ee]; // violet to cyan

    for (let i = 0; i < 16; i++) {
      // Create elongated trail geometry
      const trailLength = 0.8 + Math.random() * 1.2;
      const trailGeometry = new THREE.CylinderGeometry(0.015, 0.04, trailLength, 6);
      const trailMaterial = new THREE.MeshBasicMaterial({
        color: trailColors[i % trailColors.length],
        transparent: true,
        opacity: 0,
      });
      trailMaterials.push(trailMaterial);
      const trail = new THREE.Mesh(trailGeometry, trailMaterial);

      // Random direction outward
      const theta = Math.random() * Math.PI * 2;
      const phi = Math.random() * Math.PI * 0.6 + Math.PI * 0.2;
      const distance = 0.6 + Math.random() * 0.8;

      const x = Math.sin(phi) * Math.cos(theta) * distance;
      const y = Math.cos(phi) * distance * 0.5;
      const z = Math.sin(phi) * Math.sin(theta) * distance;

      trail.userData = { baseX: x, baseY: y, baseZ: z, theta, phi, distance };
      trail.position.set(0, 0, 0);

      // Orient trail to point outward from center
      trail.lookAt(x * 2, y * 2, z * 2);
      trail.rotateX(Math.PI / 2);

      trails.push(trail);
      trailGroup.add(trail);
    }
    impactGroup.add(trailGroup);

    // Bright cyan/white spark tips
    const tipMaterials: THREE.MeshBasicMaterial[] = [];
    const tips: THREE.Mesh[] = [];
    const tipGeometry = new THREE.SphereGeometry(0.04, 8, 8);
    const tipColors = [0xffffff, 0x67e8f9, 0xa5f3fc]; // white, light cyan variations

    for (let i = 0; i < 10; i++) {
      const tipMaterial = new THREE.MeshBasicMaterial({
        color: tipColors[i % tipColors.length],
        transparent: true,
        opacity: 0,
      });
      tipMaterials.push(tipMaterial);
      const tip = new THREE.Mesh(tipGeometry, tipMaterial);
      const theta = Math.random() * Math.PI * 2;
      const phi = Math.random() * Math.PI * 0.5 + Math.PI * 0.25;
      const distance = 1.2 + Math.random() * 0.6;

      tip.userData = {
        baseX: Math.sin(phi) * Math.cos(theta) * distance,
        baseY: Math.cos(phi) * distance * 0.4,
        baseZ: Math.sin(phi) * Math.sin(theta) * distance,
      };
      tip.position.set(0, 0, 0);
      tips.push(tip);
      trailGroup.add(tip);
    }

    // Position impact group at contact point
    // Adjust these values to position the explosion:
    // X: negative = toward car, positive = toward ball
    // Y: up/down
    // Z: depth
    impactGroup.position.set(-1.0, 3.2, -0.8);
    impactGroup.lookAt(1.2, 5.0, -0.3); // Look toward ball position

    // Load models
    const loader = new GLTFLoader();
    let ballModel: THREE.Object3D | null = null;
    let carGroup: THREE.Group | null = null;
    let modelsLoaded = 0;
    const totalModels = 2;

    const checkAllLoaded = () => {
      modelsLoaded++;
      if (modelsLoaded >= totalModels) {
        setLoading(false);
      }
    };

    // Animation positions - single straight line trajectory
    const carStartPos = new THREE.Vector3(-14, -2, 4);
    const carEndPos = new THREE.Vector3(6, 5, -4); // Straight line through impact point

    // Ball trajectory: comes from one side, gets hit, flies off
    const ballInitialPos = new THREE.Vector3(6, 8, 3); // Where ball comes from
    const ballImpactPos = new THREE.Vector3(1.2, 5.0, -0.3); // Impact point
    const ballEndPos = new THREE.Vector3(8, 12, -4); // Where ball flies after hit

    // Load the Octane GLB model
    loader.load(
      '/models/octane_hero.glb',
      (gltf) => {
        const model = gltf.scene;

        // Scale car
        const box = new THREE.Box3().setFromObject(model);
        const size = box.getSize(new THREE.Vector3());
        const maxDim = Math.max(size.x, size.y, size.z);
        const scale = 5.0 / maxDim;
        model.scale.setScalar(scale);

        // Center the model first
        box.setFromObject(model);
        const center = box.getCenter(new THREE.Vector3());
        model.position.x = -center.x;
        model.position.z = -center.z;
        model.position.y = -center.y;

        // Create a pivot group to rotate the car for aerial pose
        carGroup = new THREE.Group();
        carGroup.add(model);

        // Start position (rotation will be set in animation loop)
        carGroup.position.copy(carStartPos);

        carGroup.traverse((child) => {
          if (child instanceof THREE.Mesh) {
            child.castShadow = true;
            child.receiveShadow = true;
          }
        });

        scene.add(carGroup);
        checkAllLoaded();
      },
      undefined,
      (err) => {
        console.error('Error loading Octane model:', err);
        setError('Failed to load 3D model');
        checkAllLoaded();
      }
    );

    // Load the Ball model
    loader.load(
      '/models/ball/scene.gltf',
      (gltf) => {
        ballModel = gltf.scene;

        // Scale the ball - much bigger
        const box = new THREE.Box3().setFromObject(ballModel);
        const size = box.getSize(new THREE.Vector3());
        const maxDim = Math.max(size.x, size.y, size.z);
        const ballScale = 6.0 / maxDim;
        ballModel.scale.setScalar(ballScale);

        // Start position
        ballModel.position.copy(ballInitialPos);

        // Random rotation so patterns don't look too aligned
        ballModel.rotation.set(
          Math.random() * Math.PI * 2,
          Math.random() * Math.PI * 2,
          Math.random() * Math.PI * 2
        );

        ballModel.traverse((child) => {
          if (child instanceof THREE.Mesh) {
            child.castShadow = true;
            child.receiveShadow = true;
          }
        });

        scene.add(ballModel);
        checkAllLoaded();
      },
      undefined,
      (err) => {
        console.error('Error loading Ball model:', err);
        // Don't set error - ball is optional
        checkAllLoaded();
      }
    );

    // Animation
    let animationId: number;
    const clock = new THREE.Clock();

    // Impact timing in normalized progress (0-1)
    const IMPACT_TIME = 0.58; // Impact happens at 58% of animation

    const animate = () => {
      animationId = requestAnimationFrame(animate);
      const now = clock.getElapsedTime();
      const deltaTime = now - lastTimeRef.current;
      lastTimeRef.current = now;

      // Update progress if playing
      if (isPlayingRef.current) {
        const progressIncrement = (deltaTime / ANIMATION_DURATION) * speedRef.current;
        progressRef.current += progressIncrement;

        // First play: pause just after impact to show explosion start (at ~59% progress)
        if (isFirstPlayRef.current && progressRef.current >= 0.59) {
          isFirstPlayRef.current = false;
          setIsPlaying(false);
          progressRef.current = 0.59; // Stop exactly at this point
        }

        // Loop animation
        if (progressRef.current >= 1) {
          progressRef.current = 0;
        }

        // Update state periodically for UI
        setProgress(progressRef.current);
      }

      const t = progressRef.current;

      // ===== CAR ANIMATION =====
      if (carGroup) {
        // Car travels in a straight line from start to end
        carGroup.position.lerpVectors(carStartPos, carEndPos, t);

        // Fixed orientation angles (pointing toward ball trajectory)
        const yaw = 0.5;   // Angle toward the ball
        const pitch = 0.3; // Nose slightly up

        // Continuous air roll (barrel roll) - rotation around car's forward axis (X)
        const rollAngle = t * Math.PI * 3; // 1.5 full rotations

        // Apply rotations: Yaw first, then Pitch, then Roll on local X axis
        carGroup.rotation.order = 'YZX';
        carGroup.rotation.set(rollAngle, yaw, pitch);
      }

      // ===== BALL ANIMATION =====
      if (ballModel) {
        // Rotation speeds (in radians over full phase)
        const preImpactRotX = Math.PI * 0.5;
        const preImpactRotZ = Math.PI * 0.3;
        const postImpactRotX = Math.PI * 0.8; // Slightly faster after hit
        const postImpactRotZ = Math.PI * 0.5;

        if (t < IMPACT_TIME) {
          // Ball arrives from initial position to impact point
          const ballT = t / IMPACT_TIME;
          ballModel.position.lerpVectors(ballInitialPos, ballImpactPos, ballT);

          // Gentle spin while approaching
          ballModel.rotation.x = ballT * preImpactRotX;
          ballModel.rotation.z = ballT * preImpactRotZ;
        } else {
          // Ball launches after impact
          const ballT = (t - IMPACT_TIME) / (1 - IMPACT_TIME);
          const easedBallT = easeOutCubic(ballT);

          // Arc trajectory after being hit
          const arcHeight = Math.sin(ballT * Math.PI) * 3;
          ballModel.position.lerpVectors(ballImpactPos, ballEndPos, easedBallT);
          ballModel.position.y += arcHeight;

          // Continuous rotation: start from where pre-impact ended + add post-impact rotation
          ballModel.rotation.x = preImpactRotX + ballT * postImpactRotX;
          ballModel.rotation.z = preImpactRotZ + ballT * postImpactRotZ;
        }
      }

      // ===== IMPACT EFFECTS ANIMATION =====
      // Flash effect at impact
      const flashIntensity = t > IMPACT_TIME - 0.01 && t < IMPACT_TIME + 0.08
        ? Math.max(0, 1 - Math.abs(t - IMPACT_TIME) * 20)
        : 0;

      // Ring animations - start exactly at impact, grow from 0
      if (t >= IMPACT_TIME) {
        // Inner ring: grows slower but continues expanding (0 to 3.5 over longer time)
        const ringT = Math.min((t - IMPACT_TIME) / 0.4, 1);
        const ringScale = easeOutCubic(ringT) * 3.5;
        impactRing.scale.setScalar(ringScale);
        ringMaterial.opacity = Math.max(0, 0.9 - ringT * 0.9);

        // Outer ring: grows from 0 to 5, faster
        const outerRingT = Math.min((t - IMPACT_TIME) / 0.25, 1);
        const outerRingScale = easeOutCubic(outerRingT) * 5;
        outerRing.scale.setScalar(outerRingScale);
        outerRingMaterial.opacity = Math.max(0, 0.6 - outerRingT * 0.6);
      } else {
        ringMaterial.opacity = 0;
        outerRingMaterial.opacity = 0;
        impactRing.scale.setScalar(0);
        outerRing.scale.setScalar(0);
      }

      // Glow effect - grows from 0 with enhanced flash
      if (t >= IMPACT_TIME - 0.01 && t < IMPACT_TIME + 0.35) {
        const glowT = (t - (IMPACT_TIME - 0.01)) / 0.36; // 0 to 1 over full duration

        // Scale grows quickly then stays
        const scaleT = Math.min(glowT * 3, 1); // Reach full scale in first third
        const coreScale = easeOutCubic(scaleT) * 2.0 + flashIntensity * 2.0;
        glowSphere.scale.setScalar(coreScale);

        // Intensity: quick rise then smooth fade to 0
        const fadeT = Math.max(0, (glowT - 0.2) / 0.8); // Start fading after 20%
        const coreIntensity = (1.0 - fadeT) * (1.0 + flashIntensity * 0.8);
        glowShaderMaterial.uniforms.intensity.value = Math.max(0, coreIntensity);

        // Outer glow - larger, softer
        const outerScale = easeOutCubic(scaleT) * 3.5 + flashIntensity * 2.5;
        outerGlowSphere.scale.setScalar(outerScale);
        const outerIntensity = (1.0 - fadeT) * (0.8 + flashIntensity * 0.6);
        outerGlowMaterial.uniforms.intensity.value = Math.max(0, outerIntensity);

        // Light intensity fades with glow
        impactLight.intensity = (1.0 - fadeT) * 5 + flashIntensity * 10;
      } else {
        glowSphere.scale.setScalar(0);
        glowShaderMaterial.uniforms.intensity.value = 0;
        outerGlowSphere.scale.setScalar(0);
        outerGlowMaterial.uniforms.intensity.value = 0;
        impactLight.intensity = 0;
      }

      // Shockwave - expands very fast, fades quickly
      if (t >= IMPACT_TIME && t < IMPACT_TIME + 0.15) {
        const shockT = (t - IMPACT_TIME) / 0.15; // 0 to 1 very fast
        // Expands rapidly with easing
        const shockScale = easeOutCubic(shockT) * 12; // Gets very large
        shockwaveSphere.scale.setScalar(shockScale);
        // Opacity peaks early then fades
        const shockOpacity = Math.sin(shockT * Math.PI) * 1.2; // Peaks at middle
        shockwaveMaterial.uniforms.opacity.value = shockOpacity;
      } else {
        shockwaveSphere.scale.setScalar(0);
        shockwaveMaterial.uniforms.opacity.value = 0;
      }

      // Spark animations - expand outward from impact
      if (t >= IMPACT_TIME - 0.02) {
        const sparkT = Math.min((t - (IMPACT_TIME - 0.02)) / 0.5, 1);
        const sparkExpand = easeOutCubic(sparkT);

        sparkGroup.children.forEach((spark, i) => {
          const data = spark.userData;
          const expandedRadius = data.baseRadius * (1 + sparkExpand * 2);
          spark.position.set(
            Math.cos(data.angle) * expandedRadius,
            data.spreadY * (1 + sparkExpand),
            Math.sin(data.angle) * expandedRadius
          );
          sparkMaterials[i].opacity = Math.max(0, 0.9 - sparkT * 0.9);
        });
      } else {
        sparkMaterials.forEach(m => m.opacity = 0);
      }

      // Trail animations
      if (t >= IMPACT_TIME) {
        const trailT = Math.min((t - IMPACT_TIME) / 0.4, 1);
        const trailExpand = easeOutCubic(trailT);

        trails.forEach((trail, i) => {
          const data = trail.userData;
          trail.position.set(
            data.baseX * trailExpand * 2,
            data.baseY * trailExpand * 2,
            data.baseZ * trailExpand * 2
          );
          trailMaterials[i].opacity = Math.max(0, 0.85 - trailT);
        });

        tips.forEach((tip, i) => {
          const data = tip.userData;
          tip.position.set(
            data.baseX * trailExpand * 2,
            data.baseY * trailExpand * 2,
            data.baseZ * trailExpand * 2
          );
          tipMaterials[i].opacity = Math.max(0, 0.95 - trailT);
        });
      } else {
        trailMaterials.forEach(m => m.opacity = 0);
        tipMaterials.forEach(m => m.opacity = 0);
        trails.forEach(trail => trail.position.set(0, 0, 0));
        tips.forEach(tip => tip.position.set(0, 0, 0));
      }

      controls.update();
      renderer.render(scene, camera);
    };
    animate();

    // Resize handler
    const handleResize = () => {
      const newWidth = container.clientWidth;
      const newHeight = container.clientHeight;
      camera.aspect = newWidth / newHeight;
      camera.updateProjectionMatrix();
      renderer.setSize(newWidth, newHeight);
    };

    window.addEventListener('resize', handleResize);

    // Cleanup
    return () => {
      window.removeEventListener('resize', handleResize);
      cancelAnimationFrame(animationId);
      controls.dispose();
      pmremGenerator.dispose();
      envMap.dispose();
      renderer.dispose();

      if (container.contains(renderer.domElement)) {
        container.removeChild(renderer.domElement);
      }

      scene.traverse((object) => {
        if (object instanceof THREE.Mesh) {
          object.geometry.dispose();
          if (Array.isArray(object.material)) {
            object.material.forEach((m) => m.dispose());
          } else {
            object.material.dispose();
          }
        }
      });
    };
  }, []);

  if (error) {
    return (
      <div className={`flex items-center justify-center ${className}`}>
        <div className="text-gray-500 text-sm">{error}</div>
      </div>
    );
  }

  return (
    <div className={`relative ${className}`}>
      {/* Background glow and rays - behind the 3D scene */}
      <div className="absolute inset-0 pointer-events-none -z-10">
        {/* Main gradient blob */}
        <div
          className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[600px] h-[600px] rounded-full"
          style={{ background: 'radial-gradient(circle, rgba(139,92,246,0.4) 0%, rgba(59,130,246,0.25) 40%, transparent 70%)', filter: 'blur(40px)' }}
        />

        {/* Neon ray segments - explosion effect radiating from center, synced with player */}
        <div className="absolute top-1/2 left-1/2">
          {raySegments.map((segment, i) => {
            // Animate distance based on progress - rays expand outward during playback
            const animatedDistance = segment.distance + progress * 80;
            // Slight opacity pulse at impact
            const impactPulse = progress > 0.55 && progress < 0.65 ? 1.3 : 1;
            return (
              <div
                key={i}
                className="absolute rounded-full"
                style={{
                  width: `${segment.width}px`,
                  height: `${segment.height}px`,
                  transform: `rotate(${segment.angle}deg) translateX(${animatedDistance}px)`,
                  background: segment.color,
                  boxShadow: `0 0 ${segment.height * 2 * impactPulse}px ${segment.glow}, 0 0 ${segment.height * 4 * impactPulse}px ${segment.glow}, 0 0 ${segment.height * 6 * impactPulse}px ${segment.glow}`,
                  opacity: impactPulse > 1 ? 1 : 0.85,
                }}
              />
            );
          })}
        </div>
      </div>

      {/* Loading spinner - centered */}
      {loading && (
        <div className="absolute inset-0 flex items-center justify-center z-10">
          <div
            className="w-10 h-10 rounded-full border-2 border-transparent animate-spin"
            style={{
              borderTopColor: '#a855f7',
              borderRightColor: '#3b82f6',
              filter: 'drop-shadow(0 0 8px rgba(139,92,246,0.6))',
              animationDuration: '1s',
            }}
          />
        </div>
      )}

      {/* 3D Canvas - hidden until loaded */}
      <div
        ref={containerRef}
        className={`w-full h-full cursor-grab active:cursor-grabbing transition-opacity duration-500 ${loading ? 'opacity-0' : 'opacity-100'}`}
        onMouseEnter={() => setShowDragHint(false)}
      />

      {/* Drag hint overlay - positioned above the player controls */}
      {!loading && showDragHint && (
        <div
          className="absolute bottom-28 left-1/2 -translate-x-1/2 z-30 pointer-events-none"
        >
          <div className="flex items-center gap-2 bg-gray-900/90 backdrop-blur-sm rounded-full px-4 py-2 border border-gray-700/50 shadow-xl">
            <div style={{ animation: 'dragHint 1.5s ease-in-out infinite' }}>
              <Mouse className="w-5 h-5 text-violet-400" />
            </div>
            <span className="text-sm text-gray-200 font-medium">Drag to rotate</span>
          </div>
        </div>
      )}

      {/* Player Controls Overlay */}
      {!loading && (
        <div className="absolute bottom-4 left-4 right-4 z-20">
          <div className="bg-gray-900/80 backdrop-blur-sm rounded-xl border border-gray-700/50 p-3 shadow-xl">
            <div className="flex items-center gap-3">
              {/* Play/Pause Button */}
              <button
                onClick={() => setIsPlaying(!isPlaying)}
                className="w-10 h-10 flex items-center justify-center rounded-lg bg-gradient-to-br from-violet-600 to-blue-600 hover:from-violet-500 hover:to-blue-500 transition-all shadow-lg shadow-violet-500/20"
              >
                {isPlaying ? (
                  <Pause className="w-5 h-5 text-white" />
                ) : (
                  <Play className="w-5 h-5 text-white ml-0.5" />
                )}
              </button>

              {/* Timeline */}
              <div
                ref={timelineRef}
                className="flex-1 h-2 bg-gray-700 rounded-full cursor-pointer relative group select-none"
                onMouseDown={handleTimelineMouseDown}
                draggable={false}
              >
                {/* Progress bar */}
                <div
                  className="absolute top-0 left-0 h-full bg-gradient-to-r from-violet-500 to-blue-500 rounded-full"
                  style={{ width: `${progress * 100}%` }}
                />
                {/* Scrubber handle */}
                <div
                  className="absolute top-1/2 -translate-y-1/2 w-4 h-4 bg-white rounded-full shadow-lg opacity-0 group-hover:opacity-100 transition-opacity"
                  style={{ left: `calc(${progress * 100}% - 8px)` }}
                />
              </div>

              {/* Speed Control */}
              <div className="flex items-center gap-1">
                {SPEED_OPTIONS.map((s) => (
                  <button
                    key={s}
                    onClick={() => setSpeed(s)}
                    className={`px-2 py-1 text-xs font-medium rounded transition-all ${speed === s
                      ? 'bg-violet-600 text-white'
                      : 'bg-gray-700 text-gray-400 hover:bg-gray-600 hover:text-gray-200'
                      }`}
                  >
                    {s}x
                  </button>
                ))}
              </div>

              {/* Auto-rotate Toggle */}
              <button
                onClick={() => setAutoRotate(!autoRotate)}
                className={`p-2 rounded-lg transition-all ${autoRotate
                  ? 'bg-violet-600 text-white'
                  : 'bg-gray-700 text-gray-400 hover:bg-gray-600 hover:text-gray-200'
                  }`}
                title={autoRotate ? 'Désactiver rotation auto' : 'Activer rotation auto'}
              >
                <RotateCw className={`w-4 h-4 ${autoRotate ? 'animate-spin' : ''}`} style={{ animationDuration: '3s' }} />
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
