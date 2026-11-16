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
    ["corepack", ["pnpm", "format:check"]],
    ["corepack", ["pnpm", "lint"]],
    ["corepack", ["pnpm", "exec", "tsc", "--noEmit"]],
    ["corepack", ["pnpm", "exec", "vite", "build"]],
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

test("clean-checkout route preparation precedes standalone TypeScript without Vite", async () => {
  const { prepareRoutes } = await import("./prepare-generated.mjs");
  const files = new Map([["src/routeTree.gen.ts", "committed tree"]]);
  const calls = [];
  check("check", {
    prepare(mode) {
      calls.push(["routes", mode]);
      prepareRoutes(mode, {
        render: () => ({ "src/routeTree.gen.ts": "committed tree" }),
        read: (name) => files.get(name),
        write() {
          assert.fail("clean check wrote files");
        },
      });
    },
    run(command, args) {
      calls.push([command, ...args]);
      return { status: 0 };
    },
  });
  assert.deepEqual(calls[0], ["routes", "--check"]);
  assert.equal(calls[1].includes("tsc"), true);
  assert.equal(calls.filter((call) => call.includes("tsc")).length, 1);
  assert.equal(
    calls.some((call) => call.includes("vite")),
    false,
  );
});

test("stale route output fails CI before TypeScript or bundling can repair it", async () => {
  const { prepareRoutes } = await import("./prepare-generated.mjs");
  assert.throws(
    () =>
      check("ci", {
        prepare(mode) {
          prepareRoutes(mode, {
            render: () => ({ "src/routeTree.gen.ts": "current" }),
            read: () => "stale",
            write() {
              assert.fail("drift check repaired output");
            },
          });
        },
        run() {
          assert.fail("consumer ran before drift failure");
        },
      }),
    /route drift/,
  );
});

test("packaging claims the selected Cargo target without a staged executable", () => {
  const calls = [];
  assert.equal(
    build({
      generated: (mode) => calls.push(["generated", mode]),
      metadata: () => ({ target_directory: "/private/owned-target" }),
      claim: (target) => calls.push(["claim", target]),
      run(command, args, options) {
        calls.push([command, args]);
        assert.equal(options.env.CARGO_TARGET_DIR, "/private/owned-target");
        return { status: 0 };
      },
    }),
    0,
  );
  assert.deepEqual(calls, [
    ["generated", "--check"],
    ["claim", "/private/owned-target"],
    [
      "corepack",
      ["pnpm", "tauri", "build", "--bundles", "app", "--", "--locked"],
    ],
  ]);
});
