import { Cluster } from './Cluster';

export interface SimilaritySearchResult {
  source: string;
  target: string;
  similarity: number;
}

export class SimilarityMatrix {
  similarityMatrix: {
    [key: string]: {
      [key: string]: number;
    };
  };
  similarityFunction: (a: Cluster, b: Cluster) => number;

  constructor(
    similarityFunction: (cluster: Cluster, otherCluster: Cluster) => number
  ) {
    this.similarityMatrix = {};
    this.similarityFunction = similarityFunction;
  }

  updateSimilarityMatrix(cluster: Cluster, otherCluster: Cluster): undefined {
    console.debug(
      'Calculating similarity between ' +
        cluster.getId() +
        ' and ' +
        otherCluster.getId()
    );
    if (cluster.items == otherCluster.items) {
      return;
    }

    let similarity = this.similarityFunction(cluster, otherCluster);
    if (similarity > 0) console.log({ cluster, otherCluster, similarity });

    if (!this.similarityMatrix.hasOwnProperty(cluster.getId())) {
      this.similarityMatrix[cluster.getId()] = {};
    }

    this.similarityMatrix[cluster.getId()][otherCluster.getId()] = similarity;
  }

  maxSimilarity(): SimilaritySearchResult {
    let maxSim = 0;
    let source = '';
    let target = '';

    Object.entries(this.similarityMatrix).forEach(([key, values]) =>
      Object.entries(values).forEach(([key2, value]) => {
        if (value > maxSim) {
          source = key;
          target = key2;
          maxSim = value;
        }
      })
    );
    return { source, target, similarity: maxSim };
  }

  removeCluster(clusterName: string) {
    delete this.similarityMatrix[clusterName];

    // Iteratively find rows in the matrix and remove those, too.
    Object.values(this.similarityMatrix).forEach(
      (rowObject) => delete rowObject[clusterName]
    );

    console.log('Removed ' + clusterName + ' from similarityMatrix.');
  }
}
