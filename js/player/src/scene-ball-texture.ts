import * as THREE from "three";

const BALL_TEXTURE_SIZE = 1024;

function drawBallLatitudeBand(
  context: CanvasRenderingContext2D,
  width: number,
  height: number,
  normalizedY: number,
  amplitude: number,
  phase: number,
  lineWidth: number,
  color: string,
): void {
  context.beginPath();
  for (let x = 0; x <= width; x += 8) {
    const t = x / width;
    const y =
      normalizedY * height +
      Math.sin(t * Math.PI * 2 + phase) * amplitude +
      Math.sin(t * Math.PI * 4 + phase * 0.5) * amplitude * 0.35;
    if (x === 0) {
      context.moveTo(x, y);
    } else {
      context.lineTo(x, y);
    }
  }
  context.lineWidth = lineWidth;
  context.strokeStyle = color;
  context.stroke();
}

function drawBallLongitudeBand(
  context: CanvasRenderingContext2D,
  width: number,
  height: number,
  normalizedX: number,
  amplitude: number,
  phase: number,
  lineWidth: number,
  color: string,
): void {
  context.beginPath();
  for (let y = 0; y <= height; y += 8) {
    const t = y / height;
    const x =
      normalizedX * width +
      Math.sin(t * Math.PI * 2 + phase) * amplitude +
      Math.sin(t * Math.PI * 6 + phase * 0.3) * amplitude * 0.18;
    if (y === 0) {
      context.moveTo(x, y);
    } else {
      context.lineTo(x, y);
    }
  }
  context.lineWidth = lineWidth;
  context.strokeStyle = color;
  context.stroke();
}

function drawBallMarker(
  context: CanvasRenderingContext2D,
  centerX: number,
  centerY: number,
  radius: number,
  fillStyle: string,
  strokeStyle: string,
): void {
  context.beginPath();
  context.arc(centerX, centerY, radius, 0, Math.PI * 2);
  context.fillStyle = fillStyle;
  context.fill();
  context.lineWidth = Math.max(6, radius * 0.15);
  context.strokeStyle = strokeStyle;
  context.stroke();
}

export function createBallTexture(renderer: THREE.WebGLRenderer): THREE.CanvasTexture {
  const canvas = document.createElement("canvas");
  canvas.width = BALL_TEXTURE_SIZE;
  canvas.height = BALL_TEXTURE_SIZE;
  const context = canvas.getContext("2d");
  if (!context) {
    throw new Error("Unable to create ball texture canvas");
  }

  const { width, height } = canvas;
  const background = context.createLinearGradient(0, 0, width, height);
  background.addColorStop(0, "#faf7ee");
  background.addColorStop(0.55, "#e7e1d0");
  background.addColorStop(1, "#d5cfbe");
  context.fillStyle = background;
  context.fillRect(0, 0, width, height);

  context.globalAlpha = 0.22;
  for (let row = 0; row < 28; row += 1) {
    const y = (row / 27) * height;
    context.fillStyle = row % 2 === 0 ? "#ffffff" : "#d3cbb6";
    context.fillRect(0, y, width, height / 54);
  }
  context.globalAlpha = 1;

  const seamColor = "#2d313b";
  context.lineCap = "round";
  drawBallLatitudeBand(context, width, height, 0.24, 22, 0.35, 18, seamColor);
  drawBallLatitudeBand(context, width, height, 0.5, 14, 1.1, 20, seamColor);
  drawBallLatitudeBand(context, width, height, 0.77, 20, 2.35, 18, seamColor);
  drawBallLongitudeBand(context, width, height, 0.2, 24, 0.2, 18, seamColor);
  drawBallLongitudeBand(context, width, height, 0.48, 18, 1.6, 18, seamColor);
  drawBallLongitudeBand(context, width, height, 0.76, 26, 2.7, 18, seamColor);

  context.globalAlpha = 0.92;
  drawBallMarker(context, width * 0.28, height * 0.32, 88, "#f1a63a", "#fff4d7");
  drawBallMarker(context, width * 0.68, height * 0.6, 72, "#4db0ff", "#eef8ff");
  drawBallMarker(context, width * 0.76, height * 0.2, 54, "#1f232c", "#f0ece1");
  context.globalAlpha = 1;

  context.beginPath();
  context.moveTo(width * 0.08, height * 0.86);
  context.quadraticCurveTo(width * 0.28, height * 0.72, width * 0.42, height * 0.8);
  context.quadraticCurveTo(width * 0.58, height * 0.9, width * 0.82, height * 0.78);
  context.lineWidth = 24;
  context.strokeStyle = "rgba(255, 246, 220, 0.9)";
  context.stroke();

  const texture = new THREE.CanvasTexture(canvas);
  texture.colorSpace = THREE.SRGBColorSpace;
  texture.anisotropy = Math.min(8, renderer.capabilities.getMaxAnisotropy());
  return texture;
}
