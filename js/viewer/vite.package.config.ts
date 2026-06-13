import path from "node:path";
import { defineConfig } from "vite";
import wasm from "vite-plugin-wasm";

export default defineConfig({
  plugins: [wasm()],
  build: {
    copyPublicDir: false,
    lib: {
      entry: path.resolve(import.meta.dirname, "src/lib.ts"),
      formats: ["es"],
      fileName: () => "index.js",
    },
    rollupOptions: {
      external: [
        "@rlrml/player",
        "@rlrml/subtr-actor",
        "camera-controls",
        "eventemitter3",
        "three",
        /^three\//,
      ],
    },
  },
});
