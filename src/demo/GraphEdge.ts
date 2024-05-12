import * as THREE from 'three'

export class GraphEdge extends THREE.Line {
  constructor(source: THREE.Vector3, target: THREE.Vector3) {
    const curve = new THREE.CatmullRomCurve3([source, target])

    const points = curve.getPoints(20)
    const geometry = new THREE.BufferGeometry().setFromPoints(points)

    const material = new THREE.LineBasicMaterial({ color: 0xff0000 })

    super(geometry, material)
  }
}
