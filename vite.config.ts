import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

// @ts-expect-error type error without @types/node package
import process from "node:process";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig(() => ({
  plugins: [vue()],

  // Prevent Vite from obscuring Rust errors.
  clearScreen: false,

  server: {
    // Must match devUrl in tauri.conf.json.
    port: 1420,
    strictPort: true,

    host: host || false,

    hmr: host
      ? {
        protocol: "ws",
        host,
        port: 1421,
      }
      : undefined,

    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
}));