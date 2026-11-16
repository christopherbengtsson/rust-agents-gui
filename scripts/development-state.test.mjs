import assert from "node:assert/strict";
import test from "node:test";
import { developmentEnvironment } from "./development-state.mjs";
const environment = (checkout = "/worktree", env = {}) =>
  developmentEnvironment({
    env,
    checkout,
    home: "/Users/test",
    canonical: (value) => value.replace("/alias", "/worktree"),
  });
test("native dev namespace is stable per canonical worktree and current schema", () => {
  const state = environment().AGENT_WORKSPACE_STATE_DIR;
  assert.equal(state, environment("/alias").AGENT_WORKSPACE_STATE_DIR);
  assert.notEqual(state, environment("/other").AGENT_WORKSPACE_STATE_DIR);
  assert.match(
    state,
    /\/.agent-workspace\/dev\/[a-f0-9]{12}\/native-schema-4$/,
  );
  assert.equal(environment().AGENT_WORKSPACE_SUPERVISOR, undefined);
});
test("explicit native state override remains exact", () => {
  assert.equal(
    environment("/worktree", { AGENT_WORKSPACE_STATE_DIR: "../chosen" })
      .AGENT_WORKSPACE_STATE_DIR,
    "../chosen",
  );
});
