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
    color: THREE.Color,
    statistics: GraphNodeStatistics
  ) {
    const geometry = new THREE.SphereGeometry(radius);
    const material = new THREE.MeshPhysicalMaterial({
      color: color.getHex(),
      emissive: color.getHex(),
      emissiveIntensity: 0.25,
    });

    super(geometry, material);
    this.castShadow = true; //default is false
    this.receiveShadow = true; //default

    this.uniqueId = uniqueId;
    this.nodeData = nodeData;
  }
}
