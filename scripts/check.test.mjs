import assert from "node:assert/strict";
import test from "node:test";
import { check } from "./check.mjs";

test("the complete gate rejects drift before any consumer and never requests write mode", () => {
  const calls = [];
  assert.throws(
    () =>
      check("verify", {
        prepare(mode) {
          calls.push(mode);
          throw new Error("drift");
        },
        run() {
          assert.fail("consumer started after drift");
        },
      }),
    /drift/,
  );
  assert.deepEqual(calls, ["--check"]);
});

test("CI checks preparation once, builds assets separately, then selects every retained check once", () => {
  const calls = [];
  check("ci", {
    prepare(mode) {
      calls.push(["prepare", mode]);
    },
    run(command, args) {
      calls.push([command, args]);
      return { status: 0 };
    },
  });
  assert.deepEqual(calls, [
    ["prepare", "--check"],
    ["node", ["scripts/prepare-supervisor.mjs", "debug"]],
    ["corepack", ["pnpm", "exec", "vite", "build"]],
    ["corepack", ["pnpm", "format:check"]],
    ["corepack", ["pnpm", "lint"]],
    ["corepack", ["pnpm", "exec", "tsc", "--noEmit"]],
    [
      "node",
      [
        "scripts/cargo.mjs",
        "clippy",
        "--locked",
        "--workspace",
        "--all-targets",
        "--all-features",
        "--",
        "-D",
        "warnings",
      ],
    ],
    ["node", ["scripts/test-unit.mjs"]],
  ]);
});

import { build } from "./build.mjs";
test("packaging rejects drift before resource staging or Tauri invocation", () => {
  assert.throws(
    () =>
      build({
        generated(mode) {
          assert.equal(mode, "--check");
          throw new Error("drift");
        },
        prepare() {
          assert.fail("staged after drift");
        },
        run() {
          assert.fail("packaged after drift");
        },
      }),
    /drift/,
  );
});

test("a failed static gate prevents later application units", () => {
  const calls = [];
  assert.throws(
    () =>
      check("verify", {
        prepare() {},
        run(command, args) {
          calls.push([command, args]);
          return { status: args.includes("tsc") ? 2 : 0 };
        },
      }),
    /Gate failed/,
  );
  assert.equal(calls.length, 3);
  assert.equal(calls.at(-1)[1].includes("tsc"), true);
});
