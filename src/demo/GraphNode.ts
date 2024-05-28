import * as THREE from 'three';
import { DbtNode } from '../client/local';

export interface GraphNodeStatistics {
  betweenness: number;
  degree: number;
}

const MIN_EMISSIVITY = 0.5;

export class GraphNode extends THREE.Mesh {
  uniqueId: string;
  nodeData: DbtNode;

  selected: boolean;
  dimmed: boolean;

  constructor(
    uniqueId: string,
    nodeData: DbtNode,
    radius: number,
    color: THREE.Color,
    statistics: GraphNodeStatistics
  ) {
    let geometry = undefined;
    if (
      nodeData.resource_type == 'model' ||
      nodeData.resource_type == 'seed' ||
      nodeData.resource_type == 'snapshot'
    ) {
      geometry = new THREE.SphereGeometry(radius);
    } else {
      geometry = new THREE.ConeGeometry(radius, radius);
      color = new THREE.Color(0xffffff);
    }

    const material = new THREE.MeshPhysicalMaterial({
      color: color.getHex(),
      emissive: color.getHex(),
      emissiveIntensity: MIN_EMISSIVITY,
    });

    super(geometry, material);
    this.name = nodeData['unique_id'];
    this.castShadow = true; //default is false
    this.receiveShadow = true; //default
    this.selected = false;
    this.dimmed = false;

    this.uniqueId = uniqueId;
    this.nodeData = nodeData;
  }

  select() {
    this.selected = true;

    this.material.setValues({ emissiveIntensity: 1 });
  }

  deselect() {
    this.selected = false;
    this.material.setValues({ emissiveIntensity: MIN_EMISSIVITY });
  }

  dim() {
    this.dimmed = true;
    this.material.setValues({ emissiveIntensity: 0.05 });
  }

  dedim() {
    this.dimmed = false;
    this.material.setValues({ emissiveIntensity: MIN_EMISSIVITY });
  }
}
