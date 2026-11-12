import { spawnSync } from "node:child_process";
import { copyFileSync, mkdirSync } from "node:fs";
const mode = process.argv[2];
if (!["debug", "release"].includes(mode))
  throw new Error("Expected debug or release");
const result = spawnSync(
  "cargo",
  [
    "build",
    "--locked",
    "-p",
    "workspace-supervisor",
    ...(mode === "release" ? ["--release"] : []),
  ],
  { stdio: "inherit" },
);
if (result.status !== 0) process.exit(result.status ?? 1);
mkdirSync("target/sidecar", { recursive: true });
copyFileSync(
  `target/${mode}/workspace-supervisor`,
  "target/sidecar/workspace-supervisor",
);
