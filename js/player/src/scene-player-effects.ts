import * as THREE from "three";

export interface BoostMeter {
  group: THREE.Group;
  fillMesh: THREE.Mesh;
  fillMaterial: THREE.MeshBasicMaterial;
  labelTexture: THREE.CanvasTexture;
  labelContext: CanvasRenderingContext2D;
  labelCanvas: HTMLCanvasElement;
  lastPercent: number | null;
}

export interface DemoIndicator {
  group: THREE.Group;
  ring: THREE.Mesh;
  label: THREE.Mesh;
}

export function createBoostTrail(): THREE.Group {
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

export function createBoostMeter(): BoostMeter {
  const group = new THREE.Group();
  group.visible = false;
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

export function createDemoIndicator(): DemoIndicator {
  const group = new THREE.Group();
  group.visible = false;

  const ringMaterial = new THREE.MeshBasicMaterial({
    color: 0xffd15c,
    transparent: true,
    opacity: 0.86,
    depthWrite: false,
  });
  const ring = new THREE.Mesh(new THREE.TorusGeometry(170, 8, 8, 48), ringMaterial);
  ring.position.z = 16;
  group.add(ring);

  const canvas = document.createElement("canvas");
  canvas.width = 512;
  canvas.height = 192;
  const context = canvas.getContext("2d");
  if (!context) {
    throw new Error("Unable to create demo indicator label context");
  }

  context.textAlign = "center";
  context.textBaseline = "middle";
  context.lineJoin = "round";
  context.font = "800 86px sans-serif";
  context.lineWidth = 20;
  context.strokeStyle = "rgba(7, 19, 29, 0.94)";
  context.strokeText("DEMO", canvas.width / 2, 88);
  context.fillStyle = "#fff0b8";
  context.fillText("DEMO", canvas.width / 2, 88);
  context.font = "700 34px sans-serif";
  context.lineWidth = 10;
  context.strokeText("RESPAWNING", canvas.width / 2, 150);
  context.fillStyle = "#ffbd4a";
  context.fillText("RESPAWNING", canvas.width / 2, 150);

  const texture = new THREE.CanvasTexture(canvas);
  texture.colorSpace = THREE.SRGBColorSpace;
  const labelMaterial = new THREE.MeshBasicMaterial({
    map: texture,
    transparent: true,
    depthWrite: false,
    side: THREE.DoubleSide,
  });
  const label = new THREE.Mesh(new THREE.PlaneGeometry(310, 116), labelMaterial);
  label.position.z = 300;
  group.add(label);

  return { group, ring, label };
}

export function updateBoostMeter(
  meter: BoostMeter,
  fraction: number,
  amount: number,
  camera: THREE.Camera,
): void {
  meter.fillMesh.scale.x = Math.max(0.001, fraction);
  const halfWidth = 94;
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

  meter.group.quaternion.copy(camera.quaternion);
}
