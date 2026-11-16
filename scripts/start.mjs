// Native launcher. Tauri owns core startup and completed shutdown.
import { spawn } from "node:child_process";
import path from "node:path";
import { developmentEnvironment } from "./development-state.mjs";
import { cargoMetadata, root } from "./cargo.mjs";
import { prepareGenerated } from "./prepare-generated.mjs";

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
  // Native signal handling routes ordinary stop through the same owned Quit.
  await Promise.all(
    [...children].map(async (child) => {
      const exited = waitForExit(child);
      const pending = setTimeout(() => {
        console.error(
          "Native shutdown is still pending; no forced termination was sent. A manual force-kill requires recovery and is not a completed drain.",
        );
      }, 3000);
      try {
        await exited;
      } finally {
        clearTimeout(pending);
      }
    }),
  );
  process.exitCode = code;
}

for (const signal of ["SIGINT", "SIGTERM"]) {
  process.once(signal, () => void stop(0));
}

try {
  prepareGenerated("--check");
  // Ticket 07 removes the still-declared bundle resource. Preparation only
  // stages that unreachable resource; native startup never executes it.
  if (mode === "dev")
    await waitForExit(
      launch("node", ["scripts/prepare-supervisor.mjs", "debug"]),
    );
  if (!stopping) {
    let env = {
      ...process.env,
      CARGO_TARGET_DIR: cargoMetadata().target_directory,
    };
    if (mode === "dev") {
      env = developmentEnvironment({ env });
      console.log(`Native dev state: ${env.AGENT_WORKSPACE_STATE_DIR}`);
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
