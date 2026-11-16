import { useEffect, useState } from "react";
import { RouterProvider, type RouterHistory } from "@tanstack/react-router";
import { QueryClientProvider } from "@tanstack/react-query";
import { AppObservations } from "@/lib/app-observations";
import { createAppRouter } from "@/router";
import { installNativeRetention } from "@/test-support/native-retention";

export function App({ history }: { history?: RouterHistory }) {
  const [app] = useState(() => new AppObservations());
  const [router] = useState(() => createAppRouter(app, history));
  useEffect(() => router.options.context.navigation.start(), [router]);
  useEffect(() => {
    if (import.meta.env.VITE_NATIVE_QUALIFICATION === "1")
      return installNativeRetention(app.client);
  }, [app]);
  return (
    <QueryClientProvider client={app.client}>
      <RouterProvider router={router} />
    </QueryClientProvider>
  );
}
