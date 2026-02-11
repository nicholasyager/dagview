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

type CachedLayout = {
  positions: { [nodeId: string]: { x: number; y: number; z: number } };
  degree: { [nodeId: string]: number };
  betweenness: { [nodeId: string]: number };
  routingPaths: { [edgeKey: string]: string[] };
};

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

  private getCacheKey(): string {
    const manifestPath = this.resources.find((r) => r.name === 'manifest')?.path ?? '';
    const powergraphPath = this.resources.find((r) => r.name === 'powergraph')?.path ?? '';
    return `layout:${manifestPath}|${powergraphPath}`;
  }

  private loadCache(key: string): CachedLayout | null {
    try {
      const raw = localStorage.getItem(key);
      if (!raw) return null;
      return JSON.parse(raw) as CachedLayout;
    } catch {
      return null;
    }
  }

  private saveCache(key: string, data: CachedLayout): void {
    // Clear stale cache entries for other manifests
    for (let i = localStorage.length - 1; i >= 0; i--) {
      const k = localStorage.key(i);
      if (k && k.startsWith('layout:') && k !== key) {
        localStorage.removeItem(k);
      }
    }

    const roundingReplacer = (_k: string, v: unknown) =>
      typeof v === 'number' ? Math.round(v * 1e4) / 1e4 : v;

    try {
      localStorage.setItem(key, JSON.stringify(data, roundingReplacer));
    } catch {
      // Full cache too large — save without routing paths
      // (layout + centrality are the most expensive to recompute)
      try {
        const partial: CachedLayout = {
          positions: data.positions,
          degree: data.degree,
          betweenness: data.betweenness,
          routingPaths: {},
        };
        localStorage.setItem(key, JSON.stringify(partial, roundingReplacer));
        console.warn('Layout cached without routing paths (storage limit)');
      } catch (e) {
        console.warn('Failed to save layout cache:', e);
      }
    }
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

    const cacheKey = this.getCacheKey();
    const cached = this.loadCache(cacheKey);

    let positions: { [nodeId: string]: { x: number; y: number; z: number } };
    let degree: { [key: string]: number };
    let directedBetweenness: { [key: string]: number };
    let routingPaths: { [edgeKey: string]: string[] } | null;

    if (cached) {
      console.log('Cache hit — using cached layout and centrality');
      positions = cached.positions;
      degree = cached.degree;
      directedBetweenness = cached.betweenness;
      routingPaths = cached.routingPaths;
    } else {
      console.log('Cache miss — computing layout and centrality');

      const layout = createLayout(graph, {
        dimensions: 3,
        springLength: 0.05,
        gravity: -6,
      });

      var energyHistory: number[] = [];
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

        let meanForceDiff =
          latestEnergyChange.reduce((acc, value) => acc + value, 0) /
          latestEnergyChange.length;

        if (energyHistory.length % 10 == 0) {
          console.log({
            event: 'Layout',
            step: energyHistory.length,
            forceVector: energyHistory[energyHistory.length - 1],
            forceDiff: meanForceDiff,
          });
        }

        if (Math.abs(meanForceDiff) < MAX_ENERGY) {
          break;
        }
      }

      positions = {};
      graph.forEachNode((node) => {
        let pos = layout.getNodePosition(node.id);
        positions[node.id as string] = {
          x: pos.x,
          y: pos.y,
          z: pos.z || 0,
        };
      });

      degree = centrality.degree(graph);
      directedBetweenness = centrality.betweenness(graph, true);
      routingPaths = null;
    }

    const degreeValues = Object.values(degree);
    const maxDegree = degreeValues.reduce((a, b) => (a > b ? a : b), -Infinity);

    const sizeInterpolator = generateInterpolator([1, maxDegree], [0.2, 1]);

    const betweennessValues = Object.values(directedBetweenness);
    const maxBetweenness = betweennessValues.reduce(
      (a, b) => (a > b ? a : b),
      -Infinity
    );

    const interpolator = generateInterpolator([0, maxBetweenness], [1, 2]);

    let colorScale = d3.scaleOrdinal(d3.schemeCategory10);

    graph.forEachNode((node) => {
      let position = positions[node.id as string];
      if (!position) return;

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
        interpolator(directedBetweenness[node.id as string]) *
          sizeInterpolator(degree[node.id as string]),
        new THREE.Color(color),
        {
          betweenness: directedBetweenness[node.id as string],
          degree: degree[node.id as string],
        }
      );

      graphNode.castShadow = false;

      graphNode.position.set(position.x, position.y, position.z);

      if (!Object(this.nodes).hasOwnProperty(node.id)) {
        this.nodes[node.id as string] = graphNode;
      } else {
        console.error(
          'Found a duplicate injection of ' + node.id,
          graphNode,
          this.nodes[node.id as string]
        );
      }

      this.engine.scene.add(graphNode);
    });

    if (routingPaths && Object.keys(routingPaths).length > 0) {
      manifestGraph.forEachLink((link) => {
        let sourceNode = graph.getNode(link.fromId);
        if (!sourceNode) return;

        let edgeKey = link.fromId + '|' + link.toId;
        let cachedPath = routingPaths![edgeKey];
        if (!cachedPath || cachedPath.length < 2) return;

        let pathObjects = cachedPath.map((id) => this.nodes[id]);
        if (pathObjects.some((obj) => !obj)) return;

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
    } else {
      routingPaths = {};
      let pathFinder = path.aStar(graph);

      manifestGraph.forEachLink((link) => {
        let sourceNode = graph.getNode(link.fromId);
        if (!sourceNode) return;

        let foundPath = pathFinder.find(link.fromId, link.toId);
        if (foundPath.length < 2) return;

        let reversedIds = foundPath
          .map((node) => node.id as string)
          .toReversed();
        routingPaths![link.fromId + '|' + link.toId] = reversedIds;

        let pathObjects = reversedIds.map((id) => this.nodes[id]);
        if (pathObjects.some((obj) => !obj)) return;

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

      this.saveCache(cacheKey, {
        positions,
        degree,
        betweenness: directedBetweenness,
        routingPaths,
      });
    }

    let distances: number[] = [];
    for (const pos of Object.values(positions)) {
      let vec = new THREE.Vector3(pos.x, pos.y, pos.z);
      distances.push(vec.length());
    }

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
