// Bootstrap harness only. Product discovery/reconnection and shutdown are ticket 02/09.
import { spawn } from "node:child_process";
import { chmod, mkdtemp, rm } from "node:fs/promises";
import { createInterface } from "node:readline";
import { fileURLToPath } from "node:url";
import path from "node:path";

const root = fileURLToPath(new URL("../", import.meta.url));
const mode = process.argv[2];
if (!["dev", "release"].includes(mode)) {
  throw new Error("usage: node scripts/start.mjs <dev|release>");
}
const children = new Set();
let stopping = false;
let privateState;

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

async function stop(code) {
  if (stopping) return;
  stopping = true;
  for (const child of children) {
    try {
      process.kill(-child.pid, "SIGTERM");
    } catch (error) {
      if (error.code !== "ESRCH") console.error(error);
    }
  }
  // These are bootstrap processes with no admitted work or durable storage.
  await Promise.all(
    [...children].map(async (child) => {
      const exited = waitForExit(child);
      const deadline = setTimeout(() => {
        try {
          process.kill(-child.pid, "SIGKILL");
        } catch (error) {
          if (error.code !== "ESRCH") console.error(error);
        }
      }, 3000);
      try {
        await exited;
      } finally {
        clearTimeout(deadline);
      }
    }),
  );
  if (privateState) await rm(privateState, { recursive: true, force: true });
  process.exitCode = code;
}

for (const signal of ["SIGINT", "SIGTERM"]) {
  process.once(signal, () => void stop(0));
}

try {
  if (mode === "dev") {
    await waitForExit(
      launch("cargo", ["build", "--locked", "-p", "workspace-supervisor"]),
    );
  }
  if (!stopping) {
    // macOS Unix socket paths are short; avoid the long per-user TMPDIR.
    privateState = await mkdtemp("/tmp/agent-workspace-");
    await chmod(privateState, 0o700);
    const socket = path.join(privateState, "supervisor.sock");
    const supervisor = launch(
      path.join(
        root,
        "target",
        mode === "dev" ? "debug" : "release",
        "workspace-supervisor",
      ),
      [socket],
      { stdio: ["ignore", "pipe", "inherit"] },
    );
    let readyCleanup;
    const ready = new Promise((resolve, reject) => {
      const lines = createInterface({ input: supervisor.stdout });
      const deadline = setTimeout(
        () => reject(new Error("Supervisor startup timed out")),
        5000,
      );
      supervisor.once("error", reject);
      supervisor.once("exit", () =>
        reject(new Error("Supervisor exited before startup")),
      );
      lines.on("line", (line) => {
        if (line === "ready") resolve();
      });
      lines.once("close", () => clearTimeout(deadline));
      // Keep draining output after readiness; no unbounded in-memory output.
      readyCleanup = () => clearTimeout(deadline);
    });
    await ready.finally(() => readyCleanup?.());
    console.log(`Supervisor PID ${supervisor.pid} · ${socket}`);
    const env = { ...process.env, AGENT_WORKSPACE_SOCKET: socket };
    const desktop =
      mode === "dev"
        ? launch("corepack", ["pnpm", "tauri", "dev"], { env })
        : launch(
            path.join(
              root,
              "target/release/bundle/macos/Agent Workspace.app/Contents/MacOS/workspace-desktop",
            ),
            [],
            {
              env,
            },
          );
    supervisor.once("exit", () => {
      if (!stopping) {
        console.error(
          "Supervisor exited. Stop and restart the bootstrap to recover.",
        );
        void stop(1);
      }
    });
    await waitForExit(desktop);
    await stop(0);
  }
} catch (error) {
  console.error(error.message);
  await stop(1);
}
