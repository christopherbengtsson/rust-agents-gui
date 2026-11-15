import { expect, test } from "vitest";
import corpus from "../contracts/observations-v9.json";
import {
  validateRegistration,
  validateWorkspace,
  validateSupervisorSnapshot,
} from "./generated/contracts/validators.js";
import { decodeWorkspaceReply } from "./lib/workspaces";
import { decodeSnapshot } from "./lib/observation-contracts";

test("generated serialization contracts require output defaults and allow flattened variants", () => {
  const registration = {
    create_initial_workspace: false,
    initial_workspace_id: null,
    action_id: "00000000-0000-0000-0000-000000000002",
    path: "/projects/one",
    state: "completed",
    repository_id: "00000000-0000-0000-0000-000000000001",
  };
  expect(validateRegistration(registration)).toBe(true);
  const { initial_workspace_id: omitted, ...missing } = registration;
  expect(omitted).toBe(null);
  expect(validateRegistration(missing)).toBe(false);
  expect(validateRegistration({ ...registration, action_id: "invalid" })).toBe(
    false,
  );
});

test("structurally valid workspace paths still require normalized absolute locations", () => {
  const workspace = corpus.workspaces[0].reply.result.Ok.workspace;
  if (!workspace) throw new Error("Missing workspace fixture");
  const request = { operation: "get", id: workspace.id } as const;
  const ready = {
    ...workspace,
    name: "one",
    branch: "work/one",
    status: { state: "ready" },
  };
  for (const path of [
    "relative",
    "/projects/../one",
    "/projects/./one",
    "/projects//one",
    "/projects/one/",
  ]) {
    const value = { kind: "workspace", workspace: { ...ready, path } };
    expect(validateWorkspace(value.workspace)).toBe(true);
    expect(() => decodeWorkspaceReply(request, value)).toThrow();
  }
  expect(() =>
    decodeWorkspaceReply(request, {
      kind: "workspace",
      workspace: { ...ready, path: "/projects/one" },
    }),
  ).not.toThrow();
});

test("schema-valid numeric and multibyte payloads still require contextual budgets", () => {
  const unsafe = {
    ...corpus.snapshot,
    cursor: { owner: corpus.owner, revision: Number.MAX_SAFE_INTEGER + 1 },
  };
  expect(validateSupervisorSnapshot(unsafe)).toBe(true);
  expect(() => decodeSnapshot(unsafe)).toThrow();
  const largePid = {
    ...corpus.snapshot,
    status: { ...corpus.snapshot.status, supervisor_pid: 2 ** 32 },
  };
  expect(validateSupervisorSnapshot(largePid)).toBe(true);
  expect(() => decodeSnapshot(largePid)).toThrow();
  const multibyte = {
    ...corpus.snapshot,
    status: { ...corpus.snapshot.status, application_version: "🌍".repeat(40) },
  };
  expect(validateSupervisorSnapshot(multibyte)).toBe(true);
  expect(() => decodeSnapshot(multibyte)).toThrow();
});

import {
  validateDesktopPid,
  validateFeedBatch,
  validateFeedEvent,
  validateOwnedObservation,
  validateOwnedRepositoryReply,
  validateOwnedWorkspaceReply,
  validateRepositoryReply,
  validateRepositoryRequest,
  validateRequestFailure,
  validateWorkspaceReply,
  validateWorkspaceRequest,
} from "./generated/contracts/validators.js";
import {
  decodeOwnedReply,
  decodeRequestFailure,
} from "./lib/observation-contracts";
import { decodeRepositoryReply } from "./lib/repositories";
import { decodeFeedBatch } from "./lib/observation-feed";

test("every current synthetic corpus root passes its generated serialization validator", () => {
  expect(corpus.protocol).toBe(9);
  expect(validateDesktopPid(corpus.desktop_pid)).toBe(true);
  expect(validateSupervisorSnapshot(corpus.snapshot)).toBe(true);
  for (const entry of corpus.repositories) {
    expect(validateRepositoryRequest(entry.request)).toBe(true);
    expect(validateOwnedRepositoryReply(entry.reply)).toBe(true);
  }
  for (const entry of corpus.workspaces) {
    expect(validateWorkspaceRequest(entry.request)).toBe(true);
    expect(validateOwnedWorkspaceReply(entry.reply)).toBe(true);
  }
  for (const failure of corpus.failures)
    expect(validateRequestFailure(failure)).toBe(true);
  for (const batch of corpus.feed) {
    expect(validateFeedBatch(batch)).toBe(true);
    expect(validateFeedEvent({ kind: "batch", batch })).toBe(true);
  }
  expect(
    validateFeedEvent({
      kind: "stopped",
      subscription_id: "subscription",
      message: "stopped",
    }),
  ).toBe(true);
});

test("flattened schemas allow additional properties while contextual assertions reject contradictory states", () => {
  const registration = {
    create_initial_workspace: false,
    initial_workspace_id: null,
    action_id: "00000000-0000-0000-0000-000000000002",
    path: "/projects/one",
    state: "pending",
  };
  const request = {
    operation: "action",
    action_id: registration.action_id,
  } as const;
  const open = {
    kind: "action",
    registration: { ...registration, future_field: "allowed" },
  };
  expect(validateRepositoryReply(open)).toBe(true);
  expect(() => decodeRepositoryReply(request, open)).not.toThrow();
  for (const extra of [
    { repository_id: "00000000-0000-0000-0000-000000000001" },
    { error: { code: "storage", message: "contradiction" } },
  ]) {
    const reply = {
      kind: "action",
      registration: { ...registration, ...extra },
    };
    expect(validateRepositoryReply(reply)).toBe(true);
    expect(() => decodeRepositoryReply(request, reply)).toThrow();
  }
  const workspace = corpus.workspaces[0].reply.result.Ok.workspace;
  if (!workspace) throw new Error("Missing workspace fixture");
  const reply = {
    kind: "workspace",
    workspace: {
      ...workspace,
      status: { state: "queued", stage: "creating_branch" },
    },
  };
  expect(validateWorkspaceReply(reply)).toBe(true);
  expect(() =>
    decodeWorkspaceReply({ operation: "get", id: workspace.id }, reply),
  ).toThrow();
});

test("structurally valid owner, delivery, target and feed relationships still fail contextual assertions", () => {
  const envelope = { ...corpus.repositories[0].reply, owner: "wrong-owner" };
  expect(validateOwnedObservation(envelope)).toBe(true);
  expect(() =>
    decodeOwnedReply(corpus.owner, envelope, () => undefined),
  ).toThrow();
  const failure = {
    kind: "transport",
    error: {
      code: "busy",
      delivery: "unknown",
      message: "Contradictory evidence",
    },
  };
  expect(validateRequestFailure(failure)).toBe(true);
  expect(decodeRequestFailure(failure)).toMatchObject({
    code: "reply_unavailable",
    delivery: "unknown",
  });
  const batch = corpus.feed[0];
  for (const value of [
    { ...batch, cursor: { ...batch.cursor, owner: "wrong-owner" } },
    { ...batch, batch_id: 10 },
    {
      ...batch,
      sample: { ...batch.sample, watermark: batch.cursor.sequence + 1 },
    },
    {
      ...batch,
      cursor: { ...batch.cursor, sequence: Number.MAX_SAFE_INTEGER + 1 },
      sample: { ...batch.sample, watermark: Number.MAX_SAFE_INTEGER + 1 },
    },
  ]) {
    expect(validateFeedBatch(value)).toBe(true);
    expect(() =>
      decodeFeedBatch(value, corpus.owner, batch.subscription_id, 0),
    ).toThrow();
  }
});

test("malformed fields, variants, results and noncanonical UUIDs fail generated structure", () => {
  for (const value of [
    null,
    [],
    {},
    { kind: "unknown" },
    { kind: "workspace", workspace: {} },
  ])
    expect(validateWorkspaceReply(value)).toBe(false);
  for (const result of [
    {},
    { Ok: {}, Err: { code: "storage", message: "Both" } },
    { Err: { code: "invented", message: "Invalid" } },
  ])
    expect(validateOwnedObservation({ owner: corpus.owner, result })).toBe(
      false,
    );
  const workspace = corpus.workspaces[0].reply.result.Ok.workspace;
  if (!workspace) throw new Error("Missing workspace fixture");
  expect(
    validateWorkspace({ ...workspace, id: `urn:uuid:${workspace.id}` }),
  ).toBe(false);
  expect(
    validateRegistration({
      create_initial_workspace: false,
      initial_workspace_id: null,
      action_id: workspace.id,
      path: "/one",
      state: "completed",
    }),
  ).toBe(false);
});
