import { execFileSync } from "node:child_process";
import { mkdtemp, mkdir, rm, writeFile } from "node:fs/promises";
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

async function main() {
  const scratchDir = await mkdtemp(
    path.join(os.tmpdir(), "subtr-actor-player-smoke-")
  );
  let publishDir = null;

  try {
    run("npm", ["--prefix", jsDir, "run", "build"], packageDir);
    run("npm", ["run", "build"], packageDir);

    const packDir = path.join(scratchDir, "pack");
    const consumerDir = path.join(scratchDir, "consumer");
    const sourceDir = path.join(consumerDir, "src");

    await mkdir(packDir, { recursive: true });
    await mkdir(sourceDir, { recursive: true });

    const bindingsPackOutput = execFileSync(
      "npm",
      ["pack", "--json", "--pack-destination", packDir],
      {
        cwd: path.resolve(jsDir, "pkg"),
        encoding: "utf8",
      }
    );
    const [{ filename: bindingsFilename }] = JSON.parse(bindingsPackOutput);

    publishDir = execFileSync("npm", ["run", "--silent", "prepare:package"], {
      cwd: packageDir,
      encoding: "utf8",
    }).trim();
    const playerPackOutput = execFileSync(
      "npm",
      ["pack", "--json", "--pack-destination", packDir],
      {
        cwd: publishDir,
        encoding: "utf8",
      }
    );
    const [{ filename: playerFilename }] = JSON.parse(playerPackOutput);

    const bindingsTarballPath = path.join(packDir, bindingsFilename);
    const playerTarballPath = path.join(packDir, playerFilename);

    await writeFile(
      path.join(consumerDir, "package.json"),
      JSON.stringify(
        {
          name: "subtr-actor-smoke-consumer",
          private: true,
          type: "module",
          scripts: {
            check: "tsc --noEmit",
            build: "vite build",
          },
          dependencies: {
            "subtr-actor": `file:${path.relative(consumerDir, bindingsTarballPath)}`,
            "subtr-actor-player": `file:${path.relative(consumerDir, playerTarballPath)}`,
            three: "^0.180.0",
          },
          devDependencies: {
            typescript: "^5.9.2",
            vite: "^7.3.1",
          },
        },
        null,
        2
      )
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
        2
      )
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
      ].join("\n")
    );

    await writeFile(
      path.join(sourceDir, "main.ts"),
      [
        'import {',
        '  ReplayPlayer,',
        '  ReplayPlaylistPlayer,',
        '  createReplayBytesSource,',
        '  ensureBindingsReady,',
        '  frameBound,',
        '  parsePlaylistManifest,',
        '  type PlaylistManifest,',
        '  type ReplayPlayerOptions,',
        '} from "subtr-actor-player";',
        "",
        "const manifest: PlaylistManifest = parsePlaylistManifest({",
        "  items: [",
        "    {",
        '      replay: "demo",',
        '      start: { kind: "time", value: 0 },',
        '      end: { kind: "time", value: 1 },',
        "    },",
        "  ],",
        "});",
        "",
        "const options: ReplayPlayerOptions = {",
        "  initialPlaybackRate: 1.25,",
        "};",
        "",
        "void ReplayPlayer;",
        "void ReplayPlaylistPlayer;",
        "void createReplayBytesSource;",
        "void ensureBindingsReady;",
        "void frameBound;",
        "console.log(manifest, options);",
        "",
      ].join("\n")
    );

    run("npm", ["install"], consumerDir);
    run("npm", ["run", "check"], consumerDir);
    run("npm", ["run", "build"], consumerDir);
  } finally {
    if (publishDir) {
      await rm(publishDir, { force: true, recursive: true });
    }
    await rm(scratchDir, { force: true, recursive: true });
  }
}

await main();
