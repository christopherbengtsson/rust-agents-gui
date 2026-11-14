import { createFileRoute, Outlet, redirect } from "@tanstack/react-router";
import { SettingsLayout } from "@/components/blocks/settings-layout";
import { useAppRuntime } from "@/app-runtime";
export const Route = createFileRoute("/settings")({
  beforeLoad: ({ location }) => {
    if (location.pathname === "/settings")
      throw redirect({ to: "/settings/general", replace: true });
  },
  component: SettingsScreen,
});

function SettingsScreen() {
  const { navigation } = useAppRuntime();
  return (
    <SettingsLayout onBack={navigation.returnFromSettings}>
      <Outlet />
    </SettingsLayout>
  );
}
