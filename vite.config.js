import glsl from 'vite-plugin-glsl';
import basicSsl from '@vitejs/plugin-basic-ssl';
import { defineConfig } from 'vite';
import wasmPack from './src/wasmPack';

export default defineConfig({
  plugins: [glsl(), basicSsl(), wasmPack('./src/wasm/powergraph')],
});
