import { createHash, randomUUID } from "node:crypto";
import {
  copyFileSync,
  mkdirSync,
  readFileSync,
  renameSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { cargoMetadata, claimTarget, root, runCommand } from "./cargo.mjs";

export function prepareSupervisor(
  mode,
  {
    run = runCommand,
    copyFile = copyFileSync,
    mkdir = mkdirSync,
    readFile = readFileSync,
    writeFile = writeFileSync,
    rename = renameSync,
    remove = rmSync,
    claim = claimTarget,
  } = {},
) {
  if (!["debug", "release"].includes(mode))
    throw new Error("Expected debug or release");
  const metadata = cargoMetadata(run);
  const supervisor = metadata.packages.find(
    (pkg) => pkg.name === "workspace-supervisor",
  );
  if (!supervisor)
    throw new Error("Supervisor package missing from Cargo metadata");
  claim(metadata.target_directory);
  const sourceRevision = run("git", ["rev-parse", "HEAD"]).trim();
  const dirty =
    run("git", ["status", "--porcelain", "--untracked-files=normal"]).trim()
      .length > 0;
  const output = run("cargo", [
    "build",
    "--locked",
    "--message-format=json-render-diagnostics",
    "-p",
    "workspace-supervisor",
    "--bin",
    "workspace-supervisor",
    ...(mode === "release" ? ["--release"] : []),
  ]);
  const artifacts = output
    .split("\n")
    .filter((line) => line.startsWith("{"))
    .map((line) => JSON.parse(line))
    .filter(
      (message) =>
        message.reason === "compiler-artifact" &&
        message.package_id === supervisor.id &&
        message.target?.name === "workspace-supervisor" &&
        message.target.kind.includes("bin") &&
        !message.profile.test &&
        message.executable,
    );
  if (artifacts.length !== 1)
    throw new Error(
      "Cargo must report exactly one supervisor executable; nothing staged",
    );
  const artifact = artifacts[0];
  const staging = path.join(root, "src-tauri/gen/sidecar");
  const binary = path.join(staging, "workspace-supervisor");
  mkdir(staging, { recursive: true });
  // macOS caches code signatures by inode. Never overwrite executable bytes in
  // place: publish a fresh same-directory file, preserving the old binary if
  // copying fails. See Apple's "Updating Mac Software" guidance.
  const temporary = path.join(staging, `.workspace-supervisor-${randomUUID()}`);
  try {
    copyFile(artifact.executable, temporary);
    rename(temporary, binary);
  } finally {
    remove(temporary, { force: true });
  }
  const provenance = {
    sourceRevision,
    applicationVersion: supervisor.version,
    dirty,
    targetDirectory: metadata.target_directory,
    profile: mode === "debug" ? "dev" : "release",
    cargoProfile: artifact.profile,
    executable: artifact.executable,
    sha256: createHash("sha256").update(readFile(binary)).digest("hex"),
  };
  writeFile(
    path.join(staging, "provenance.json"),
    `${JSON.stringify(provenance, null, 2)}\n`,
  );
  return provenance;
}

if (
  process.argv[1] &&
  path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)
) {
  prepareSupervisor(process.argv[2]);
}
