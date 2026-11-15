import { useEffect, useRef, useState, useSyncExternalStore } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import type { AppObservations } from "@/lib/app-observations";
import type { ProviderObservation } from "@/generated/contracts/types";

/** Native chooser interaction only. Validation, persistence and retry belong to Rust. */
export function useProviderSelection(
  app: AppObservations,
  owner: string | undefined,
  observation: ProviderObservation | undefined,
) {
  const [picker, setPicker] = useState<{
    scope: string;
    picking: boolean;
    error?: string;
  }>();
  const generation = useRef(0);
  const target = observation?.target.id;
  const scope = `${owner}:${target}`;
  const picking = picker?.scope === scope && picker.picking;
  const error = picker?.scope === scope ? picker.error : undefined;
  useEffect(
    () => () => {
      generation.current += 1;
      setPicker(undefined);
    },
    [owner, target],
  );
  const action = useSyncExternalStore(app.providers.subscribeActions, () =>
    owner && target && app.providers.action(owner, target)?.kind === "selection"
      ? app.providers.action(owner, target)
      : undefined,
  );
  async function locate() {
    if (
      !owner ||
      !observation ||
      picking ||
      (action && action.state !== "rejected")
    )
      return;
    const current = generation.current;
    setPicker({ scope, picking: true });
    try {
      const path = await open({
        directory: false,
        multiple: false,
        title: "Locate Codex executable",
      });
      if (generation.current === current && typeof path === "string")
        await app.providers.selectExecutable(owner, observation, path);
    } catch {
      if (generation.current === current)
        setPicker({
          scope,
          picking: false,
          error:
            "The executable chooser could not be opened. Try Locate Codex again.",
        });
    } finally {
      if (generation.current === current)
        setPicker((previous) =>
          previous?.scope === scope
            ? { ...previous, picking: false }
            : previous,
        );
    }
  }
  return {
    locate,
    picking,
    error: error ?? action?.message,
    uncertain: action?.state === "uncertain",
    reconcile: () => {
      if (owner && target && action)
        void app.providers.reconcileSelection(owner, target, action.attempt);
    },
  };
}
