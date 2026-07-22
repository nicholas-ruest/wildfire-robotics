import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const packageDirectory = resolve(process.argv[2] ?? "");
const manifest = JSON.parse(readFileSync(resolve(packageDirectory, "package.json"), "utf8"));
const lock = JSON.parse(readFileSync(resolve(packageDirectory, "package-lock.json"), "utf8"));

if (lock.lockfileVersion !== 3 || lock.packages === undefined) {
  throw new Error("a package-lock v3 packages inventory is required");
}

const purl = (name, version) =>
  `pkg:npm/${encodeURIComponent(name).replace("%40", "@")}%40${encodeURIComponent(version)}`;
const components = Object.entries(lock.packages)
  .filter(([path, item]) => path.startsWith("node_modules/") && item.version)
  .map(([path, item]) => {
    const name = item.name ?? path.slice(path.lastIndexOf("node_modules/") + 13);
    return {
    type: "library",
    "bom-ref": `${path}@${item.version}`,
    name,
    version: item.version,
    purl: purl(name, item.version),
    properties: [
      { name: "wildfire:npm:path", value: path },
      { name: "wildfire:npm:development", value: String(item.dev === true) },
    ],
    };
  })
  .sort((left, right) => left["bom-ref"].localeCompare(right["bom-ref"]));

const document = {
  $schema: "http://cyclonedx.org/schema/bom-1.5.schema.json",
  bomFormat: "CycloneDX",
  specVersion: "1.5",
  version: 1,
  metadata: {
    component: {
      type: "application",
      "bom-ref": `${manifest.name}@${manifest.version}`,
      name: manifest.name,
      version: manifest.version,
      purl: purl(manifest.name, manifest.version),
    },
    properties: [
      { name: "wildfire:source", value: "package-lock.json" },
      { name: "wildfire:lockfile-version", value: String(lock.lockfileVersion) },
    ],
  },
  components,
};

process.stdout.write(`${JSON.stringify(document, null, 2)}\n`);
