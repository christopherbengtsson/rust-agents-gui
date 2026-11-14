import { readFileSync, readdirSync, statSync, writeFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { gzipSync, brotliCompressSync } from "node:zlib";
const assets = Object.fromEntries(
  readdirSync("dist/assets")
    .sort()
    .map((name) => {
      const content = readFileSync(`dist/assets/${name}`);
      return [
        name,
        {
          bytes: content.length,
          gzip: gzipSync(content).length,
          brotli: brotliCompressSync(content).length,
          sha256: createHash("sha256").update(content).digest("hex"),
        },
      ];
    }),
);
const html = readFileSync("dist/index.html", "utf8");
const initial = [...html.matchAll(/(?:src|href)="\/assets\/([^"]+\.js)"/g)].map(
  (match) => match[1],
);
const sum = (names) =>
  names.reduce(
    (total, name) => ({
      bytes: total.bytes + assets[name].bytes,
      gzip: total.gzip + assets[name].gzip,
      brotli: total.brotli + assets[name].brotli,
    }),
    { bytes: 0, gzip: 0, brotli: 0 },
  );
const report = {
  assets,
  initial,
  initialJavaScript: sum(initial),
  allJavaScript: sum(
    Object.keys(assets).filter((name) => name.endsWith(".js")),
  ),
  desktopExecutableBytes: statSync(
    "target/release/bundle/macos/Agent Workspace.app/Contents/MacOS/workspace-desktop",
  ).size,
  optionalMain: Object.keys(assets).filter((name) => name.startsWith("main-")),
  optionalSettings: Object.keys(assets).filter((name) =>
    name.startsWith("settings.general-"),
  ),
  sourceShikiImports:
    "none in src; package installation alone does not establish bundle inclusion",
  compression: "Node default gzip/Brotli options",
};
writeFileSync(
  "docs/evidence/frontend-navigation/f06/artifacts.json",
  JSON.stringify(report, null, 2) + "\n",
);
console.log(
  JSON.stringify({
    initialJavaScript: report.initialJavaScript,
    allJavaScript: report.allJavaScript,
    optionalMain: report.optionalMain,
    optionalSettings: report.optionalSettings,
  }),
);
