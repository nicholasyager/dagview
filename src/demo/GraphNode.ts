import * as THREE from 'three'
import vertexShader from './shader.vert'
import fragmentShader from './shader.frag'

export class GraphNode extends THREE.Mesh {
  uniqueId: string
  nodeData: object

  constructor(uniqueId: string, nodeData: Object) {
    const geometry = new THREE.SphereGeometry(0.1)
    const material = new THREE.ShaderMaterial({
      vertexShader,
      fragmentShader,
    })

    super(geometry, material)

    this.uniqueId = uniqueId
    this.nodeData = nodeData
  }
}
