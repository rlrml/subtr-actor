#!/usr/bin/env node

import { mkdtemp, rm } from "node:fs/promises";
import { existsSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { spawn, spawnSync } from "node:child_process";
import net from "node:net";
import { createServer } from "vite";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const packageDir = path.resolve(scriptDir, "..");
const repoRoot = path.resolve(packageDir, "../..");
const resultPrefix = "__SUBTR_ACTOR_LOAD_BENCH__";

function parseArgs(argv) {
  const options = {
    replay: path.resolve(repoRoot, "assets/rlcs-2025-worlds-grand-final-flcn-nrg-g5.replay"),
    iterations: 1,
    chrome: process.env.CHROME_BIN ?? null,
    timeoutMs: 120_000,
    headless: true,
  };

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    const readValue = () => {
      const value = argv[index + 1];
      if (!value || value.startsWith("--")) {
        throw new Error(`Missing value for ${arg}`);
      }
      index += 1;
      return value;
    };

    if (arg === "--replay") {
      options.replay = path.resolve(readValue());
    } else if (arg === "--iterations") {
      options.iterations = Number.parseInt(readValue(), 10);
    } else if (arg === "--chrome") {
      options.chrome = readValue();
    } else if (arg === "--timeout-ms") {
      options.timeoutMs = Number.parseInt(readValue(), 10);
    } else if (arg === "--headed") {
      options.headless = false;
    } else if (arg === "--help" || arg === "-h") {
      printUsage();
      process.exit(0);
    } else {
      throw new Error(`Unknown argument: ${arg}`);
    }
  }

  if (!Number.isFinite(options.iterations) || options.iterations < 1) {
    throw new Error("--iterations must be a positive integer");
  }
  if (!Number.isFinite(options.timeoutMs) || options.timeoutMs < 1_000) {
    throw new Error("--timeout-ms must be at least 1000");
  }

  return options;
}

function printUsage() {
  console.log(`Usage: npm run bench:load -- [options]

Options:
  --replay <path>       Replay file to load.
  --iterations <count>  Number of Chrome runs. Default: 1.
  --chrome <path>       Chrome/Chromium executable. Defaults to CHROME_BIN or platform lookup.
  --timeout-ms <ms>     Per-iteration timeout. Default: 120000.
  --headed              Run Chrome with a visible window.
`);
}

async function findFreePort() {
  const server = net.createServer();
  await new Promise((resolve, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", resolve);
  });
  const address = server.address();
  await new Promise((resolve) => server.close(resolve));
  if (!address || typeof address === "string") {
    throw new Error("Failed to allocate a local port");
  }
  return address.port;
}

function findChromeExecutable(explicitPath) {
  if (explicitPath) {
    return explicitPath;
  }

  const absoluteCandidates = [
    "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
    "/Applications/Chromium.app/Contents/MacOS/Chromium",
  ];
  for (const candidate of absoluteCandidates) {
    if (existsSync(candidate)) {
      return candidate;
    }
  }

  for (const candidate of [
    "google-chrome",
    "google-chrome-stable",
    "chromium",
    "chromium-browser",
  ]) {
    const result = spawnSync("sh", ["-lc", `command -v ${candidate}`], {
      encoding: "utf8",
    });
    const resolved = result.stdout.trim();
    if (result.status === 0 && resolved) {
      return resolved;
    }
  }

  throw new Error("Could not find Chrome or Chromium. Pass --chrome <path>.");
}

function shellQuote(value) {
  return `'${String(value).replaceAll("'", "'\\''")}'`;
}

function chromeArgs({ port, userDataDir, headless }) {
  return [
    `--remote-debugging-port=${port}`,
    `--user-data-dir=${userDataDir}`,
    "--no-first-run",
    "--no-default-browser-check",
    "--disable-background-networking",
    "--disable-extensions",
    "--disable-sync",
    "--disable-features=Translate,OptimizationHints",
    ...(headless ? ["--headless=new", "--disable-gpu"] : []),
    "about:blank",
  ];
}

async function waitForJson(url, timeoutMs) {
  const startedAt = Date.now();
  let lastError;
  while (Date.now() - startedAt < timeoutMs) {
    try {
      const response = await fetch(url);
      if (response.ok) {
        return response.json();
      }
      lastError = new Error(`${response.status} ${response.statusText}`);
    } catch (error) {
      lastError = error;
    }
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  throw new Error(`Timed out waiting for ${url}: ${lastError?.message ?? "unknown error"}`);
}

function connectCdp(wsUrl) {
  let nextId = 1;
  const pending = new Map();
  const listeners = new Map();
  const socket = new WebSocket(wsUrl);

  const opened = new Promise((resolve, reject) => {
    socket.addEventListener("open", resolve, { once: true });
    socket.addEventListener("error", reject, { once: true });
  });

  socket.addEventListener("message", (event) => {
    const message = JSON.parse(event.data);
    if (message.id !== undefined) {
      const entry = pending.get(message.id);
      if (!entry) {
        return;
      }
      pending.delete(message.id);
      if (message.error) {
        entry.reject(new Error(message.error.message));
      } else {
        entry.resolve(message.result);
      }
      return;
    }

    const callbacks = listeners.get(message.method);
    if (callbacks) {
      for (const callback of callbacks) {
        callback(message.params);
      }
    }
  });

  socket.addEventListener("close", () => {
    for (const entry of pending.values()) {
      entry.reject(new Error("CDP socket closed"));
    }
    pending.clear();
  });

  return {
    async send(method, params = {}) {
      await opened;
      const id = nextId;
      nextId += 1;
      socket.send(JSON.stringify({ id, method, params }));
      return new Promise((resolve, reject) => {
        pending.set(id, { resolve, reject });
      });
    },
    on(method, callback) {
      const callbacks = listeners.get(method) ?? new Set();
      callbacks.add(callback);
      listeners.set(method, callbacks);
      return () => callbacks.delete(callback);
    },
    close() {
      socket.close();
    },
  };
}

function benchmarkHtml(replayPath) {
  const replayUrlPath = `/@fs${pathToFileURL(replayPath).pathname}`;
  return `<!doctype html>
<meta charset="utf-8">
<title>subtr-actor replay load benchmark</title>
<pre id="status">Starting benchmark...</pre>
<script type="module">
const resultPrefix = ${JSON.stringify(resultPrefix)};
const replayUrl = ${JSON.stringify(replayUrlPath)};
const replayName = ${JSON.stringify(path.basename(replayPath))};
const statusEl = document.getElementById("status");
const startedAt = performance.now();
const stageTimings = new Map();

function now() {
  return performance.now();
}

function recordProgress(progress) {
  const timestamp = now();
  const stage = progress.stage ?? "unknown";
  const timing = stageTimings.get(stage) ?? {
    stage,
    startMs: timestamp - startedAt,
    endMs: timestamp - startedAt,
    events: 0,
    lastProgress: null,
  };
  timing.endMs = timestamp - startedAt;
  timing.events += 1;
  timing.lastProgress = progress.progress ?? null;
  stageTimings.set(stage, timing);

  statusEl.textContent = stage + " " + Math.round((progress.progress ?? 0) * 100) + "%";
}

try {
  const { loadReplayBundleInWorker } = await import("/src/replayLoader.ts");
  const fetchStart = now();
  const response = await fetch(replayUrl);
  if (!response.ok) {
    throw new Error("Failed to fetch replay: " + response.status + " " + response.statusText);
  }
  const bytes = new Uint8Array(await response.arrayBuffer());
  const fetchEnd = now();

  const loadStart = now();
  const bundle = await loadReplayBundleInWorker(bytes, {
    reportEveryNFrames: 100,
    onProgress: recordProgress,
  });
  const loadEnd = now();

  const result = {
    ok: true,
    userAgent: navigator.userAgent,
    replayName,
    replayBytes: bytes.byteLength,
    fetchMs: fetchEnd - fetchStart,
    loadMs: loadEnd - loadStart,
    totalMs: loadEnd - startedAt,
    replayFrames: bundle.replay.frameCount,
    statsFrames: bundle.statsTimeline.frames.length,
    players: bundle.replay.players.length,
    stageTimings: Array.from(stageTimings.values()).map((timing) => ({
      ...timing,
      durationMs: timing.endMs - timing.startMs,
    })),
  };
  statusEl.textContent = JSON.stringify(result, null, 2);
  console.log(resultPrefix + JSON.stringify(result));
} catch (error) {
  const result = {
    ok: false,
    message: error instanceof Error ? error.message : String(error),
    stack: error instanceof Error ? error.stack : null,
  };
  statusEl.textContent = JSON.stringify(result, null, 2);
  console.error(resultPrefix + JSON.stringify(result));
}
</script>`;
}

async function startViteServer(replayPath) {
  const server = await createServer({
    configFile: path.resolve(packageDir, "vite.config.ts"),
    root: packageDir,
    plugins: [
      {
        name: "subtr-actor-load-benchmark-page",
        configureServer(viteServer) {
          viteServer.middlewares.use("/__subtr_actor_load_benchmark", (_req, res) => {
            res.statusCode = 200;
            res.setHeader("Content-Type", "text/html; charset=utf-8");
            res.end(benchmarkHtml(replayPath));
          });
        },
      },
    ],
    server: {
      host: "127.0.0.1",
      port: 0,
      strictPort: false,
      fs: {
        allow: [repoRoot],
      },
    },
  });
  await server.listen();
  const url = server.resolvedUrls?.local?.[0];
  if (!url) {
    await server.close();
    throw new Error("Vite did not report a local server URL");
  }
  return { server, url: new URL("/__subtr_actor_load_benchmark", url).href };
}

async function runChromeBenchmark({ url, chrome, headless, timeoutMs }) {
  const port = await findFreePort();
  const userDataDir = await mkdtemp(path.join(tmpdir(), "subtr-actor-load-bench-"));
  const executable = findChromeExecutable(chrome);
  const child = spawn(executable, chromeArgs({ port, userDataDir, headless }), {
    stdio: ["ignore", "ignore", "pipe"],
  });
  let stderr = "";
  child.stderr.on("data", (chunk) => {
    stderr += chunk.toString();
  });

  try {
    const targetsUrl = `http://127.0.0.1:${port}/json/list`;
    await waitForJson(`http://127.0.0.1:${port}/json/version`, 10_000);
    const targets = await waitForJson(targetsUrl, 10_000);
    const target = targets.find((entry) => entry.type === "page");
    if (!target?.webSocketDebuggerUrl) {
      throw new Error("Could not find Chrome page target");
    }

    const cdp = connectCdp(target.webSocketDebuggerUrl);
    const consoleMessages = [];
    let settled = false;
    const resultPromise = new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        if (!settled) {
          settled = true;
          reject(new Error(`Timed out after ${timeoutMs}ms`));
        }
      }, timeoutMs);

      cdp.on("Runtime.consoleAPICalled", (params) => {
        const values = (params.args ?? []).map((arg) => arg.value).filter((value) =>
          typeof value === "string"
        );
        for (const value of values) {
          consoleMessages.push(value);
          if (value.startsWith(resultPrefix) && !settled) {
            settled = true;
            clearTimeout(timeout);
            resolve(JSON.parse(value.slice(resultPrefix.length)));
          }
        }
      });

      cdp.on("Runtime.exceptionThrown", (params) => {
        if (!settled) {
          settled = true;
          clearTimeout(timeout);
          reject(new Error(params.exceptionDetails?.text ?? "Runtime exception"));
        }
      });
    });

    await cdp.send("Runtime.enable");
    await cdp.send("Page.enable");
    await cdp.send("Page.navigate", { url });
    const result = await resultPromise;
    cdp.close();
    return { result, consoleMessages };
  } catch (error) {
    const command = [executable, ...chromeArgs({ port, userDataDir, headless })]
      .map(shellQuote)
      .join(" ");
    error.message = `${error.message}\nChrome command: ${command}\nChrome stderr:\n${stderr}`;
    throw error;
  } finally {
    child.kill("SIGTERM");
    await rm(userDataDir, {
      recursive: true,
      force: true,
      maxRetries: 5,
      retryDelay: 100,
    }).catch(() => {});
  }
}

function summarize(results) {
  const okResults = results.filter((result) => result.ok);
  if (okResults.length === 0) {
    return null;
  }

  const average = (values) => values.reduce((sum, value) => sum + value, 0) / values.length;
  const stageNames = Array.from(new Set(
    okResults.flatMap((result) => result.stageTimings.map((stage) => stage.stage)),
  ));

  return {
    runs: okResults.length,
    averageLoadMs: average(okResults.map((result) => result.loadMs)),
    averageTotalMs: average(okResults.map((result) => result.totalMs)),
    stages: stageNames.map((stageName) => {
      const durations = okResults.flatMap((result) =>
        result.stageTimings
          .filter((stage) => stage.stage === stageName)
          .map((stage) => stage.durationMs)
      );
      return {
        stage: stageName,
        averageDurationMs: average(durations),
      };
    }),
  };
}

const options = parseArgs(process.argv.slice(2));
const { server, url } = await startViteServer(options.replay);
try {
  const results = [];
  for (let iteration = 0; iteration < options.iterations; iteration += 1) {
    console.error(`Benchmark iteration ${iteration + 1}/${options.iterations}: ${options.replay}`);
    const { result } = await runChromeBenchmark({
      url,
      chrome: options.chrome,
      headless: options.headless,
      timeoutMs: options.timeoutMs,
    });
    results.push(result);
    console.log(JSON.stringify(result, null, 2));
  }
  const summary = summarize(results);
  if (summary) {
    console.log(JSON.stringify({ summary }, null, 2));
  }
} finally {
  await server.close();
}
