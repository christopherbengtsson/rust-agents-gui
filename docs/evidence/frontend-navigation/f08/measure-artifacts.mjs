// Explicit artifact measurement; no application behavior or process qualification.
import { readFileSync, readdirSync, statSync, writeFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { gzipSync, brotliCompressSync } from "node:zlib";
import path from "node:path";
const evidence = "docs/evidence/frontend-navigation/f08";
const baseline = JSON.parse(readFileSync(`${evidence}/baseline-context.json`));
function totalBytes(directory) {
  return readdirSync(directory, { withFileTypes: true }).reduce(
    (n, item) =>
      n +
      (item.isDirectory()
        ? totalBytes(path.join(directory, item.name))
        : statSync(path.join(directory, item.name)).size),
    0,
  );
}
function inspect(root, target) {
  const html = readFileSync(path.join(root, "dist/index.html"), "utf8");
  const initial = [
    ...html.matchAll(/(?:src|href)="\/assets\/([^"?]+\.js)"/g),
  ].map((match) => match[1]);
  const assets = Object.fromEntries(
    readdirSync(path.join(root, "dist/assets"))
      .filter((file) => file.endsWith(".js"))
      .sort()
      .map((file) => {
        const body = readFileSync(path.join(root, "dist/assets", file));
        return [
          file,
          {
            raw: body.length,
            gzip: gzipSync(body).length,
            brotli: brotliCompressSync(body).length,
            sha256: createHash("sha256").update(body).digest("hex"),
          },
        ];
      }),
  );
  const sum = (names) =>
    names.reduce(
      (a, name) =>
        Object.fromEntries(
          ["raw", "gzip", "brotli"].map((kind) => [
            kind,
            a[kind] + assets[name][kind],
          ]),
        ),
      { raw: 0, gzip: 0, brotli: 0 },
    );
  const lazy = Object.keys(assets).filter((name) => !initial.includes(name));
  const bundle = path.join(target, "release/bundle/macos/Agent Workspace.app");
  const executables = [
    "Contents/MacOS/workspace-desktop",
    "Contents/Resources/workspace-supervisor",
  ];
  return {
    root,
    target,
    initial,
    lazy,
    assets,
    initialJavaScript: sum(initial),
    lazyJavaScript: sum(lazy),
    totalJavaScript: sum(Object.keys(assets)),
    bundleBytes: totalBytes(bundle),
    executables: Object.fromEntries(
      executables.map((name) => {
        const body = readFileSync(path.join(bundle, name));
        return [
          name,
          {
            bytes: body.length,
            sha256: createHash("sha256").update(body).digest("hex"),
          },
        ];
      }),
    ),
    sidecarProvenance: JSON.parse(
      readFileSync(path.join(root, "src-tauri/gen/sidecar/provenance.json")),
    ),
  };
}
const results = {
  compression:
    "Node 26.13.1 zlib gzipSync and brotliCompressSync defaults; independently compressed emitted chunks",
  baseline: process.argv.includes("--retained-baseline")
    ? JSON.parse(readFileSync(`${evidence}/artifacts.json`)).baseline
    : inspect(baseline.checkout, baseline.target),
  current: inspect(process.cwd(), path.resolve("target")),
};
writeFileSync(
  `${evidence}/${process.argv[2] ?? "artifacts.json"}`,
  JSON.stringify(results, null, 2) + "\n",
  { flag: "wx" },
);
console.log(
  JSON.stringify(
    Object.fromEntries(
      Object.entries(results).map(([key, value]) => [
        key,
        typeof value === "string"
          ? value
          : {
              initial: value.initialJavaScript,
              lazy: value.lazyJavaScript,
              total: value.totalJavaScript,
              bundle: value.bundleBytes,
            },
      ]),
    ),
  ),
);
