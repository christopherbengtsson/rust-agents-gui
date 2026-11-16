// Opt-in actual storage/process qualification; never part of application unit gates.
import { spawnSync } from "node:child_process";
import { mkdtempSync, existsSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { cargoMetadata, claimTarget, root, runCommand } from "./cargo.mjs";

const [kind, ...args] = process.argv.slice(2).filter((arg) => arg !== "--");
if (kind === "native") {
  if (args.length !== 2)
    throw new Error("Usage: qualify:native <qualified.app> <new-report.json>");
  const result = spawnSync("python3", ["scripts/qualify-native.py", ...args], {
    cwd: root,
    stdio: "inherit",
  });
  if (result.error) throw result.error;
  process.exitCode = result.status ?? 1;
} else if (["storage", "effects"].includes(kind)) {
  if (args.length !== 1 || existsSync(args[0]))
    throw new Error("Expected one new report.json path");
  const target = cargoMetadata().target_directory;
  claimTarget(target);
  const name =
    kind === "storage" ? "qualify-storage" : "qualify-process-effects";
  const pkg = kind === "storage" ? "storage-checkpoint" : "workspace-core";
  const messages = runCommand("cargo", [
    "build",
    "--locked",
    "-p",
    pkg,
    "--features",
    "qualification",
    "--bin",
    name,
    "--message-format=json-render-diagnostics",
  ]);
  const artifacts = messages
    .split("\n")
    .filter((line) => line.startsWith("{"))
    .map((line) => JSON.parse(line))
    .filter(
      (message) =>
        message.reason === "compiler-artifact" &&
        message.target.name === name &&
        message.executable,
    );
  if (artifacts.length !== 1)
    throw new Error("Expected one reported qualification executable");
  const directory = mkdtempSync(
    path.join(tmpdir(), `agent-workspace-${kind}-`),
  );
  const invocation = [
    path.join(directory, "fresh"),
    ...(kind === "effects"
      ? [path.join(root, "fixtures/process-effects/runtime.py")]
      : []),
  ];
  const started = performance.now();
  const result = spawnSync(artifacts[0].executable, invocation, {
    cwd: root,
    encoding: "utf8",
    timeout: 90000,
    maxBuffer: 16 * 1024 * 1024,
  });
  const report = {
    kind,
    directory,
    executable: artifacts[0].executable,
    command: invocation,
    seconds: (performance.now() - started) / 1000,
    exit: result.status,
    signal: result.signal,
    error: result.error?.message,
    stdout: result.stdout,
    stderr: result.stderr,
  };
  writeFileSync(args[0], `${JSON.stringify(report, null, 2)}\n`, {
    flag: "wx",
  });
  console.log(
    `Qualification ${kind}: ${result.status === 0 ? "passed" : "failed"}; retained ${directory}`,
  );
  process.exitCode = result.status ?? 1;
} else throw new Error("Expected storage, effects or native qualification");
