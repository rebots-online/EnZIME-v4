import { readFileSync } from "node:fs";
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Single source of truth: the canonical `version.txt` at the repo root,
// written by `scripts/update-version.sh` on every dev/build run. The version
// is never hand-edited here — this file only reads what the stamper produced.
const APP_VERSION = readFileSync(
  new URL("../../version.txt", import.meta.url),
  "utf8",
).trim();

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 41847,
    strictPort: true,
  },
  build: {
    outDir: "dist",
  },
  define: {
    __APP_VERSION__: JSON.stringify(APP_VERSION),
  },
});
