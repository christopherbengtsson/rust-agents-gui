import { createHash } from "node:crypto";
import { realpathSync } from "node:fs";
import { homedir } from "node:os";
import path from "node:path";
import { root } from "./cargo.mjs";

const hash = (text, length) =>
  createHash("sha256").update(text).digest("hex").slice(0, length);

// Native metadata namespace follows the current storage schema, never a peer binary.
export function developmentEnvironment({
  env = process.env,
  checkout = root,
  home = homedir(),
  canonical = realpathSync,
} = {}) {
  return {
    ...env,
    AGENT_WORKSPACE_STATE_DIR:
      env.AGENT_WORKSPACE_STATE_DIR ??
      path.join(
        home,
        ".agent-workspace/dev",
        hash(canonical(checkout), 12),
        "native-schema-4",
      ),
  };
}
