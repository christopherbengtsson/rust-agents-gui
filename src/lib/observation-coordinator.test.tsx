import { expect, test, vi } from "vitest";
import { ObservationCoordinator } from "./observation-coordinator";

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (error: unknown) => void;
  const promise = new Promise<T>((yes, no) => {
    resolve = yes;
    reject = no;
  });
  return { promise, resolve, reject };
}

test("two physical reads run; identical queued and active targets coalesce", async () => {
  const coordinator = new ObservationCoordinator();
  const scope = coordinator.scope();
  const a = deferred<number>();
  const b = deferred<number>();
  const start = vi.fn(() => Promise.resolve(3));
  const first = scope.read(["a"], () => a.promise);
  const second = scope.read(["b"], () => b.promise);
  const third = scope.read(["c"], start);
  const duplicate = scope.read(["c"], start);
  const activeDuplicate = scope.read(["a"], () => Promise.resolve(99));
  expect(start).not.toHaveBeenCalled();
  a.resolve(1);
  expect(await first).toBe(1);
  expect(await activeDuplicate).toBe(1);
  expect(await third).toBe(3);
  expect(await duplicate).toBe(3);
  expect(start).toHaveBeenCalledTimes(1);
  b.resolve(2);
  expect(await second).toBe(2);
});

test("disposal drops queued work and drains active work without admitting a replacement early", async () => {
  const coordinator = new ObservationCoordinator();
  const old = coordinator.scope();
  const current = coordinator.scope();
  const a = deferred<number>();
  const b = deferred<number>();
  const first = old.read(["old-a"], () => a.promise);
  const second = old.read(["old-b"], () => b.promise);
  const queuedRun = vi.fn(() => Promise.resolve(1));
  const queued = old.read(["obsolete"], queuedRun);
  const rejected = [first, second, queued].map((promise) =>
    promise.catch((error: unknown) => error),
  );
  old.dispose();
  expect(await rejected[2]).toBeInstanceOf(Error);
  const nextRun = vi.fn(() => Promise.resolve(4));
  const next = current.read(["current"], nextRun);
  expect(nextRun).not.toHaveBeenCalled();
  a.resolve(9);
  expect(await rejected[0]).toBeInstanceOf(Error);
  expect(await next).toBe(4);
  expect(queuedRun).not.toHaveBeenCalled();
  b.reject(new Error("obsolete failure"));
  expect(await rejected[1]).toBeInstanceOf(Error);
});

test("64 queued targets retain admission and reject only the newest distinct target", async () => {
  const coordinator = new ObservationCoordinator();
  const scope = coordinator.scope();
  const a = deferred<number>();
  const first = scope.read(["active-a"], () => a.promise);
  const second = scope.read(["active-b"], () => a.promise);
  const start = vi.fn(() => Promise.resolve(2));
  const queued = Array.from({ length: 64 }, (_, i) => scope.read([i], start));
  const duplicate = scope.read([63], start);
  await expect(scope.read([64], start)).rejects.toMatchObject({
    code: "busy",
    source: "local",
    delivery: "not_sent",
  });
  expect(start).not.toHaveBeenCalled();
  a.resolve(1);
  await Promise.all([first, second, ...queued, duplicate]);
  expect(start).toHaveBeenCalledTimes(64);
});

test("owner, page and behavior flags distinguish reads; equivalent object keys coalesce", async () => {
  const scope = new ObservationCoordinator().scope();
  const held = deferred<number>();
  const start = vi.fn(() => held.promise);
  const first = scope.read(["owner-a", { id: "one", refresh: false }], start);
  const duplicate = scope.read(
    ["owner-a", { refresh: false, id: "one" }],
    start,
  );
  const requests = [
    scope.read(["owner-b", { id: "one", refresh: false }], start),
    scope.read(["owner-a", { id: "one", refresh: true }], start),
    scope.read(
      ["owner-a", { id: "one", refresh: false, after: "page-2" }],
      start,
    ),
  ];
  expect(start).toHaveBeenCalledTimes(2);
  held.resolve(1);
  await Promise.all([first, duplicate, ...requests]);
  expect(start).toHaveBeenCalledTimes(4);
});

test("one consumer can leave a coalesced read; failures and synchronous throws release capacity without retry", async () => {
  const coordinator = new ObservationCoordinator();
  const old = coordinator.scope();
  const current = coordinator.scope();
  const held = deferred<number>();
  const obsolete = old
    .read(["shared"], () => held.promise)
    .catch((error: unknown) => error);
  const read = current.read(["shared"], () => Promise.resolve(99));
  old.dispose();
  held.resolve(1);
  expect(await read).toBe(1);
  expect(await obsolete).toBeInstanceOf(Error);
  const start = vi.fn(() => {
    throw new Error("controlled throw");
  });
  await expect(current.read(["throws"], start)).rejects.toThrow(
    "controlled throw",
  );
  await expect(current.read(["after"], () => Promise.resolve(2))).resolves.toBe(
    2,
  );
  expect(start).toHaveBeenCalledTimes(1);
  await expect(old.read(["disposed"], start)).rejects.toThrow("obsolete");
});

test("a new interest coalesces with a physically draining identical read while the disposed interest stays obsolete", async () => {
  const coordinator = new ObservationCoordinator();
  const old = coordinator.scope();
  const current = coordinator.scope();
  const held = deferred<number>();
  const obsolete = old
    .read(["same-target"], () => held.promise)
    .catch((error: unknown) => error);
  old.dispose();
  const replacement = vi.fn(() => Promise.resolve(99));
  const read = current.read(["same-target"], replacement);
  expect(replacement).not.toHaveBeenCalled();
  held.resolve(1);
  expect(await read).toBe(1);
  expect(await obsolete).toBeInstanceOf(Error);
});
