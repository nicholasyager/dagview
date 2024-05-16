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

  constructor(private engine: Engine) {
    super();
    this.raycaster = new THREE.Raycaster();
    this.pointer = new THREE.Vector2();

    const pointerState: PointerState = {
      isDragging: false,
      startX: 0,
      startY: 0,
    };

    document.addEventListener('mousedown', (event: MouseEvent) => {
      pointerState.startX = event.clientX;
      pointerState.startY = event.clientY;
      pointerState.isDragging = false;
    });

    document.addEventListener('mousemove', (event) => {
      const x = (event.clientX / this.engine.sizes.width) * 2 - 1;
      const y = -(event.clientY / this.engine.sizes.height) * 2 + 1;
      this.setPointer(x, y);
      this.update();
      if (this.listenerCount('move')) {
        this.emit('move', this.getIntersections());
      }

      const diffX = Math.abs(event.clientX - pointerState.startX);
      const diffY = Math.abs(event.clientY - pointerState.startY);

      if (diffX > DRAG_THRESHOLD || diffY > DRAG_THRESHOLD) {
        pointerState.isDragging = true;
      }
    });

    document.addEventListener('mouseup', (event: MouseEvent) => {
      if (!(event.target instanceof HTMLCanvasElement)) return;

      if (pointerState.isDragging) {
        console.log('Drag event detected');
      } else {
        const point = this.mouseEventToVector2(event);
        this.setPointer(point.x, point.y);
        this.update();
        if (this.listenerCount('click')) {
          this.emit('click', this.getIntersections());
        }
      }

      // Reset state
      pointerState.isDragging = false;
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

  private mouseEventToVector2(event: MouseEvent) {
    const x = (event.clientX / this.engine.sizes.width) * 2 - 1;
    const y = -(event.clientY / this.engine.sizes.height) * 2 + 1;
    return new THREE.Vector2(x, y);
  }
}
