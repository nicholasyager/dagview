import createGraph, { Graph, Node } from 'ngraph.graph';
import { SimilarityMatrix } from './powerGraphs/SimilarityMatrix';
import { Cluster, difference, intersection } from './powerGraphs/Cluster';

type CartesianProduct<Inputs> = [string, string];

function cartesianProduct<Sets extends ReadonlyArray<ReadonlyArray<unknown>>>(
  sets: Sets
): CartesianProduct<Sets> {
  return sets.reduce((a, b) =>
    a.flatMap((d) => b.map((e) => [d, e].flat()))
  ) as CartesianProduct<Sets>;
}

class PowerEdge {
  from: string;
  to: string;
  size: number;

  constructor(from: string, to: string, size: number) {
    this.from = from;
    this.to = to;
    this.size = size;
  }
}

class HyperGraph {
  graph: Graph;
  nodeParents: { [key: string]: string };

  constructor() {
    this.graph = createGraph();
    this.nodeParents = {};
  }

  addNode(nodeId: string, data?: object) {
    this.graph.addNode(nodeId, data);
  }

  addEdge(from: string, to: string, data?: object) {
    this.graph.addLink(from, to, data);
  }

  addPowerNode(id: string, nodes: Set<string>, data?: object) {
    this.graph.addNode(id, data);

    nodes.forEach((node) => {
      this.nodeParents[node] = id;
    });
  }

  forEachNode(func: Function) {
    this.graph.forEachNode(
      func as (node: Node<any>) => boolean | void | null | undefined
    );
  }

  getNodesCount() {
    return this.graph.getNodesCount();
  }
}

export class PowerGraph {
  graph: Graph;
  hypergraph: HyperGraph;
  powerEdges: Set<PowerEdge>;
  clusters: { [key: string]: Cluster };
  remainingClusters: { [key: string]: Cluster };
  minimumSimilarity: number;

  similarityMatrix: SimilarityMatrix;

  getClusterPeers(): [Cluster, Cluster][] {
    return [];
  }

  getClusterPairs(): [Cluster, Cluster][] {
    return Object.values(this.clusters).flatMap((cluster, index) =>
      Object.values(this.clusters)
        .slice(index)
        .map((nextCluster) => {
          return [cluster, nextCluster] as [Cluster, Cluster];
        })
    );
  }

  parents(nodes: Set<string>): Set<string> {
    // Find the parent nodes for a set of nodes.
    let parents: Set<string> = new Set();

    nodes.forEach((node) => {
      console.log(node);
      const links = this.graph.getLinks(node);
      if (!links) return;

      Array.from(links).forEach((link) => {
        if (link.fromId == node) return;
        parents.add(link.fromId as string);
      });
    });

    return parents;
  }

  neighborhood(cluster: Cluster): Set<string> {
    // Find all of the neighbors within the neighborhood of a cluster.
    let neighborhood: Set<string> = new Set();

    cluster.forEach((node) => {
      const nodeLinks = this.graph.getLinks(node);
      if (!nodeLinks) return;
      Array.from(nodeLinks)
        .map((item) => item.toId)
        .forEach((nodeId) => {
          if (nodeId != node) {
            // console.log({ nodeId, source: node });
            neighborhood.add(nodeId as string);
          }
        });
    });

    return neighborhood;
  }

  calculateNeighborhoodSimilarity(nodes: Cluster, otherNodes: Cluster): number {
    console.log('Calculating neighborhood similarity');
    let setA: Set<string> = this.neighborhood(nodes);
    let setB: Set<string> = this.neighborhood(otherNodes);

    // console.log({ nodes, otherNodes, setA, setB });

    let intersection = new Set<string>();
    for (let elem of setA) {
      if (setB.has(elem)) {
        intersection.add(elem);
      }
    }

    let union = new Set<string>(setA);
    for (let elem of setB) {
      union.add(elem);
    }

    return intersection.size / union.size;
  }

  constructor(graph: Graph) {
    this.graph = graph;
    this.hypergraph = new HyperGraph();
    this.powerEdges = new Set<PowerEdge>();
    this.clusters = {};
    this.remainingClusters = {};
    this.minimumSimilarity = 0.9;

    this.graph.forEachNode((node) => {
      if (!node || !node.id) return;

      let nodeId = node.id;

      let parents: string[] = [];

      const links = this.graph.getLinks(nodeId);
      if (links == null) return;

      links.forEach((item) => {
        if (item.toId == node.id) {
          parents.push(item.fromId as string);
        }
      });

      let cluster = new Cluster([node.id as string], parents);

      this.clusters[cluster.getId()] = cluster;
      this.remainingClusters[cluster.getId()] = cluster;
    });

    this.similarityMatrix = new SimilarityMatrix((cluster, otherCluster) =>
      this.calculateNeighborhoodSimilarity(cluster, otherCluster)
    );

    const pairs = this.getClusterPairs();

    pairs.forEach(([u, w]) =>
      this.similarityMatrix.updateSimilarityMatrix(u, w)
    );

    let clustersWithMaxSimilarity = this.similarityMatrix.maxSimilarity();
    console.log(clustersWithMaxSimilarity);

    // This starts line 8 of the power graph algorithm. At this point, we'll want
    // to be creating the hierarchical clusters for the graph using max similarity.

    const maxIterations = 100;
    let iteration = 0;

    while (
      Object.keys(this.remainingClusters).length > 1 &&
      clustersWithMaxSimilarity.similarity > this.minimumSimilarity &&
      iteration < maxIterations
    ) {
      console.log('Remaining Clusters', this.remainingClusters);

      // Combined `u` and `w` into one cluster `uw`. Add `uw` to the cluster list
      const combinedCluster = this.remainingClusters[
        clustersWithMaxSimilarity.source
      ].union(this.remainingClusters[clustersWithMaxSimilarity.target]);
      console.log(
        'Adding ' +
          combinedCluster.getId() +
          ' to clusters and remainingClusters.',
        combinedCluster
      );

      this.clusters[combinedCluster.getId()] = combinedCluster;
      this.remainingClusters[combinedCluster.getId()] = combinedCluster;

      console.log(this.remainingClusters);

      // Remove u and w from the similarity matrix and remaining clusters object.

      delete this.remainingClusters[clustersWithMaxSimilarity.source];
      console.log(
        'Removed ' +
          clustersWithMaxSimilarity.source +
          ' from remainingClusters.'
      );
      delete this.remainingClusters[clustersWithMaxSimilarity.target];
      console.log(
        'Removed ' +
          clustersWithMaxSimilarity.target +
          ' from remainingClusters.'
      );
      this.similarityMatrix.removeCluster(clustersWithMaxSimilarity.source);
      this.similarityMatrix.removeCluster(clustersWithMaxSimilarity.target);

      console.log(this.remainingClusters);

      // Update the similarity matrix for `uw` and all other clusters.
      Object.values(this.remainingClusters).forEach((w) => {
        this.similarityMatrix.updateSimilarityMatrix(combinedCluster, w);
      });

      console.log('Similarity matrix updated', this.similarityMatrix);

      clustersWithMaxSimilarity = this.similarityMatrix.maxSimilarity();
      iteration++;
      console.log(clustersWithMaxSimilarity);
    }

    console.debug(
      'Completed greedy clustering exercise.',
      this.remainingClusters
    );

    // For each Cluster in this.clusters, find the neighborhood and add it if the similarity
    // between the Cluster and the Neighborhood is less than the minimum similarity threshold.
    Object.entries(this.clusters).forEach(([index, cluster]) => {
      const neighbors = this.neighborhood(cluster);
      const neighborhood = new Cluster(neighbors, this.parents(neighbors));
      const similarity = this.calculateNeighborhoodSimilarity(
        cluster,
        neighborhood
      );
      console.log(
        'Calculating similarity between ' +
          index +
          ' and its neighbors ' +
          neighborhood.getId() +
          '.',
        { similarity }
      );

      if (similarity >= this.minimumSimilarity) {
        this.clusters[neighborhood.getId()] = neighborhood;
      }
    });

    // Do it again! This way, we get the second-degree neighbors, too.
    // For each Cluster in this.clusters, find the neighborhood and add it if the similarity
    // between the Cluster and the Neighborhood is less than the minimum similarity threshold.
    Object.entries(this.clusters).forEach(([index, cluster]) => {
      const neighbors = this.neighborhood(cluster);
      const neighborhood = new Cluster(neighbors, this.parents(neighbors));
      const similarity = this.calculateNeighborhoodSimilarity(
        cluster,
        neighborhood
      );
      console.log(
        'Calculating similarity between ' +
          index +
          ' and its neighbors ' +
          neighborhood.getId() +
          '.',
        { similarity }
      );

      if (similarity >= this.minimumSimilarity) {
        this.clusters[neighborhood.getId()] = neighborhood;
      }
    });

    // Start populating the hypergraph. We'll begin by adding all singleton clusters as vertices in the hypergraph.
    Object.values(this.clusters).forEach((cluster) => {
      if (cluster.size() > 1) return;
      const node = this.graph.getNode(cluster.getId());

      this.hypergraph.addNode(cluster.getId(), {
        cluster: cluster,
        dbt: node?.data,
      });
    });

    // Line 17 - 23: For each unordered pair of clusters, identify poweredges
    this.getClusterPairs().forEach(([cluster, otherCluster]) => {
      console.log({ cluster, otherCluster });

      // If U ∩ W = ∅ and (U ∪ W, U × W) isasub-graph in G.
      const intersection = cluster.intersection(otherCluster);

      const union = cluster.union(otherCluster);

      if (intersection.size() == 0) {
        console.log('Finding disjoint clusters', {
          cluster,
          otherCluster,
          union,
        });

        cartesianProduct([
          Array.from(cluster.items),
          Array.from(otherCluster.items),
        ])
          .filter((edge) => {
            if (edge.length != 2) {
              throw new Error();
            }

            return !!this.graph.hasLink(edge[0], edge[1]);
          })
          .forEach((edge) => {
            this.powerEdges.add(
              new PowerEdge(
                cluster.getId(),
                otherCluster.getId(),
                cluster.size() + otherCluster.size()
              )
            );
          });
      }

      // If U = W and the U-induced graph in G isa clique

      if (cluster.getId() == otherCluster.getId()) {
        const isSubGraph = cartesianProduct([
          Array.from(cluster.items),
          Array.from(otherCluster.items),
        ])
          .filter((edge) => {
            if (edge.length != 2) {
              throw new Error();
            }

            console.log(edge);

            return edge[0] != edge[1];
          })
          .map((edge) => {
            if (!edge) {
              return;
            }
            console.log(
              edge,
              edge[0],
              edge[1],
              !!this.graph.hasLink(edge[0], edge[1]) ||
                !!this.graph.hasLink(edge[1], edge[0])
            );
            return (
              !!this.graph.hasLink(edge[0], edge[1]) ||
              !!this.graph.hasLink(edge[1], edge[0])
            );
          })
          .every((value) => value == true);

        if (isSubGraph) {
          console.log('Edge is a subgraph.', cluster, otherCluster, isSubGraph);

          // Removing for now, since dbt DAGs don't really have self-loops.
          //   this.powerEdges.add(
          //     new PowerEdge(
          //       cluster.getId(),
          //       otherCluster.getId(),
          //       (cluster.size() + otherCluster.size()) / 2
          //     )
          //   );
        }
      }
    });

    // Sort power edges
    let iterations = 0;
    while (this.powerEdges.size > 0 && iterations < 100000) {
      const edges = Array.from(this.powerEdges).sort((a, b) => a.size - b.size);

      console.log(edges);
      const edge = edges.pop();
      if (!!edge) this.powerEdges.delete(edge);
      console.log(edge);

      if (!edge) continue;

      const sourceCluster = this.clusters[edge.from];
      const targetCluster = this.clusters[edge.to];

      // If the size of power edge (U, W) is two and U = W then do nothing
      if (edge?.from == edge?.to && edge?.size == 2) continue;
      else if (null) {
        // Add new candidates for non-trivial overlap with source
      } else if (null) {
        // Add new candidates for non-trivial overlap with target
      } else if (null) {
        // Else if (U, W) is a clique (U = W):  Add power node U to V′ and power edge (U, U) to E′
      } else {
        // Else: Add power nodes U and W to V′ and power edge (U, W) to E
        console.log(
          'Adding ' + sourceCluster.getId() + ' and ' + targetCluster.getId()
        );

        let sourceGraphNode = this.graph.getNode(sourceCluster.getId());
        console.log(sourceGraphNode);
        this.hypergraph.addPowerNode(
          sourceCluster.getId(),
          sourceCluster.items,
          {
            type: 'cluster',
            cluster: sourceCluster,
            dbt: sourceGraphNode
              ? sourceGraphNode.data
              : { unique_id: sourceCluster.getId() },
          }
        );

        let targetGraphNode = this.graph.getNode(targetCluster.getId());
        this.hypergraph.addPowerNode(
          targetCluster.getId(),
          targetCluster.items,
          {
            type: 'cluster',
            cluster: targetCluster,
            dbt: targetGraphNode
              ? targetGraphNode.data
              : { unique_id: targetCluster.getId() },
          }
        );

        this.hypergraph.addEdge(sourceCluster.getId(), targetCluster.getId());
      }

      // For each
      // this.hypergraph.forEachNode((cluster: Node) => {
      //   let nodeIntersection = sourceCluster.intersection(cluster.data.cluster);

      //   let differenceA = sourceCluster.difference(cluster.data.cluster);
      //   let differenceB = cluster.data.cluster.difference(sourceCluster);

      //   console.log(nodeIntersection, differenceA, differenceB);

      //   if (
      //     nodeIntersection.size() > 0 &&
      //     differenceA.size() > 0 &&
      //     differenceB.size() > 0
      //   ) {
      //     this.clusters[differenceA.getId()] = differenceA;
      //     this.powerEdges.add(
      //       new PowerEdge(
      //         differenceA.getId(),
      //         targetCluster.getId(),
      //         differenceA.size() + targetCluster.size()
      //       )
      //     );

      //     this.clusters[nodeIntersection.getId()] = nodeIntersection;
      //     this.powerEdges.add(
      //       new PowerEdge(
      //         nodeIntersection.getId(),
      //         targetCluster.getId(),
      //         nodeIntersection.size() + targetCluster.size()
      //       )
      //     );
      //   }
      // });

      // this.hypergraph.forEachNode((cluster: Node) => {
      //   let nodeIntersection = targetCluster.intersection(cluster.data.cluster);

      //   let differenceA = targetCluster.difference(cluster.data.cluster);
      //   let differenceB = cluster.data.cluster.difference(targetCluster);

      //   console.log(nodeIntersection, differenceA, differenceB);

      //   if (
      //     nodeIntersection.size() > 0 &&
      //     differenceA.size() > 0 &&
      //     differenceB.size() > 0
      //   ) {
      //     this.clusters[differenceA.getId()] = differenceA;
      //     this.powerEdges.add(
      //       new PowerEdge(
      //         differenceA.getId(),
      //         targetCluster.getId(),
      //         differenceA.size() + targetCluster.size()
      //       )
      //     );

      //     this.clusters[nodeIntersection.getId()] = nodeIntersection;
      //     this.powerEdges.add(
      //       new PowerEdge(
      //         nodeIntersection.getId(),
      //         targetCluster.getId(),
      //         nodeIntersection.size() + targetCluster.size()
      //       )
      //     );
      //   }
      // });

      // Else: Add power nodes U and W to V′ and power edge (U, W) to E′

      iterations++;
      console.log({ edges, iterations });
      // break;
    }

    console.log(this.hypergraph.getNodesCount());
    console.log(this.powerEdges);

    // Line 15.

    // Lines 41 - 42: Add all missing edges from graph to hypergraph.
    this.graph.forEachLink((link) => {
      // Check of there is a missing link between the singleton clusters
      // in the hypergraph

      if (!this.hypergraph.graph.hasLink(link.fromId, link.toId)) {
        this.hypergraph.addEdge(
          link.fromId as string,
          link.toId as string,
          link.data
        );
      }
    });
  }
}
