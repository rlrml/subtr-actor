import { defineConfig } from 'vite';

export default defineConfig({
  server: {
    fs: {
      allow: ['..']
    }
  },
  optimizeDeps: {
    exclude: ['rl-replay-subtr-actor']
  },
  assetsInclude: ['**/*.wasm']
});