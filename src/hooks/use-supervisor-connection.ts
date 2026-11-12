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

// One owner supplies observations to the shell and diagnostics. Rust owns
// discovery, reconnect and recovery; these reads never submit execution actions.
export function useSupervisorConnection() {
  const [connection, setConnection] = useState<Connection>({
    state: "checking",
  });
  const mounted = useRef(false);
  const pending = useRef<symbol | null>(null);

  // Stable identity lets the mount Effect depend on the same refresh path as
  // the button without starting a request on each render.
  const refreshStatus = useCallback(async () => {
    if (!mounted.current || pending.current !== null) return;
    const request = Symbol();
    pending.current = request;
    setConnection((last) => ({
      state: "checking",
      observation: last.observation,
    }));
    try {
      const [snapshot, desktopPid] = await Promise.all([
        invoke<SupervisorSnapshot>("supervisor_status"),
        invoke<number>("desktop_pid"),
      ]);
      if (mounted.current && pending.current === request)
        setConnection({
          state: "connected",
          observation: { snapshot, desktopPid },
        });
    } catch (error) {
      if (mounted.current && pending.current === request)
        setConnection((last) => ({
          state: "error",
          error: String(error),
          observation: last.observation,
        }));
    } finally {
      if (pending.current === request) pending.current = null;
    }
  }, []);

  useEffect(() => {
    mounted.current = true;
    void refreshStatus();
    return () => {
      mounted.current = false;
      // Ignore this setup's late replies, including after Strict Mode re-setup.
      // This invalidates presentation ownership; it does not cancel Rust work.
      pending.current = null;
    };
  }, [refreshStatus]);

  return { connection, refreshStatus };
}
