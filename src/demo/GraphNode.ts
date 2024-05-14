import * as THREE from 'three';

export interface GraphNodeStatistics {
  betweenness: number;
}

export class GraphNode extends THREE.Mesh {
  uniqueId: string;
  nodeData: object;

  constructor(
    uniqueId: string,
    nodeData: Object,
    radius: number,
    statistics: GraphNodeStatistics
  ) {
    const geometry = new THREE.SphereGeometry(radius);
    const material = new THREE.MeshStandardMaterial({
      color: 0x222222,
      emissive: 0xff0000,
    });

    super(geometry, material);

    this.uniqueId = uniqueId;
    this.nodeData = nodeData;
  }
}
