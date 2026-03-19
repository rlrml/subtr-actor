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
      })
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
          server.config.logger.error(
            error instanceof Error ? error.message : String(error)
          );
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
    resolve: {
      alias: useLocalAliases
        ? {
          "@colonelpanic8/subtr-actor": path.resolve(
            import.meta.dirname,
            "../pkg/rl_replay_subtr_actor.js",
          ),
          "subtr-actor-player": path.resolve(import.meta.dirname, "../player/src/lib.ts"),
          three: path.resolve(import.meta.dirname, "node_modules/three"),
        }
        : undefined,
    },
    server: {
      fs: {
        allow: [path.resolve(import.meta.dirname, "..")],
      },
    },
    worker: {
      rollupOptions: {
        external: ["@colonelpanic8/subtr-actor", "subtr-actor-player"],
      },
    },
    optimizeDeps: {
      exclude: ["@colonelpanic8/subtr-actor", "subtr-actor-player"],
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
          external: ["@colonelpanic8/subtr-actor", "subtr-actor-player", "three"],
        },
      },
  };
});
