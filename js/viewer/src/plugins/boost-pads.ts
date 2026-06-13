/**
 * createBoostPadsPlugin — boost pad rendering, ported from the original ballcam
 * GameEngine (`createBoostPads` / `updateBoostPads` / `dispose`, commit
 * e7686161). The mesh construction, materials, glow layers, point lights, and
 * availability styling are the original code, restructured onto the
 * ViewerPlugin hooks and fed by the adapter's `boostPads` map (which carries
 * subtr-actor's resolved pad layout + exact pickup/availability events) instead
 * of framework's Player.
 */
import * as THREE from "three";
import type { ViewerPlugin, ViewerPluginContext, ViewerRenderContext } from "../types.js";

type PadMesh = THREE.Mesh<THREE.BufferGeometry, THREE.MeshStandardMaterial>;

export function createBoostPadsPlugin(): ViewerPlugin {
  let boostPadMeshes = new Map<number, PadMesh>();

  function createBoostPads(ctx: ViewerPluginContext): void {
    const boostPads = ctx.player.adapter.boostPads;
    if (!boostPads || boostPads.size === 0) return;

    console.log(`[boost-pads] Creating ${boostPads.size} boost pads...`);

    boostPadMeshes = new Map();

    boostPads.forEach((pad, padId) => {
      const isBig = pad.isBig;

      let geometry: THREE.BufferGeometry;
      let material: THREE.MeshStandardMaterial;
      let mesh: PadMesh;

      if (isBig) {
        // Big pads: Glowing sphere (dimensions in Unreal Units)
        // Big boost pads in RL are ~144 UU diameter, we use a smaller visual sphere
        const radius = 50; // Visual sphere radius in UU
        geometry = new THREE.SphereGeometry(radius, 16, 16);
        material = new THREE.MeshStandardMaterial({
          color: 0xffdd44, // Bright yellow/orange core
          emissive: 0xffaa00,
          emissiveIntensity: 1.0,
          metalness: 0.3,
          roughness: 0.2,
          transparent: true, // CRITICAL: Required for opacity changes
          opacity: 1.0,
          depthWrite: false, // Fix transparency sorting with arena walls
        });
        mesh = new THREE.Mesh(geometry, material);
        mesh.renderOrder = 100; // Render after arena walls

        // Add glow effect - larger transparent sphere with additive blending
        const glowGeometry = new THREE.SphereGeometry(radius * 2.0, 16, 16);
        const glowMaterial = new THREE.MeshBasicMaterial({
          color: 0xffaa00,
          transparent: true,
          opacity: 0.3,
          blending: THREE.AdditiveBlending,
          side: THREE.BackSide, // Render inside of sphere for halo effect
          depthWrite: false,
        });
        const glowMesh = new THREE.Mesh(glowGeometry, glowMaterial);
        glowMesh.renderOrder = 99;
        mesh.add(glowMesh);
        mesh.userData.glowMesh = glowMesh;

        // Add second smaller glow layer for more intensity
        const innerGlowGeometry = new THREE.SphereGeometry(radius * 1.4, 16, 16);
        const innerGlowMaterial = new THREE.MeshBasicMaterial({
          color: 0xffcc00,
          transparent: true,
          opacity: 0.4,
          blending: THREE.AdditiveBlending,
          side: THREE.BackSide,
          depthWrite: false,
        });
        const innerGlowMesh = new THREE.Mesh(innerGlowGeometry, innerGlowMaterial);
        innerGlowMesh.renderOrder = 99;
        mesh.add(innerGlowMesh);
        mesh.userData.innerGlowMesh = innerGlowMesh;

        mesh.userData.needsLight = true;
      } else {
        // Small pads: Flat cylinder at ground level (dimensions in Unreal Units)
        // Small boost pads in RL are ~64 UU diameter
        const radius = 40; // Visual radius in UU
        const height = 5; // Flat disk height in UU
        geometry = new THREE.CylinderGeometry(radius, radius, height, 16);
        material = new THREE.MeshStandardMaterial({
          color: 0xffcc00, // Yellow/orange
          emissive: 0xff9900,
          emissiveIntensity: 0.4,
          metalness: 0.2,
          roughness: 0.8,
          transparent: true, // CRITICAL: Required for opacity changes
          opacity: 1.0,
          depthWrite: false, // Fix transparency sorting with arena walls
        });
        mesh = new THREE.Mesh(geometry, material);
        mesh.renderOrder = 100; // Render after arena walls
      }

      // Position the boost pad (adapter provides positions in UU)
      // The pad data uses Unreal coordinates: x, y (length), z (height)
      // Three.js expects: x, y (height), z (depth)
      // So we need to swap Y and Z here (unlike physics which does it in the compiler)
      const groundLevel = 10; // Just above ground to avoid z-fighting (in UU)
      const floatHeight = isBig ? 150 : groundLevel; // Big pads at 150 UU, small pads near ground

      mesh.position.set(
        pad.position.x, // X stays the same
        floatHeight, // Y = height (use our custom float height)
        pad.position.y, // Z = Unreal Y (position along the field length)
      );

      // Store metadata (preserve existing userData like light reference)
      mesh.userData.padId = padId;
      mesh.userData.isBig = isBig;
      mesh.userData.isAvailable = true;

      ctx.scene.add(mesh);
      boostPadMeshes.set(padId, mesh);

      // Add point light for big pads directly to scene
      if (mesh.userData.needsLight) {
        const light = new THREE.PointLight(0xffaa00, 1.0, 600);
        light.decay = 0; // No decay, but limited by distance
        light.position.set(pad.position.x, floatHeight - 50, pad.position.y);
        ctx.scene.add(light);
        mesh.userData.light = light;
      }
    });

    console.log(`[boost-pads] ✓ Created ${boostPadMeshes.size} boost pad meshes`);
  }

  function updateBoostPads(ctx: ViewerRenderContext): void {
    const boostPads = ctx.player.adapter.boostPads;

    boostPads.forEach((pad, padId) => {
      const mesh = boostPadMeshes.get(padId);
      if (!mesh) return;

      // Update visibility and color based on availability
      const isAvailable = pad.isAvailable;
      if (mesh.userData.isAvailable === isAvailable) return;
      mesh.userData.isAvailable = isAvailable;

      if (isAvailable) {
        // Available - orange/yellow (normal color)
        mesh.material.color.setHex(pad.isBig ? 0xffdd44 : 0xffcc00);
        mesh.material.emissive.setHex(pad.isBig ? 0xffaa00 : 0xff9900);
        mesh.material.emissiveIntensity = pad.isBig ? 1.0 : 0.4;
        mesh.material.opacity = 1.0;
        mesh.visible = true;
        // Turn on light and glow for big pads
        if (mesh.userData.light) {
          mesh.userData.light.intensity = 1.0;
        }
        if (mesh.userData.glowMesh) {
          mesh.userData.glowMesh.visible = true;
        }
        if (mesh.userData.innerGlowMesh) {
          mesh.userData.innerGlowMesh.visible = true;
        }
      } else {
        // Picked up - transparent (faded out)
        mesh.material.color.setHex(pad.isBig ? 0xffaa00 : 0xffcc00);
        mesh.material.emissive.setHex(0x000000); // No emission when inactive
        mesh.material.emissiveIntensity = 0.0;
        mesh.material.opacity = 0.2; // Very transparent
        mesh.visible = true; // Still visible but faded
        // Turn off light and glow for big pads
        if (mesh.userData.light) {
          mesh.userData.light.intensity = 0;
        }
        if (mesh.userData.glowMesh) {
          mesh.userData.glowMesh.visible = false;
        }
        if (mesh.userData.innerGlowMesh) {
          mesh.userData.innerGlowMesh.visible = false;
        }
      }
    });
  }

  return {
    id: "boost-pads",

    setup(ctx) {
      createBoostPads(ctx);
    },

    beforeRender(ctx) {
      updateBoostPads(ctx);
    },

    teardown(ctx) {
      // Cleanup boost pad meshes (original GameEngine.dispose, plus the glow
      // children and per-pad lights it leaked)
      boostPadMeshes.forEach((mesh) => {
        ctx.scene.remove(mesh);
        mesh.geometry.dispose();
        mesh.material.dispose();
        for (const key of ["glowMesh", "innerGlowMesh"] as const) {
          const glow = mesh.userData[key] as
            | THREE.Mesh<THREE.BufferGeometry, THREE.MeshBasicMaterial>
            | undefined;
          if (glow) {
            glow.geometry.dispose();
            glow.material.dispose();
          }
        }
        const light = mesh.userData.light as THREE.PointLight | undefined;
        if (light) {
          ctx.scene.remove(light);
          light.dispose();
        }
      });
      boostPadMeshes.clear();
    },
  };
}
