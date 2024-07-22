import { EventEmitter } from './utilities/EventEmitter';
import * as THREE from 'three';
import { Engine } from './Engine';

interface PointerState {
  isDragging: boolean;
  startX: number;
  startY: number;
}

const DRAG_THRESHOLD = 10;

export type RaycasterEvent = THREE.Intersection<
  THREE.Object3D<THREE.Object3DEventMap>
>;

export class Raycaster extends EventEmitter {
  private raycaster: THREE.Raycaster;
  private pointer: THREE.Vector2;
  // private cameraViewProjectionMatrix: THREE.Matrix4;
  // private frustum: THREE.Frustum;
  pointerState: PointerState;

  constructor(private engine: Engine) {
    super();
    this.raycaster = new THREE.Raycaster();
    this.pointer = new THREE.Vector2();

    this.pointerState = {
      isDragging: false,
      startX: 0,
      startY: 0,
    };

    document.addEventListener('mousedown', (event: MouseEvent) => {
      this.pointerState.startX = event.clientX;
      this.pointerState.startY = event.clientY;
      this.pointerState.isDragging = false;
    });

    document.addEventListener('mousemove', (event) => {
      const x = (event.clientX / this.engine.sizes.width) * 2 - 1;
      const y = -(event.clientY / this.engine.sizes.height) * 2 + 1;
      this.setPointer(x, y);
      this.update();
      if (this.listenerCount('move')) {
        this.emit('move', this.getIntersections());
      }

      const diffX = Math.abs(event.clientX - this.pointerState.startX);
      const diffY = Math.abs(event.clientY - this.pointerState.startY);

      if (diffX > DRAG_THRESHOLD || diffY > DRAG_THRESHOLD) {
        this.pointerState.isDragging = true;
      }
    });

    document.addEventListener('wheel', (event: WheelEvent) => {
      if (!(event.target instanceof HTMLCanvasElement)) return;
      if (this.listenerCount('cameraZoom')) {
        this.emit('cameraZoom', this);
      }
    });

    document.addEventListener('mouseup', (event: MouseEvent) => {
      if (!(event.target instanceof HTMLCanvasElement)) return;

      if (this.pointerState.isDragging) {
        console.log('Drag event detected');
        if (this.listenerCount('cameraMove')) {
          this.emit('cameraMove', this);
        }
      } else {
        const point = this.mouseEventToVector2(event);
        this.setPointer(point.x, point.y);
        this.update();
        if (this.listenerCount('click')) {
          this.emit('click', this.getIntersections());
        }
      }

      // Reset state
      this.pointerState.isDragging = false;
    });

    document.addEventListener('dblclick', (event: MouseEvent) => {
      if (!(event.target instanceof HTMLCanvasElement)) return;

      const point = this.mouseEventToVector2(event);
      this.setPointer(point.x, point.y);
      this.update();
      if (this.listenerCount('dblclick')) {
        this.emit('dblclick', this.getIntersections());
      }
    });
  }

  public update() {
    this.raycaster.setFromCamera(this.pointer, this.engine.camera.instance);
  }

  public setPointer(x: number, y: number) {
    this.pointer.x = x;
    this.pointer.y = y;
  }

  public getIntersections() {
    let intersections = this.raycaster.intersectObjects(
      this.engine.scene.children,
      true
    );
    return intersections;
  }

  public isSeen(object: THREE.Object3D) {
    this.engine.camera.instance.updateMatrix();
    this.engine.camera.instance.updateMatrixWorld();
    var frustum = new THREE.Frustum();
    frustum.setFromProjectionMatrix(
      new THREE.Matrix4().multiplyMatrices(
        this.engine.camera.instance.projectionMatrix,
        this.engine.camera.instance.matrixWorldInverse
      )
    );

    // Check if the object's bounding box is within the frustum
    return frustum.intersectsObject(object);
  }

  private mouseEventToVector2(event: MouseEvent) {
    const x = (event.clientX / this.engine.sizes.width) * 2 - 1;
    const y = -(event.clientY / this.engine.sizes.height) * 2 + 1;
    return new THREE.Vector2(x, y);
  }
}
