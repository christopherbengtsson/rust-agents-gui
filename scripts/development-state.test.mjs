import assert from "node:assert/strict";
import test from "node:test";
import { developmentEnvironment } from "./development-state.mjs";
const built = {
  executable: "/target/debug/workspace-supervisor",
  applicationVersion: "0.1.0",
};
function environment({
  checkout = "/worktree",
  protocol = "4\n",
  version = "0.1.0",
  env = {},
} = {}) {
  const calls = [];
  const result = developmentEnvironment(
    { ...built, applicationVersion: version },
    {
      env,
      checkout,
      home: "/Users/test",
      canonical: (value) => value.replace("/alias", "/worktree"),
      run: (exe, args) => {
        calls.push([exe, args]);
        return protocol;
      },
    },
  );
  assert.deepEqual(calls, [[built.executable, ["--protocol-version"]]]);
  return result;
}
test("dev state is persistent, canonical-worktree and compatibility specific", () => {
  const a = environment();
  assert.equal(
    a.AGENT_WORKSPACE_STATE_DIR,
    environment().AGENT_WORKSPACE_STATE_DIR,
  );
  assert.equal(
    a.AGENT_WORKSPACE_STATE_DIR,
    environment({ checkout: "/alias" }).AGENT_WORKSPACE_STATE_DIR,
  );
  assert.notEqual(
    a.AGENT_WORKSPACE_STATE_DIR,
    environment({ checkout: "/other" }).AGENT_WORKSPACE_STATE_DIR,
  );
  assert.notEqual(
    a.AGENT_WORKSPACE_STATE_DIR,
    environment({ protocol: "5" }).AGENT_WORKSPACE_STATE_DIR,
  );
  assert.notEqual(
    a.AGENT_WORKSPACE_STATE_DIR,
    environment({ version: "0.2.0" }).AGENT_WORKSPACE_STATE_DIR,
  );
  assert.match(
    a.AGENT_WORKSPACE_STATE_DIR,
    /\/\.agent-workspace\/dev\/[a-f0-9]{12}\/p4-[a-f0-9]{8}$/,
  );
  assert.equal(a.AGENT_WORKSPACE_SUPERVISOR, built.executable);
});
test("an explicit state override is honored exactly and stale supervisor overrides are replaced", () => {
  const a = environment({
    env: {
      AGENT_WORKSPACE_STATE_DIR: "../my-state",
      AGENT_WORKSPACE_SUPERVISOR: "/old/binary",
    },
  });
  assert.equal(a.AGENT_WORKSPACE_STATE_DIR, "../my-state");
  assert.equal(a.AGENT_WORKSPACE_SUPERVISOR, built.executable);
  assert.throws(() => environment({ protocol: "ready" }), /protocol metadata/);
});
