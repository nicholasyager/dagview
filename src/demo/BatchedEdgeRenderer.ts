import * as THREE from 'three';
import * as d3 from 'd3';
import { Engine } from '../engine/Engine';
import { GraphNode } from './GraphNode';
import { GraphEdge2 } from './GraphEdge';

export interface EdgeDef {
  id: string;
  pathObjects: GraphNode[];
  color: THREE.Color;
}

export interface EdgeRecord {
  id: string;
  source: GraphNode;
  target: GraphNode;
  pathObjects: GraphNode[];
  color: THREE.Color;
  vertexOffset: number;
  vertexCount: number;
  dimmed: boolean;
  selected: boolean;
  baseOpacity: number;
}

const CURVE_POINTS = 20;
const SEGMENTS_PER_EDGE = CURVE_POINTS;
const VERTS_PER_EDGE = SEGMENTS_PER_EDGE * 2;

// Pre-compute once — same basis values every frame
const opacityInterp = d3.interpolateBasis([0.01, 0.01, 0.35]);

export class BatchedEdgeRenderer {
  mesh: THREE.LineSegments;
  edges: Map<string, EdgeRecord> = new Map();
  edgesByNode: Map<number, EdgeRecord[]> = new Map();
  activeSelections: Map<string, GraphEdge2> = new Map();

  private alphaAttr!: THREE.BufferAttribute;

  constructor() {
    this.mesh = new THREE.LineSegments(
      new THREE.BufferGeometry(),
      this.createMaterial()
    );
    this.mesh.frustumCulled = false;
  }

  private createMaterial(): THREE.ShaderMaterial {
    return new THREE.ShaderMaterial({
      vertexShader: `
        attribute float alpha;
        varying vec3 vColor;
        varying float vAlpha;
        void main() {
          vColor = color;
          vAlpha = alpha;
          gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
        }
      `,
      fragmentShader: `
        varying vec3 vColor;
        varying float vAlpha;
        void main() {
          if (vAlpha < 0.001) discard;
          gl_FragColor = vec4(vColor, vAlpha);
        }
      `,
      transparent: true,
      depthWrite: false,
      vertexColors: true,
    });
  }

  build(edgeDefs: EdgeDef[]): void {
    const totalVerts = edgeDefs.length * VERTS_PER_EDGE;
    const positions = new Float32Array(totalVerts * 3);
    const colors = new Float32Array(totalVerts * 3);
    const alphas = new Float32Array(totalVerts);

    let offset = 0;

    for (const def of edgeDefs) {
      const source = def.pathObjects[0];
      const target = def.pathObjects[def.pathObjects.length - 1];

      const curve = new THREE.CatmullRomCurve3(
        def.pathObjects.map((n) => n.position)
      );
      const pts = curve.getPoints(CURVE_POINTS);

      // Darken like GraphEdge2: color.lerp(black, 0.8)
      const edgeColor = def.color.clone().lerp(new THREE.Color(0x000000), 0.8);

      const vertexOffset = offset;

      for (let i = 0; i < SEGMENTS_PER_EDGE; i++) {
        const v0 = offset;
        const v1 = offset + 1;

        positions[v0 * 3] = pts[i].x;
        positions[v0 * 3 + 1] = pts[i].y;
        positions[v0 * 3 + 2] = pts[i].z;

        positions[v1 * 3] = pts[i + 1].x;
        positions[v1 * 3 + 1] = pts[i + 1].y;
        positions[v1 * 3 + 2] = pts[i + 1].z;

        colors[v0 * 3] = edgeColor.r;
        colors[v0 * 3 + 1] = edgeColor.g;
        colors[v0 * 3 + 2] = edgeColor.b;

        colors[v1 * 3] = edgeColor.r;
        colors[v1 * 3 + 1] = edgeColor.g;
        colors[v1 * 3 + 2] = edgeColor.b;

        alphas[v0] = 0.075;
        alphas[v1] = 0.075;

        offset += 2;
      }

      const vertexCount = offset - vertexOffset;

      const record: EdgeRecord = {
        id: def.id,
        source,
        target,
        pathObjects: def.pathObjects,
        color: def.color,
        vertexOffset,
        vertexCount,
        dimmed: false,
        selected: false,
        baseOpacity: 0.075,
      };

      this.edges.set(def.id, record);

      for (const node of [source, target]) {
        let list = this.edgesByNode.get(node.id);
        if (!list) {
          list = [];
          this.edgesByNode.set(node.id, list);
        }
        list.push(record);
      }
    }

    const geometry = new THREE.BufferGeometry();
    const posAttr = new THREE.BufferAttribute(positions, 3);
    const colorAttr = new THREE.BufferAttribute(colors, 3);
    this.alphaAttr = new THREE.BufferAttribute(alphas, 1);

    geometry.setAttribute('position', posAttr);
    geometry.setAttribute('color', colorAttr);
    geometry.setAttribute('alpha', this.alphaAttr);

    this.mesh.geometry.dispose();
    this.mesh.geometry = geometry;
  }

  updateOpacities(engine: Engine): void {
    const zoom = engine.camera.instance.position.distanceTo(
      engine.camera.controls.target
    );
    const cameraTarget = engine.camera.controls.target;
    const alphaArr = this.alphaAttr.array as Float32Array;

    // Cache per-node visibility and distance — avoids redundant work
    // when many edges share the same endpoint nodes.
    const nodeVisibility = new Map<number, boolean>();
    const nodeDistance = new Map<number, number>();

    const isNodeVisible = (node: GraphNode): boolean => {
      let v = nodeVisibility.get(node.id);
      if (v === undefined) {
        v = engine.raycaster.isSeen(node);
        nodeVisibility.set(node.id, v);
      }
      return v;
    };

    const getNodeDistance = (node: GraphNode): number => {
      let d = nodeDistance.get(node.id);
      if (d === undefined) {
        d = node.position.distanceTo(cameraTarget);
        nodeDistance.set(node.id, d);
      }
      return d;
    };

    for (const record of this.edges.values()) {
      if (record.selected) continue;

      const distance = Math.min(
        getNodeDistance(record.source),
        getNodeDistance(record.target)
      );

      let percentDistance = zoom / (distance + 0.0001);
      if (percentDistance > 1) percentDistance = 1;

      let lineOpacity = opacityInterp(percentDistance);

      if (record.dimmed) {
        lineOpacity = lineOpacity / zoom;
      }

      if (!isNodeVisible(record.source) && !isNodeVisible(record.target)) {
        lineOpacity = 0;
      }

      record.baseOpacity = lineOpacity;

      alphaArr.fill(
        lineOpacity,
        record.vertexOffset,
        record.vertexOffset + record.vertexCount
      );
    }

    this.alphaAttr.needsUpdate = true;
  }

  getVisibleNodes(threshold: number): Set<GraphNode> {
    const visible = new Set<GraphNode>();
    for (const record of this.edges.values()) {
      if (record.baseOpacity >= threshold) {
        visible.add(record.source);
        visible.add(record.target);
      }
    }
    return visible;
  }

  selectEdge(id: string, scene: THREE.Scene): GraphEdge2 | null {
    const record = this.edges.get(id);
    if (!record || record.selected) return null;

    record.selected = true;

    // Zero alpha in batch
    const alphaArr = this.alphaAttr.array as Float32Array;
    alphaArr.fill(
      0,
      record.vertexOffset,
      record.vertexOffset + record.vertexCount
    );
    this.alphaAttr.needsUpdate = true;

    // Create standalone edge for particle animation
    const standalone = new GraphEdge2(
      record.id,
      record.pathObjects,
      record.color.clone()
    );
    standalone.select();
    scene.add(standalone);
    this.activeSelections.set(id, standalone);

    return standalone;
  }

  deselectEdge(id: string, scene: THREE.Scene): void {
    const standalone = this.activeSelections.get(id);
    if (standalone) {
      standalone.deselect();
      scene.remove(standalone);
      this.activeSelections.delete(id);
    }

    const record = this.edges.get(id);
    if (record) {
      record.selected = false;
    }
  }

  deselectAll(scene: THREE.Scene): void {
    for (const [id, standalone] of this.activeSelections) {
      standalone.deselect();
      scene.remove(standalone);
      const record = this.edges.get(id);
      if (record) record.selected = false;
    }
    this.activeSelections.clear();
  }

  dimAll(): void {
    for (const record of this.edges.values()) {
      record.dimmed = true;
    }
  }

  dedimAll(): void {
    for (const record of this.edges.values()) {
      record.dimmed = false;
    }
  }

  dedimEdge(id: string): void {
    const record = this.edges.get(id);
    if (record) record.dimmed = false;
  }

  getEdgesForNode(node: GraphNode): EdgeRecord[] {
    return this.edgesByNode.get(node.id) || [];
  }
}
