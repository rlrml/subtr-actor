import { promises as fs } from "node:fs";
import path from "node:path";
import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";

const scriptDir = fileURLToPath(new URL(".", import.meta.url));
const jsDir = path.resolve(scriptDir, "..");
const repoRoot = path.resolve(jsDir, "..");
const pkgDir = path.resolve(jsDir, "pkg");
const sourceDirs = [path.resolve(repoRoot, "src"), path.resolve(jsDir, "src")];
const sourceFiles = [
  path.resolve(repoRoot, "Cargo.toml"),
  path.resolve(repoRoot, "Cargo.lock"),
  path.resolve(jsDir, "Cargo.toml"),
  path.resolve(jsDir, "package.json"),
];
const outputFiles = [
  path.resolve(pkgDir, "rl_replay_subtr_actor.js"),
  path.resolve(pkgDir, "rl_replay_subtr_actor_bg.wasm"),
  path.resolve(pkgDir, "rl_replay_subtr_actor.d.ts"),
];

async function pathExists(targetPath) {
  try {
    await fs.access(targetPath);
    return true;
  } catch {
    return false;
  }
}

async function walkFiles(rootDir) {
  if (!(await pathExists(rootDir))) {
    return [];
  }

  const entries = await fs.readdir(rootDir, { withFileTypes: true });
  const files = await Promise.all(
    entries.map(async (entry) => {
      const fullPath = path.join(rootDir, entry.name);
      if (entry.isDirectory()) {
        return walkFiles(fullPath);
      }
      return fullPath;
    })
  );

  return files.flat();
}

async function latestMtime(paths) {
  let latest = 0;

  for (const targetPath of paths) {
    if (!(await pathExists(targetPath))) {
      continue;
    }
    const stat = await fs.stat(targetPath);
    latest = Math.max(latest, stat.mtimeMs);
  }

  return latest;
}

async function earliestMtime(paths) {
  let earliest = Number.POSITIVE_INFINITY;

  for (const targetPath of paths) {
    if (!(await pathExists(targetPath))) {
      return 0;
    }
    const stat = await fs.stat(targetPath);
    earliest = Math.min(earliest, stat.mtimeMs);
  }

  return Number.isFinite(earliest) ? earliest : 0;
}

async function collectSourceFiles() {
  const nestedFiles = (await Promise.all(sourceDirs.map(walkFiles))).flat();
  return [...sourceFiles, ...nestedFiles];
}

async function isWasmPackageStale() {
  const [latestSourceMtime, earliestOutputMtime] = await Promise.all([
    latestMtime(await collectSourceFiles()),
    earliestMtime(outputFiles),
  ]);

  return latestSourceMtime > earliestOutputMtime;
}

function runBuildScript() {
  return new Promise((resolve, reject) => {
    const child = spawn(
      "npm",
      ["--prefix", path.resolve(jsDir, "player"), "run", "build:bindings"],
      {
        cwd: jsDir,
        stdio: "inherit",
      }
    );

    child.on("exit", (code) => {
      if (code === 0) {
        resolve();
        return;
      }
      reject(new Error(`wasm rebuild exited with code ${code}`));
    });

    child.on("error", reject);
  });
}

export function getWasmWatchTargets() {
  return [...sourceDirs, ...sourceFiles];
}

export function isWasmSourcePath(filePath) {
  const resolvedPath = path.resolve(filePath);
  return getWasmWatchTargets().some(
    (targetPath) =>
      resolvedPath === targetPath ||
      resolvedPath.startsWith(`${targetPath}${path.sep}`)
  );
}

export async function ensureWasmPackageFresh({
  force = false,
  log = console.log,
} = {}) {
  if (process.env.SUBTR_ACTOR_SKIP_WASM_BUILD === "1") {
    if ((await earliestMtime(outputFiles)) === 0) {
      throw new Error("SUBTR_ACTOR_SKIP_WASM_BUILD=1 but js/pkg is missing");
    }

    log("[wasm] using prebuilt js/pkg");
    return false;
  }

  if (!force && !(await isWasmPackageStale())) {
    log("[wasm] js/pkg is up to date");
    return false;
  }

  log(force ? "[wasm] rebuilding js/pkg" : "[wasm] js/pkg is stale, rebuilding");
  await runBuildScript();
  return true;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const force = process.argv.includes("--force");
  await ensureWasmPackageFresh({ force });
}
