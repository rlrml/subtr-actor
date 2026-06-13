import { defineConfig } from "vite";
import path from "node:path";
import wasm from "vite-plugin-wasm";

// Dev/bring-up config for the high-fidelity viewer. Serves public/ (GLB models +
// draco) at the web root and resolves the subtr-actor WASM from ../pkg.
export default defineConfig(() => ({
  resolve: {
    alias: {
      "@rlrml/subtr-actor": path.resolve(import.meta.dirname, "../pkg/rl_replay_subtr_actor.js"),
      // Dedupe three so examples/jsm and addons resolve against one copy.
      three: path.resolve(import.meta.dirname, "node_modules/three"),
    },
  },
  plugins: [wasm()],
  optimizeDeps: {
    exclude: ["@rlrml/subtr-actor"],
  },
  server: {
    fs: {
      // Allow importing the wasm package and repo assets from outside the package.
      allow: [path.resolve(import.meta.dirname, ".."), path.resolve(import.meta.dirname, "../..")],
    },
  },
}));
