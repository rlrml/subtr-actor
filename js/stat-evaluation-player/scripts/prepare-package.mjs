import { mkdtemp, cp, readFile, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const packageDir = path.resolve(scriptDir, "..");
const distDir = path.resolve(packageDir, "dist");

async function main() {
  const sourcePackage = JSON.parse(
    await readFile(path.resolve(packageDir, "package.json"), "utf8"),
  );
  const bindingsPackage = JSON.parse(
    await readFile(path.resolve(packageDir, "..", "package.json"), "utf8"),
  );
  const publishPackage = {
    ...sourcePackage,
    dependencies: {
      [bindingsPackage.name]: sourcePackage.version,
      "subtr-actor-player": sourcePackage.version,
    },
  };
  delete publishPackage.scripts;
  delete publishPackage.devDependencies;

  const outputDir = await mkdtemp(
    path.join(os.tmpdir(), "subtr-actor-stat-evaluation-player-package-"),
  );

  await cp(distDir, path.join(outputDir, "dist"), { recursive: true });
  await cp(path.resolve(packageDir, "README.md"), path.join(outputDir, "README.md"));
  await cp(path.resolve(packageDir, "LICENSE"), path.join(outputDir, "LICENSE"));
  await writeFile(
    path.join(outputDir, "package.json"),
    `${JSON.stringify(publishPackage, null, 2)}\n`,
  );

  process.stdout.write(outputDir);
}

await main();
