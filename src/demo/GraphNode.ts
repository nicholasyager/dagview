import * as THREE from 'three';
import { DbtNode } from '../client/local';

export interface GraphNodeStatistics {
  betweenness: number;
}

export class GraphNode extends THREE.Mesh {
  uniqueId: string;
  nodeData: DbtNode;

  selected: boolean;

  constructor(
    uniqueId: string,
    nodeData: DbtNode,
    radius: number,
    color: THREE.Color,
    statistics: GraphNodeStatistics
  ) {
    let geometry = undefined;
    if (nodeData.resource_type == 'model' || nodeData.resource_type == 'seed') {
      geometry = new THREE.SphereGeometry(radius);
    } else {
      geometry = new THREE.ConeGeometry(radius, radius);
      color = new THREE.Color(0xffffff);
    }

    const material = new THREE.MeshPhysicalMaterial({
      color: color.getHex(),
      emissive: color.getHex(),
      emissiveIntensity: 0.5,
    });

    super(geometry, material);
    this.name = nodeData['unique_id'];
    this.castShadow = true; //default is false
    this.receiveShadow = true; //default
    this.selected = false;

    this.uniqueId = uniqueId;
    this.nodeData = nodeData;
  }

  select() {
    this.selected = true;

    this.material.setValues({ emissiveIntensity: 1 });
  }

  deselect() {
    this.selected = true;
    this.material.setValues({ emissiveIntensity: 0.5 });
  }
}
