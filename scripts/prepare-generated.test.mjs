import assert from "node:assert/strict";
import test from "node:test";
import { reconcileArtifacts } from "./prepare-generated.mjs";

test("read-only drift reports missing/stale/unexpected artifacts without repair", () => {
  const writes = [];
  const files = new Map([
    ["types.ts", "stale"],
    ["old.js", "obsolete"],
  ]);
  const effects = {
    list: () => [...files.keys()],
    read: (name) => files.get(name),
    write: (...args) => writes.push(args),
    remove: (...args) => writes.push(args),
  };
  assert.throws(
    () =>
      reconcileArtifacts(
        { "types.ts": "current", "validators.js": "new" },
        "--check",
        effects,
      ),
    /stale: types.ts.*missing: validators.js.*unexpected: old.js/s,
  );
  assert.deepEqual(writes, []);
});

import { prepareGenerated } from "./prepare-generated.mjs";

test("preparation claims the target and uses only Cargo's reported exporter before reconciliation", () => {
  const calls = [];
  const artifact = {
    reason: "compiler-artifact",
    package_id: "core-id",
    target: { name: "export-contracts", kind: ["bin"] },
    profile: { test: false },
    executable: "/private/target/debug/export-contracts",
  };
  prepareGenerated("--check", {
    run(command, args) {
      calls.push([command, args]);
      if (args[0] === "metadata")
        return JSON.stringify({
          target_directory: "/private/target",
          packages: [{ name: "workspace-core", id: "core-id" }],
        });
      if (args[0] === "build") return JSON.stringify(artifact);
      assert.equal(command, artifact.executable);
      return '{"declarations":"wire types","schemas":{}}';
    },
    claim(target) {
      calls.push(["claim", target]);
    },
    render(exported) {
      assert.equal(exported.declarations, "wire types");
      calls.push(["render"]);
      return { "types.ts": "wire types" };
    },
    reconcile(expected, mode) {
      calls.push(["reconcile", expected, mode]);
    },
  });
  assert.deepEqual(calls, [
    ["cargo", ["metadata", "--locked", "--no-deps", "--format-version=1"]],
    ["claim", "/private/target"],
    [
      "cargo",
      [
        "build",
        "--locked",
        "-p",
        "workspace-core",
        "--bin",
        "export-contracts",
        "--features",
        "codegen",
        "--message-format=json-render-diagnostics",
      ],
    ],
    [artifact.executable, []],
    ["render"],
    ["reconcile", { "types.ts": "wire types" }, "--check"],
  ]);
});

test("explicit write mode repairs expected files and removes only obsolete managed outputs", () => {
  const files = new Map([
    ["types.ts", "stale"],
    ["obsolete.js", "old"],
  ]);
  reconcileArtifacts(
    { "types.ts": "current", "schemas.json": "new" },
    "--write",
    {
      list: () => [...files.keys()],
      read: (name) => files.get(name),
      write: (name, content) => files.set(name, content),
      remove: (name) => files.delete(name),
    },
  );
  assert.deepEqual(
    [...files.entries()],
    [
      ["types.ts", "current"],
      ["schemas.json", "new"],
    ],
  );
});

test("clean checkout checking accepts the complete committed artifact set without writes", () => {
  const expected = { "types.ts": "types", "validators.js": "code" };
  reconcileArtifacts(expected, "--check", {
    list: () => Object.keys(expected),
    read: (name) => expected[name],
    write() {
      assert.fail("check wrote files");
    },
    remove() {
      assert.fail("check removed files");
    },
  });
});

for (const failure of [
  "build",
  "export",
  "render",
  "missing",
  "ambiguous",
  "test-artifact",
  "foreign-artifact",
]) {
  test(`preparation ${failure} failure cannot reconcile or repair committed files`, () => {
    const artifact = {
      reason: "compiler-artifact",
      package_id: "core-id",
      target: { name: "export-contracts", kind: ["bin"] },
      profile: { test: false },
      executable: "/private/exporter",
    };
    assert.throws(() =>
      prepareGenerated("--write", {
        claim() {},
        run(command, args) {
          if (args[0] === "metadata")
            return JSON.stringify({
              target_directory: "/private/target",
              packages: [{ name: "workspace-core", id: "core-id" }],
            });
          if (args[0] === "build") {
            if (failure === "build") throw new Error("failed build");
            const artifacts =
              failure === "missing"
                ? []
                : failure === "ambiguous"
                  ? [artifact, artifact]
                  : [
                      {
                        ...artifact,
                        ...(failure === "test-artifact"
                          ? { profile: { test: true } }
                          : {}),
                        ...(failure === "foreign-artifact"
                          ? { package_id: "foreign" }
                          : {}),
                      },
                    ];
            return artifacts.map((value) => JSON.stringify(value)).join("\n");
          }
          assert.equal(command, artifact.executable);
          if (failure === "export") throw new Error("failed export");
          return "{}";
        },
        render() {
          if (failure === "render") throw new Error("failed render");
          assert.fail("unexpected render");
        },
        reconcile() {
          assert.fail("reconciled after failure");
        },
      }),
    );
  });
}
