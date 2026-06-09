import { defineConfig } from "vite";
import path from "node:path";
import { fileURLToPath } from "node:url";

const rootDir = fileURLToPath(new URL(".", import.meta.url));
const srcDir = path.resolve(rootDir, "src");
const distDir = path.resolve(rootDir, "dist");

export default defineConfig({
  base: "./",
  resolve: {
    alias: {
      "@": srcDir,
    },
  },
  worker: {
    rollupOptions: {
      // The player library keeps the wasm package external so consumers resolve it.
      external: ["@rlrml/subtr-actor"],
    },
  },
  server: {
    fs: {
      allow: [path.resolve(rootDir, "..")],
    },
  },
  build: {
    outDir: distDir,
    emptyOutDir: true,
    lib: {
      // `boost-units` is a dependency-free entry so consumers can rescale boost
      // for display without pulling in the full player (three.js / wasm).
      entry: {
        index: path.resolve(srcDir, "lib.ts"),
        "boost-units": path.resolve(srcDir, "boost-units.ts"),
      },
      formats: ["es"],
    },
    rollupOptions: {
      external: ["@rlrml/subtr-actor", "three"],
    },
  },
});
