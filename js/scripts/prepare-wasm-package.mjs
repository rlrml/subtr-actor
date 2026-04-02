import { readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = fileURLToPath(new URL(".", import.meta.url));
const jsDir = path.resolve(scriptDir, "..");
const sourcePackagePath = path.resolve(jsDir, "package.json");
const generatedPackagePath = path.resolve(jsDir, "pkg", "package.json");

const sourcePackage = JSON.parse(await readFile(sourcePackagePath, "utf8"));
const generatedPackage = JSON.parse(await readFile(generatedPackagePath, "utf8"));

// wasm-pack re-reads pkg/package.json on subsequent builds and expects a few
// metadata fields in the string forms it writes itself. Normalize the fields
// we customize so repeated builds against the same out-dir stay idempotent.
delete generatedPackage.collaborators;
generatedPackage.name = sourcePackage.name;
generatedPackage.version = sourcePackage.version;
generatedPackage.description = sourcePackage.description;
generatedPackage.repository =
  typeof sourcePackage.repository === "string"
    ? sourcePackage.repository
    : sourcePackage.repository?.url;
generatedPackage.keywords = sourcePackage.keywords;
generatedPackage.author = sourcePackage.author;
generatedPackage.license = sourcePackage.license;
generatedPackage.publishConfig = { access: "public" };

await writeFile(
  generatedPackagePath,
  `${JSON.stringify(generatedPackage, null, 2)}\n`
);
