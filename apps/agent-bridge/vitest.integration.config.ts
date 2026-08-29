import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    include: ["tests/smoke.test.ts"],
    testTimeout: 60_000,
  },
});
