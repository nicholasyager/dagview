import * as THREE from 'three';
import { RenderEngine } from './RenderEngine';
import { RenderLoop } from './RenderLoop';
import { DebugUI } from './interface/DebugUI';
import { Sizes } from './Sizes';
import { Camera } from './Camera';
import { Resources } from './Resources';
import { InfoConfig, InfoUI } from './interface/InfoUI';
import { Experience, ExperienceConstructor } from './Experience';
import { Loader } from './interface/Loader';
import { Raycaster } from './Raycaster';

export interface BloomParameters {
  threshold: number;
  strength: number;
  radius: number;
  exposure: number;
}

export interface EngineParameters {
  bloom: BloomParameters;
}

export class Engine {
  public readonly camera!: Camera;
  public readonly scene!: THREE.Scene;
  public readonly renderEngine!: RenderEngine;
  public readonly time!: RenderLoop;
  public readonly debug!: DebugUI;
  public readonly raycaster!: Raycaster;
  public readonly infoUI!: InfoUI;
  public readonly sizes!: Sizes;
  public readonly canvas!: HTMLCanvasElement;
  public readonly resources!: Resources;
  public readonly experience!: Experience;
  private readonly loader!: Loader;

  hasUpdated: boolean;
  hasMoved: boolean;

  params: EngineParameters;

  constructor({
    canvas,
    experience,
    info,
  }: {
    canvas: HTMLCanvasElement;
    experience: ExperienceConstructor;
    info?: InfoConfig;
  }) {
    if (!canvas) {
      throw new Error('No canvas provided');
    }

    this.params = {
      bloom: {
        threshold: 0, //0.15,
        strength: 2,
        radius: 0.5,
        exposure: 2,
      },
    };

    this.hasUpdated = true;
    this.hasMoved = true;

    this.canvas = canvas;
    this.sizes = new Sizes(this);
    this.debug = new DebugUI();

    this.scene = new THREE.Scene();
    this.camera = new Camera(this);
    this.raycaster = new Raycaster(this);
    this.infoUI = new InfoUI(info);
    this.renderEngine = new RenderEngine(this);
    this.time = new RenderLoop(this);
    this.experience = new experience(this);
    this.resources = new Resources(this.experience.resources);
    this.loader = new Loader();

    this.resources.on('loaded', () => {
      this.experience.init();
      this.loader.complete();
    });

    this.resources.on('progress', (progress: number) => {
      this.loader.setProgress(progress);
    });

    const bloomFolder = this.debug.gui.addFolder('Bloom');
    bloomFolder
      .add(this.params.bloom, 'strength', 0.0, 10.0)
      .onChange((value) => {
        this.renderEngine.composer.passes[1].strength = Number(value);
        this.time.update();
      });

    bloomFolder
      .add(this.params.bloom, 'radius', 0.0, 10.0)
      .onChange((value) => {
        this.renderEngine.composer.passes[1].radius = Number(value);
        this.time.update();
      });
  }

  update(delta: number) {
    if (!this.loader.isComplete) return;

    if (!this.hasUpdated) return;
    this.hasUpdated = false;

    this.camera.update();
    this.renderEngine.update();
    this.experience.update(delta);
    this.debug.update();
  }

  resize() {
    this.camera.resize();
    this.renderEngine.resize();
    if (this.experience.resize) {
      this.experience.resize();
    }
  }
}
