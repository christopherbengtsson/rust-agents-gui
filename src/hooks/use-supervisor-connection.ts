import { completeOperation } from "@/lib/complete-operation";
import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

import {
  decodeDesktopPid,
  decodeRequestFailure,
  decodeSnapshot,
  type SupervisorSnapshot,
} from "@/lib/observation-contracts";

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
    const [snapshot, desktopPid] = await Promise.allSettled([
      invoke<unknown>("supervisor_status").then(decodeSnapshot),
      invoke<unknown>("desktop_pid").then(decodeDesktopPid),
    ]);
    if (snapshot.status === "rejected") throw snapshot.reason;
    if (desktopPid.status === "rejected") throw desktopPid.reason;
    return {
      ok: true,
      observation: { snapshot: snapshot.value, desktopPid: desktopPid.value },
    };
  } catch (error) {
    return {
      ok: false,
      error:
        typeof error === "string" ? error : decodeRequestFailure(error).message,
    };
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
  const generation = useRef(0);
  const queued = useRef(false);

  // Mount reads observations without an extra synchronous checking update.
  // Request identity and cleanup remain shared with explicit refresh.
  const readStatus = useCallback(async function read() {
    if (!mounted.current) return;
    if (pending.current !== null) {
      queued.current = true;
      return;
    }
    const epoch = generation.current;
    const request = Symbol();
    pending.current = request;
    return completeOperation(
      async () => {
        await readObservation().then((outcome) => {
          if (
            !mounted.current ||
            generation.current !== epoch ||
            pending.current !== request
          )
            return;
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
      },
      undefined,
      () => {
        if (pending.current === request) pending.current = null;
        if (mounted.current && queued.current) {
          queued.current = false;
          void read();
        }
      },
    );
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
      // Invalidate presentation, retaining the physical request until settlement.
      generation.current += 1;
      queued.current = false;
    };
  }, [readStatus]);

  return { connection, refreshStatus };
}
