import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { prepareGenerated } from "./prepare-generated.mjs";
import { cargoMetadata, claimTarget, root } from "./cargo.mjs";

export function build({
  generated = prepareGenerated,
  metadata = cargoMetadata,
  claim = claimTarget,
  run = spawnSync,
} = {}) {
  generated("--check");
  const target = metadata().target_directory;
  claim(target);
  const result = run(
    "corepack",
    ["pnpm", "tauri", "build", "--bundles", "app", "--", "--locked"],
    {
      cwd: root,
      stdio: "inherit",
      env: { ...process.env, CARGO_TARGET_DIR: target },
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
