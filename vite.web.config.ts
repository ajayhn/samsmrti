import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import { VitePWA } from "vite-plugin-pwa";

// Review-only PWA build. Reuses the desktop app's stores/components where
// they're already platform-agnostic (see streamed-yawning-coral.md Phase 2)
// by aliasing every `../lib/tauri` / `../../lib/tauri` import -- the Tauri
// invoke()-based api object -- to webApi.ts, which implements the same
// interface via a dedicated Worker + wasm core instead. Nothing outside
// src/lib/tauri.ts's own module needs to change for this to work.
export default defineConfig({
  plugins: [
    react(),
    tailwindcss(),
    VitePWA({
      registerType: "autoUpdate",
      injectRegister: "auto",
      includeAssets: ["favicon.svg", "favicon.png", "icon.svg"],
      manifest: {
        name: "Samsmrti",
        short_name: "Samsmrti",
        description: "Spaced-repetition review, synced from your desktop decks.",
        start_url: "/index.web.html",
        scope: "/",
        display: "standalone",
        background_color: "#18181b",
        theme_color: "#4f46e5",
        icons: [
          { src: "/pwa-192.png", sizes: "192x192", type: "image/png" },
          { src: "/pwa-512.png", sizes: "512x512", type: "image/png" },
        ],
      },
      workbox: {
        // The wasm binary alone is a few MB (dev build); default workbox
        // cutoff is 2MB. Precached so review works offline once installed.
        maximumFileSizeToCacheInBytes: 10 * 1024 * 1024,
      },
    }),
  ],
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
