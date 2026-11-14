import { createFileRoute } from "@tanstack/react-router";
import { MainScreen } from "@/screens/main";
export const Route = createFileRoute("/repository/$repositoryId/")({
  component: MainScreen,
});
