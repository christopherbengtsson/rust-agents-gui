import assert from "node:assert/strict";
import test from "node:test";
import { ESLint } from "eslint";

const eslint = new ESLint();
async function lint(source, filePath = "src/components/blocks/example.tsx") {
  const [result] = await eslint.lintText(source, { filePath });
  assert.equal(result.fatalErrorCount, 0, JSON.stringify(result.messages));
  return result.messages.map((message) => message.ruleId);
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
    ).includes("react-hooks/rules-of-hooks"),
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
    ).includes("react-hooks/exhaustive-deps"),
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
    ).includes("react-hooks/immutability"),
  );
  assert.ok(
    (
      await lint(
        'export function Picture() { return <img src="image.png" />; }',
      )
    ).includes("jsx-a11y/alt-text"),
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
        rules.includes("no-restricted-imports") ||
          rules.includes("no-restricted-syntax"),
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
