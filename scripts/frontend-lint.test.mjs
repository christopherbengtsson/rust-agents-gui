import assert from "node:assert/strict";
import test from "node:test";
import { spawnSync } from "node:child_process";
import {
  mkdtempSync,
  mkdirSync,
  copyFileSync,
  writeFileSync,
  rmSync,
} from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { after } from "node:test";

const root = fileURLToPath(new URL("../", import.meta.url));
const fixtures = mkdtempSync(path.join(tmpdir(), "frontend-lint-"));
after(() => rmSync(fixtures, { recursive: true, force: true }));
function run(sources, extra = []) {
  copyFileSync(
    path.join(root, ".oxlintrc.json"),
    path.join(fixtures, ".oxlintrc.json"),
  );
  for (const [file, source] of Object.entries(sources)) {
    const destination = path.join(fixtures, file);
    mkdirSync(path.dirname(destination), { recursive: true });
    writeFileSync(destination, source);
  }
  const result = spawnSync(
    process.execPath,
    [
      path.join(root, "node_modules/oxlint/bin/oxlint"),
      "--config",
      ".oxlintrc.json",
      "--format",
      "json",
      "--max-warnings",
      "0",
      "--report-unused-disable-directives-severity",
      "warn",
      ...extra,
      ...Object.keys(sources),
    ],
    { cwd: fixtures, encoding: "utf8", timeout: 15000 },
  );
  assert.ifError(result.error);
  assert.equal(result.signal, null);
  const output = JSON.parse(result.stdout);
  assert.ok(Array.isArray(output.diagnostics), result.stderr);
  for (const diagnostic of output.diagnostics) {
    assert.ok(
      typeof diagnostic.code === "string" ||
        /^Unused (?:oxlint|eslint)-disable directive/.test(diagnostic.message),
      JSON.stringify(diagnostic),
    );
    assert.ok(["error", "warning"].includes(diagnostic.severity));
  }
  assert.equal(result.status, output.diagnostics.length ? 1 : 0, result.stderr);
  return output.diagnostics;
}
async function lint(source, filePath = "src/components/blocks/example.tsx") {
  return run({ [filePath]: source }).map((message) => message.code);
}

test("frontend lint accepts typed composition and direct UI imports", async () => {
  assert.deepEqual(
    await lint(`
      import { Button } from "@/components/ui/button";
      export function Action({ name }: { name: string }) {
        return <Button variant="outline">{name}</Button>;
      }
    `),
    [],
  );
});

test("frontend lint rejects conditional Hooks and incomplete dependencies", async () => {
  assert.ok(
    (
      await lint(`
      import { useState } from "react";
      export function Action({ enabled }: { enabled: boolean }) {
        if (enabled) useState(0);
        return null;
      }
    `)
    ).includes("react-hooks(rules-of-hooks)"),
  );
  assert.ok(
    (
      await lint(`
      import { useEffect } from "react";
      export function Action({ name }: { name: string }) {
        useEffect(() => { console.log(name); }, []);
        return null;
      }
    `)
    ).includes("react-hooks(exhaustive-deps)"),
  );
});

test("frontend lint surfaces compiler and accessible-name defects", async () => {
  assert.ok(
    (
      await lint(`
      export function Action({ item }: { item: { name: string } }) {
        item.name = "changed during render";
        return <span>{item.name}</span>;
      }
    `)
    ).includes("react(immutability)"),
  );
  assert.ok(
    (
      await lint(
        'export function Picture() { return <img src="image.png" />; }',
      )
    ).includes("jsx-a11y(alt-text)"),
  );
});

test("toolkit layers reject static, dynamic, type and re-export Tauri dependencies", async () => {
  for (const layer of ["ui", "primitives"]) {
    const filePath = `src/components/${layer}/example.tsx`;
    for (const source of [
      'import { invoke } from "@tauri-apps/api/core";',
      'import type { Event } from "@tauri-apps/api/event";',
      'export { invoke } from "@tauri-apps/api/core";',
      'export * from "@tauri-apps/api/core";',
      'export const load = () => import("@tauri-apps/api/core");',
    ]) {
      const rules = await lint(source, filePath);
      assert.ok(
        rules.includes("eslint(no-restricted-imports)"),
        `${filePath}: ${source}`,
      );
    }
  }
  assert.deepEqual(
    await lint(
      'import { invoke } from "@tauri-apps/api/core";',
      "src/hooks/use-example.ts",
    ),
    [],
  );
});

test("native boundary rules cover nested paths, type and namespace exports", () => {
  const sources = {};
  const invalid = [
    'import { type Event } from "@tauri-apps/api/event";',
    'export type { Event } from "@tauri-apps/api/event";',
    'export type * from "@tauri-apps/api/event";',
    'export * as api from "@tauri-apps/api/core";',
  ];
  for (const layer of ["ui", "primitives"]) {
    invalid.forEach((source, i) => {
      sources[`src/components/${layer}/nested/case-${i}.ts`] = source;
    });
  }
  sources["src/components/blocks/allowed.ts"] =
    'import { invoke } from "@tauri-apps/api/core";';
  const diagnostics = run(sources);
  assert.equal(diagnostics.length, 8);
  for (const diagnostic of diagnostics) {
    assert.equal(diagnostic.code, "eslint(no-restricted-imports)");
    assert.equal(diagnostic.severity, "error");
    assert.ok(!diagnostic.filename.includes("blocks/"));
  }
  assert.equal(new Set(diagnostics.map((d) => d.filename)).size, 8);
});

test("warnings fail and native suppression names remain checked", () => {
  const [warning] = run({
    "src/hooks/deps.ts":
      'import {useEffect} from "react"; export function useExample(name:string){useEffect(()=>console.log(name),[]);}',
  });
  assert.equal(warning.code, "react-hooks(exhaustive-deps)");
  assert.equal(warning.severity, "warning");
  assert.deepEqual(
    run({
      "src/components/blocks/suppressed.tsx":
        '// oxlint-disable-next-line jsx-a11y/alt-text\nexport function Image(){return <img src="x"/>;}',
    }),
    [],
  );
  const [unused] = run({
    "src/components/blocks/unused.tsx":
      "// oxlint-disable-next-line jsx-a11y/alt-text\nexport function Text(){return <span>Text</span>;}",
  });
  assert.equal(unused.severity, "warning");
  assert.match(unused.message, /Unused.*disable/i);
});

test("native table-cell compatibility preserves other redundancy checks", () => {
  const diagnostics = run({
    "src/components/blocks/grid.tsx":
      'export function Grid(){return <td role="gridcell">Value</td>;}',
    "src/components/blocks/cell.tsx":
      'export function Cell(){return <td role="cell">Value</td>;}',
  });
  assert.equal(
    diagnostics.filter((d) => d.code === "jsx-a11y(no-redundant-roles)").length,
    1,
  );
  assert.deepEqual(
    new Set(diagnostics.map((d) => d.code)),
    new Set([
      "jsx-a11y(no-redundant-roles)",
      "jsx-a11y(no-interactive-element-to-noninteractive-role)",
    ]),
  );
  assert.ok(diagnostics.every((d) => d.filename.endsWith("cell.tsx")));
});
