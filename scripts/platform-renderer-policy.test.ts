import { expect, it } from "bun:test";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(import.meta.dir, "..");
const source = readFileSync(resolve(root, ".github/workflows/motion-platform-renderer.yml"), "utf8");
const validate = (text: string) => {
  const workflow = Bun.YAML.parse(text) as any;
  expect(workflow.on.pull_request.branches).toEqual(["main"]);
  expect(workflow.on.push.branches).toEqual(["main"]);
  const job = workflow.jobs.native;
  expect(job["runs-on"]).toBe("${{ matrix.os }}");
  expect(job["timeout-minutes"]).toBe(135);
  expect(job.strategy["fail-fast"]).toBe(false);
  expect(job.strategy.matrix.os).toEqual(["ubuntu-latest", "windows-latest", "macos-latest"]);
  expect(job.if).toBeUndefined();
  expect(job["continue-on-error"]).toBeUndefined();
  const steps = job.steps as any[];
  for (const step of steps) {
    expect(step["continue-on-error"]).toBeUndefined();
    if (step.run) {
      expect(step.run).not.toMatch(/\|\|\s*true|exit\s+0|OPENCUT_PLATFORM_REQUIRED\s*=\s*0|--passWithNoTests/u);
    }
  }
  const python = steps.find((step) => step.uses === "actions/setup-python@v5");
  expect(python?.with["python-version"]).toBe("3.11");
  const toolchain = steps.find((step) => step.uses === "moonrepo/setup-toolchain@v0");
  expect(toolchain?.with).toEqual({ "auto-install": true, "auto-setup": true, "moon-version": "2.3.3" });
  for (const command of [
    "python scripts/setup-platform-renderer.py",
    "python scripts/test_setup_platform_renderer.py",
    "bun install --frozen-lockfile",
    "bun --config=bunfig.toml --no-env-file test scripts/platform-renderer-policy.test.ts",
    "bun x --no-install vitest run --config vitest.unit.config.ts tests/package-runtime.test.ts",
    "bun run apps/agent-bridge/scripts/run-platform-renderer.ts",
  ]) {
    const matches = steps.filter((step) => step.run === command);
    expect(matches).toHaveLength(1);
    expect(matches[0].if).toBeUndefined();
  }
  expect(steps.find((step) => step.name === "Windows FFmpeg")?.run).toBe("choco install ffmpeg --yes --no-progress --version=7.1.1");
  expect(steps.find((step) => step.name === "macOS FFmpeg")?.run).toContain("brew install ffmpeg@7");
  const upload = steps.find((step) => step.uses === "actions/upload-artifact@v4");
  expect(upload?.with["if-no-files-found"]).toBe("error");
};

it("requires actual default renderer on all three platforms", () => validate(source));
for (const [original, replacement] of [
  ["ubuntu-latest, windows-latest, macos-latest", "ubuntu-latest, macos-latest"],
  ["timeout-minutes: 135", "timeout-minutes: 1"],
  ['python-version: "3.11"', 'python-version: "3.12"'],
  ["moon-version: 2.3.3", "moon-version: 2.0.0"],
  ["fail-fast: false", "fail-fast: true"],
  ["run: bun run apps/agent-bridge/scripts/run-platform-renderer.ts", "run: bun run apps/agent-bridge/scripts/run-platform-renderer.ts || true"],
  ["run: bun run apps/agent-bridge/scripts/run-platform-renderer.ts", "if: false\n        run: bun run apps/agent-bridge/scripts/run-platform-renderer.ts"],
  ["run: bun run apps/agent-bridge/scripts/run-platform-renderer.ts", "continue-on-error: true\n        run: bun run apps/agent-bridge/scripts/run-platform-renderer.ts"],
  ["run-platform-renderer.ts", "run-packaged-smoke.ts"],
  ["if-no-files-found: error", "if-no-files-found: ignore"],
] as const) {
  it(`rejects weakened platform execution: ${replacement}`, () => {
    expect(source).toContain(original);
    expect(() => validate(source.replace(original, replacement))).toThrow();
  });
}

const validateDriver = (driver: string) => {
  expect(driver).toContain('"--release", "-p", "opencut-headless"');
  expect(driver).toContain('OPENCUT_PLATFORM_REQUIRED: "1"');
  expect(driver).toContain('"target/release"');
  expect(driver).toContain("requirePlatformNativeReport(nativeReport)");
  expect(driver).toContain('"--reporter=json"');
  expect(driver).not.toContain("OPENCUT_REFERENCE_REQUIRED");
  expect(driver).not.toContain("OPENCUT_PREVIEW_TEST_HEADLESS");
};
const driverSource = readFileSync(resolve(root, "apps/agent-bridge/scripts/run-platform-renderer.ts"), "utf8");
it("driver uses repository default binaries and mandatory native mode", () => {
  validateDriver(driverSource);
  const pins = readFileSync(resolve(root, ".prototools"), "utf8");
  expect(pins).toContain('bun  = "1.4.0"');
  expect(pins).toContain('rust = "1.97.0"');
});

for (const replacement of ["OPENCUT_PREVIEW_TEST_HEADLESS", "OPENCUT_REFERENCE_REQUIRED"]) {
  it(`rejects private/optional driver substitution: ${replacement}`, () => {
    expect(() => validateDriver(driverSource.replace("OPENCUT_PLATFORM_SOURCE_HEADLESS", replacement))).toThrow();
  });
}
