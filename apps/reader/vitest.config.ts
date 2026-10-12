import { defineConfig } from "vitest/config";
export default defineConfig({
  esbuild: { jsx: "automatic" },
  test: {
    environment: "jsdom",
    include: ["src/routes/reader/reader-canvas.test.tsx", "src/components/annotation/annotation-save.test.tsx"],
    restoreMocks: true,
  },
});
