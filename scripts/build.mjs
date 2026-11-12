import { spawnSync } from "node:child_process";
import { root } from "./cargo.mjs";
import { prepareSupervisor } from "./prepare-supervisor.mjs";

const provenance = prepareSupervisor("release");
const result = spawnSync(
  "corepack",
  ["pnpm", "tauri", "build", "--bundles", "app", "--", "--locked"],
  {
    cwd: root,
    stdio: "inherit",
    env: { ...process.env, CARGO_TARGET_DIR: provenance.targetDirectory },
  },
);
if (result.error) throw result.error;
process.exitCode = result.status ?? 1;
