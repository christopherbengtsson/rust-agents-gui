// Retained native child ownership. Ordinary stop waits for the core's completed Quit.
import { spawn } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { developmentEnvironment } from "./development-state.mjs";
import { cargoMetadata, claimTarget, root } from "./cargo.mjs";
import { prepareGenerated } from "./prepare-generated.mjs";

export function createLauncher({
  spawn: launchProcess = spawn,
  warn = console.error,
  schedule = setTimeout,
  unschedule = clearTimeout,
} = {}) {
  const active = new Set();
  const records = new WeakMap();
  let stopping;
  function launch(command, args, options = {}, kind = "native") {
    if (stopping) throw new Error("Launcher is stopping");
    // Isolate retained children from foreground Ctrl-C. Only this owner requests
    // their shutdown; ignored stdin prevents background terminal reads.
    const child = launchProcess(command, args, {
      cwd: root,
      stdio: ["ignore", "inherit", "inherit"],
      detached: true,
      ...options,
    });
    const record = { child, kind, done: false, requested: false };
    records.set(child, record);
    active.add(record);
    record.exited = new Promise((resolve, reject) => {
      child.once("error", (error) => {
        record.done = true;
        active.delete(record);
        reject(error);
      });
      child.once("exit", (code, signal) => {
        record.done = true;
        active.delete(record);
        if (
          code === 0 ||
          (kind === "service" &&
            record.requested &&
            (signal === "SIGTERM" || code === 143))
        )
          resolve();
        else
          reject(
            new Error(
              `Process exited with ${signal ?? code}; native termination requires recovery, not a completed drain`,
            ),
          );
      });
    });
    // A service can fail before its owner starts awaiting it. Retain the rejection.
    record.exited.catch(() => {});
    return child;
  }
  const wait = (child) => records.get(child).exited;
  function stop() {
    if (stopping) return stopping;
    stopping = (async () => {
      const failures = [];
      // Keep presentation available while native cleanup is pending or failed.
      for (const kind of ["native", "service"]) {
        const retained = [...active].filter((record) => record.kind === kind);
        for (const record of retained) {
          if (!record.done) {
            record.requested = true;
            record.child.kill("SIGTERM");
          }
        }
        const timer = retained.length
          ? schedule(
              () =>
                warn(
                  "Native shutdown is pending; keep the app open. No forced termination was sent; a force-kill requires recovery and is not success.",
                ),
              3000,
            )
          : undefined;
        try {
          const results = await Promise.allSettled(
            retained.map((record) => record.exited),
          );
          failures.push(
            ...results
              .filter((result) => result.status === "rejected")
              .map((result) => result.reason),
          );
        } finally {
          if (timer !== undefined) unschedule(timer);
        }
      }
      if (failures.length)
        throw new AggregateError(
          failures,
          "Shutdown failed; native recovery is required",
        );
    })();
    return stopping;
  }
  return {
    launch,
    wait,
    stop,
    get stopping() {
      return Boolean(stopping);
    },
  };
}

export async function start(mode) {
  if (!["dev", "release"].includes(mode))
    throw new Error("usage: node scripts/start.mjs <dev|release>");
  const launcher = createLauncher();
  const stop = () =>
    launcher.stop().catch((error) => {
      console.error(error);
      process.exitCode = 1;
    });
  for (const signal of ["SIGINT", "SIGTERM"]) process.on(signal, stop);
  try {
    prepareGenerated("--check");
    const target = cargoMetadata().target_directory;
    claimTarget(target);
    let env = { ...process.env, CARGO_TARGET_DIR: target };
    if (mode === "dev") {
      env = developmentEnvironment({ env });
      console.log(`Native dev state: ${env.AGENT_WORKSPACE_STATE_DIR}`);
      // Native edits require completed Quit and an explicit restart. Vite retains HMR.
      await launcher.wait(
        launcher.launch(
          "cargo",
          ["build", "--locked", "-p", "workspace-desktop"],
          { env },
        ),
      );
      if (launcher.stopping) return;
      const frontend = launcher.launch(
        process.execPath,
        ["node_modules/vite/bin/vite.js"],
        { env, stdio: ["ignore", "pipe", "inherit"] },
        "service",
      );
      frontend.stdout.on("data", (bytes) => process.stdout.write(bytes));
      // A failed strict-port server must not leave a native owner behind.
      launcher.wait(frontend).then(stop, (error) => {
        console.error(error);
        process.exitCode = 1;
        void stop();
      });
      let ready;
      let timer;
      let readiness = "";
      try {
        await Promise.race([
          new Promise((resolve, reject) => {
            ready = (bytes) => {
              readiness = (readiness + bytes.toString()).slice(-1024);
              if (readiness.includes("http://127.0.0.1:1420/")) resolve();
            };
            frontend.stdout.on("data", ready);
            timer = setTimeout(
              () =>
                reject(
                  new Error(
                    "Development server did not become ready within 15 seconds",
                  ),
                ),
              15000,
            );
          }),
          launcher.wait(frontend).then(() => {
            throw new Error("Development server exited before native launch");
          }),
        ]);
      } finally {
        clearTimeout(timer);
        frontend.stdout.off("data", ready);
      }
    }
    if (!launcher.stopping) {
      const executable =
        mode === "dev"
          ? path.join(target, "debug/workspace-desktop")
          : path.join(
              target,
              "release/bundle/macos/Agent Workspace.app/Contents/MacOS/workspace-desktop",
            );
      await launcher.wait(launcher.launch(executable, [], { env }));
    }
  } catch (error) {
    console.error(error);
    process.exitCode = 1;
  } finally {
    await stop();
    for (const signal of ["SIGINT", "SIGTERM"]) process.off(signal, stop);
  }
}
if (
  process.argv[1] &&
  path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)
)
  await start(process.argv[2]);
