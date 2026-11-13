import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import { transformSync } from "oxc-transform-react";

for (const file of [
  "src/hooks/use-supervisor-connection.ts",
  "src/hooks/use-repositories.ts",
  "src/hooks/use-workspaces.ts",
  "src/components/blocks/repository-registration.tsx",
]) {
  test(`installed React compiler accepts lifecycle control flow in ${file}`, () => {
    const result = transformSync(file, readFileSync(file, "utf8"), {
      jsx: "preserve",
      reactCompiler: {
        compilationMode: "infer",
        panicThreshold: "none",
        target: "19",
        reportDiagnostics: true,
      },
    });
    assert.equal(result.fatal, false);
    assert.deepEqual(result.errors, []);
    assert.match(result.code, /_c\(/);
  });
}
