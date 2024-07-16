import { LoadingManager } from 'three';

export interface DbtNode {
  resource_type: string;
  unique_id: string;
}

export interface Manifest {
  nodes: {
    [key: string]: DbtNode;
  };
  sources: {
    [key: string]: DbtNode;
  };

  child_map: {
    [key: string]: string[];
  };
  parent_map: {
    [key: string]: string[];
  };
}

export interface Set<T> {
  items: T[];
}

export interface Cluster {
  id: string;
  items: Set<string>;
}

export interface PowerNodeObject {
  id: string;
  cluster: Cluster;
}

export interface PowerEdgeObject {
  from: string;
  to: string;
}

export interface JsonPowerGraph {
  power_nodes: PowerNodeObject[];
  power_edges: PowerEdgeObject[];
  clusters: Cluster[];
}

export async function getManifest(path: string): Promise<Manifest> {
  const response = await fetch(path);
  const manifest = await response.json();
  return manifest;
}

export async function getPowergraph(path: string): Promise<JsonPowerGraph> {
  const response = await fetch(path);
  const manifest = await response.json();
  return manifest;
}

export class ManifestLoader {
  loadingManager: LoadingManager;

  constructor(loadingManager: LoadingManager) {
    console.log('Created ManifestLoader');
    this.loadingManager = loadingManager;
  }

  load(
    url: string,
    onLoad?: (data: Manifest) => void,
    onProgress?: (event: Object) => void,
    onError?: (err: unknown) => void
  ): void {
    fetch(url)
      .then((response) => {
        if (onProgress) {
          onProgress({ url, loaded: 1, total: 2 });
          this.loadingManager.onProgress(url, 1, 1);
        }
        return response.json();
      })
      .then((manifest) => {
        if (onLoad) onLoad(manifest);
        this.loadingManager.onLoad();
      })
      .catch((error) => {
        if (onError) onError(error);
      });
  }
}

export class PowerGraphLoader {
  loadingManager: LoadingManager;

  constructor(loadingManager: LoadingManager) {
    console.log('Created PowerGraphLoader');
    this.loadingManager = loadingManager;
  }

  load(
    url: string,
    onLoad?: (data: JsonPowerGraph) => void,
    onProgress?: (event: Object) => void,
    onError?: (err: unknown) => void
  ): void {
    fetch(url)
      .then((response) => {
        if (onProgress) {
          onProgress({ url, loaded: 1, total: 1 });
          this.loadingManager.onProgress(url, 1, 1);
        }
        return response.json();
      })
      .then((power_graph) => {
        if (onLoad) onLoad(power_graph);
        this.loadingManager.onLoad();
      })
      .catch((error) => {
        if (onError) onError(error);
      });
  }
}
