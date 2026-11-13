import { useCallback, useEffect, useRef, useState } from "react";
import { completeOperation } from "@/lib/complete-operation";
import {
  RUNTIME_PENDING,
  readRuntimePending,
  runtimeError,
  runtimeRequest,
  terminal,
  type RuntimeError,
  type RuntimeMutation,
  type RuntimeReply,
  type RuntimeAction,
} from "@/lib/runtimes";

// One App-owned observation/action lifetime. Settings owns only discardable drafts.
export function useRuntimes(owner: string | undefined, visible: boolean) {
  const [observation, setObservation] = useState<RuntimeReply>();
  const [pending, setPending] = useState(readRuntimePending);
  const [error, setError] = useState<RuntimeError>();
  const [actionError, setActionError] = useState<RuntimeError>();
  const [preferenceError, setPreferenceError] = useState<string>();
  const [loading, setLoading] = useState(false);
  const [reading, setReading] = useState(false);
  const [recovered, setRecovered] = useState<RuntimeAction>();
  const intent = useRef(pending);
  const epoch = useRef(0);
  const mounted = useRef(false);
  const lane = useRef(false);
  const submitting = useRef(false);
  const queued = useRef(false);
  const latestRead = useRef<(() => Promise<void>) | undefined>(undefined);
  const [sending, setSending] = useState(false);
  const forget = useCallback(() => {
    intent.current = null;
    setPending(null);
    try {
      localStorage.removeItem(RUNTIME_PENDING);
    } catch {
      setPreferenceError(
        "The completed setup action could not be cleared from desktop preferences. Reopening will read its status.",
      );
    }
  }, []);
  const read = useCallback(async () => {
    if (!owner || !mounted.current) return;
    if (lane.current || submitting.current) {
      queued.current = true;
      return;
    }
    lane.current = true;
    setReading(true);
    const generation = epoch.current;
    const active = () => mounted.current && epoch.current === generation;
    return completeOperation(
      async () => {
        const retained = intent.current;
        const target = { owner, runtime: "codex" as const };
        const request = retained
          ? {
              operation: "action" as const,
              target,
              action_id: retained.action_id,
            }
          : { operation: "observe" as const, target };
        const reply = await runtimeRequest(request, retained);
        if (!active()) return;
        setObservation(reply);
        setError(undefined);
        if (reply.lookup?.complete && reply.lookup.action)
          setRecovered(reply.lookup.action);
        if (
          retained &&
          reply.configuration &&
          reply.lookup?.complete &&
          (!reply.lookup.action || terminal(reply.lookup.action.phase)) &&
          !reply.busy &&
          !reply.blocked
        )
          forget();
      },
      (failure) => {
        if (active()) setError(runtimeError(failure));
      },
      () => {
        lane.current = false;
        if (mounted.current) setReading(false);
        if (mounted.current) setLoading(false);
        if (queued.current && mounted.current) {
          queued.current = false;
          queueMicrotask(() => void latestRead.current?.());
        }
      },
    );
  }, [owner, forget]);
  const refresh = useCallback(async () => {
    if (!owner) return;
    setLoading(true);
    await read();
  }, [owner, read]);
  useEffect(() => {
    mounted.current = true;
    latestRead.current = read;
    void read();
    return () => {
      mounted.current = false;
      epoch.current += 1;
    };
  }, [read]);
  useEffect(() => {
    if (
      !owner ||
      error ||
      (!visible && !pending && !observation?.busy) ||
      submitting.current
    )
      return;
    const timer = setInterval(
      () => void read(),
      pending || observation?.busy ? 250 : 2000,
    );
    return () => clearInterval(timer);
  }, [owner, visible, pending, observation?.busy, error, read]);
  const submit = useCallback(
    async (operation: "save" | "check", executable: string | null = null) => {
      if (
        !owner ||
        !observation?.configuration ||
        observation.target.owner !== owner ||
        observation.busy ||
        observation.blocked ||
        error ||
        intent.current ||
        lane.current ||
        submitting.current
      )
        return;
      const request: RuntimeMutation =
        operation === "save"
          ? {
              operation,
              target: { owner, runtime: "codex" },
              action_id: crypto.randomUUID(),
              revision: observation.configuration.revision,
              executable,
            }
          : {
              operation,
              target: { owner, runtime: "codex" },
              action_id: crypto.randomUUID(),
              revision: observation.configuration.revision,
            };
      try {
        localStorage.setItem(RUNTIME_PENDING, JSON.stringify(request));
      } catch {
        setPreferenceError(
          "The desktop could not retain setup recovery identity. No action was submitted.",
        );
        return;
      }
      intent.current = request;
      setPending(request);
      submitting.current = true;
      setSending(true);
      setActionError(undefined);
      setRecovered(undefined);
      const generation = epoch.current;
      const active = () => mounted.current && epoch.current === generation;
      await completeOperation(
        async () => {
          const reply = await runtimeRequest(request);
          if (!active()) return;
          setObservation(reply);
          setError(undefined);
          // Settlement always comes from an explicit status read, including a lost
          // admission reply or an owner change. Never resubmit the mutation.
        },
        (failure) => {
          if (!active()) return;
          const e = runtimeError(failure);
          if (["invalid_request", "stale_target", "busy"].includes(e.code)) {
            setActionError(e);
            forget();
          } else setError(e);
        },
        () => {
          submitting.current = false;
          if (mounted.current) setSending(false);
          if (queued.current && mounted.current) {
            queued.current = false;
            queueMicrotask(() => void latestRead.current?.());
          }
        },
      );
      if (active()) await read();
    },
    [owner, observation, error, forget, read],
  );
  return {
    observation,
    pending,
    error,
    actionError,
    preferenceError,
    loading,
    sending,
    reading,
    recovered,
    stale:
      !!observation &&
      (!owner || observation.target.owner !== owner || !!error),
    canAct:
      !!owner &&
      observation?.target.owner === owner &&
      !!observation.configuration &&
      !observation.busy &&
      !observation.blocked &&
      !error &&
      !pending &&
      !sending &&
      !reading,
    refresh,
    save: (executable: string | null) => submit("save", executable),
    check: () => submit("check"),
  };
}
export type Runtimes = ReturnType<typeof useRuntimes>;
