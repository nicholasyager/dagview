import { Engine } from '../engine/Engine';
import * as THREE from 'three';

import { Experience } from '../engine/Experience';
import { Resource } from '../engine/Resources';
import {
  Cluster,
  JsonPowerGraph,
  Manifest,
  PowerEdgeObject,
  PowerNodeObject,
} from '../client/local';
import { GraphNode } from './GraphNode';

import createLayout from 'ngraph.forcelayout';
import centrality from 'ngraph.centrality';

import createGraph, { Graph, Link, Node as NGraphNode } from 'ngraph.graph';
import path from 'ngraph.path';
import { EventedType } from 'ngraph.events';
import { GraphEdge2 } from './GraphEdge';

import * as d3 from 'd3';
import { RaycasterEvent } from '../engine/Raycaster';
import { Selector } from '../engine/interface/SearchUI';

const excludedResources = ['test', 'unit_test'];

const MAX_ENERGY = 0.1;

function getRelative(
  graph: Graph,
  node: NGraphNode,
  maxDepth: number | undefined,
  direction: 'parents' | 'children',
  depth?: number | undefined
): Set<NGraphNode> {
  let parents: Set<NGraphNode> = new Set();

  if (!depth) {
    depth = 1;
  }

  console.log(node, depth, maxDepth);

  graph.forEachLinkedNode(
    node.id,
    (_, link) => {
      let searchNode;
      if (direction == 'children') {
        searchNode = graph.getNode(link.toId);
      } else {
        searchNode = graph.getNode(link.fromId);
      }

      if (!searchNode) {
        return;
      }

      if (searchNode.id == node.id) {
        return;
      }

      parents.add(searchNode);

      if (!maxDepth || depth + 1 <= maxDepth) {
        parents = parents.union(
          getRelative(graph, searchNode, maxDepth, direction, depth + 1)
        );
      }
    },
    false
  );

  return parents;
}

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
  manifestGraph: Graph | undefined;

  resources: Resource[] = [
    {
      name: 'manifest',
      type: 'manifest',
      path: 'assets/manifest.20260210.json',
      // path: 'assets/manifest.huge.json',
      // path: 'assets/manifest.big.json',
      // path: 'assets/manifest.small.json',
    },
    {
      name: 'powergraph',
      type: 'powergraph',
      path: 'assets/powergraph.manifest.20260210.json',
      // path: 'assets/powergraph.manifest.huge.json',
      // path: 'assets/powergraph.manifest.big.json',
      // path: 'assets/powergraph.manifest.small.json',
    },
  ];

  constructor(private engine: Engine) {
    this.nodes = {};
    this.edges = {};
    this.iterations = 0;
    this.selectedNodes = [];
    this.manifestGraph = undefined;
  }

  init() {
    this.engine.raycaster.on('move', (e: RaycasterEvent[]) => {
      this.handlePointer(e);

      if (this.engine.raycaster.pointerState.isDragging) {
        this.engine.hasUpdated = true;
      }
      // this.engine.hasMoved = true;
    });
    this.engine.raycaster.on('click', (e: RaycasterEvent[]) => {
      this.handleClick(e);
      this.engine.hasMoved = true;

      this.engine.hasUpdated = true;
    });
    this.engine.raycaster.on('dblclick', (e: RaycasterEvent[]) => {
      this.handleDoubleClick(e);
      this.engine.hasMoved = true;
      this.engine.hasUpdated = true;
    });

    this.engine.raycaster.on('cameraMove', (e: RaycasterEvent[]) => {
      this.handleCameraMove(e);
      if (this.engine.raycaster.pointerState.isDragging) {
        this.engine.hasUpdated = true;
        this.engine.hasMoved = true;
      }
    });

    this.engine.raycaster.on('cameraZoom', (e: RaycasterEvent[]) => {
      this.handleCameraMove(e);

      this.engine.hasUpdated = true;
      this.engine.hasMoved = true;
    });

    this.engine.searchUI.on('search', (e: Selector) => {
      this.handleSearch(e);

      // if (this.engine.raycaster.pointerState.isDragging) {
      //   this.engine.hasUpdated = true;
      // }
      // this.engine.hasMoved = true;
    });

    let manifest: Manifest = this.engine.resources.getItem('manifest');
    let pg_object: JsonPowerGraph = this.engine.resources.getItem('powergraph');

    let clusters: { [key: string]: Cluster } = {};

    pg_object.power_nodes.forEach((item) => {
      clusters[item.id] = item.cluster;
    });

    const manifestGraph = this.generateGraphFromManifest(manifest);
    this.manifestGraph = manifestGraph;

    // const powerGraph = new PowerGraph(baseGraph);

    // const graph = powerGraph.hypergraph.graph;

    console.log(pg_object);

    // return;

    let graph = createGraph();
    pg_object.power_nodes.forEach((node: PowerNodeObject) => {
      console.log(node.id);
      let graphNode = manifestGraph.getNode(node.id);
      console.log(graphNode);

      let nodeData = !!graphNode ? graphNode.data : {};

      if (node.cluster.items.items.length > 1) {
        nodeData['resource_type'] = 'cluster';
      }

      graph.addNode(node.id, {
        unique_id: node.id,
        cluster: node.cluster,

        ...nodeData,
      });

      node.cluster.items.items.forEach((target: string) => {
        if (node.id == target) return;

        let graphNode = manifestGraph.getNode(node.id);

        let existingNode = graph.hasNode(target);
        if (!existingNode) {
          let nodeData = !!graphNode ? graphNode.data : {};
          graph.addNode(target, {
            unique_id: target,
            ...nodeData,
          });
        } else if (!Object(existingNode.data).hasOwnProperty('name')) {
          let nodeData = !!graphNode ? graphNode.data : {};
          existingNode.data = nodeData;
        }

        graph.addLink(node.id, target);
      });
    });

    pg_object.power_edges.forEach((edge: PowerEdgeObject) => {
      if (edge.from == edge.to) return;
      graph.addLink(edge.from, edge.to);
    });

    var degree: { [key: string]: number } = centrality.degree(graph);

    // Split routing nodes! If a `cluster` node has multiple
    // in edges and out edges, then split it into an "in" and an "out",
    // and re-write all of the surrounding links accordingly.
    graph.forEachNode((node) => {
      if (Object.hasOwn(clusters, node.id)) {
        if (clusters[node.id].items.items.length <= 1) {
          return;
        }
      } else {
        return;
      }

      let links = node.links;

      if (links == null) {
        return;
      }

      let inDegree = Array.from(links).filter((link: Link<any>) => {
        return link.toId == node.id;
      }).length;
      let outDegree = links.size - inDegree;

      if (inDegree <= 1 && outDegree <= 1) return;

      graph.addNode(node.id + '.out', {
        unique_id: node.id + '.out',
        ...node.data,
      });
      graph.addNode(node.id + '.in', {
        unique_id: node.id + '.out',
        ...node.data,
      });
      graph.addLink(node.id + '.in', node.id + '.out');

      node.links?.forEach((link) => {
        if (link.fromId == node.id) {
          graph.addLink(node.id + '.out', link.toId);
          graph.removeLink(link);
        } else if (link.toId == node.id) {
          graph.addLink(link.fromId, node.id + '.in');
          graph.removeLink(link);
        }
      });
    });

    // return;

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

      if (energyHistory.length % 10 == 0) {
        console.log({
          event: 'Layout',
          step: energyHistory.length,
          forceVector: energyHistory[energyHistory.length - 1],
          forceDiff: meanForceDiff,
          // forcePercent: Math.abs(percentChange),
        });
      }

      if (Math.abs(meanForceDiff) < MAX_ENERGY) {
        break;
      }
    }

    let pathFinder = path.aStar(graph);

    var directedBetweenness: { [key: string]: number } = centrality.betweenness(
      graph,
      true
    );

    const degreeValues = Object.values(degree);
    const maxDegree = degreeValues.reduce((a, b) => (a > b ? a : b), -Infinity);

    const sizeInterpolator = generateInterpolator(
      [1, maxDegree],
      [0.2, 1]
    );

    const betweennessValues = Object.values(directedBetweenness);
    const maxBetweenness = betweennessValues.reduce(
      (a, b) => (a > b ? a : b),
      -Infinity
    );

    const interpolator = generateInterpolator([0, maxBetweenness], [1, 2]);

    // const distanceInterpolator = generateInterpolator([0, 3000], [0, 1]);
    let colorScale = d3.scaleOrdinal(d3.schemeCategory10);

    graph.forEachNode((node) => {
      let position = layout.getNodePosition(node.id);

      if (!node.data) {
        node.data = {};
      }

      node.data['owner'] = undefined;
      node.data['schema'];
      let metadata = node.data['meta'] || {};
      if (metadata.hasOwnProperty('atlan')) {
        node.data['owner'] =
          metadata['atlan']?.['attributes']?.['ownerGroups']?.[0];
      } else if (
        node.data.hasOwnProperty('config') &&
        node.data.config.hasOwnProperty('group') &&
        !!node.data.config.group
      ) {
        node.data['owner'] = node.data.config.group;
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

      if (!Object(this.nodes).hasOwnProperty(node.id)) {
        this.nodes[node.id] = graphNode;
      } else {
        console.error(
          'Found a duplicate injection of ' + node.id,
          graphNode,
          this.nodes[node.id]
        );
      }

      this.engine.scene.add(graphNode);
    });

    manifestGraph.forEachLink((link) => {
      // We need to map the manifest graph to the power graph. With this in mind, we need to compute pathing
      // in the power graph between the two ends of the manifest graph link.
      let sourceNode = graph.getNode(link.fromId);

      if (!sourceNode) {
        return;
      }

      // let targetNode = graph.getNode(link.toId);

      let routingPath = pathFinder.find(link.fromId, link.toId);

      if (routingPath.length < 2) {
        return;
      }

      let pathObjects = routingPath
        .map((node) => {
          return this.nodes[node.id];
        })
        .toReversed();

      if (pathObjects.some((obj) => !obj)) {
        return;
      }

      let graphEdge = new GraphEdge2(
        link.id,
        pathObjects,
        new THREE.Color(
          sourceNode.data['resource_type'] == 'source'
            ? 0xaaaaaa
            : colorScale(sourceNode.data['owner'])
        )
      );
      this.edges[link.id] = graphEdge;
      this.engine.scene.add(graphEdge);
    });

    // Let's see everything by default! We can do this by measuring the max difference
    // between nodes and set this as the initial camera position.
    let distances: number[] = [];
    layout.forEachBody((body) => {
      let vec = new THREE.Vector3(body.pos.x, body.pos.y, body.pos.z);
      distances.push(vec.length());
    });

    const maxDistance = distances.reduce((a, b) => (a > b ? a : b), 0);
    this.engine.camera.instance.position.z = maxDistance * 2;
  }

  generateGraphFromManifest(manifest: Manifest): Graph<any, any> & EventedType {
    let graph = createGraph();
    for (let [key, value] of Object.entries(manifest.nodes)) {
      let resourceType = key.split('.')[0];

      if (excludedResources.includes(resourceType)) continue;
      graph.addNode(key, value);
    }

    for (let [key, value] of Object.entries(manifest.sources)) {
      graph.addNode(key, value);
    }

    for (let [source, targets] of Object.entries(manifest.child_map)) {
      let resourceType = source.split('.')[0];
      if (excludedResources.includes(resourceType)) continue;

      if (source.startsWith('exposure')) continue;
      if (!graph.hasNode(source)) continue;

      targets.forEach((target: string) => {
        if (!graph.hasNode(target)) return;
        graph?.addLink(source, target);
      });
    }

    for (let [target, sources] of Object.entries(manifest.parent_map)) {
      let resourceType = target.split('.')[0];
      if (excludedResources.includes(resourceType)) continue;
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
    let subtitle = document.getElementById('node-owner');
    if (!subtitle && element?.parentElement) {
      subtitle = document.createElement('p');
      subtitle.id = 'node-owner';
      element.after(subtitle);
    }

    if (!!selected && !!element) {
      let object = selected.object as GraphNode;
      element.textContent = object.nodeData.unique_id;
      if (subtitle) {
        subtitle.textContent = object.nodeData.owner || '';
      }
    } else {
      element.textContent = '';
      if (subtitle) {
        subtitle.textContent = '';
      }
    }
  }

  selectNodes(nodes: GraphNode[], containedSelection?: boolean) {
    if (!containedSelection) {
      containedSelection = false;
    }

    Object.values(this.edges).forEach((edge) => {
      edge.dim();
    });

    Object.values(this.nodes).forEach((node) => {
      node.dim();
    });

    let selectedNodeIds: Set<number> = new Set(nodes.map((node) => node.id));

    nodes.forEach((node) => {
      this.selectedNodes.push(node.id);

      let selectedObject: GraphNode | undefined =
        this.engine.scene.getObjectById(node.id) as GraphNode;
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
          if (containedSelection) {
            if (
              !selectedNodeIds.has(edge.source.id) ||
              !selectedNodeIds.has(edge.target.id)
            ) {
              return;
            }
          }
          edge.select();
          edge.dedim();

          edge.source.dedim();
          edge.target.dedim();

          this.selectedNodes.push(edge.id);
        });
      }
    });
  }

  clearSelections() {
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
  }

  handleClick(intersections: RaycasterEvent[]) {
    this.clearSelections();

    const selected = intersections.filter(
      (element: RaycasterEvent) => element.object.type == 'Mesh'
    )[0];

    if (!selected) {
      return;
    }

    if (selected.object instanceof GraphNode) {
      this.selectNodes([selected.object]);
    }
  }

  handleDoubleClick(intersections: RaycasterEvent[]) {
    const selected = intersections.filter(
      (element: RaycasterEvent) => element.object.type == 'Mesh'
    )[0];

    if (!selected) {
      return;
    }
    // console.log(selected);

    this.engine.camera.controls.target = new THREE.Vector3(
      ...selected.object.position
    );
  }

  handleCameraMove(_: RaycasterEvent[]) {
    let target = this.engine.camera.controls.target;
    let distance = this.engine.camera.instance.position.distanceTo(target);
    this.engine.camera.instance.far = Math.min(distance * 2, 2000);
  }

  update(delta: number) {
    this.engine.hasUpdated = false;
    if (this.selectedNodes.length > 0) {
      this.engine.hasUpdated = true;
    }

    Object.values(this.edges).forEach((edge) => {
      edge.update(delta, this.engine);

      if (this.engine.hasMoved) {
        if (
          !this.engine.raycaster.isSeen(edge.source) &&
          !this.engine.raycaster.isSeen(edge.target)
        ) {
          let object = edge.children[0] as THREE.Line;

          if (object.material instanceof THREE.Material) {
            object.material.opacity = 0.01;
          } else {
            object.material[0].opacity = 0.01;
          }
        } else {
          edge.updateEdgeOpacity(this.engine);
          edge.source.updateDistance(this.engine);
          edge.target.updateDistance(this.engine);
        }
      }
    });

    this.engine.hasMoved = false;
  }

  handleSearch(selector: Selector) {
    this.clearSelections();
    console.log(selector);

    let selected_nodes: Set<NGraphNode> = new Set();
    if (!this.manifestGraph) {
      return;
    }

    this.manifestGraph.forEachNode((node) => {
      console.log(node);
      if (
        (Object.hasOwn(node.data, 'name') &&
          node.data.name == selector.value) ||
        (Object.hasOwn(node.data, 'alias') && node.data.alias == selector.value)
      ) {
        selected_nodes.add(node);
      }
    });

    // Get Parents
    let parents: Set<NGraphNode> = new Set();
    if (selector.parents) {
      Array.from(selected_nodes).forEach((node) => {
        if (!this.manifestGraph) {
          return;
        }

        let nodeParents = getRelative(
          this.manifestGraph,
          node,
          selector.parents_depth,
          'parents'
        );
        parents = parents.union(nodeParents);
      });
    }

    let children: Set<NGraphNode> = new Set();
    if (selector.children) {
      Array.from(selected_nodes).forEach((node) => {
        if (!this.manifestGraph) {
          return;
        }

        let nodeChildren = getRelative(
          this.manifestGraph,
          node,
          selector.children_depth,
          'children'
        );
        children = children.union(nodeChildren);
      });
    }

    let all_nodes = selected_nodes.union(parents).union(children);

    // Target center mass for the selection.

    let selectedObjects = Array.from(all_nodes)
      .map(
        (node) =>
          this.engine.scene.getObjectByName(node.id as string) as GraphNode
      )
      .filter((node) => !!node);

    // Find the center of the selected nodes
    let vectorSum = selectedObjects.reduce(
      (output, object) => {
        output.position.add(object.position);
        output.items += 1;
        return output;
      },
      { position: new THREE.Vector3(0, 0, 0), items: 0 }
    );

    let center_of_mass = vectorSum.position.divideScalar(vectorSum.items);

    // Find the correct zoom level for the selected nodes
    let distance = selectedObjects.reduce((distance, object) => {
      let itemDistance = object.position.distanceTo(center_of_mass);
      if (itemDistance > distance) {
        return itemDistance;
      }
      return distance;
    }, 0);

    this.engine.camera.instance.position.x = 0;
    this.engine.camera.instance.position.y = 0;
    this.engine.camera.instance.position.z = distance * 2;
    this.engine.camera.controls.target = center_of_mass;

    this.selectNodes(selectedObjects, true);
    this.engine.hasMoved = true;
    this.engine.hasUpdated = true;
  }
}
