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

  constructor(name: string, objectPath: GraphNode[], color: THREE.Color) {
    let source = objectPath[0];
    let target = objectPath[objectPath.length - 1];
    // console.log(color);
    const curve = new THREE.CatmullRomCurve3(
      objectPath.map((item) => item.position)
    );

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
    this.endpointVisible = raycaster.isSeen(this.children[0]);
    // raycaster.isSeen(this.source) || raycaster.isSeen(this.target);
  }

  updateEdgeOpacity(engine: Engine) {
    const minLineOpacity = 0.01;
    const maxLineOpacity = 0.5;

    // Calculate distance from mesh to camera
    if (this.children.length < 1) return;

    let terminiPositions = [this.source.position, this.target.position];

    const zoom = engine.camera.instance.position.distanceTo(
      engine.camera.controls.target
    );

    const distance = Math.min(
      ...terminiPositions.map((position) => {
        return position.distanceTo(engine.camera.controls.target);
      })
    );

    let percentDistance = zoom / (distance + 0.0001);
    if (percentDistance > 1) {
      percentDistance = 1;
    }

    let lineOpacity = d3.interpolateBasis([
      minLineOpacity,
      minLineOpacity,

      maxLineOpacity,
      // minLineOpacity,
    ])(percentDistance);

    // console.log({
    //   name: this.name,
    //   zoom,
    //   distance,
    //   percentDistance,
    //   lineOpacity,
    // });

    // this.children[0].material.color = new THREE.Color(
    //   d3.interpolateTurbo(lineOpacity)
    // );

    // lineOpacity = 0.2;

    if (this.selected) {
      lineOpacity = 1;
    } else if (this.dimmed) {
      lineOpacity = lineOpacity / 4;
    }

    if (!this.endpointVisible) {
      lineOpacity = minLineOpacity;
    }

    this.children[0].material.opacity = lineOpacity;
  }

  update(delta: number, engine: Engine) {
    // Update the parameter t

    if (!this.endpointVisible) return;

    this.time += 0.05 * delta;
    if (this.time > 1) this.time = 0; // Reset t to loop the animation

    //Update the line's opacity

    let pointOpacity = 1;

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

      // const clampedLineDistance = Math.min(
      //   Math.max(distance, distanceRange[0]),
      //   distanceRange[1]
      // );
      let percentage =
        (distance - distanceRange[0]) / (distanceRange[1] - distanceRange[0]);

      return d3.interpolateNumber(opacityRange[0], opacityRange[1])(percentage);

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
