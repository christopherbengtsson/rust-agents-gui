import assert from "node:assert/strict";
import path from "node:path";
import test from "node:test";
import { claimTarget, root } from "./cargo.mjs";

const target = "/private/build-target";
const marker = path.join(target, ".agent-workspace-owner.json");
function targetScenario({
  symlink = false,
  claimedBy,
  markerSymlink = false,
} = {}) {
  const calls = [];
  return {
    calls,
    effects: {
      lstat(file) {
        calls.push(["inspect", file]);
        return {
          isSymbolicLink: () => (file === target ? symlink : markerSymlink),
          isDirectory: () => file === target,
          isFile: () => file === marker,
        };
      },
      mkdir(directory) {
        calls.push(["mkdir", directory]);
      },
      realpath(file) {
        return file;
      },
      readFile(file) {
        calls.push(["read", file]);
        return JSON.stringify({ worktree: claimedBy });
      },
      writeFile(file, contents, options) {
        calls.push(["claim", file, JSON.parse(contents), options]);
        if (claimedBy !== undefined)
          throw Object.assign(new Error("exists"), { code: "EEXIST" });
      },
    },
  };
}

test("claims an unclaimed target exclusively for this worktree", () => {
  const scenario = targetScenario();
  claimTarget(target, scenario.effects);
  assert.deepEqual(scenario.calls.at(-1), [
    "claim",
    marker,
    { worktree: root },
    { flag: "wx" },
  ]);
});

test("permits a target already claimed by this worktree", () => {
  const scenario = targetScenario({ claimedBy: root });
  claimTarget(target, scenario.effects);
  assert.deepEqual(scenario.calls.at(-1), ["read", marker]);
});

test("rejects another worktree's target without changing its ownership", () => {
  const scenario = targetScenario({ claimedBy: "/another/worktree" });
  assert.throws(
    () => claimTarget(target, scenario.effects),
    /another worktree/,
  );
  assert.equal(
    scenario.calls.filter(([operation]) => operation === "claim").length,
    1,
  );
});

test("rejects a symlinked target before creating or claiming it", () => {
  const scenario = targetScenario({ symlink: true });
  assert.throws(() => claimTarget(target, scenario.effects), /not a symlink/);
  assert.deepEqual(scenario.calls, [["inspect", target]]);
});

test("rejects a target under a symlinked parent before claiming it", () => {
  const scenario = targetScenario();
  scenario.effects.realpath = (file) =>
    file === target ? "/another/build-target" : file;
  assert.throws(
    () => claimTarget(target, scenario.effects),
    /traverses a symlink/,
  );
  assert.equal(
    scenario.calls.some(([operation]) => operation === "claim"),
    false,
  );
});

test("rejects a symlinked ownership marker without following it", () => {
  const scenario = targetScenario({ claimedBy: root, markerSymlink: true });
  assert.throws(() => claimTarget(target, scenario.effects), /Unsafe/);
  assert.equal(
    scenario.calls.some(([operation]) => operation === "read"),
    false,
  );
});
