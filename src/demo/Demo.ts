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
import { GraphEdge2 } from './GraphEdge';

import * as d3 from 'd3';
import { RaycasterEvent } from '../engine/Raycaster';
import { PowerGraph } from './PowerGraph';
import init, { greet } from 'powergraph';

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
  // graph: Graph<any, any> & EventedType;
  // layout: Layout<any>;
  nodes: { [key: string]: GraphNode };
  edges: { [key: string]: GraphEdge2 };
  iterations: number;
  selectedNodes: number[];

  resources: Resource[] = [
    {
      name: 'manifest',
      type: 'manifest',
      // path: 'assets/manifest.huge.json',
      // path: 'assets/manifest.big.json',
      path: 'assets/manifest.small.json',
    },
  ];

  constructor(private engine: Engine) {
    this.nodes = {};
    this.edges = {};
    this.iterations = 0;
    this.selectedNodes = [];

    init().then(() => {
      console.log('init wasm-pack');
      greet('from vite!');
    });
  }

  init() {
    this.engine.raycaster.on('move', (e: RaycasterEvent[]) =>
      this.handlePointer(e)
    );
    this.engine.raycaster.on('click', (e: RaycasterEvent[]) =>
      this.handleClick(e)
    );
    this.engine.raycaster.on('dblclick', (e: RaycasterEvent[]) =>
      this.handleDoubleClick(e)
    );

    this.engine.raycaster.on('cameraMove', (e: RaycasterEvent[]) => {});

    let manifest: Manifest = this.engine.resources.getItem('manifest');

    const graph = this.generateGraphFromManifest(manifest);

    // const powerGraph = new PowerGraph(baseGraph);

    // const graph = powerGraph.hypergraph.graph;

    const layout = createLayout(graph, {
      dimensions: 3,
      // dragCoefficient: 0.99,
      springLength: 0.05,
      gravity: -6,
    });

    var energyHistory = [];
    while (true) {
      layout.step();

      energyHistory.push(layout.getForceVectorLength());

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

    var directedBetweenness: { [key: string]: number } = centrality.betweenness(
      graph,
      true
    );
    var degree: { [key: string]: number } = centrality.degree(graph);
    const sizeInterpolator = generateInterpolator(
      [1, Math.max(...Object.values(degree))],
      [0.2, 1]
    );

    const maxBetweenness = Math.max(...Object.values(directedBetweenness));

    const interpolator = generateInterpolator([0, maxBetweenness], [1, 2]);

    // const distanceInterpolator = generateInterpolator([0, 3000], [0, 1]);
    let colorScale = d3.scaleOrdinal(d3.schemeCategory10);

    graph.forEachNode((node) => {
      let position = layout.getNodePosition(node.id);
      console.log({ node, position });

      if (!node.data) {
        console.log(node);
        return;
      }

      node.data['owner'] = undefined;
      node.data['schema'];
      let metadata = node.data['meta'] || {};
      if (metadata.hasOwnProperty('atlan')) {
        node.data['owner'] = metadata['atlan']['attributes']['ownerGroups'][0];
      }

      let color = colorScale(node.data['owner']);

      let graphNode = new GraphNode(
        node.data.unique_id,
        node.data,
        interpolator(directedBetweenness[node.id]) *
          sizeInterpolator(degree[node.id]),
        new THREE.Color(color),
        {
          betweenness: directedBetweenness[node.id],
          degree: degree[node.id],
        }
      );

      graphNode.castShadow = false;

      graphNode.position.set(
        position.x,
        position.y,
        position.z ? position.z : 0
      );

      this.nodes[node.id] = graphNode;

      this.engine.scene.add(graphNode);
    });

    graph.forEachLink((link) => {
      let sourceNode = graph.getNode(link.fromId);
      let targetNode = graph.getNode(link.toId);

      if (!sourceNode || !targetNode) return;

      let sourceObject = this.nodes[sourceNode.id];
      let targetObject = this.nodes[targetNode.id];

      if (!sourceObject || !targetObject) return;

      let graphEdge = new GraphEdge2(
        link.id,
        sourceObject,
        targetObject,
        new THREE.Color(
          sourceNode.data['resource_type'] == 'source'
            ? 0xaaaaaa
            : colorScale(sourceNode.data['owner'])
        )
      );
      this.edges[link.id] = graphEdge;
      this.engine.scene.add(graphEdge);
    });
  }

  generateGraphFromManifest(manifest: Manifest): Graph<any, any> & EventedType {
    let graph = createGraph();
    for (let [key, value] of Object.entries(manifest.nodes)) {
      if (key.startsWith('test')) continue;
      graph.addNode(key, value);
    }

    for (let [key, value] of Object.entries(manifest.sources)) {
      graph.addNode(key, value);
    }

    for (let [source, targets] of Object.entries(manifest.child_map)) {
      if (source.startsWith('test')) continue;
      if (source.startsWith('exposure')) continue;
      if (!graph.hasNode(source)) continue;

      targets.forEach((target: string) => {
        if (!graph.hasNode(target)) return;
        graph?.addLink(source, target);
      });
    }

    for (let [target, sources] of Object.entries(manifest.parent_map)) {
      if (target.startsWith('test')) continue;
      if (target.startsWith('exposure')) continue;
      sources.forEach((source: string) => {
        if (!graph.hasNode(source)) return;
        graph?.addLink(source, target);
      });
    }
    return graph;
  }

  resize() {}

  handlePointer(intersections: RaycasterEvent[]) {
    const selected = intersections.filter(
      (element: RaycasterEvent) => element.object.type == 'Mesh'
    )[0];

    let element = document.getElementsByTagName('h1')[0];
    if (!!selected && !!element) {
      let object = selected.object as GraphNode;
      element.textContent = object.nodeData.unique_id;
    } else {
      element.textContent = '';
    }
  }

  handleClick(intersections: RaycasterEvent[]) {
    this.selectedNodes.forEach((node) => {
      let selectedObject: GraphNode | undefined =
        this.engine.scene.getObjectById(node) as GraphNode;
      selectedObject.deselect();
    });

    Object.values(this.edges).forEach((edge) => {
      edge.dedim();
    });

    Object.values(this.nodes).forEach((node) => {
      node.dedim();
    });

    this.selectedNodes = [];

    const selected = intersections.filter(
      (element: RaycasterEvent) => element.object.type == 'Mesh'
    )[0];

    if (!selected) {
      return;
    }

    this.selectedNodes.push(selected.object.id);

    Object.values(this.edges).forEach((edge) => {
      edge.dim();
    });

    Object.values(this.nodes).forEach((node) => {
      node.dim();
    });

    let selectedObject: GraphNode | undefined = this.engine.scene.getObjectById(
      selected.object.id
    ) as GraphNode;
    if (!!selectedObject) {
      selectedObject.select();

      // Find all edges and select those, too!
      let childEdges = this.engine.scene.getObjectsByProperty(
        'source',
        selectedObject
      ) as GraphEdge2[];
      let parentEdges = this.engine.scene.getObjectsByProperty(
        'target',
        selectedObject
      ) as GraphEdge2[];

      let edges = childEdges.concat(parentEdges);

      edges.forEach((edge) => {
        edge.select();
        edge.dedim();

        edge.source.dedim();
        edge.target.dedim();

        this.selectedNodes.push(edge.id);
      });
    }
  }

  handleDoubleClick(intersections: RaycasterEvent[]) {
    const selected = intersections.filter(
      (element: RaycasterEvent) => element.object.type == 'Mesh'
    )[0];

    if (!selected) {
      return;
    }
    console.log(selected);

    this.engine.camera.controls.target = new THREE.Vector3(
      ...selected.object.position
    );
  }

  update(delta: number) {
    Object.values(this.edges).forEach((edge) => {
      edge.update(delta, this.engine);
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
