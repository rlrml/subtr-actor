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

const BOOST_PAD_CHILD_MESH_KEYS = [
  "baseGroup",
  "glowMesh",
  "innerGlowMesh",
  "lensColumnMesh",
  "lensRimMesh",
  "topGlowMesh",
  "coreGlowMesh",
  "highlightMesh",
] as const;

function createHorizontalGlowDisk(
  radius: number,
  color: number,
  opacity: number,
  renderOrder: number,
): THREE.Mesh<THREE.CircleGeometry, THREE.MeshBasicMaterial> {
  const mesh = new THREE.Mesh(
    new THREE.CircleGeometry(radius, 32),
    new THREE.MeshBasicMaterial({
      color,
      transparent: true,
      opacity,
      blending: THREE.AdditiveBlending,
      side: THREE.DoubleSide,
      depthWrite: false,
    }),
  );
  mesh.rotation.x = -Math.PI / 2;
  mesh.renderOrder = renderOrder;
  return mesh;
}

function setBasicGroupOpacity(group: THREE.Object3D | undefined, opacityScale: number): void {
  if (!group) {
    return;
  }
  group.traverse((child) => {
    const mesh = child as THREE.Mesh<THREE.BufferGeometry, THREE.MeshBasicMaterial>;
    if (!mesh.isMesh || !(mesh.material instanceof THREE.MeshBasicMaterial)) {
      return;
    }
    const baseOpacity = mesh.userData.baseOpacity as number | undefined;
    mesh.material.opacity = (baseOpacity ?? mesh.material.opacity) * opacityScale;
  });
}

function configureBigBaseMesh(
  mesh: THREE.Mesh<THREE.BufferGeometry, THREE.MeshBasicMaterial>,
  opacity: number,
  renderOrder: number,
): void {
  mesh.rotation.x = -Math.PI / 2;
  mesh.renderOrder = renderOrder;
  mesh.frustumCulled = false;
  mesh.userData.baseOpacity = opacity;
  mesh.material.transparent = true;
  mesh.material.opacity = opacity;
  mesh.material.side = THREE.DoubleSide;
  mesh.material.depthWrite = false;
}

function createBigBoostBase(radius: number): THREE.Group {
  const group = new THREE.Group();
  group.renderOrder = 98;
  group.frustumCulled = false;

  const darkMaterial = new THREE.MeshBasicMaterial({ color: 0x11110d });
  const panelMaterial = new THREE.MeshBasicMaterial({
    color: 0xffa000,
    blending: THREE.AdditiveBlending,
  });

  const center = new THREE.Mesh(new THREE.CircleGeometry(radius * 0.55, 48), darkMaterial.clone());
  configureBigBaseMesh(center, 0.86, 98);
  group.add(center);

  const centerRing = new THREE.Mesh(
    new THREE.RingGeometry(radius * 0.45, radius * 0.62, 48),
    new THREE.MeshBasicMaterial({
      color: 0xffd13a,
      blending: THREE.AdditiveBlending,
    }),
  );
  configureBigBaseMesh(centerRing, 0.78, 100);
  centerRing.position.y = 1.4;
  group.add(centerRing);

  function createArmShape(
    innerRadius: number,
    outerRadius: number,
    halfAngle: number,
  ): THREE.Shape {
    const shape = new THREE.Shape();
    const points: Array<[number, number]> = [
      [innerRadius * Math.cos(-halfAngle * 0.72), innerRadius * Math.sin(-halfAngle * 0.72)],
      [outerRadius * Math.cos(-halfAngle), outerRadius * Math.sin(-halfAngle)],
      [outerRadius * Math.cos(halfAngle), outerRadius * Math.sin(halfAngle)],
      [innerRadius * Math.cos(halfAngle * 0.72), innerRadius * Math.sin(halfAngle * 0.72)],
    ];
    points.forEach(([x, z], index) => {
      if (index === 0) {
        shape.moveTo(x, z);
      } else {
        shape.lineTo(x, z);
      }
    });
    shape.closePath();
    return shape;
  }

  for (let index = 0; index < 3; index += 1) {
    const rotation = (index * (Math.PI * 2)) / 3 + Math.PI / 2;
    const arm = new THREE.Mesh(
      new THREE.ShapeGeometry(createArmShape(radius * 0.52, radius * 1.42, 0.33)),
      darkMaterial.clone(),
    );
    configureBigBaseMesh(arm, 0.86, 98);
    arm.rotation.z = rotation;
    group.add(arm);

    const panel = new THREE.Mesh(
      new THREE.ShapeGeometry(createArmShape(radius * 0.66, radius * 1.2, 0.21)),
      panelMaterial.clone(),
    );
    configureBigBaseMesh(panel, 0.86, 99);
    panel.position.y = 1.1;
    panel.rotation.z = rotation;
    group.add(panel);
  }

  return group;
}

function setChildMeshAvailability(mesh: PadMesh, available: boolean): void {
  for (const key of BOOST_PAD_CHILD_MESH_KEYS) {
    const child = mesh.userData[key] as
      | THREE.Mesh<THREE.BufferGeometry, THREE.MeshBasicMaterial>
      | THREE.Group
      | undefined;
    if (!child) {
      continue;
    }
    child.visible = available;
  }
}

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
        // Big pads: translucent golden lens over a dark/gold base.
        const radius = 37; // Visual sphere radius in UU
        geometry = new THREE.SphereGeometry(radius, 24, 18);
        material = new THREE.MeshPhysicalMaterial({
          color: 0xffb21a,
          emissive: 0xff8a00,
          emissiveIntensity: 0.42,
          metalness: 0.04,
          roughness: 0.08,
          clearcoat: 1.0,
          clearcoatRoughness: 0.025,
          transmission: 0.18,
          thickness: 30,
          ior: 1.42,
          envMapIntensity: 1.9,
          blending: THREE.AdditiveBlending,
          transparent: true, // CRITICAL: Required for opacity changes
          opacity: 0.68,
          depthWrite: false, // Fix transparency sorting with arena walls
        });
        mesh = new THREE.Mesh(geometry, material);
        mesh.renderOrder = 100; // Render after arena walls

        const baseGroup = createBigBoostBase(radius * 2.05);
        baseGroup.position.y = -140;
        mesh.add(baseGroup);
        mesh.userData.baseGroup = baseGroup;

        const lensColumnMesh = new THREE.Mesh(
          new THREE.CylinderGeometry(radius * 0.12, radius * 0.18, 112, 24, 1, true),
          new THREE.MeshBasicMaterial({
            color: 0xffc340,
            transparent: true,
            opacity: 0.28,
            blending: THREE.AdditiveBlending,
            side: THREE.DoubleSide,
            depthWrite: false,
          }),
        );
        lensColumnMesh.position.y = -62;
        lensColumnMesh.renderOrder = 99;
        mesh.add(lensColumnMesh);
        mesh.userData.lensColumnMesh = lensColumnMesh;

        const lensRimMesh = new THREE.Mesh(
          new THREE.SphereGeometry(radius * 1.03, 24, 14),
          new THREE.MeshBasicMaterial({
            color: 0xffdf7a,
            transparent: true,
            opacity: 0.32,
            blending: THREE.AdditiveBlending,
            side: THREE.BackSide,
            depthWrite: false,
          }),
        );
        lensRimMesh.renderOrder = 101;
        mesh.add(lensRimMesh);
        mesh.userData.lensRimMesh = lensRimMesh;

        // Add glow effect - larger transparent sphere with additive blending
        const glowGeometry = new THREE.SphereGeometry(radius * 1.3, 20, 14);
        const glowMaterial = new THREE.MeshBasicMaterial({
          color: 0xffb62b,
          transparent: true,
          opacity: 0.16,
          blending: THREE.AdditiveBlending,
          side: THREE.BackSide, // Render inside of sphere for halo effect
          depthWrite: false,
        });
        const glowMesh = new THREE.Mesh(glowGeometry, glowMaterial);
        glowMesh.renderOrder = 99;
        mesh.add(glowMesh);
        mesh.userData.glowMesh = glowMesh;

        // Add second smaller glow layer for more intensity
        const innerGlowGeometry = new THREE.SphereGeometry(radius * 1.12, 20, 14);
        const innerGlowMaterial = new THREE.MeshBasicMaterial({
          color: 0xffc12a,
          transparent: true,
          opacity: 0.22,
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
        const radius = 45; // Visual radius in UU
        const height = 8; // Low reflective puck height in UU
        geometry = new THREE.CylinderGeometry(radius, radius * 0.92, height, 32);
        material = new THREE.MeshPhysicalMaterial({
          color: 0xffc21a,
          emissive: 0xff9700,
          emissiveIntensity: 0.72,
          metalness: 0.88,
          roughness: 0.14,
          clearcoat: 1.0,
          clearcoatRoughness: 0.05,
          envMapIntensity: 2.0,
          transparent: true, // CRITICAL: Required for opacity changes
          opacity: 1.0,
          depthWrite: false, // Fix transparency sorting with arena walls
        });
        mesh = new THREE.Mesh(geometry, material);
        mesh.renderOrder = 100; // Render after arena walls

        const topGlowMesh = createHorizontalGlowDisk(radius * 1.42, 0xffb000, 0.34, 101);
        topGlowMesh.position.y = height / 2 + 0.15;
        mesh.add(topGlowMesh);
        mesh.userData.topGlowMesh = topGlowMesh;

        const coreGlowMesh = createHorizontalGlowDisk(radius * 0.74, 0xffff9a, 0.42, 102);
        coreGlowMesh.position.y = height / 2 + 0.35;
        mesh.add(coreGlowMesh);
        mesh.userData.coreGlowMesh = coreGlowMesh;

        const highlightMesh = createHorizontalGlowDisk(radius * 0.42, 0xfff8d0, 0.46, 103);
        highlightMesh.position.set(-radius * 0.18, height / 2 + 0.55, -radius * 0.12);
        highlightMesh.scale.y = 0.34;
        mesh.add(highlightMesh);
        mesh.userData.highlightMesh = highlightMesh;
      }

      // Position the boost pad (adapter provides positions in UU)
      // The pad data uses Unreal coordinates: x, y (length), z (height)
      // Three.js expects: x, y (height), z (depth)
      // So we need to swap Y and Z here (unlike physics which does it in the compiler)
      const groundLevel = 10; // Just above ground to avoid z-fighting (in UU)
      const floatHeight = isBig ? 130 : groundLevel; // Big pads at 130 UU, small pads near ground

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
        const light = new THREE.PointLight(0xff9d00, 0.7, 480);
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
        mesh.material.color.setHex(pad.isBig ? 0xffb21a : 0xffc21a);
        mesh.material.emissive.setHex(pad.isBig ? 0xff8a00 : 0xff9700);
        mesh.material.emissiveIntensity = pad.isBig ? 0.42 : 0.72;
        mesh.material.opacity = pad.isBig ? 0.68 : 1.0;
        mesh.visible = true;
        setChildMeshAvailability(mesh, true);
        setBasicGroupOpacity(mesh.userData.baseGroup as THREE.Group | undefined, 1);
        // Turn on light and glow for big pads
        if (mesh.userData.light) {
          mesh.userData.light.intensity = 0.85;
        }
        if (mesh.userData.glowMesh) {
          mesh.userData.glowMesh.visible = true;
        }
        if (mesh.userData.innerGlowMesh) {
          mesh.userData.innerGlowMesh.visible = true;
        }
      } else {
        // Picked up - transparent (faded out)
        mesh.material.color.setHex(pad.isBig ? 0x8a4c00 : 0x8a5400);
        mesh.material.emissive.setHex(0x000000); // No emission when inactive
        mesh.material.emissiveIntensity = 0.0;
        mesh.material.opacity = 0.2; // Very transparent
        mesh.visible = true; // Still visible but faded
        setChildMeshAvailability(mesh, false);
        if (mesh.userData.baseGroup) {
          mesh.userData.baseGroup.visible = true;
          setBasicGroupOpacity(mesh.userData.baseGroup as THREE.Group, 0.26);
        }
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
        for (const key of BOOST_PAD_CHILD_MESH_KEYS) {
          const glow = mesh.userData[key] as
            | THREE.Mesh<THREE.BufferGeometry, THREE.MeshBasicMaterial>
            | THREE.Group
            | undefined;
          if (glow) {
            glow.traverse((child) => {
              const childMesh = child as THREE.Mesh<THREE.BufferGeometry, THREE.MeshBasicMaterial>;
              if (!childMesh.isMesh) {
                return;
              }
              childMesh.geometry.dispose();
              childMesh.material.dispose();
            });
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
