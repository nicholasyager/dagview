import { max } from 'd3';
import { Graph } from 'ngraph.graph';
import { SimilarityMatrix } from './powerGraphs/SimilarityMatrix';
import { Cluster } from './powerGraphs/Cluster';

export class PowerGraph {
  graph: Graph;
  clusters: { [key: string]: Cluster };
  remainingClusters: { [key: string]: Cluster };
  minimumSimilarity: number;

  similarityMatrix: SimilarityMatrix;

  getClusterPairs(): [Cluster, Cluster][] {
    return Object.values(this.clusters).flatMap((cluster, index) =>
      Object.values(this.clusters)
        .slice(index + 1)
        .map((nextCluster) => {
          return [cluster, nextCluster] as [Cluster, Cluster];
        })
    );
  }

  calculateNeighborhoodSimilarity(nodes: Cluster, otherNodes: Cluster): number {
    console.log('Calculating neighborhood similarity');
    let setA: Set<string> = new Set();
    let setB: Set<string> = new Set();

    nodes.forEach((node) => {
      const nodeLinks = this.graph.getLinks(node);
      if (!nodeLinks) return;
      Array.from(nodeLinks)
        .map((item) => item.toId)
        .forEach((nodeId) => {
          if (nodeId != node) {
            // console.log({ nodeId, source: node });
            setA.add(nodeId as string);
          }
        });
    });

    otherNodes.forEach((node) => {
      const nodeLinks = this.graph.getLinks(node);
      if (!nodeLinks) return;
      Array.from(nodeLinks)
        .map((item) => item.toId)
        .forEach((nodeId) => {
          if (nodeId != node) {
            setB.add(nodeId as string);
          }
        });
    });

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
    this.clusters = {};
    this.remainingClusters = {};
    this.minimumSimilarity = 0.9;

    this.graph.forEachNode((node) => {
      if (!node.id) return;

      let cluster = new Cluster([node.id as string]);

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
  }
}
