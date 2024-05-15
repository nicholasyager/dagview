import * as THREE from 'three';
import * as d3 from 'd3';

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

export class GraphEdge2 extends THREE.Points {
  points: THREE.Vector3[];
  curve: THREE.CatmullRomCurve3;
  // sampledPointIndices: number[];
  time: number;

  constructor(
    source: THREE.Vector3,
    target: THREE.Vector3,
    color: THREE.Color
  ) {
    const curve = new THREE.CatmullRomCurve3([source, target]);

    const particleCount = source.distanceTo(target); // Adjust the number of particles as desired
    const points = curve.getSpacedPoints(particleCount);

    const particleSize = 0.05; // Adjust the size of the particles
    const particleColor = color; // Adjust the color of the particles

    // let sampledPoints = [];
    // let sampledPointIndices = [];
    // for (let i = 0; i < points.length; i += pointIndexSize) {
    //   sampledPoints.push(points[i]);
    //   sampledPointIndices.push(i);
    //   console.log(sampledPoints);
    // }

    const geometry = new THREE.BufferGeometry().setFromPoints(points);
    const material = new THREE.PointsMaterial({
      size: particleSize,
      color: particleColor,
    });

    super(geometry, material);
    this.curve = curve;
    this.points = points;

    this.time = 0;
  }

  update(delta: number) {
    // Update the parameter t

    this.time += 0.025 * delta;
    if (this.time > 1) this.time = 0; // Reset t to loop the animation

    // Interpolate the point's position along the line
    let currentPositions = this.geometry.getAttribute('position').array;

    for (let i = 0; i < this.points.length; i++) {
      let initialPosition = (i + 1) / this.points.length;
      let newPosition = this.time + initialPosition;
      // console.log({
      //   i,
      //   points: this.points.length,
      //   initialPosition,
      //   newPosition,
      //   time: this.time,
      // });
      if (newPosition >= 1) newPosition -= 1;
      // console.log({ time: this.time, initialPosition, newPosition });
      this.points[i] = this.curve.getPoint(newPosition);

      currentPositions[i * 3] = this.points[i].x;
      currentPositions[i * 3 + 1] = this.points[i].y;
      currentPositions[i * 3 + 2] = this.points[i].z;
    }

    // let currentPositions = this.geometry.getAttribute('position').array;

    // for (let i = 0; i < this.sampledPointIndices.length; i++) {
    //   let pointIndex = this.sampledPointIndices[i];
    //   // Get the next point

    //   if (pointIndex + 1 >= this.sampledPointIndices.length) pointIndex = -1;

    //   let nextPoint = this.points[pointIndex + 1];
    //   currentPositions[i] = nextPoint.x;
    //   currentPositions[i + 1] = nextPoint.y;
    //   currentPositions[i + 2] = nextPoint.z;
    // }

    this.geometry.getAttribute('position').needsUpdate = true;
  }
}
