// Preparation and each retained gate run once; application units never generate files.
import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { root } from "./cargo.mjs";
import { prepareGenerated } from "./prepare-generated.mjs";

export function check(
  mode,
  { prepare = prepareGenerated, run = spawnSync } = {},
) {
  if (!["check", "verify", "ci"].includes(mode))
    throw new Error("Expected check, verify or ci");
  prepare("--check");
  const commands = [
    ...(mode === "ci"
      ? [
          ["node", ["scripts/prepare-supervisor.mjs", "debug"]],
          ["corepack", ["pnpm", "exec", "vite", "build"]],
        ]
      : []),
    ...(mode !== "check"
      ? [
          ["corepack", ["pnpm", "format:check"]],
          ["corepack", ["pnpm", "lint"]],
        ]
      : []),
    ["corepack", ["pnpm", "exec", "tsc", "--noEmit"]],
    [
      "node",
      [
        "scripts/cargo.mjs",
        "clippy",
        "--locked",
        "--workspace",
        "--all-targets",
        "--all-features",
        "--",
        "-D",
        "warnings",
      ],
    ],
    ...(mode !== "check" ? [["node", ["scripts/test-unit.mjs"]]] : []),
  ];
  for (const [command, args] of commands) {
    console.log(`Gate: ${command} ${args.join(" ")}`);
    const result = run(command, args, { cwd: root, stdio: "inherit" });
    if (result.error) throw result.error;
    if (result.status !== 0)
      throw new Error(
        `Gate failed (${result.status ?? result.signal}): ${command} ${args.join(" ")}`,
      );
  }
}
if (
  process.argv[1] &&
  path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)
)
  check(process.argv[2]);
