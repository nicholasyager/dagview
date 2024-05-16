import * as THREE from 'three';
import * as d3 from 'd3';
import { Engine } from '../engine/Engine';
import { GraphNode } from './GraphNode';

export class GraphEdge extends THREE.Line {
  constructor(source: THREE.Vector3, target: THREE.Vector3) {
    const curve = new THREE.CatmullRomCurve3([source, target]);

    const points = curve.getPoints(10);
    const geometry = new THREE.BufferGeometry().setFromPoints(points);

    const material = new THREE.LineBasicMaterial({
      color: 0x7d7d7d7,
      linewidth: 3,
    });

    super(geometry, material);
  }
}

export class GraphEdge2 extends THREE.Group {
  source: THREE.Object3D;
  target: THREE.Object3D;
  selected: Boolean;
  points: THREE.Vector3[];
  curve: THREE.CatmullRomCurve3;
  // sampledPointIndices: number[];
  time: number;

  constructor(
    source: THREE.Object3D,
    target: THREE.Object3D,
    color: THREE.Color
  ) {
    const curve = new THREE.CatmullRomCurve3([
      source.position,
      target.position,
    ]);

    const particleCount = source.position.distanceTo(target.position) / 5; // Adjust the number of particles as desired
    const points = curve.getSpacedPoints(particleCount);

    const particleSize = 0.075; // Adjust the size of the particles
    const particleColor = color; // Adjust the color of the particles

    // let sampledPoints = [];
    // let sampledPointIndices = [];
    // for (let i = 0; i < points.length; i += pointIndexSize) {
    //   sampledPoints.push(points[i]);
    //   sampledPointIndices.push(i);
    //   console.log(sampledPoints);
    // }

    const material = new THREE.PointsMaterial({
      size: particleSize,
      color: particleColor,
      transparent: true,
      opacity: 0.5,
    });

    super();
    this.add(
      new THREE.Points(
        new THREE.BufferGeometry().setFromPoints(points),
        material
      )
    );

    const line = new THREE.Line(
      new THREE.BufferGeometry().setFromPoints(curve.getPoints(20)),
      new THREE.LineDashedMaterial({
        color: color, //.lerp(new THREE.Color(0x000000), 0.75),
        scale: 10,
        gapSize: 10,
        dashSize: 1,
        transparent: true,
        opacity: 1,
      })
    );
    line.computeLineDistances();
    this.add(line);

    this.selected = false;
    this.source = source;
    this.target = target;

    this.curve = curve;
    this.points = points;

    this.time = 0;
  }

  update(delta: number, engine: Engine) {
    // Update the parameter t

    this.time += 0.05 * delta;
    if (this.time > 1) this.time = 0; // Reset t to loop the animation

    // Interpolate the point's position along the line
    let currentPositions =
      this.children[0].geometry.getAttribute('position').array;

    for (let i = 0; i < this.points.length; i++) {
      let initialPosition = (i + 1) / this.points.length;
      let newPosition = this.time + initialPosition;

      if (newPosition >= 1) newPosition -= 1;
      // console.log({ time: this.time, initialPosition, newPosition });
      this.points[i] = this.curve.getPoint(newPosition);

      currentPositions[i * 3] = this.points[i].x;
      currentPositions[i * 3 + 1] = this.points[i].y;
      currentPositions[i * 3 + 2] = this.points[i].z;
    }

    //Update the line's opacity

    // Calculate distance from mesh to camera
    if (this.children.length <= 1) return;

    const position = new THREE.Vector3(
      ...this.children[1].geometry.getAttribute('position').array
    );
    const distance = position.distanceTo(engine.camera.instance.position);

    // Map the distance to an opacity value (for example, using linear mapping)
    // Adjust the mapping function as needed
    const maxDistance = 25; // Maximum distance at which the mesh should be fully transparent
    const minDistance = 5; // Minimum distance at which the mesh should be fully opaque
    const maxOpacity = 1.0; // Full opacity
    const minOpacity = 0.3; // Fully transparent

    // Ensure distance is within bounds
    const clampedDistance = Math.min(
      Math.max(distance, minDistance),
      maxDistance
    );

    // Linearly interpolate opacity based on the distance
    let opacity =
      minOpacity +
      ((maxOpacity - minOpacity) * (maxDistance - clampedDistance)) /
        (maxDistance - minDistance);

    if (this.selected) {
      opacity = 1;
    }

    // Update the material opacity
    this.children[1].material.opacity = opacity;

    this.children[0].geometry.getAttribute('position').needsUpdate = true;
  }

  select() {
    this.selected = true;
    this.children[1].material.setValues({ gapSize: 2, dashSize: 1 });
  }

  deselect() {
    this.selected = false;
    this.children[1].material.setValues({ gapSize: 10, dashSize: 1 });
  }
}
