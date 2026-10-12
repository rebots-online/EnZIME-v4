import { defineConfig } from "vitest/config";
export default defineConfig({
  esbuild: { jsx: "automatic" },
  test: {
    environment: "jsdom",
    include: ["src/routes/library/pack-entry.test.tsx", "src/routes/reader/reader-canvas.test.tsx", "src/components/annotation/annotation-save.test.tsx", "src/routes/reader/canvas-modalities.test.tsx", "src/routes/library/library-dyndon.test.tsx"],
    restoreMocks: true,
  },
});
