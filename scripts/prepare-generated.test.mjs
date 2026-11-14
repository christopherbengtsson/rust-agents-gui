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
    routes() {},
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
        routes() {},
        routes() {},
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

test("route checking rejects stale output without writes and explicit write repairs it", async () => {
  const { prepareRoutes } = await import("./prepare-generated.mjs");
  const calls = [];
  const effects = {
    render: () => ({ "src/routeTree.gen.ts": "current" }),
    read: () => "stale",
    write: (...args) => calls.push(args),
  };
  assert.throws(() => prepareRoutes("--check", effects), /route drift/);
  assert.deepEqual(calls, []);
  prepareRoutes("--write", effects);
  assert.deepEqual(calls, [["src/routeTree.gen.ts", "current"]]);
});

test("route rendering invokes the pinned CLI only in an isolated source mirror and cleans it", async () => {
  const { renderRoutes } = await import("./prepare-generated.mjs");
  const calls = [];
  const output = renderRoutes({
    create: () => "/fixture/isolated",
    copy: (source, destination) => calls.push(["copy", source, destination]),
    run(command, args, options) {
      assert.equal(command, process.execPath);
      assert.match(args[0], /router-cli.*bin\/tsr.cjs$/);
      assert.equal(args[1], "generate");
      assert.equal(options.cwd, "/fixture/isolated");
      assert.equal(options.timeout, 30000);
      calls.push(["generate"]);
      return { status: 0 };
    },
    list: () => ["index.tsx"],
    read: (name) => `generated ${name}`,
    cleanup: (directory) => calls.push(["cleanup", directory]),
  });
  assert.equal(calls[0][0], "copy");
  assert.equal(calls[1][0], "copy");
  assert.deepEqual(calls.slice(2), [
    ["generate"],
    ["cleanup", "/fixture/isolated"],
  ]);
  assert.deepEqual(Object.keys(output), [
    "src/routeTree.gen.ts",
    "src/routes/index.tsx",
  ]);
});
for (const failure of ["copy", "process", "timeout", "read"]) {
  test(`route rendering cleans its isolated mirror after ${failure} failure`, async () => {
    const { renderRoutes } = await import("./prepare-generated.mjs");
    const cleaned = [];
    assert.throws(() =>
      renderRoutes({
        create: () => "/fixture/isolated",
        copy: () => {
          if (failure === "copy") throw new Error("copy failed");
        },
        run: () =>
          failure === "timeout"
            ? { error: new Error("timeout") }
            : { status: failure === "process" ? 1 : 0 },
        list: () => [],
        read: () => {
          throw new Error("read failed");
        },
        cleanup: (directory) => cleaned.push(directory),
      }),
    );
    assert.deepEqual(cleaned, ["/fixture/isolated"]);
  });
}

test("generated preparation reconciles contracts before routes with one explicit mode", () => {
  const calls = [];
  prepareGenerated("--check", {
    run(command, args) {
      if (args[0] === "metadata")
        return JSON.stringify({
          target_directory: "/fixture/target",
          packages: [{ name: "workspace-core", id: "core" }],
        });
      if (args[0] === "build")
        return JSON.stringify({
          reason: "compiler-artifact",
          package_id: "core",
          target: { name: "export-contracts", kind: ["bin"] },
          profile: { test: false },
          executable: "/fixture/exporter",
        });
      return "{}";
    },
    claim() {},
    render: () => ({}),
    reconcile: (_, mode) => calls.push(["contracts", mode]),
    routes: (mode) => calls.push(["routes", mode]),
  });
  assert.deepEqual(calls, [
    ["contracts", "--check"],
    ["routes", "--check"],
  ]);
});
