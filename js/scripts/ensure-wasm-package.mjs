import { promises as fs } from "node:fs";
import path from "node:path";
import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";

const scriptDir = fileURLToPath(new URL(".", import.meta.url));
const jsDir = path.resolve(scriptDir, "..");
const repoRoot = path.resolve(jsDir, "..");
const sourceDirs = [path.resolve(repoRoot, "src"), path.resolve(jsDir, "src")];
const sourceFiles = [
  path.resolve(repoRoot, "Cargo.toml"),
  path.resolve(repoRoot, "Cargo.lock"),
  path.resolve(jsDir, "Cargo.toml"),
  path.resolve(jsDir, "package.json"),
];

function outputFilesFor(pkgDirName) {
  const pkgDir = path.resolve(jsDir, pkgDirName);
  return [
    path.resolve(pkgDir, "rl_replay_subtr_actor.js"),
    path.resolve(pkgDir, "rl_replay_subtr_actor_bg.wasm"),
    path.resolve(pkgDir, "rl_replay_subtr_actor.d.ts"),
  ];
}

const WASM_TARGETS = {
  web: {
    label: "js/pkg",
    outputFiles: outputFilesFor("pkg"),
    buildCommand: ["npm", ["--prefix", path.resolve(jsDir, "player"), "run", "build:bindings"]],
  },
  node: {
    label: "js/pkg-node",
    outputFiles: outputFilesFor("pkg-node"),
    buildCommand: ["npm", ["--prefix", jsDir, "run", "build:nodejs"]],
  },
};

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
    }),
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

async function isWasmTargetStale(target) {
  const [latestSourceMtime, earliestOutputMtime] = await Promise.all([
    latestMtime(await collectSourceFiles()),
    earliestMtime(target.outputFiles),
  ]);

  return latestSourceMtime > earliestOutputMtime;
}

function runBuildCommand([command, args]) {
  return new Promise((resolve, reject) => {
    const child = spawn(command, args, {
      cwd: jsDir,
      stdio: "inherit",
    });

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
      resolvedPath === targetPath || resolvedPath.startsWith(`${targetPath}${path.sep}`),
  );
}

async function ensureWasmTargetFresh(target, { force, log }) {
  if (process.env.SUBTR_ACTOR_SKIP_WASM_BUILD === "1") {
    if ((await earliestMtime(target.outputFiles)) === 0) {
      throw new Error(`SUBTR_ACTOR_SKIP_WASM_BUILD=1 but ${target.label} is missing`);
    }

    log(`[wasm] using prebuilt ${target.label}`);
    return false;
  }

  if (!force && !(await isWasmTargetStale(target))) {
    log(`[wasm] ${target.label} is up to date`);
    return false;
  }

  log(force ? `[wasm] rebuilding ${target.label}` : `[wasm] ${target.label} is stale, rebuilding`);
  await runBuildCommand(target.buildCommand);
  return true;
}

export async function ensureWasmPackageFresh({
  force = false,
  log = console.log,
  targets = ["web"],
} = {}) {
  let rebuilt = false;

  for (const targetName of targets) {
    const target = WASM_TARGETS[targetName];
    if (!target) {
      throw new Error(`unknown wasm target: ${targetName}`);
    }
    if (await ensureWasmTargetFresh(target, { force, log })) {
      rebuilt = true;
    }
  }

  return rebuilt;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const force = process.argv.includes("--force");
  await ensureWasmPackageFresh({ force, targets: ["web", "node"] });
}
