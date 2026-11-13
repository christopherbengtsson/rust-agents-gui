import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface SupervisorSnapshot {
  status: {
    state: "ready";
    supervisor_pid: number;
    application_version: string;
  };
  cursor: { owner: string; revision: number };
  reset: boolean;
}

export type Observation = {
  snapshot: SupervisorSnapshot;
  desktopPid: number;
};

export type Connection =
  | { state: "checking"; observation?: Observation }
  | { state: "connected"; observation: Observation }
  | { state: "error"; error: string; observation?: Observation };

// Convert IPC failures into outcomes before the Hook commits presentation state.
async function readObservation(): Promise<
  { ok: true; observation: Observation } | { ok: false; error: string }
> {
  try {
    const [snapshot, desktopPid] = await Promise.all([
      invoke<SupervisorSnapshot>("supervisor_status"),
      invoke<number>("desktop_pid"),
    ]);
    return { ok: true, observation: { snapshot, desktopPid } };
  } catch (error) {
    return { ok: false, error: String(error) };
  }
}

// One owner supplies observations to the shell and diagnostics. Rust owns
// discovery, reconnect and recovery; these reads never submit execution actions.
export function useSupervisorConnection() {
  const [connection, setConnection] = useState<Connection>({
    state: "checking",
  });
  const mounted = useRef(false);
  const pending = useRef<symbol | null>(null);

  // Mount reads observations without an extra synchronous checking update.
  // Request identity and cleanup remain shared with explicit refresh.
  const readStatus = useCallback(async () => {
    if (!mounted.current || pending.current !== null) return;
    const request = Symbol();
    pending.current = request;
    try {
      await readObservation().then((outcome) => {
        if (!mounted.current || pending.current !== request) return;
        if (outcome.ok)
          setConnection({
            state: "connected",
            observation: outcome.observation,
          });
        else
          setConnection((last) => ({
            state: "error",
            error: outcome.error,
            observation: last.observation,
          }));
      });
    } finally {
      if (pending.current === request) pending.current = null;
    }
  }, []);

  const refreshStatus = useCallback(async () => {
    if (!mounted.current || pending.current !== null) return;
    setConnection((last) => ({
      state: "checking",
      observation: last.observation,
    }));
    await readStatus();
  }, [readStatus]);

  useEffect(() => {
    mounted.current = true;
    void readStatus();
    return () => {
      mounted.current = false;
      // Ignore this setup's late replies, including after Strict Mode re-setup.
      // This invalidates presentation ownership; it does not cancel Rust work.
      pending.current = null;
    };
  }, [readStatus]);

  return { connection, refreshStatus };
}
