import * as THREE from "three";

export const OPAQUE_WALL_OPACITY = 1;
export const OUTSIDE_WALL_OPACITY = 0.32;

export interface WallPanel {
  mesh: THREE.Mesh;
  material: THREE.Material;
  outwardLocal: THREE.Vector3;
  fixedOpacity: number | null;
}

export function createExampleSoccarField(scale: number): {
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
    fixedOpacity: number | null = null,
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
        createVerticalWallBox(sideWallWidth, SOCCAR_DEPTH, WALL_THICKNESS, wallMaterial),
        new THREE.Vector3(0, 1, 0),
      );
      sideWall.position.set(xSign * (sideWallWidth / 2 + GOAL_WIDTH / 2), 0, SOCCAR_DEPTH / 2);
      backWall.add(sideWall);

      const corner = registerWall(
        createVerticalWallBox(cornerWidth, SOCCAR_DEPTH, WALL_THICKNESS, wallMaterial),
        new THREE.Vector3(0, 1, 0),
      );
      corner.position.set(
        xSign * (SOCCAR_XSIZE / 2 - STADIUM_CORNER / 2),
        -STADIUM_CORNER / 2,
        SOCCAR_DEPTH / 2,
      );
      corner.rotateZ((-xSign * Math.PI) / 4);
      backWall.add(corner);
    }

    const top = registerWall(
      createVerticalWallBox(GOAL_WIDTH, SOCCAR_DEPTH - GOAL_HEIGHT, WALL_THICKNESS, wallMaterial),
      new THREE.Vector3(0, 1, 0),
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
      new THREE.Mesh(new THREE.ShapeGeometry(floor), floorMaterial),
      new THREE.Vector3(0, 0, -1),
    );
    floorMesh.receiveShadow = true;
    res.add(floorMesh);

    for (const xSign of mirroredSigns) {
      const goalPost = registerWall(
        createVerticalWallBox(GOAL_DEPTH, GOAL_HEIGHT, WALL_THICKNESS, wallMaterial),
        new THREE.Vector3(0, -xSign, 0),
        OUTSIDE_WALL_OPACITY,
      );
      goalPost.position.set(
        (xSign * GOAL_WIDTH) / 2,
        SOCCAR_YSIZE / 2 + GOAL_DEPTH / 2,
        GOAL_HEIGHT / 2,
      );
      goalPost.rotateZ(Math.PI / 2);
      res.add(goalPost);
    }

    const goalRoof = registerWall(
      createHorizontalWallBox(GOAL_WIDTH, GOAL_DEPTH, WALL_THICKNESS, wallMaterial),
      new THREE.Vector3(0, 0, 1),
      OUTSIDE_WALL_OPACITY,
    );
    goalRoof.position.set(0, SOCCAR_YSIZE / 2 + GOAL_DEPTH / 2, GOAL_HEIGHT);
    res.add(goalRoof);

    const goalBack = registerWall(
      createVerticalWallBox(GOAL_WIDTH, GOAL_HEIGHT, WALL_THICKNESS, wallMaterial),
      new THREE.Vector3(0, 1, 0),
      OUTSIDE_WALL_OPACITY,
    );
    goalBack.position.set(0, SOCCAR_YSIZE / 2 + GOAL_DEPTH, GOAL_HEIGHT / 2);
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
          wallMaterial,
        ),
        new THREE.Vector3(0, -xSign, 0),
      );
      sideWall.position.set(
        (xSign * SOCCAR_XSIZE) / 2,
        (SOCCAR_YSIZE / 2 - STADIUM_CORNER) / 2,
        SOCCAR_DEPTH / 2,
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
  material: THREE.Material,
): THREE.Mesh {
  return new THREE.Mesh(new THREE.BoxGeometry(length, thickness, height, 6, 1, 6), material);
}

function createHorizontalWallBox(
  width: number,
  depth: number,
  thickness: number,
  material: THREE.Material,
): THREE.Mesh {
  return new THREE.Mesh(new THREE.BoxGeometry(width, depth, thickness, 6, 6, 1), material);
}
