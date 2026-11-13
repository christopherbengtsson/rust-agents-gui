import { useCallback, useLayoutEffect, useRef, useState } from "react";

export type SelectionEligibility = {
  capture: () => number;
  allows: (generation: number) => boolean;
  navigate: () => void;
};

// Owns screen/focus intent and permission to select after an asynchronous action.
// Domain hooks remain the only owners of repository/workspace selection.
export function useAppNavigation() {
  const [screen, setScreen] = useState<"workbench" | "settings">("workbench");
  const [visit, setVisit] = useState(0);
  const currentScreen = useRef(screen);
  const currentVisit = useRef(0);
  const generation = useRef(0);
  const focusIntent = useRef(false);
  const settingsHeading = useRef<HTMLHeadingElement>(null);
  const settingsTrigger = useRef<HTMLButtonElement>(null);
  const workspaceMain = useRef<HTMLElement>(null);
  const [selection] = useState<SelectionEligibility>(() => ({
    capture: () => generation.current,
    allows: (token) =>
      currentScreen.current === "workbench" && generation.current === token,
    navigate: () => {
      generation.current++;
    },
  }));
  // A portal from an earlier visit must not restore focus even after Back.
  const canRestoreFocus = useCallback(
    () =>
      currentScreen.current === "workbench" && currentVisit.current === visit,
    [visit],
  );

  useLayoutEffect(() => {
    if (!focusIntent.current) return;
    focusIntent.current = false;
    const target =
      screen === "settings"
        ? settingsHeading.current
        : settingsTrigger.current?.isConnected
          ? settingsTrigger.current
          : workspaceMain.current;
    target?.focus({ preventScroll: true });
  }, [screen]);

  function openSettings() {
    if (currentScreen.current === "settings") return;
    selection.navigate();
    currentScreen.current = "settings";
    focusIntent.current = true;
    currentVisit.current++;
    setVisit(currentVisit.current);
    setScreen("settings");
  }
  function back() {
    if (currentScreen.current === "workbench") return;
    selection.navigate();
    currentScreen.current = "workbench";
    focusIntent.current = true;
    setScreen("workbench");
  }
  return {
    screen,
    visit,
    selection,
    openSettings,
    back,
    settingsHeading,
    settingsTrigger,
    workspaceMain,
    canRestoreFocus,
  };
}
