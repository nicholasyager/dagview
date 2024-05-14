import { LoadingManager } from 'three'

export interface DbtNode {
  resource_type: string
  unique_id: string
}

export interface Manifest {
  nodes: {
    [key: string]: DbtNode
  }
  child_map: {
    [key: string]: string[]
  }
  parent_map: {
    [key: string]: string[]
  }
}

export async function getManifest(path: string): Promise<Manifest> {
  const response = await fetch(path)
  const manifest = await response.json()
  return manifest
}

export class ManifestLoader {
  loadingManager: LoadingManager

  constructor(loadingManager: LoadingManager) {
    console.log('Created ManifestLoader')
    this.loadingManager = loadingManager
  }

  load(
    url: string,
    onLoad?: (data: Manifest) => void,
    onProgress?: (event: Object) => void,
    onError?: (err: unknown) => void
  ): void {
    fetch(url)
      .then((response) => {
        if (onProgress) onProgress({ url, loaded: 1, total: 1 })
        return response.json()
      })
      .then((manifest) => {
        if (onLoad) onLoad(manifest)
        this.loadingManager.onLoad()
      })
      .catch((error) => {
        if (onError) onError(error)
      })
  }
}
