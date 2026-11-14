import {
  readFileSync,
  readdirSync,
  statSync,
  writeFileSync,
  existsSync,
} from "node:fs";
import path from "node:path";
import { gzipSync, brotliCompressSync } from "node:zlib";
import { createHash } from "node:crypto";
const evidence = "docs/evidence/frontend-navigation/f04g";
if (!process.argv[2] || existsSync(process.argv[2]))
  throw new Error("Choose a fresh output path; retain existing evidence");
const environment = JSON.parse(
  readFileSync(path.join(evidence, "environment.json")),
);
function bytes(directory) {
  return readdirSync(directory, { withFileTypes: true }).reduce(
    (n, file) =>
      n +
      (file.isDirectory()
        ? bytes(path.join(directory, file.name))
        : statSync(path.join(directory, file.name)).size),
    0,
  );
}
if (readdirSync("dist/assets").some((file) => file.endsWith(".map")))
  throw new Error(
    "Build release frontend assets before measuring; debug maps are present",
  );
const results = {
  compression: "Node zlib gzipSync defaults; brotliCompressSync defaults",
};
for (const [label, root, target] of [
  ["current", process.cwd(), path.join(process.cwd(), "target")],
  ["baseline", environment.baseline, environment.target],
]) {
  const assets = readdirSync(path.join(root, "dist/assets"))
    .filter((file) => file.endsWith(".js"))
    .map((file) => {
      const data = readFileSync(path.join(root, "dist/assets", file));
      return {
        file,
        raw: data.length,
        gzip: gzipSync(data).length,
        brotli: brotliCompressSync(data).length,
        sha256: createHash("sha256").update(data).digest("hex"),
      };
    });
  const bundle = path.join(target, "release/bundle/macos/Agent Workspace.app");
  results[label] = {
    assets,
    bundleBytes: bytes(bundle),
    sidecar: JSON.parse(
      readFileSync(path.join(root, "src-tauri/gen/sidecar/provenance.json")),
    ),
  };
}
writeFileSync(process.argv[2], JSON.stringify(results, null, 2) + "\n");
console.log(JSON.stringify(results, null, 2));
