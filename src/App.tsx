import { useEffect, useState } from "react";
import { RouterProvider, type RouterHistory } from "@tanstack/react-router";
import { QueryClientProvider } from "@tanstack/react-query";
import { AppObservations } from "@/lib/app-observations";
import { createAppRouter } from "@/router";

export function App({ history }: { history?: RouterHistory }) {
  const [app] = useState(() => new AppObservations());
  const [router] = useState(() => createAppRouter(app, history));
  useEffect(() => router.options.context.navigation.start(), [router]);
  return (
    <QueryClientProvider client={app.client}>
      <RouterProvider router={router} />
    </QueryClientProvider>
  );
}
