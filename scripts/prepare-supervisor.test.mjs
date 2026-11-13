import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import path from "node:path";
import test from "node:test";
import { root } from "./cargo.mjs";
import { prepareSupervisor } from "./prepare-supervisor.mjs";

const packageId =
  "path+file:///checkout/crates/supervisor#workspace-supervisor@0.1.0";
const artifact = {
  reason: "compiler-artifact",
  package_id: packageId,
  target: { name: "workspace-supervisor", kind: ["bin"] },
  profile: { opt_level: "3", test: false },
  executable: "/private/build-target/release/workspace-supervisor",
  fresh: true,
};
const binary = Buffer.from("current supervisor executable");

function stagingScenario({
  messages = [artifact],
  dirty = false,
  failBuild = false,
} = {}) {
  const commands = [];
  const copies = [];
  const directories = [];
  const writes = [];
  const reads = [];
  return {
    commands,
    copies,
    directories,
    writes,
    reads,
    effects: {
      claim(target) {
        assert.equal(target, "/private/build-target");
      },
      run(command, args) {
        commands.push([command, args]);
        if (command === "cargo" && args[0] === "metadata") {
          return JSON.stringify({
            target_directory: "/private/build-target",
            packages: [
              { name: "workspace-supervisor", id: packageId, version: "0.1.0" },
            ],
          });
        }
        if (command === "git" && args[0] === "rev-parse")
          return "revision-123\n";
        if (command === "git" && args[0] === "status")
          return dirty ? " M source.rs\n" : "";
        if (command === "cargo" && args[0] === "build") {
          if (failBuild) throw new Error("Cargo build failed");
          return [
            "a non-JSON tool diagnostic",
            ...messages.map((message) => JSON.stringify(message)),
            "",
          ].join("\n");
        }
        assert.fail(`Unexpected command: ${command} ${args.join(" ")}`);
      },
      copyFile(source, destination) {
        copies.push([source, destination]);
      },
      mkdir(directory, options) {
        directories.push([directory, options]);
      },
      readFile(file) {
        reads.push(file);
        return binary;
      },
      writeFile(file, content) {
        writes.push([file, content]);
      },
    },
  };
}

test("stages Cargo's exact executable from a private target and records its provenance", () => {
  const scenario = stagingScenario({
    dirty: true,
    messages: [
      {
        ...artifact,
        package_id: "another-package",
        executable: "/stale/workspace-supervisor",
      },
      { reason: "build-finished", success: true },
      artifact,
    ],
  });
  const provenance = prepareSupervisor("release", scenario.effects);
  const staged = path.join(root, "src-tauri/gen/sidecar/workspace-supervisor");
  assert.deepEqual(scenario.copies, [[artifact.executable, staged]]);
  assert.deepEqual(scenario.reads, [staged]);
  assert.deepEqual(scenario.directories, [
    [path.dirname(staged), { recursive: true }],
  ]);
  assert.deepEqual(provenance, {
    sourceRevision: "revision-123",
    applicationVersion: "0.1.0",
    dirty: true,
    targetDirectory: "/private/build-target",
    profile: "release",
    cargoProfile: artifact.profile,
    executable: artifact.executable,
    sha256: createHash("sha256").update(binary).digest("hex"),
  });
  assert.deepEqual(scenario.writes, [
    [
      path.join(path.dirname(staged), "provenance.json"),
      `${JSON.stringify(provenance, null, 2)}\n`,
    ],
  ]);
  assert.deepEqual(scenario.commands.at(-1), [
    "cargo",
    [
      "build",
      "--locked",
      "--message-format=json-render-diagnostics",
      "-p",
      "workspace-supervisor",
      "--bin",
      "workspace-supervisor",
      "--release",
    ],
  ]);
});

test("debug staging records the dev profile and a clean source tree", () => {
  const scenario = stagingScenario({
    messages: [
      {
        ...artifact,
        executable: "/private/build-target/debug/workspace-supervisor",
        profile: { opt_level: "0", test: false },
      },
    ],
  });
  const provenance = prepareSupervisor("debug", scenario.effects);
  assert.equal(provenance.profile, "dev");
  assert.equal(provenance.dirty, false);
  assert.equal(scenario.commands.at(-1)[1].includes("--release"), false);
  assert.equal(scenario.copies[0][0], provenance.executable);
});

for (const [name, messages] of [
  ["missing executable", [{ ...artifact, executable: null }]],
  ["test executable", [{ ...artifact, profile: { test: true } }]],
  [
    "ambiguous executables",
    [artifact, { ...artifact, executable: "/another/binary" }],
  ],
]) {
  test(`rejects ${name} without touching staged resources`, () => {
    const scenario = stagingScenario({ messages });
    assert.throws(
      () => prepareSupervisor("release", scenario.effects),
      /exactly one/,
    );
    assert.deepEqual(scenario.directories, []);
    assert.deepEqual(scenario.copies, []);
    assert.deepEqual(scenario.writes, []);
  });
}

test("a failed build cannot stage a previously available executable", () => {
  const scenario = stagingScenario({ failBuild: true });
  assert.throws(
    () => prepareSupervisor("release", scenario.effects),
    /Cargo build failed/,
  );
  assert.deepEqual(scenario.copies, []);
  assert.deepEqual(scenario.writes, []);
});
