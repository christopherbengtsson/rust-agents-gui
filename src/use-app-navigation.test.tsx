import { Activity } from "react";
import {
  act,
  cleanup,
  fireEvent,
  render,
  renderHook,
  screen,
} from "@testing-library/react";
import { afterEach, expect, test } from "vitest";
import { useAppNavigation } from "./use-app-navigation";

afterEach(cleanup);

function NavigationWithMissingTrigger({
  showTrigger,
}: {
  showTrigger: boolean;
}) {
  const {
    screen: destination,
    workspaceMain,
    settingsTrigger,
    settingsHeading,
    openSettings,
    back,
  } = useAppNavigation();
  return (
    <>
      <Activity mode={destination === "workbench" ? "visible" : "hidden"}>
        <main ref={workspaceMain} tabIndex={-1}>
          {showTrigger && (
            <button ref={settingsTrigger} onClick={openSettings}>
              Settings
            </button>
          )}
        </main>
      </Activity>
      {destination === "settings" && (
        <>
          <h1 ref={settingsHeading} tabIndex={-1}>
            Settings
          </h1>
          <button onClick={back}>Back</button>
        </>
      )}
    </>
  );
}

test("Back focuses the revealed main when the original Settings trigger no longer exists", () => {
  const { rerender } = render(<NavigationWithMissingTrigger showTrigger />);
  fireEvent.click(screen.getByRole("button", { name: "Settings" }));
  rerender(<NavigationWithMissingTrigger showTrigger={false} />);
  fireEvent.click(screen.getByRole("button", { name: "Back" }));
  expect(document.activeElement).toBe(screen.getByRole("main"));
});

test("an earlier portal's focus permission stays invalid after returning to the workbench", () => {
  const { result } = renderHook(useAppNavigation);
  const oldPortalFocus = result.current.canRestoreFocus;
  expect(oldPortalFocus()).toBe(true);
  act(() => result.current.openSettings());
  expect(oldPortalFocus()).toBe(false);
  act(() => result.current.back());
  expect(oldPortalFocus()).toBe(false);
  expect(result.current.canRestoreFocus()).toBe(true);
});
