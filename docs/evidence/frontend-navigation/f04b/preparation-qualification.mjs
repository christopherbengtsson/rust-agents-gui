import assert from "node:assert/strict";
import { readFileSync, readdirSync, writeFileSync, unlinkSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
const directory = "src/generated/contracts/";
const original = Object.fromEntries(
  readdirSync(directory).map((name) => [name, readFileSync(directory + name)]),
);
function run(command, args, expected = 0) {
  const result = spawnSync(command, args, { encoding: "utf8" });
  console.log(
    `$ ${command} ${args.join(" ")}\n${result.stdout}${result.stderr}\nexit ${result.status}`,
  );
  assert.equal(result.status, expected);
  return result;
}
try {
  run("node", ["scripts/prepare-generated.mjs", "--write"]);
  for (const [name, bytes] of Object.entries(original))
    assert.deepEqual(readFileSync(directory + name), bytes);
  run("node", ["scripts/prepare-generated.mjs", "--write"]);
  for (const [name, bytes] of Object.entries(original))
    assert.deepEqual(readFileSync(directory + name), bytes);
  console.log("Two regenerations match every original artifact byte");
  const stale = Buffer.concat([
    original["types.ts"],
    Buffer.from("// deliberate drift\n"),
  ]);
  writeFileSync(directory + "types.ts", stale);
  const check = run("corepack", ["pnpm", "check"], 1);
  assert.match(check.stdout + check.stderr, /stale: types.ts/);
  assert.doesNotMatch(check.stdout, /Gate:/);
  assert.deepEqual(readFileSync(directory + "types.ts"), stale);
  writeFileSync(directory + "types.ts", original["types.ts"]);
  unlinkSync(directory + "validators.js");
  const missing = run("corepack", ["pnpm", "frontend:build"], 1);
  assert.match(missing.stdout + missing.stderr, /missing: validators.js/);
  assert.equal(readdirSync(directory).includes("validators.js"), false);
  writeFileSync(directory + "validators.js", original["validators.js"]);
  writeFileSync(directory + "unexpected.txt", "deliberate unexpected output");
  const unexpected = run(
    "node",
    ["scripts/prepare-generated.mjs", "--check"],
    1,
  );
  assert.match(
    unexpected.stdout + unexpected.stderr,
    /unexpected: unexpected.txt/,
  );
  assert.equal(
    readFileSync(directory + "unexpected.txt", "utf8"),
    "deliberate unexpected output",
  );
  unlinkSync(directory + "unexpected.txt");
  run("node", ["scripts/prepare-generated.mjs", "--check"]);
  console.log(
    JSON.stringify(
      Object.fromEntries(
        Object.entries(original).map(([name, bytes]) => [
          name,
          createHash("sha256").update(bytes).digest("hex"),
        ]),
      ),
    ),
  );
} finally {
  for (const [name, bytes] of Object.entries(original))
    writeFileSync(directory + name, bytes);
  if (readdirSync(directory).includes("unexpected.txt"))
    unlinkSync(directory + "unexpected.txt");
}
