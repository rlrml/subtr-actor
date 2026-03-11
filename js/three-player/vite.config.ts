import { defineConfig } from "vite";
import path from "node:path";
import { fileURLToPath } from "node:url";

const rootDir = fileURLToPath(new URL(".", import.meta.url));
const srcDir = path.resolve(rootDir, "src");
const distDir = path.resolve(rootDir, "dist");

export default defineConfig(({ mode }) => ({
  resolve: {
    alias: {
      "@": srcDir,
      "@subtr-actor-wasm": path.resolve(
        rootDir,
        "../pkg/rl_replay_subtr_actor.js"
      ),
    },
  },
  server: {
    fs: {
      allow: [path.resolve(rootDir, "..")],
    },
  },
  build:
    mode === "library"
      ? {
          outDir: path.resolve(distDir, "lib"),
          emptyOutDir: false,
          lib: {
            entry: path.resolve(srcDir, "lib.ts"),
            name: "SubtrActorThreePlayer",
            fileName: "subtr-actor-three-player",
            formats: ["es"],
          },
          rollupOptions: {
            external: ["three"],
          },
        }
      : {
          outDir: path.resolve(distDir, "demo"),
          emptyOutDir: true,
        },
}));
