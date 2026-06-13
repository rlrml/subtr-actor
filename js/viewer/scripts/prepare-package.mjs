import { cp, mkdtemp, readFile, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const packageDir = path.resolve(scriptDir, "..");
const repoJsDir = path.resolve(packageDir, "..");
const distDir = path.resolve(packageDir, "dist");

async function main() {
  const sourcePackage = JSON.parse(
    await readFile(path.resolve(packageDir, "package.json"), "utf8"),
  );
  const bindingsPackage = JSON.parse(
    await readFile(path.resolve(repoJsDir, "package.json"), "utf8"),
  );
  const playerPackage = JSON.parse(
    await readFile(path.resolve(repoJsDir, "player", "package.json"), "utf8"),
  );
  const publishPackage = {
    ...sourcePackage,
    dependencies: {
      ...sourcePackage.dependencies,
      "@rlrml/player": playerPackage.version,
      "@rlrml/subtr-actor": bindingsPackage.version,
    },
  };
  delete publishPackage.scripts;
  delete publishPackage.devDependencies;

  const outputDir = await mkdtemp(path.join(os.tmpdir(), "subtr-actor-viewer-package-"));

  await cp(distDir, path.join(outputDir, "dist"), { recursive: true });
  await cp(path.resolve(packageDir, "public"), path.join(outputDir, "public"), {
    recursive: true,
  });
  await cp(path.resolve(packageDir, "docs"), path.join(outputDir, "docs"), {
    recursive: true,
  });
  await cp(path.resolve(packageDir, "README.md"), path.join(outputDir, "README.md"));
  await cp(path.resolve(repoJsDir, "LICENSE"), path.join(outputDir, "LICENSE"));
  await writeFile(
    path.join(outputDir, "package.json"),
    `${JSON.stringify(publishPackage, null, 2)}\n`,
  );

  process.stdout.write(outputDir);
}

await main();
