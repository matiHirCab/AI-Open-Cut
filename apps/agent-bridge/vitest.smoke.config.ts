import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    include: ["tests/packaged-smoke.test.ts"],
    testTimeout: 30_000,
  },
});
