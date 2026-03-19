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
      external: ["@colonelpanic8/subtr-actor"],
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
      entry: path.resolve(srcDir, "lib.ts"),
      name: "SubtrActorPlayer",
      fileName: "index",
      formats: ["es"],
    },
    rollupOptions: {
      external: ["@colonelpanic8/subtr-actor", "three"],
    },
  },
});
