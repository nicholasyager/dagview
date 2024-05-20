import { WebGLRenderer } from 'three';
import { Engine } from './Engine';
import * as THREE from 'three';
import { VRButton } from 'three/examples/jsm/webxr/VRButton.js';
import { GameEntity } from './GameEntity';
import { EffectComposer } from 'three/examples/jsm/postprocessing/EffectComposer';
import { RenderPass } from 'three/examples/jsm/postprocessing/RenderPass';
import { UnrealBloomPass } from 'three/examples/jsm/postprocessing/UnrealBloomPass.js';

const BLOOM_SCENE = 1;

export class RenderEngine implements GameEntity {
  renderer: WebGLRenderer;
  composer: EffectComposer;

  constructor(private engine: Engine) {
    this.renderer = new WebGLRenderer({
      canvas: this.engine.canvas,
      antialias: true,
    });

    this.renderer.outputEncoding = THREE.sRGBEncoding;
    this.renderer.toneMapping = THREE.CineonToneMapping;
    this.renderer.toneMappingExposure = 1.75;
    this.renderer.shadowMap.enabled = true;
    this.renderer.shadowMap.type = THREE.PCFSoftShadowMap;

    this.renderer.setClearColor('#000000');
    this.renderer.setSize(this.engine.sizes.width, this.engine.sizes.height);
    this.renderer.setPixelRatio(Math.min(this.engine.sizes.pixelRatio, 2));

    this.composer = new EffectComposer(this.renderer);

    const renderPass = new RenderPass(
      this.engine.scene,
      this.engine.camera.instance
    );
    this.composer.addPass(renderPass);

    const bloomLayer = new THREE.Layers();
    bloomLayer.set(BLOOM_SCENE);

    const bloomPass = new UnrealBloomPass(
      new THREE.Vector2(window.innerWidth, window.innerHeight),
      1.5,
      0.4,
      0.85
    );
    bloomPass.threshold = this.engine.params.bloom.threshold;
    bloomPass.strength = this.engine.params.bloom.strength;
    bloomPass.radius = this.engine.params.bloom.radius;
    this.composer.addPass(bloomPass);

    // document.body.appendChild(VRButton.createButton(this.renderer));
    // this.renderer.xr.enabled = true;
  }

  update() {
    this.composer.render();
  }

  resize() {
    // Check if the VR device is presenting
    const isVRPresenting = this.renderer.xr.isPresenting;

    // Change the size only if the VR device is not presenting
    if (!isVRPresenting) {
      this.renderer.setSize(this.engine.sizes.width, this.engine.sizes.height);
      this.composer.setSize(this.engine.sizes.width, this.engine.sizes.height);
    }

    this.composer.render();
  }
}
