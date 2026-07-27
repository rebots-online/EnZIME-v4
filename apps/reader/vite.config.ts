import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

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
    __APP_VERSION__: JSON.stringify("1.0.52684"),
  },
});
