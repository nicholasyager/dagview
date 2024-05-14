import { Engine } from '../engine/Engine';
import * as THREE from 'three';

import { Experience } from '../engine/Experience';
import { Resource } from '../engine/Resources';
import { Manifest } from '../client/local';
import { GraphNode } from './GraphNode';

import createLayout, { Layout } from 'ngraph.forcelayout';
import centrality from 'ngraph.centrality';

import createGraph, { Graph } from 'ngraph.graph';
import { EventedType } from 'ngraph.events';
import { GraphEdge, GraphEdge2 } from './GraphEdge';

import * as d3 from 'd3';

const MAX_ENERGY = 0.1;

function generateInterpolator(
  domain: [number, number],
  range: [number, number]
): (input: number) => number {
  return (input: number) => {
    var percentage = (input - domain[0]) / domain[1];
    return range[0] + (range[1] - range[0]) * percentage;
  };
}

export class Demo implements Experience {
  graph: Graph<any, any> & EventedType;
  layout: Layout<any>;
  nodes: { [key: string]: GraphNode };
  edges: { [key: string]: GraphEdge2 };
  iterations: number;

  resources: Resource[] = [
    {
      name: 'manifest',
      type: 'manifest',
      path: 'assets/manifest.huge.json',
    },
  ];

  constructor(private engine: Engine) {
    this.graph = createGraph();
    this.layout = createLayout(this.graph, {
      dimensions: 3,
      dragCoefficient: 0.99,
      springLength: 0.05,
      gravity: -6,
    });
    this.nodes = {};
    this.edges = {};
    this.iterations = 0;
  }

  init() {
    this.engine.raycaster.on('move', this.handlePointer);

    let manifest: Manifest = this.engine.resources.getItem('manifest');

    for (let [key, value] of Object.entries(manifest.nodes)) {
      if (key.startsWith('test')) continue;
      this.graph.addNode(key, value);
    }

    for (let [source, targets] of Object.entries(manifest.child_map)) {
      if (source.startsWith('test')) continue;
      if (!this.graph.hasNode(source)) continue;

      targets.forEach((target: string) => {
        if (!this.graph.hasNode(target)) return;
        this.graph?.addLink(source, target);
      });
    }

    for (let [target, sources] of Object.entries(manifest.parent_map)) {
      if (target.startsWith('test')) continue;
      sources.forEach((source: string) => {
        if (!this.graph.hasNode(source)) return;
        this.graph?.addLink(source, target);
      });
    }

    // for (var i = 0; i < ITERATIONS_MAX; ++i) {
    var energyHistory = [];
    while (true) {
      this.layout.step();

      energyHistory.push(this.layout.getForceVectorLength());

      let evaluationRange = energyHistory.slice(
        energyHistory.length -
          (energyHistory.length > 5 ? 5 : energyHistory.length)
      );
      let latestEnergyChange = evaluationRange
        .slice(1)
        .map((value, index) => value - evaluationRange[index]);
      // const percentChange = (endingEnergy - startingEnergy) / startingEnergy

      let meanForceDiff =
        latestEnergyChange.reduce((acc, value) => acc + value, 0) /
        latestEnergyChange.length;

      console.log({
        event: 'Layout',
        step: energyHistory.length,
        forceVector: energyHistory[energyHistory.length - 1],
        forceDiff: meanForceDiff,
        // forcePercent: Math.abs(percentChange),
      });

      if (Math.abs(meanForceDiff) < MAX_ENERGY) {
        break;
      }
    }

    // const plane = new THREE.Mesh(
    //   new THREE.PlaneGeometry(10, 10),
    //   new THREE.MeshStandardMaterial({ color: 0xffffff })
    // );

    // plane.rotation.x = -Math.PI / 2;
    // plane.receiveShadow = false;

    // this.engine.scene.add(plane)
    // this.engine.scene.add(new THREE.AmbientLight(0xffffff, 0.5));

    // let directionalLight = new THREE.DirectionalLight(0xffffff, 1);
    // directionalLight.castShadow = true;
    // directionalLight.position.set(2, 2, 2);

    // this.engine.scene.add(directionalLight);

    var directedBetweenness: { [key: string]: number } = centrality.betweenness(
      this.graph,
      true
    );
    console.log(Object.values(directedBetweenness));

    const maxBetweenness = Math.max(...Object.values(directedBetweenness));

    console.log(maxBetweenness);
    const interpolator = generateInterpolator([0, maxBetweenness], [0.1, 1]);
    let colorScale = d3.scaleOrdinal(d3.schemeCategory10);

    this.graph.forEachNode((node) => {
      let position = this.layout.getNodePosition(node.id);

      if (!node.data) {
        return;
      }

      let color = colorScale(node.data['schema']);

      let graphNode = new GraphNode(
        node.data.unique_id,
        node.data,
        interpolator(directedBetweenness[node.id]),
        new THREE.Color(color),
        {
          betweenness: directedBetweenness[node.id],
        }
      );

      graphNode.castShadow = false;

      graphNode.position.set(
        position.x,
        position.y,
        position.z ? position.z : 0
      );

      this.nodes[node.data.unique_id] = graphNode;
      this.engine.scene.add(graphNode);
    });

    this.graph.forEachLink((link) => {
      let source = this.layout.getNodePosition(link.fromId);
      let target = this.layout.getNodePosition(link.toId);
      let sourceNode = this.graph.getNode(link.fromId);

      if (!sourceNode) return;

      let graphEdge = new GraphEdge2(
        new THREE.Vector3(source.x, source.y, source.z ? source.z : 0),
        new THREE.Vector3(target.x, target.y, target.z ? target.z : 0),
        new THREE.Color(colorScale(sourceNode.data['schema']))
      );
      this.edges[link.id] = graphEdge;
      this.engine.scene.add(graphEdge);
    });

    console.log('Fin');
  }

  resize() {}

  handlePointer(intersections: Any) {
    const selected = intersections.filter(
      (element: Any) => element.object.type == 'Mesh'
    )[0];
    let element = document.getElementsByTagName('h1')[0];
    if (!!selected && !!element) {
      element.textContent = selected.object.nodeData.unique_id;
    } else {
      element.textContent = '';
    }
  }

  update(delta: number) {
    Object.values(this.edges).forEach((edge) => {
      edge.update(delta);
    });
    // if (this.iterations < ITERATIONS_MAX) this.layout.step()
    // this.graph.forEachNode((node) => {
    //   let position = this.layout.getNodePosition(node.id)
    //   if (!node.data) {
    //     return
    //   }
    //   let graphNode = this.nodes[node.data.unique_id]
    //   graphNode.castShadow = true
    //   graphNode.position.set(
    //     position.x,
    //     position.y,
    //     position.z ? position.z : 0
    //   )
    // })
    // this.iterations++
    // this.graph.forEachLink((link) => {
    //   let oldGraphEdge = this.edges[link.id]
    //   this.engine.scene.remove(oldGraphEdge)
    //   let source = this.layout.getNodePosition(link.fromId)
    //   let target = this.layout.getNodePosition(link.toId)
    //   let graphEdge = new GraphEdge(
    //     new THREE.Vector3(source.x, source.y, source.z ? source.z : 0),
    //     new THREE.Vector3(target.x, target.y, target.z ? target.z : 0)
    //   )
    //   this.edges[link.id] = graphEdge
    //   this.engine.scene.add(graphEdge)
    // })
  }
}
