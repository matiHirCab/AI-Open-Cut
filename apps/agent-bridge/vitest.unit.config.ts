import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    exclude: [
      "tests/smoke.test.ts",
      "tests/packaged-smoke.test.ts",
      "tests/tts-real.test.ts",
    ],
    include: ["tests/**/*.test.ts"],
  },
});
