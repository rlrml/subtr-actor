import * as THREE from "three";

export function createExampleCarMesh(color: string): THREE.Group {
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
  bodyGeometry.setAttribute("position", new THREE.Float32BufferAttribute(vertices.flat(), 3));
  bodyGeometry.setIndex(faces.flat());
  bodyGeometry.computeVertexNormals();

  const outer = new THREE.Group();
  const inner = new THREE.Group();
  const body = new THREE.Mesh(bodyGeometry, new THREE.MeshLambertMaterial({ color }));
  body.castShadow = true;
  inner.add(body);

  const windowMaterial = new THREE.MeshPhongMaterial({
    color: 0x1a1b2e,
    shininess: 120,
    transparent: true,
    opacity: 0.82,
  });
  const windowVertices = [
    [100, -100, 100],
    [-100, -100, 100],
    [150, -220, 20],
    [-150, -220, 20],
    [100, 100, 100],
    [-100, 100, 100],
    [140, 170, 25],
    [-140, 170, 25],
    [100, -100, 100],
    [100, 100, 100],
    [150, -220, 20],
    [140, 170, 25],
    [-100, -100, 100],
    [-100, 100, 100],
    [-150, -220, 20],
    [-140, 170, 25],
  ];
  const windowFaces = [
    [0, 2, 3],
    [0, 3, 1],
    [4, 6, 7],
    [4, 7, 5],
    [8, 10, 11],
    [8, 11, 9],
    [12, 14, 15],
    [12, 15, 13],
  ];
  const windowGeometry = new THREE.BufferGeometry();
  windowGeometry.setAttribute(
    "position",
    new THREE.Float32BufferAttribute(windowVertices.flat(), 3),
  );
  windowGeometry.setIndex(windowFaces.flat());
  windowGeometry.computeVertexNormals();
  const windowMesh = new THREE.Mesh(windowGeometry, windowMaterial);
  windowMesh.position.z = 1;
  inner.add(windowMesh);

  const windshieldMaterial = new THREE.MeshBasicMaterial({
    color: 0x88d7ff,
    transparent: true,
    opacity: 0.34,
    side: THREE.DoubleSide,
  });
  const windshieldGeometry = new THREE.BufferGeometry();
  windshieldGeometry.setAttribute(
    "position",
    new THREE.Float32BufferAttribute(
      [90, -110, 95, -90, -110, 95, 140, -210, 25, -140, -210, 25],
      3,
    ),
  );
  windshieldGeometry.setIndex([0, 2, 3, 0, 3, 1]);
  windshieldGeometry.computeVertexNormals();
  const windshieldMesh = new THREE.Mesh(windshieldGeometry, windshieldMaterial);
  windshieldMesh.position.z = 2;
  inner.add(windshieldMesh);

  const wheelMaterial = new THREE.MeshPhongMaterial({
    color: 0x222222,
    shininess: 48,
  });
  const makeWheel = (x: number, y: number, z: number, width: number): THREE.Mesh => {
    const wheel = new THREE.Mesh(new THREE.CylinderGeometry(70, 70, width, 10), wheelMaterial);
    wheel.rotateZ(Math.PI / 2);
    wheel.position.set(x, y, z);
    wheel.castShadow = true;
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
