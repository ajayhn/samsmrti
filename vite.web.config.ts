import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

// Review-only PWA build. Reuses the desktop app's stores/components where
// they're already platform-agnostic (see streamed-yawning-coral.md Phase 2)
// by aliasing every `../lib/tauri` / `../../lib/tauri` import -- the Tauri
// invoke()-based api object -- to webApi.ts, which implements the same
// interface via a dedicated Worker + wasm core instead. Nothing outside
// src/lib/tauri.ts's own module needs to change for this to work.
export default defineConfig({
  plugins: [react(), tailwindcss()],
  clearScreen: false,
  resolve: {
    alias: [
      {
        find: /^(\.\.\/)+lib\/tauri$/,
        replacement: new URL("./src/lib/webApi.ts", import.meta.url).pathname,
      },
    ],
  },
  worker: {
    // dbWorker.ts uses ES `import` for the wasm-pack glue module.
    format: "es",
  },
  assetsInclude: ["**/*.wasm"],
  server: {
    port: 4243,
    strictPort: true,
  },
  build: {
    outDir: "dist-web",
    rollupOptions: {
      input: "index.web.html",
    },
  },
});
