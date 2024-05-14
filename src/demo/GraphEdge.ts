import * as THREE from 'three';

export class GraphEdge extends THREE.Line {
  constructor(source: THREE.Vector3, target: THREE.Vector3) {
    const curve = new THREE.CatmullRomCurve3([source, target]);

    const points = curve.getPoints(20);
    const geometry = new THREE.BufferGeometry().setFromPoints(points);

    const material = new THREE.LineBasicMaterial({
      color: 0x7d7d7d7,
      linewidth: 3,
    });

    super(geometry, material);
  }
}
