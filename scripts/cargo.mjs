import { execFileSync, spawnSync } from "node:child_process";
import {
  lstatSync,
  mkdirSync,
  readFileSync,
  realpathSync,
  writeFileSync,
} from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

export const root = fileURLToPath(new URL("../", import.meta.url));

export function runCommand(command, args) {
  return execFileSync(command, args, {
    cwd: root,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "inherit"],
    maxBuffer: 16 * 1024 * 1024,
  });
}

export function cargoMetadata(run = runCommand) {
  return JSON.parse(
    run("cargo", ["metadata", "--locked", "--no-deps", "--format-version=1"]),
  );
}

export function claimTarget(
  target,
  {
    lstat = lstatSync,
    mkdir = mkdirSync,
    readFile = readFileSync,
    realpath = realpathSync,
    writeFile = writeFileSync,
  } = {},
) {
  const inspect = (file) => {
    try {
      return lstat(file);
    } catch (error) {
      if (error.code === "ENOENT") return undefined;
      throw error;
    }
  };
  const existing = inspect(target);
  if (existing && (existing.isSymbolicLink() || !existing.isDirectory()))
    throw new Error(
      `Cargo target must be a real directory, not a symlink: ${target}`,
    );
  mkdir(target, { recursive: true });
  if (realpath(target) !== path.resolve(target))
    throw new Error(`Cargo target path traverses a symlink: ${target}`);
  const marker = path.join(target, ".agent-workspace-owner.json");
  const owner = realpath(root);
  try {
    writeFile(marker, `${JSON.stringify({ worktree: owner })}\n`, {
      flag: "wx",
    });
  } catch (error) {
    if (error.code !== "EEXIST") throw error;
    const info = inspect(marker);
    if (!info?.isFile() || info.isSymbolicLink())
      throw new Error(`Unsafe Cargo target ownership marker: ${marker}`);
    if (JSON.parse(readFile(marker, "utf8")).worktree !== owner)
      throw new Error(
        `Cargo target belongs to another worktree; select a private CARGO_TARGET_DIR: ${target}`,
      );
  }
}

if (
  process.argv[1] &&
  path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)
) {
  const metadata = cargoMetadata();
  claimTarget(metadata.target_directory);
  const result = spawnSync("cargo", process.argv.slice(2), {
    cwd: root,
    env: { ...process.env, CARGO_TARGET_DIR: metadata.target_directory },
    stdio: "inherit",
  });
  if (result.error) throw result.error;
  process.exitCode = result.status ?? 1;
}
