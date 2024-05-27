import * as THREE from 'three';
import * as d3 from 'd3';
import { Engine } from '../engine/Engine';
import { Raycaster } from '../engine/Raycaster';
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

function findIndexOfRange(arr: [number, number][], num: number): number {
  for (let i = 0; i < arr.length; i++) {
    const [min, max] = arr[i];
    if (num >= min && num <= max) {
      return i;
    }
  }
  return -1; // If the number is not within any range
}

export class GraphEdge2 extends THREE.Group {
  source: GraphNode;
  target: GraphNode;
  selected: Boolean;
  dimmed: Boolean;
  points: THREE.Vector3[];
  curve: THREE.CatmullRomCurve3;
  // sampledPointIndices: number[];
  time: number;

  // Point properties
  particleSize: number;
  particleColor: THREE.Color;
  endpointVisible: boolean;

  constructor(
    name: string,
    source: GraphNode,
    target: GraphNode,
    color: THREE.Color
  ) {
    // console.log(color);
    const curve = new THREE.CatmullRomCurve3([
      source.position,
      target.position,
    ]);

    const particleCount = source.position.distanceTo(target.position); // Adjust the number of particles as desired
    const points = curve.getSpacedPoints(particleCount);

    // let sampledPoints = [];
    // let sampledPointIndices = [];
    // for (let i = 0; i < points.length; i += pointIndexSize) {
    //   sampledPoints.push(points[i]);
    //   sampledPointIndices.push(i);
    //   console.log(sampledPoints);
    // }

    super();
    this.name = name;
    this.endpointVisible = true;

    const line = new THREE.Line(
      new THREE.BufferGeometry().setFromPoints(curve.getPoints(20)),
      new THREE.LineDashedMaterial({
        color: color.lerp(new THREE.Color(0x000000), 0.8),
        scale: 10,
        gapSize: 0,
        dashSize: 1,
        transparent: true,
        opacity: 0.075,
      })
    );
    line.computeLineDistances();
    this.add(line);

    this.selected = false;
    this.dimmed = false;
    this.source = source;
    this.target = target;

    this.curve = curve;
    this.points = points;

    this.time = 0;

    this.particleSize = 0.075; // Adjust the size of the particles
    this.particleColor = color; // Adjust the color of the particles
  }

  handleCameraMove(raycaster: Raycaster) {
    // Confirm if the edge should be viewable.
    this.endpointVisible =
      raycaster.isSeen(this.source) || raycaster.isSeen(this.target);
  }

  update(delta: number, engine: Engine) {
    // Update the parameter t

    this.time += 0.05 * delta;
    if (this.time > 1) this.time = 0; // Reset t to loop the animation

    //Update the line's opacity

    // Calculate distance from mesh to camera
    if (this.children.length < 1) return;

    const position = new THREE.Vector3(
      ...this.children[0].geometry.getAttribute('position').array
    );
    const distance = position.distanceTo(engine.camera.instance.position);

    let distanceColorScale = d3.scaleSequential([0, 5], d3.interpolatePuOr);

    const distanceArrays: [number, number][] = [
      [0, 5],
      [5, 10],
      [10, 30],
      [30, 75],
      [75, 500],
      [500, 1000],
    ];

    // this.children[0].material.color = new THREE.Color(
    //   distanceColorScale(findIndexOfRange(distanceArrays, distance))
    // );

    // Map the distance to an opacity value (for example, using linear mapping)
    // Adjust the mapping function as needed
    const maxPointDistance = 25; // Maximum distance at which the mesh should be fully transparent
    const minPointDistance = 5; // Minimum distance at which the mesh should be fully opaque
    const maxOpacity = 1; // Full opacity

    const minOpacity = 0.05; // Fully transparent

    const minLineOpacity = 0.1;
    const maxLineOpacity = 0.5;

    let lineOpacity = this.calculateLineOpacity(distance, distanceArrays, [
      [minLineOpacity, minLineOpacity],
      [maxLineOpacity, minLineOpacity],
      [maxLineOpacity, maxLineOpacity],
      [minLineOpacity, maxLineOpacity],
      [minLineOpacity, minLineOpacity],
      [minLineOpacity, 0.0],
    ]);

    if (this.selected) {
      lineOpacity = 1;
    }

    if (this.dimmed) {
      lineOpacity = lineOpacity / 2;
    }

    if (!this.endpointVisible) {
      lineOpacity = minLineOpacity;
    }

    this.children[0].material.opacity = lineOpacity;

    // Ensure distance is within bounds
    const clampedPointDistance = Math.min(
      Math.max(distance, minPointDistance),
      maxPointDistance
    );

    // Linearly interpolate opacity based on the distance
    let pointOpacity =
      minOpacity +
      ((maxOpacity - minOpacity) * (maxPointDistance - clampedPointDistance)) /
        (maxPointDistance - minPointDistance);

    if (this.selected) {
      pointOpacity = 1;
    }

    // Update the material opacity

    if (this.selected && this.children.length >= 2) {
      this.children[1].material.opacity = pointOpacity;
      // Interpolate the point's position along the line
      let currentPositions =
        this.children[1].geometry.getAttribute('position').array;

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
      this.children[1].geometry.getAttribute('position').needsUpdate = true;
    }
  }

  select() {
    this.selected = true;
    this.children[0].material.setValues({ gapSize: 2, dashSize: 1 });

    this.add(
      new THREE.Points(
        new THREE.BufferGeometry().setFromPoints(this.points),
        new THREE.PointsMaterial({
          size: this.particleSize,
          color: this.particleColor,
          transparent: true,
          opacity: 0.5,
        })
      )
    );
  }

  dim() {
    this.dimmed = true;
  }

  dedim() {
    this.dimmed = false;
  }

  deselect() {
    this.selected = false;
    this.children[0].material.setValues({ gapSize: 0, dashSize: 1 });

    this.remove(this.children[1]);
  }

  calculateLineOpacity(
    distance: number,
    distanceRanges: [number, number][],
    opacityRanges: [number, number][]
  ): number {
    // For a given distance, and ranges of opacities, calculate the appropriate opacity.

    for (let rangeIndex = 0; rangeIndex < distanceRanges.length; rangeIndex++) {
      const distanceRange = distanceRanges[rangeIndex];
      const opacityRange = opacityRanges[rangeIndex];
      if (!(distance >= distanceRange[0] && distance < distanceRange[1])) {
        continue;
      }

      const clampedLineDistance = Math.min(
        Math.max(distance, distanceRange[0]),
        distanceRange[1]
      );

      let output =
        opacityRange[0] +
        ((opacityRange[1] - opacityRange[0]) *
          (distanceRange[1] - clampedLineDistance)) /
          (distanceRange[1] - distanceRange[0]);

      // if (rangeIndex >= 1 && rangeIndex <= 3)
      // console.log(distance, distanceRange, opacityRange, output);
      return output;
    }

    return 1;
  }
}
