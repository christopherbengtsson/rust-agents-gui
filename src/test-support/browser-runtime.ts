import { beforeEach, vi } from "vitest";

// These units run browser code in jsdom. p-queue otherwise detects Node's
// optional builtins and passes jsdom's EventTarget to Node's setMaxListeners.
// Match the WebView environment, where process.getBuiltinModule is absent.
vi.spyOn(process, "getBuiltinModule").mockReturnValue(undefined);

beforeEach(() => {
  vi.spyOn(process, "getBuiltinModule").mockReturnValue(undefined);
});
