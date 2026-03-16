import * as THREE from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
import type { ReplayModel } from "./types";

export interface ReplayScene {
  scene: THREE.Scene;
  replayRoot: THREE.Group;
  camera: THREE.PerspectiveCamera;
  renderer: THREE.WebGLRenderer;
  controls: OrbitControls;
  resize: () => void;
  dispose: () => void;
  ballMesh: THREE.Mesh;
  playerMeshes: Map<string, THREE.Object3D>;
  playerBoostTrails: Map<string, THREE.Group>;
  playerBoostMeters: Map<string, BoostMeter>;
  updateWallVisibility: () => void;
}

interface WallPanel {
  mesh: THREE.Mesh;
  material: THREE.Material;
  outwardLocal: THREE.Vector3;
  fixedOpacity: number | null;
}

const OPAQUE_WALL_OPACITY = 1;
const OUTSIDE_WALL_OPACITY = 0.32;

function createWallMaterial(color: number): THREE.MeshBasicMaterial {
  const material = new THREE.MeshBasicMaterial({
    color,
    transparent: true,
    opacity: OPAQUE_WALL_OPACITY,
    side: THREE.DoubleSide,
  });
  material.forceSinglePass = true;
  return material;
}

function createFloorMaterial(color: number): THREE.MeshLambertMaterial {
  return new THREE.MeshLambertMaterial({
    color,
    side: THREE.DoubleSide,
    transparent: true,
    opacity: OPAQUE_WALL_OPACITY,
  });
}

function createVerticalWallBox(
  length: number,
  height: number,
  thickness: number,
  material: THREE.Material
): THREE.Mesh {
  return new THREE.Mesh(
    new THREE.BoxGeometry(length, thickness, height, 6, 1, 6),
    material
  );
}

function createHorizontalWallBox(
  width: number,
  depth: number,
  thickness: number,
  material: THREE.Material
): THREE.Mesh {
  return new THREE.Mesh(
    new THREE.BoxGeometry(width, depth, thickness, 6, 6, 1),
    material
  );
}

function createExampleSoccarField(scale: number): {
  stadium: THREE.Group;
  wallPanels: WallPanel[];
} {
  type MirrorSign = 1 | -1;

  const SOCCAR_YSIZE = 10280 * scale;
  const SOCCAR_XSIZE = 8240 * scale;
  const SOCCAR_DEPTH = 1960 * scale;
  const STADIUM_CORNER = 1000 * scale;
  const GOAL_WIDTH = 1900 * scale;
  const GOAL_HEIGHT = 800 * scale;
  const GOAL_DEPTH = 900 * scale;
  const WALL_THICKNESS = Math.max(1, scale);
  const wallPanels: WallPanel[] = [];
  const mirroredSigns: MirrorSign[] = [1, -1];

  function registerWall(
    mesh: THREE.Mesh,
    outwardLocal: THREE.Vector3,
    fixedOpacity: number | null = null
  ): THREE.Mesh {
    const material = (mesh.material as THREE.Material).clone();
    mesh.material = material;
    wallPanels.push({
      mesh,
      material,
      outwardLocal: outwardLocal.clone().normalize(),
      fixedOpacity,
    });
    return mesh;
  }

  function createBackWall(color: number): THREE.Group {
    const backWall = new THREE.Group();
    const wallMaterial = createWallMaterial(color);
    const sideWallWidth = SOCCAR_XSIZE / 2 - STADIUM_CORNER - GOAL_WIDTH / 2;

    const cornerWidth = Math.sqrt(2 * Math.pow(STADIUM_CORNER, 2));

    for (const xSign of mirroredSigns) {
      const sideWall = registerWall(
        createVerticalWallBox(
          sideWallWidth,
          SOCCAR_DEPTH,
          WALL_THICKNESS,
          wallMaterial
        ),
        new THREE.Vector3(0, 1, 0)
      );
      sideWall.position.set(
        xSign * (sideWallWidth / 2 + GOAL_WIDTH / 2),
        0,
        SOCCAR_DEPTH / 2
      );
      backWall.add(sideWall);

      const corner = registerWall(
        createVerticalWallBox(
          cornerWidth,
          SOCCAR_DEPTH,
          WALL_THICKNESS,
          wallMaterial
        ),
        new THREE.Vector3(0, 1, 0)
      );
      corner.position.set(
        xSign * (SOCCAR_XSIZE / 2 - STADIUM_CORNER / 2),
        -STADIUM_CORNER / 2,
        SOCCAR_DEPTH / 2
      );
      corner.rotateZ(-xSign * Math.PI / 4);
      backWall.add(corner);
    }

    const top = registerWall(
      createVerticalWallBox(
        GOAL_WIDTH,
        SOCCAR_DEPTH - GOAL_HEIGHT,
        WALL_THICKNESS,
        wallMaterial
      ),
      new THREE.Vector3(0, 1, 0)
    );
    top.position.set(0, 0, SOCCAR_DEPTH / 2 + GOAL_HEIGHT / 2);
    backWall.add(top);

    return backWall;
  }

  function createHalf(color: number, mirrored: boolean): THREE.Group {
    const res = new THREE.Group();
    const floorOutline: Array<[number, number]> = [
      [SOCCAR_XSIZE / 2, 0],
      [-SOCCAR_XSIZE / 2, 0],
      [-SOCCAR_XSIZE / 2, SOCCAR_YSIZE / 2 - STADIUM_CORNER],
      [-SOCCAR_XSIZE / 2 + STADIUM_CORNER, SOCCAR_YSIZE / 2],
      [-GOAL_WIDTH / 2, SOCCAR_YSIZE / 2],
      [-GOAL_WIDTH / 2, SOCCAR_YSIZE / 2 + GOAL_DEPTH],
      [GOAL_WIDTH / 2, SOCCAR_YSIZE / 2 + GOAL_DEPTH],
      [GOAL_WIDTH / 2, SOCCAR_YSIZE / 2],
      [SOCCAR_XSIZE / 2 - STADIUM_CORNER, SOCCAR_YSIZE / 2],
      [SOCCAR_XSIZE / 2, SOCCAR_YSIZE / 2 - STADIUM_CORNER],
      [SOCCAR_XSIZE / 2, 0],
    ];
    const floor = new THREE.Shape();
    floorOutline.forEach(([x, y], index) => {
      if (index === 0) {
        floor.moveTo(x, y);
      } else {
        floor.lineTo(x, y);
      }
    });

    const floorMaterial = createFloorMaterial(color);
    const wallMaterial = createWallMaterial(color);

    const floorMesh = registerWall(
      new THREE.Mesh(
        new THREE.ShapeGeometry(floor),
        floorMaterial
      ),
      new THREE.Vector3(0, 0, -1)
    );
    floorMesh.receiveShadow = true;
    res.add(floorMesh);

    for (const xSign of mirroredSigns) {
      const goalPost = registerWall(
        createVerticalWallBox(
          GOAL_DEPTH,
          GOAL_HEIGHT,
          WALL_THICKNESS,
          wallMaterial
        ),
        new THREE.Vector3(0, -xSign, 0),
        OUTSIDE_WALL_OPACITY
      );
      goalPost.position.set(
        xSign * GOAL_WIDTH / 2,
        SOCCAR_YSIZE / 2 + GOAL_DEPTH / 2,
        GOAL_HEIGHT / 2
      );
      goalPost.rotateZ(Math.PI / 2);
      res.add(goalPost);
    }

    const goalRoof = registerWall(
      createHorizontalWallBox(
        GOAL_WIDTH,
        GOAL_DEPTH,
        WALL_THICKNESS,
        wallMaterial
      ),
      new THREE.Vector3(0, 0, 1),
      OUTSIDE_WALL_OPACITY
    );
    goalRoof.position.set(
      0,
      SOCCAR_YSIZE / 2 + GOAL_DEPTH / 2,
      GOAL_HEIGHT
    );
    res.add(goalRoof);

    const goalBack = registerWall(
      createVerticalWallBox(
        GOAL_WIDTH,
        GOAL_HEIGHT,
        WALL_THICKNESS,
        wallMaterial
      ),
      new THREE.Vector3(0, 1, 0),
      OUTSIDE_WALL_OPACITY
    );
    goalBack.position.set(
      0,
      SOCCAR_YSIZE / 2 + GOAL_DEPTH,
      GOAL_HEIGHT / 2
    );
    res.add(goalBack);

    const backWall = createBackWall(color);
    backWall.position.y = SOCCAR_YSIZE / 2;
    res.add(backWall);

    for (const xSign of mirroredSigns) {
      const sideWall = registerWall(
        createVerticalWallBox(
          SOCCAR_YSIZE / 2 - STADIUM_CORNER,
          SOCCAR_DEPTH,
          WALL_THICKNESS,
          wallMaterial
        ),
        new THREE.Vector3(0, -xSign, 0)
      );
      sideWall.position.set(
        xSign * SOCCAR_XSIZE / 2,
        (SOCCAR_YSIZE / 2 - STADIUM_CORNER) / 2,
        SOCCAR_DEPTH / 2
      );
      sideWall.rotateZ(Math.PI / 2);
      res.add(sideWall);
    }

    if (mirrored) {
      res.rotateZ(Math.PI);
    }

    return res;
  }

  const stadium = new THREE.Group();
  stadium.add(createHalf(0xffe8b3, false));
  stadium.add(createHalf(0x7fe3ff, true));
  return { stadium, wallPanels };
}

function createExampleCarMesh(color: string): THREE.Group {
  const vertices = [
    [100, -100, 100],
    [100, 100, 100],
    [-100, 100, 100],
    [-100, -100, 100],
    [150, -220, 20],
    [-150, -220, 20],
    [130, -400, -20],
    [-130, -400, -20],
    [140, 170, 25],
    [-140, 170, 25],
    [130, 240, 25],
    [-130, 240, 25],
    [130, -400, -80],
    [-130, -400, -80],
    [150, -220, -80],
    [-150, -220, -80],
    [140, 170, -80],
    [-140, 170, -80],
    [130, 240, -80],
    [-130, 240, -80],
  ];
  const faces = [
    [0, 1, 2],
    [0, 2, 3],
    [4, 0, 5],
    [0, 3, 5],
    [6, 4, 5],
    [6, 5, 7],
    [1, 8, 9],
    [1, 9, 2],
    [4, 8, 1],
    [4, 1, 0],
    [3, 2, 9],
    [3, 9, 5],
    [8, 10, 11],
    [8, 11, 9],
    [12, 6, 7],
    [12, 7, 13],
    [7, 5, 15],
    [7, 15, 13],
    [6, 14, 4],
    [12, 14, 6],
    [14, 16, 4],
    [4, 16, 8],
    [5, 9, 15],
    [15, 9, 17],
    [16, 18, 8],
    [8, 18, 10],
    [9, 11, 17],
    [17, 11, 19],
    [10, 18, 11],
    [11, 18, 19],
    [14, 12, 13],
    [14, 13, 15],
    [16, 14, 15],
    [16, 15, 17],
    [18, 16, 17],
    [18, 17, 19],
  ];

  const bodyGeometry = new THREE.BufferGeometry();
  bodyGeometry.setAttribute(
    "position",
    new THREE.Float32BufferAttribute(vertices.flat(), 3)
  );
  bodyGeometry.setIndex(faces.flat());
  bodyGeometry.computeVertexNormals();

  const outer = new THREE.Group();
  const inner = new THREE.Group();
  const body = new THREE.Mesh(
    bodyGeometry,
    new THREE.MeshLambertMaterial({ color })
  );
  body.castShadow = true;
  inner.add(body);

  const wheelMaterial = new THREE.MeshLambertMaterial({ color: 0x111111 });
  const makeWheel = (x: number, y: number, z: number, width: number): THREE.Mesh => {
    const wheel = new THREE.Mesh(
      new THREE.CylinderGeometry(70, 70, width, 10),
      wheelMaterial
    );
    wheel.rotateZ(Math.PI / 2);
    wheel.position.set(x, y, z);
    return wheel;
  };

  inner.add(makeWheel(120, -300, -60, 50));
  inner.add(makeWheel(-120, -300, -60, 50));
  inner.add(makeWheel(120, 150, -60, 70));
  inner.add(makeWheel(-120, 150, -60, 70));
  inner.position.set(0, 0, 50);
  inner.rotateZ(Math.PI / 2);
  inner.scale.set(0.35, 0.35, 0.35);
  outer.add(inner);
  return outer;
}

function createBoostTrail(): THREE.Group {
  const trail = new THREE.Group();
  trail.visible = false;
  trail.position.set(-124, 0, 8);

  const outerGeometry = new THREE.ConeGeometry(30, 220, 14, 1, true);
  outerGeometry.rotateZ(Math.PI / 2);
  outerGeometry.translate(-110, 0, 0);

  const innerGeometry = new THREE.ConeGeometry(17, 150, 12, 1, true);
  innerGeometry.rotateZ(Math.PI / 2);
  innerGeometry.translate(-75, 0, 0);

  const glowGeometry = new THREE.SphereGeometry(21, 12, 12);
  const nozzleOffsets = [-38, 38];

  for (const lateralOffset of nozzleOffsets) {
    const plume = new THREE.Group();
    plume.position.set(0, lateralOffset, 0);

    const outerMaterial = new THREE.MeshBasicMaterial({
      color: "#ff9b2f",
      transparent: true,
      opacity: 0.42,
      blending: THREE.AdditiveBlending,
      depthWrite: false,
      side: THREE.DoubleSide,
    });
    outerMaterial.forceSinglePass = true;
    const outerFlame = new THREE.Mesh(outerGeometry, outerMaterial);
    outerFlame.name = "outer-flame";
    plume.add(outerFlame);

    const innerMaterial = new THREE.MeshBasicMaterial({
      color: "#fff2ba",
      transparent: true,
      opacity: 0.9,
      blending: THREE.AdditiveBlending,
      depthWrite: false,
      side: THREE.DoubleSide,
    });
    innerMaterial.forceSinglePass = true;
    const innerFlame = new THREE.Mesh(innerGeometry, innerMaterial);
    innerFlame.name = "inner-flame";
    plume.add(innerFlame);

    const glowMaterial = new THREE.MeshBasicMaterial({
      color: "#fff8db",
      transparent: true,
      opacity: 0.62,
      blending: THREE.AdditiveBlending,
      depthWrite: false,
    });
    glowMaterial.forceSinglePass = true;
    const glow = new THREE.Mesh(glowGeometry, glowMaterial);
    glow.name = "glow";
    glow.position.x = -10;
    plume.add(glow);

    trail.add(plume);
  }

  return trail;
}

export interface BoostMeter {
  group: THREE.Group;
  fillMesh: THREE.Mesh;
  fillMaterial: THREE.MeshBasicMaterial;
  labelTexture: THREE.CanvasTexture;
  labelContext: CanvasRenderingContext2D;
  labelCanvas: HTMLCanvasElement;
  lastPercent: number | null;
}

function createBoostMeter(): BoostMeter {
  const group = new THREE.Group();
  group.visible = false;

  // Position above the car, sized to remain readable from the default camera.
  group.position.set(0, 0, 235);

  const panelWidth = 240;
  const panelHeight = 82;
  const barWidth = 188;
  const barHeight = 20;

  const panelGeometry = new THREE.PlaneGeometry(panelWidth, panelHeight);
  const panelMaterial = new THREE.MeshBasicMaterial({
    color: 0x07131d,
    transparent: true,
    opacity: 0.78,
    side: THREE.DoubleSide,
    depthWrite: false,
  });
  const panelMesh = new THREE.Mesh(panelGeometry, panelMaterial);
  panelMesh.position.z = -1;
  group.add(panelMesh);

  // Background (dark)
  const bgGeometry = new THREE.PlaneGeometry(barWidth, barHeight);
  const bgMaterial = new THREE.MeshBasicMaterial({
    color: 0x152431,
    transparent: true,
    opacity: 0.92,
    side: THREE.DoubleSide,
    depthWrite: false,
  });
  const bgMesh = new THREE.Mesh(bgGeometry, bgMaterial);
  bgMesh.position.y = -18;
  group.add(bgMesh);

  // Fill (yellow-gold, scales with boost amount)
  const fillGeometry = new THREE.PlaneGeometry(barWidth, barHeight);
  const fillMaterial = new THREE.MeshBasicMaterial({
    color: 0xffc247,
    transparent: true,
    opacity: 0.98,
    side: THREE.DoubleSide,
    depthWrite: false,
  });
  const fillMesh = new THREE.Mesh(fillGeometry, fillMaterial);
  fillMesh.position.y = -18;
  group.add(fillMesh);

  const labelCanvas = document.createElement("canvas");
  labelCanvas.width = 512;
  labelCanvas.height = 160;
  const labelContext = labelCanvas.getContext("2d");
  if (!labelContext) {
    throw new Error("Unable to create boost meter label context");
  }

  const labelTexture = new THREE.CanvasTexture(labelCanvas);
  labelTexture.colorSpace = THREE.SRGBColorSpace;
  labelTexture.needsUpdate = true;

  const labelGeometry = new THREE.PlaneGeometry(190, 48);
  const labelMaterial = new THREE.MeshBasicMaterial({
    map: labelTexture,
    transparent: true,
    depthWrite: false,
    side: THREE.DoubleSide,
  });
  const labelMesh = new THREE.Mesh(labelGeometry, labelMaterial);
  labelMesh.position.set(0, 15, 0);
  group.add(labelMesh);

  return {
    group,
    fillMesh,
    fillMaterial,
    labelTexture,
    labelContext,
    labelCanvas,
    lastPercent: null,
  };
}

export function updateBoostMeter(
  meter: BoostMeter,
  fraction: number,
  amount: number,
  camera: THREE.Camera,
): void {
  // Scale fill bar horizontally by fraction
  meter.fillMesh.scale.x = Math.max(0.001, fraction);
  // Shift fill so it stays left-aligned: at scale=1 it's centered,
  // so offset = (1 - scale) * halfWidth
  const halfWidth = 94; // barWidth / 2
  meter.fillMesh.position.x = -(1 - fraction) * halfWidth;
  meter.fillMesh.position.y = -18;

  const percent = Math.max(0, Math.min(100, Math.round((amount / 255) * 100)));
  if (meter.lastPercent !== percent) {
    const { labelContext, labelCanvas, labelTexture } = meter;
    labelContext.clearRect(0, 0, labelCanvas.width, labelCanvas.height);
    labelContext.textAlign = "center";
    labelContext.textBaseline = "middle";
    labelContext.lineJoin = "round";

    labelContext.font = "700 84px sans-serif";
    labelContext.lineWidth = 18;
    labelContext.strokeStyle = "rgba(7, 19, 29, 0.92)";
    labelContext.strokeText(`${percent}`, labelCanvas.width / 2, 78);
    labelContext.fillStyle = "#fff8e1";
    labelContext.fillText(`${percent}`, labelCanvas.width / 2, 78);

    labelContext.font = "600 30px sans-serif";
    labelContext.lineWidth = 10;
    labelContext.strokeText("BOOST", labelCanvas.width / 2, 130);
    labelContext.fillStyle = "#ffcf70";
    labelContext.fillText("BOOST", labelCanvas.width / 2, 130);

    labelTexture.needsUpdate = true;
    meter.lastPercent = percent;
  }

  // Billboard: face the camera
  meter.group.quaternion.copy(camera.quaternion);
}

function makeLights(scene: THREE.Scene): void {
  scene.add(new THREE.AmbientLight("#d8ecff", 1.6));

  const keyLight = new THREE.DirectionalLight("#fff6df", 2.4);
  keyLight.position.set(4000, -6000, 5000);
  scene.add(keyLight);

  const fillLight = new THREE.DirectionalLight("#97d7ff", 1.2);
  fillLight.position.set(-5000, 4000, 3000);
  scene.add(fillLight);
}

function makeBallMesh(): THREE.Mesh {
  return new THREE.Mesh(
    new THREE.SphereGeometry(93, 16, 16),
    new THREE.MeshLambertMaterial({ color: 0xffffff })
  );
}

export function createReplayScene(
  container: HTMLElement,
  replay: ReplayModel,
  fieldScale: number
): ReplayScene {
  const scene = new THREE.Scene();
  scene.background = new THREE.Color("#081119");

  const camera = new THREE.PerspectiveCamera(
    48,
    1,
    10 * fieldScale,
    500000 * fieldScale
  );
  camera.up.set(0, 0, 1);
  camera.position.set(0, -9000 * fieldScale, 5000 * fieldScale);
  camera.lookAt(0, 0, 0);

  const renderer = new THREE.WebGLRenderer({ antialias: true });
  renderer.setPixelRatio(window.devicePixelRatio);
  container.replaceChildren(renderer.domElement);

  const controls = new OrbitControls(camera, renderer.domElement);
  controls.enableDamping = true;
  controls.maxDistance = 160000 * fieldScale;
  controls.target.set(0, 0, 600 * fieldScale);
  controls.update();

  const { stadium, wallPanels } = createExampleSoccarField(fieldScale);
  scene.add(stadium);
  makeLights(scene);

  const replayRoot = new THREE.Group();
  replayRoot.scale.set(-fieldScale, fieldScale, fieldScale);
  scene.add(replayRoot);

  const ballMesh = makeBallMesh();
  replayRoot.add(ballMesh);

  const playerMeshes = new Map<string, THREE.Object3D>();
  const playerBoostTrails = new Map<string, THREE.Group>();
  const playerBoostMeters = new Map<string, BoostMeter>();
  for (const player of replay.players) {
    const mesh = createExampleCarMesh(player.isTeamZero ? "#57a8ff" : "#ff9c40");
    const boostTrail = createBoostTrail();
    mesh.add(boostTrail);
    const boostMeter = createBoostMeter();
    mesh.add(boostMeter.group);
    replayRoot.add(mesh);
    playerMeshes.set(player.id, mesh);
    playerBoostTrails.set(player.id, boostTrail);
    playerBoostMeters.set(player.id, boostMeter);
  }

  const resize = (): void => {
    const width = container.clientWidth || 1;
    const height = container.clientHeight || 1;
    camera.aspect = width / height;
    camera.updateProjectionMatrix();
    renderer.setSize(width, height, false);
  };

  resize();

  const wallCenter = new THREE.Vector3();
  const wallNormal = new THREE.Vector3();
  const wallQuaternion = new THREE.Quaternion();
  const wallToCamera = new THREE.Vector3();
  const updateWallVisibility = (): void => {
    scene.updateMatrixWorld(true);
    for (const panel of wallPanels) {
      if (panel.fixedOpacity !== null) {
        panel.material.transparent = true;
        panel.material.opacity = panel.fixedOpacity;
        panel.material.depthWrite = false;
        continue;
      }

      panel.mesh.getWorldPosition(wallCenter);
      panel.mesh.getWorldQuaternion(wallQuaternion);
      wallNormal.copy(panel.outwardLocal).applyQuaternion(wallQuaternion).normalize();
      wallToCamera.copy(camera.position).sub(wallCenter);
      const isOutside = wallNormal.dot(wallToCamera) > 0;
      panel.material.transparent = true;
      panel.material.opacity = isOutside
        ? OUTSIDE_WALL_OPACITY
        : OPAQUE_WALL_OPACITY;
      panel.material.depthWrite = !isOutside;
    }
  };

  const dispose = (): void => {
    controls.dispose();
    renderer.dispose();
    container.replaceChildren();
  };

  return {
    scene,
    replayRoot,
    camera,
    renderer,
    controls,
    resize,
    dispose,
    ballMesh,
    playerMeshes,
    playerBoostTrails,
    playerBoostMeters,
    updateWallVisibility,
  };
}
