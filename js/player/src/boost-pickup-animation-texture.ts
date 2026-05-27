import * as THREE from "three";
import {
  BOOST_PICKUP_TEXT_CANVAS_HEIGHT,
  BOOST_PICKUP_TEXT_CANVAS_WIDTH,
} from "./boost-pickup-animation-constants";

export function createBoostPickupCountTexture(count: number, color: string): THREE.CanvasTexture {
  const canvas = document.createElement("canvas");
  canvas.width = BOOST_PICKUP_TEXT_CANVAS_WIDTH;
  canvas.height = BOOST_PICKUP_TEXT_CANVAS_HEIGHT;
  const context = canvas.getContext("2d");
  if (!context) {
    throw new Error("Unable to create boost pickup count canvas");
  }

  context.clearRect(0, 0, canvas.width, canvas.height);
  context.textAlign = "center";
  context.textBaseline = "middle";
  context.lineJoin = "round";
  context.font = "800 124px sans-serif";
  context.lineWidth = 18;
  context.strokeStyle = "rgba(4, 10, 18, 0.88)";
  context.strokeText(`${count}`, canvas.width / 2, canvas.height / 2);
  context.fillStyle = color;
  context.fillText(`${count}`, canvas.width / 2, canvas.height / 2);

  const texture = new THREE.CanvasTexture(canvas);
  texture.colorSpace = THREE.SRGBColorSpace;
  texture.needsUpdate = true;
  return texture;
}

export function disposeBoostPickupTexture(texture: THREE.Texture | null): void {
  texture?.dispose();
}
