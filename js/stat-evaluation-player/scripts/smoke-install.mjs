import { execFileSync } from "node:child_process";
import { mkdtemp, mkdir, readFile, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const packageDir = path.resolve(scriptDir, "..");
const jsDir = path.resolve(packageDir, "..");

function run(command, args, cwd) {
  execFileSync(command, args, {
    cwd,
    encoding: "utf8",
    stdio: "inherit",
  });
}

async function packTarball(cwd, packDir) {
  const packOutput = execFileSync(
    "npm",
    ["pack", "--json", "--pack-destination", packDir],
    {
      cwd,
      encoding: "utf8",
    },
  );
  const [{ filename }] = JSON.parse(packOutput);
  return path.join(packDir, filename);
}

async function main() {
  const sourcePackage = JSON.parse(
    await readFile(path.resolve(packageDir, "package.json"), "utf8"),
  );
  const playerPackage = JSON.parse(
    await readFile(path.resolve(jsDir, "player", "package.json"), "utf8"),
  );
  const scratchDir = await mkdtemp(
    path.join(os.tmpdir(), "subtr-actor-stats-player-smoke-"),
  );
  let playerPublishDir = null;
  let statsPublishDir = null;

  try {
    run("npm", ["--prefix", jsDir, "run", "build"], packageDir);
    run("npm", ["run", "build"], path.resolve(jsDir, "player"));
    run("npm", ["run", "build"], packageDir);

    const packDir = path.join(scratchDir, "pack");
    const consumerDir = path.join(scratchDir, "consumer");
    const sourceDir = path.join(consumerDir, "src");

    await mkdir(packDir, { recursive: true });
    await mkdir(sourceDir, { recursive: true });

    const bindingsTarballPath = await packTarball(path.resolve(jsDir, "pkg"), packDir);

    playerPublishDir = execFileSync("npm", ["run", "--silent", "prepare:package"], {
      cwd: path.resolve(jsDir, "player"),
      encoding: "utf8",
    }).trim();
    const playerTarballPath = await packTarball(playerPublishDir, packDir);

    statsPublishDir = execFileSync("npm", ["run", "--silent", "prepare:package"], {
      cwd: packageDir,
      encoding: "utf8",
    }).trim();
    const statsTarballPath = await packTarball(statsPublishDir, packDir);

    await writeFile(
      path.join(consumerDir, "package.json"),
      JSON.stringify(
        {
          name: "subtr-actor-stats-player-smoke-consumer",
          private: true,
          type: "module",
          scripts: {
            check: "tsc --noEmit",
            build: "vite build",
          },
          dependencies: {
            "@colonelpanic8/subtr-actor": `file:${path.relative(consumerDir, bindingsTarballPath)}`,
            [playerPackage.name]: `file:${path.relative(consumerDir, playerTarballPath)}`,
            [sourcePackage.name]: `file:${path.relative(consumerDir, statsTarballPath)}`,
            three: "^0.180.0",
          },
          devDependencies: {
            typescript: "^5.9.2",
            vite: "^7.3.1",
          },
        },
        null,
        2,
      ),
    );

    await writeFile(
      path.join(consumerDir, "tsconfig.json"),
      JSON.stringify(
        {
          compilerOptions: {
            target: "ES2022",
            module: "ESNext",
            moduleResolution: "Bundler",
            lib: ["DOM", "ES2022"],
            strict: true,
            skipLibCheck: true,
          },
          include: ["src"],
        },
        null,
        2,
      ),
    );

    await writeFile(
      path.join(consumerDir, "index.html"),
      [
        "<!doctype html>",
        "<html>",
        "  <body>",
        '    <div id="app"></div>',
        '    <script type="module" src="/src/main.ts"></script>',
        "  </body>",
        "</html>",
        "",
      ].join("\n"),
    );

    await writeFile(
      path.join(sourceDir, "main.ts"),
      [
        'import {',
        '  buildTimeInZoneTimelineRanges,',
        '  createStatsFrameLookup,',
        '  mountStatEvaluationPlayer,',
        '  type StatEvaluationPlayerHandle,',
        `} from "${sourcePackage.name}";`,
        "",
        'const root = document.getElementById("app");',
        'if (!(root instanceof HTMLElement)) {',
        '  throw new Error("Missing app root");',
        "}",
        "",
        "const emptyTimeline = {",
        "  replay_meta: {},",
        "  timeline_events: [],",
        "  frames: [],",
        "};",
        "createStatsFrameLookup(emptyTimeline);",
        "buildTimeInZoneTimelineRanges(emptyTimeline);",
        "const handle: StatEvaluationPlayerHandle = mountStatEvaluationPlayer(root);",
        "handle.destroy();",
        "",
      ].join("\n"),
    );

    run("npm", ["install"], consumerDir);
    run("npm", ["run", "check"], consumerDir);
    run("npm", ["run", "build"], consumerDir);
  } finally {
    if (playerPublishDir) {
      await rm(playerPublishDir, { force: true, recursive: true });
    }
    if (statsPublishDir) {
      await rm(statsPublishDir, { force: true, recursive: true });
    }
    await rm(scratchDir, { force: true, recursive: true });
  }
}

await main();
