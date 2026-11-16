import assert from "node:assert/strict";
import test from "node:test";
import { EventEmitter } from "node:events";
import { createLauncher } from "./start.mjs";

function scenario() {
  const children = [];
  const warnings = [];
  let deadline;
  const launcher = createLauncher({
    spawn(command, args) {
      const child = new EventEmitter();
      child.command = command;
      child.args = args;
      child.signals = [];
      child.kill = (signal) => {
        child.signals.push(signal);
        return true;
      };
      children.push(child);
      return child;
    },
    warn: (message) => warnings.push(message),
    schedule: (callback) => {
      deadline = callback;
      return 1;
    },
    unschedule() {},
  });
  return { launcher, children, warnings, deadline: () => deadline() };
}

test("ordinary stop awaits retained native completion, reports pending and never escalates", async () => {
  const s = scenario();
  const child = s.launcher.launch("native", []);
  const stopped = s.launcher.stop();
  let completed = false;
  stopped.then(() => {
    completed = true;
  });
  s.deadline();
  await Promise.resolve();
  assert.equal(completed, false);
  assert.deepEqual(child.signals, ["SIGTERM"]);
  assert.match(s.warnings[0], /pending.*recovery/);
  assert.equal(s.launcher.stop(), stopped);
  child.emit("exit", 0, null);
  await stopped;
});

test("already exited children are never signalled and force termination is failure", async () => {
  const s = scenario();
  const old = s.launcher.launch("native", []);
  old.emit("exit", 0, null);
  await s.launcher.wait(old);
  await s.launcher.stop();
  assert.deepEqual(old.signals, []);
  for (const [code, signal] of [
    [null, "SIGKILL"],
    [143, null],
  ]) {
    const crashed = scenario();
    const child = crashed.launcher.launch("native", []);
    const stopped = crashed.launcher.stop();
    child.emit("exit", code, signal);
    await assert.rejects(stopped, /recovery/);
  }
});

test("native child shutdown completes before the development server stops", async () => {
  const s = scenario();
  const frontend = s.launcher.launch("vite", [], {}, "service");
  const native = s.launcher.launch("native", []);
  const stopped = s.launcher.stop();
  assert.deepEqual(frontend.signals, []);
  native.emit("exit", 0, null);
  await new Promise((resolve) => setImmediate(resolve));
  assert.deepEqual(frontend.signals, ["SIGTERM"]);
  frontend.emit("exit", 143, null);
  await stopped;
});
