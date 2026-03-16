import { defineConfig } from "vite";
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
        log: (message) => console.log(message),
      })
    );
    return rebuild;
  };

  return {
    name: "ensure-wasm-bindings",
    async buildStart() {
      await queueRebuild();
    },
    async configureServer(server) {
      await queueRebuild();
      server.watcher.add(getWasmWatchTargets());

      const rebuildOnChange = async (filePath) => {
        if (!isWasmSourcePath(filePath)) {
          return;
        }

        try {
          await queueRebuild(true);
          server.ws.send({ type: "full-reload" });
        } catch (error) {
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

export default defineConfig({
  plugins: [ensureWasmBindingsPlugin()],
  server: {
    fs: {
      allow: [".."],
    },
  },
  optimizeDeps: {
    exclude: ["rl-replay-subtr-actor"],
  },
  assetsInclude: ["**/*.wasm"],
});
