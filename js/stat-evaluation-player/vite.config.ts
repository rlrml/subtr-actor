import { defineConfig } from "vite";
import path from "node:path";
import wasm from "vite-plugin-wasm";
import {
  ensureWasmPackageFresh,
  getWasmWatchTargets,
  isWasmSourcePath,
} from "../scripts/ensure-wasm-package.mjs";

function ensureWasmBindingsPlugin() {
  let rebuild = Promise.resolve();

  const queueRebuild = (force = false) => {
    rebuild = rebuild.then(() =>
      ensureWasmPackageFresh({
        force,
        log: (message: string) => console.log(message),
      }),
    );
    return rebuild;
  };

  return {
    name: "ensure-wasm-bindings",
    async buildStart() {
      await queueRebuild();
    },
    async configureServer(server: any) {
      await queueRebuild();
      server.watcher.add(getWasmWatchTargets());

      const rebuildOnChange = async (filePath: string) => {
        if (!isWasmSourcePath(filePath)) {
          return;
        }
        try {
          await queueRebuild(true);
          server.ws.send({ type: "full-reload" });
        } catch (error: unknown) {
          server.config.logger.error(error instanceof Error ? error.message : String(error));
        }
      };

      server.watcher.on("change", rebuildOnChange);
      server.watcher.on("add", rebuildOnChange);
      server.watcher.on("unlink", rebuildOnChange);
    },
  };
}

export default defineConfig(({ command, mode }) => {
  const siteBuild = mode === "site";
  const useLocalAliases = command === "serve" || siteBuild;

  return {
    base: "./",
    plugins: [wasm(), ensureWasmBindingsPlugin()],
    // Site/dev builds serve the bundled 3D viewer assets beside the app. The
    // viewer resolves them through Vite's BASE_URL so subpath deployments such
    // as GitHub Pages do not depend on web-root /models or /draco paths.
    publicDir: useLocalAliases ? path.resolve(import.meta.dirname, "../player/public") : false,
    resolve: {
      alias: useLocalAliases
        ? {
            "@rlrml/subtr-actor": path.resolve(
              import.meta.dirname,
              "../pkg/rl_replay_subtr_actor.js",
            ),
            "@rlrml/player": path.resolve(import.meta.dirname, "../player/src/lib.ts"),
            "camera-controls": path.resolve(import.meta.dirname, "node_modules/camera-controls"),
            three: path.resolve(import.meta.dirname, "node_modules/three"),
          }
        : undefined,
    },
    server: {
      fs: {
        allow: [
          path.resolve(import.meta.dirname, ".."),
          path.resolve(import.meta.dirname, "../.."),
        ],
      },
    },
    worker: {
      rollupOptions: {
        // Site builds need the wasm package bundled into workers, while the
        // published library keeps it external for downstream consumers.
        external: siteBuild ? [] : ["@rlrml/subtr-actor", "@rlrml/player"],
      },
    },
    optimizeDeps: {
      exclude: ["@rlrml/subtr-actor", "@rlrml/player"],
    },
    build: siteBuild
      ? {
          outDir: path.resolve(import.meta.dirname, "dist"),
          emptyOutDir: true,
        }
      : {
          outDir: path.resolve(import.meta.dirname, "dist"),
          emptyOutDir: true,
          lib: {
            entry: path.resolve(import.meta.dirname, "src/lib.ts"),
            name: "SubtrActorStatEvaluationPlayer",
            fileName: "index",
            formats: ["es"],
          },
          rollupOptions: {
            external: ["@rlrml/subtr-actor", "@rlrml/player", "three"],
          },
        },
  };
});
