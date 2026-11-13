import { createHash } from "node:crypto";
import { realpathSync } from "node:fs";
import { homedir } from "node:os";
import path from "node:path";
import { root, runCommand } from "./cargo.mjs";

const hash = (text, length) =>
  createHash("sha256").update(text).digest("hex").slice(0, length);

// Called only after successful preparation. Resource/CI staging never executes
// a supervisor; only this freshly Cargo-reported binary supports the metadata flag.
export function developmentEnvironment(
  provenance,
  {
    env = process.env,
    checkout = root,
    home = homedir(),
    canonical = realpathSync,
    run = runCommand,
  } = {},
) {
  if (
    !path.isAbsolute(provenance.executable) ||
    typeof provenance.applicationVersion !== "string"
  )
    throw new Error("Fresh supervisor provenance is incomplete");
  const protocol = run(provenance.executable, ["--protocol-version"]).trim();
  if (!/^[1-9][0-9]{0,8}$/.test(protocol))
    throw new Error("Invalid supervisor protocol metadata");
  return {
    ...env,
    AGENT_WORKSPACE_SUPERVISOR: provenance.executable,
    AGENT_WORKSPACE_STATE_DIR:
      env.AGENT_WORKSPACE_STATE_DIR ??
      path.join(
        home,
        ".agent-workspace/dev",
        hash(canonical(checkout), 12),
        `p${protocol}-${hash(provenance.applicationVersion, 8)}`,
      ),
  };
}
