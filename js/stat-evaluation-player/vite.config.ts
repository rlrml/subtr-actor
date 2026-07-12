import { defineConfig } from "vite";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
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

const REVIEW_LABEL_DATASET_PATTERN = /^[a-z0-9][a-z0-9-_]*$/;

function reviewLabelsFilePath(dataset: string): string {
  // Repo root is two directories above js/stat-evaluation-player.
  return path.resolve(import.meta.dirname, "../..", "labels", dataset, "labels.jsonl");
}

/**
 * Flat-file review-label sink for the dev server.
 *
 * POST /review-labels/:dataset with a JSON body ({status, item_id, meta})
 * appends one JSON line to <repo-root>/labels/<dataset>/labels.jsonl,
 * folding in any query-string parameters on the endpoint URL (candidate,
 * provenance, replay, frame, player, ...). GET returns the file as text.
 */
function reviewLabelsSinkPlugin() {
  return {
    name: "review-labels-sink",
    configureServer(server: any) {
      server.middlewares.use((req: any, res: any, next: () => void) => {
        const url = new URL(req.url ?? "/", "http://localhost");
        const match = url.pathname.match(/^\/review-labels\/([^/]+)$/);
        if (!match) {
          next();
          return;
        }

        const respondJson = (statusCode: number, payload: unknown) => {
          res.statusCode = statusCode;
          res.setHeader("content-type", "application/json");
          res.end(JSON.stringify(payload));
        };

        const dataset = decodeURIComponent(match[1]);
        if (!REVIEW_LABEL_DATASET_PATTERN.test(dataset)) {
          respondJson(400, { error: `Invalid dataset name: ${dataset}` });
          return;
        }
        const filePath = reviewLabelsFilePath(dataset);

        if (req.method === "GET") {
          res.statusCode = 200;
          res.setHeader("content-type", "text/plain; charset=utf-8");
          try {
            res.end(fs.readFileSync(filePath, "utf8"));
          } catch {
            res.end("");
          }
          return;
        }

        if (req.method !== "POST") {
          respondJson(405, { error: "Only GET and POST are supported." });
          return;
        }

        let rawBody = "";
        req.on("data", (chunk: unknown) => {
          rawBody += String(chunk);
        });
        req.on("end", () => {
          let body: Record<string, unknown>;
          try {
            const parsed: unknown = rawBody ? JSON.parse(rawBody) : {};
            if (typeof parsed !== "object" || parsed === null || Array.isArray(parsed)) {
              throw new Error("body must be a JSON object");
            }
            body = parsed as Record<string, unknown>;
          } catch (error: unknown) {
            respondJson(400, {
              error: `Invalid JSON body: ${error instanceof Error ? error.message : String(error)}`,
            });
            return;
          }

          const record = {
            at: new Date().toISOString(),
            dataset,
            status: body.status ?? null,
            item_id: body.item_id ?? null,
            ...Object.fromEntries(url.searchParams.entries()),
            meta: body.meta ?? null,
          };
          try {
            fs.mkdirSync(path.dirname(filePath), { recursive: true });
            // Single synchronous single-line append keeps concurrent label
            // submissions from interleaving partial lines.
            fs.appendFileSync(filePath, `${JSON.stringify(record)}\n`);
          } catch (error: unknown) {
            respondJson(500, { error: error instanceof Error ? error.message : String(error) });
            return;
          }
          respondJson(200, { ok: true });
        });
      });
    },
  };
}

function reviewReplayFsAllowDirs(): string[] {
  return (process.env.REVIEW_REPLAY_DIRS ?? "")
    .split(/[:,]/)
    .map((entry) => entry.trim())
    .filter(Boolean)
    .map((entry) => path.resolve(entry));
}

export default defineConfig(({ command, mode }) => {
  const siteBuild = mode === "site";
  const useLocalAliases = command === "serve" || siteBuild;

  return {
    base: "./",
    plugins: [wasm(), ensureWasmBindingsPlugin(), reviewLabelsSinkPlugin()],
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
          // External replay directories referenced by review playlists,
          // served through /@fs/. Colon- or comma-separated, e.g.
          // REVIEW_REPLAY_DIRS=/path/to/replays npm run dev
          ...reviewReplayFsAllowDirs(),
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
