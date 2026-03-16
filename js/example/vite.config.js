import path from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig } from "vite";
import {
  ensureWasmPackageFresh,
  getWasmWatchTargets,
  isWasmSourcePath,
} from "../scripts/ensure-wasm-package.mjs";

const exampleDir = fileURLToPath(new URL(".", import.meta.url));

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
  base: "./",
  plugins: [ensureWasmBindingsPlugin()],
  server: {
    fs: {
      allow: [".."],
    },
  },
  optimizeDeps: {
    exclude: ["subtr-actor"],
  },
  resolve: {
    alias: {
      three: path.resolve(exampleDir, "node_modules/three"),
    },
  },
  assetsInclude: ["**/*.wasm"],
});
