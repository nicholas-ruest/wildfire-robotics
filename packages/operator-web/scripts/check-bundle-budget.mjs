import {gzipSync} from "node:zlib";
import {readdirSync, readFileSync} from "node:fs";
import {join, relative} from "node:path";
import {fileURLToPath} from "node:url";

const root = fileURLToPath(new URL("../dist/", import.meta.url));
const limits = {
  shell: 250 * 1024,
  initialScene: 350 * 1024,
  additionalScene: 300 * 1024,
};

function files(directory) {
  return readdirSync(directory, {withFileTypes: true}).flatMap(entry => {
    const path = join(directory, entry.name);
    return entry.isDirectory() ? files(path) : [path];
  });
}

const javascript = files(root).filter(path => path.endsWith(".js"));
const evidence = javascript.map(path => ({
  file: relative(root, path),
  gzipBytes: gzipSync(readFileSync(path)).byteLength,
}));
const shellBytes = evidence.reduce((total, item) => total + item.gzipBytes, 0);
const violations = [];
if (shellBytes > limits.shell) violations.push(`shell ${shellBytes} > ${limits.shell}`);
for (const item of evidence.filter(item => /scene/i.test(item.file))) {
  if (item.gzipBytes > limits.additionalScene) {
    violations.push(`${item.file} ${item.gzipBytes} > ${limits.additionalScene}`);
  }
}

console.log(JSON.stringify({
  status: violations.length ? "failed" : "passed",
  budgets: limits,
  shellGzipBytes: shellBytes,
  chunks: evidence,
}, null, 2));
if (violations.length) {
  console.error(violations.join("\n"));
  process.exitCode = 1;
}
