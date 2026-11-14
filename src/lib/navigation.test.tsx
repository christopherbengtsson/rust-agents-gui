import { expect, test } from "vitest";
import { createMemoryHistory } from "@tanstack/react-router";
import { Navigation, workspaceSearch } from "./navigation";

const repository = { kind: "repository", repositoryId: "a" } as const;
test("history and A-B-A revoke completion eligibility; preloading has no navigation authority", () => {
  const history = createMemoryHistory({ initialEntries: ["/repository/a"] });
  const navigation = new Navigation(history, (target) => {
    history.push(
      target.kind === "home" ? "/" : `/repository/${target.repositoryId}`,
    );
  });
  const stop = navigation.start();
  const token = navigation.capture();
  navigation.go({ kind: "repository", repositoryId: "b" });
  history.back();
  expect(navigation.complete(token, repository)).toBe(false);
  const next = navigation.capture();
  history.push("/settings/general");
  expect(navigation.complete(next, repository)).toBe(false);
  stop();
});
test("view validation accepts only known views and defaults only missing values", () => {
  expect(workspaceSearch({})).toEqual({ view: "workspace" });
  expect(workspaceSearch({ view: "changes" })).toEqual({ view: "changes" });
  for (const view of ["other", null, 0, ["changes"]])
    expect(() => workspaceSearch({ view })).toThrow("Invalid workspace view");
});
