import { expect, test } from "vitest";
import { completeOperation } from "./complete-operation";

test("preserves results and always settles, including a throwing error handler", async () => {
  const calls: string[] = [];
  const done = () => {
    calls.push("settled");
  };
  expect(await completeOperation(async () => false, undefined, done)).toBe(
    false,
  );
  const failure = new Error("operation");
  await expect(
    completeOperation(
      async () => {
        throw failure;
      },
      undefined,
      done,
    ),
  ).rejects.toBe(failure);
  expect(
    await completeOperation(
      async () => {
        throw failure;
      },
      () => true,
      done,
    ),
  ).toBe(true);
  const handler = new Error("handler");
  await expect(
    completeOperation(
      async () => {
        throw failure;
      },
      () => {
        throw handler;
      },
      done,
    ),
  ).rejects.toBe(handler);
  expect(calls).toEqual(Array(4).fill("settled"));
});
