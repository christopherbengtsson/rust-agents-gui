// Compile only the core library unit harness; execute core, tooling and mocked-IPC UI units.
import { spawnSync } from "node:child_process";
import { readdirSync } from "node:fs";
import { performance } from "node:perf_hooks";
import path from "node:path";
import { cargoMetadata, claimTarget, root, runCommand } from "./cargo.mjs";

const measure = process.argv.includes("--measure");
const metadata = cargoMetadata();
claimTarget(metadata.target_directory);
const started = performance.now();
const output = runCommand("cargo", [
  "test",
  "--locked",
  "-p",
  "workspace-core",
  "--lib",
  "--no-run",
  "--message-format=json-render-diagnostics",
]);
console.log(
  `Unit compilation: ${((performance.now() - started) / 1000).toFixed(6)} s; target ${metadata.target_directory}; test profile`,
);
const artifacts = output
  .split("\n")
  .filter((line) => line.startsWith("{"))
  .map((line) => JSON.parse(line))
  .filter(
    (message) =>
      message.reason === "compiler-artifact" &&
      message.target.name === "workspace_core" &&
      message.profile.test &&
      message.executable,
  );
if (artifacts.length !== 1)
  throw new Error("Expected exactly one core unit executable");
const tooling = readdirSync(path.join(root, "scripts"))
  .filter((name) => name.endsWith(".test.mjs"))
  .sort()
  .map((name) => path.join("scripts", name));
const commands = [
  [artifacts[0].executable, ["--test-threads=1"]],
  [process.execPath, ["--test", "--test-concurrency=1", ...tooling]],
  [
    process.execPath,
    ["node_modules/vitest/vitest.mjs", "run", "--maxWorkers=1"],
  ],
];
for (const [command, args] of commands) {
  const durations = [];
  for (let run = 0; run < (measure ? 6 : 1); run++) {
    const start = performance.now();
    const result = spawnSync(command, args, { cwd: root, stdio: "inherit" });
    const seconds = (performance.now() - start) / 1000;
    if (result.error) throw result.error;
    if (result.status !== 0) process.exit(result.status ?? 1);
    console.log(
      `Unit execution ${measure && run === 0 ? "warm-up" : run + (measure ? 0 : 1)}: ${seconds.toFixed(6)} s; ${command} ${args.join(" ")}`,
    );
    if (!measure || run > 0) durations.push(seconds);
  }
  if (measure) {
    durations.sort((a, b) => a - b);
    console.log(
      `Warm execution median ${durations[2].toFixed(6)} s; range ${durations[0].toFixed(6)}–${durations.at(-1).toFixed(6)} s`,
    );
  }
}
