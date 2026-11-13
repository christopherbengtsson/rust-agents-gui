import { fileURLToPath, URL } from "node:url";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vitest/config";

export default defineConfig({
  plugins: [
    react({
      compiler: {
        compilationMode: "infer",
        panicThreshold: "none",
        target: "19",
        reportDiagnostics: true,
      },
    }),
    tailwindcss(),
  ],
  clearScreen: false,
  server: {
    host: "127.0.0.1",
    port: 1420,
    strictPort: true,
    // Tauri owns Rust rebuilds; sidecar/target writes must not reload the UI.
    watch: { ignored: ["**/src-tauri/**", "**/crates/**", "**/target/**"] },
  },
  build: {
    // Tailwind 4 requires Safari 16.4; the bundle declares macOS 13.3.
    target: "safari16.4",
    minify: !process.env.TAURI_ENV_DEBUG,
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
  },
  resolve: { alias: { "@": fileURLToPath(new URL("./src", import.meta.url)) } },
  test: { environment: "jsdom", include: ["src/**/*.test.tsx"] },
});
