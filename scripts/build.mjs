import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { prepareGenerated } from "./prepare-generated.mjs";
import { root } from "./cargo.mjs";
import { prepareSupervisor } from "./prepare-supervisor.mjs";

export function build({
  generated = prepareGenerated,
  prepare = prepareSupervisor,
  run = spawnSync,
} = {}) {
  generated("--check");
  const provenance = prepare("release");
  const result = run(
    "corepack",
    ["pnpm", "tauri", "build", "--bundles", "app", "--", "--locked"],
    {
      cwd: root,
      stdio: "inherit",
      env: { ...process.env, CARGO_TARGET_DIR: provenance.targetDirectory },
    },
  );
  if (result.error) throw result.error;
  return result.status ?? 1;
}
if (
  process.argv[1] &&
  path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)
)
  process.exitCode = build();
