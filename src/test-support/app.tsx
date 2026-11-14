import { useState } from "react";
import { createMemoryHistory } from "@tanstack/react-router";
import { App as Application } from "../App";
export function App() {
  const [history] = useState(() =>
    createMemoryHistory({ initialEntries: ["/"] }),
  );
  return <Application history={history} />;
}
