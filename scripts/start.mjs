// Development/build launcher. Rust owns supervisor discovery and lifetime.
import { spawn } from "node:child_process";
import path from "node:path";
import { cargoMetadata, root } from "./cargo.mjs";

const mode = process.argv[2];
if (!["dev", "release"].includes(mode)) {
  throw new Error("usage: node scripts/start.mjs <dev|release>");
}
const children = new Set();
let stopping = false;

function launch(command, args, options = {}) {
  const child = spawn(command, args, {
    cwd: root,
    stdio: "inherit",
    detached: true,
    ...options,
  });
  children.add(child);
  child.once("exit", () => children.delete(child));
  child.once("error", () => children.delete(child));
  return child;
}

async function waitForExit(child) {
  return new Promise((resolve, reject) => {
    child.once("error", reject);
    child.once("exit", (code, signal) => {
      if (code === 0 || stopping) resolve();
      else reject(new Error(`Process exited with ${signal ?? code}`));
    });
  });
}

function signalGroup(child, signal) {
  try {
    process.kill(-child.pid, signal);
  } catch (error) {
    if (error.code !== "ESRCH") console.error(error);
  }
}

async function stop(code) {
  if (stopping) return;
  stopping = true;
  for (const child of children) signalGroup(child, "SIGTERM");
  // Supervisor is started by Rust in its own process group and is not in this set.
  await Promise.all(
    [...children].map(async (child) => {
      const exited = waitForExit(child);
      const deadline = setTimeout(() => {
        signalGroup(child, "SIGKILL");
      }, 3000);
      try {
        await exited;
      } finally {
        clearTimeout(deadline);
      }
    }),
  );
  process.exitCode = code;
}

for (const signal of ["SIGINT", "SIGTERM"]) {
  process.once(signal, () => void stop(0));
}

try {
  if (mode === "dev") {
    await waitForExit(
      launch("node", ["scripts/prepare-supervisor.mjs", "debug"]),
    );
  }
  if (!stopping) {
    const env = { ...process.env };
    env.CARGO_TARGET_DIR = cargoMetadata().target_directory;
    if (mode === "dev") {
      env.AGENT_WORKSPACE_SUPERVISOR = path.join(
        root,
        "src-tauri/gen/sidecar/workspace-supervisor",
      );
    }
    const desktop =
      mode === "dev"
        ? launch("corepack", ["pnpm", "tauri", "dev"], { env })
        : launch(
            path.join(
              cargoMetadata().target_directory,
              "release/bundle/macos/Agent Workspace.app/Contents/MacOS/workspace-desktop",
            ),
            [],
            {
              env,
            },
          );
    await waitForExit(desktop);
    await stop(0);
  }
} catch (error) {
  console.error(error.message);
  await stop(1);
}
