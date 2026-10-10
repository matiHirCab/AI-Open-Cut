import { expect, it } from "bun:test";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
const root = resolve(import.meta.dir, "..");
const workflow = readFileSync(resolve(root, ".github/workflows/motion-release-gates.yml"), "utf8");
const driver = readFileSync(resolve(root, "apps/agent-bridge/scripts/run-motion-release.ts"), "utf8");
const validate = (text: string) => {
  const parsed = Bun.YAML.parse(text) as any;
  expect(parsed.on.pull_request.branches).toEqual(["main"]);
  expect(parsed.on.push.branches).toEqual(["main"]);
  expect(parsed.env).toBeUndefined();
  const job = parsed.jobs.release;
  expect(job["runs-on"]).toBe("${{ matrix.os }}");
  expect(job["timeout-minutes"]).toBe(135);
  expect(job.strategy["fail-fast"]).toBe(false);
  expect(job.strategy.matrix.os).toEqual(["ubuntu-latest", "windows-latest", "macos-latest"]);
  expect(job.if).toBeUndefined();
  expect(job["continue-on-error"]).toBeUndefined();
  expect(job.env.OPENCUT_MOTION_REQUESTED_HEAD).toBe("${{ github.event.pull_request.head.sha || github.sha }}");
  expect(job.env.OPENCUT_MOTION_EVIDENCE_DIR).toBe("${{ github.workspace }}/target/motion-release-evidence");
  expect(Object.keys(job.env).sort()).toEqual(["OPENCUT_MOTION_EVIDENCE_DIR", "OPENCUT_MOTION_REQUESTED_HEAD"]);
  const steps = job.steps as any[];
  for (const step of steps) {
    expect(step["continue-on-error"]).toBeUndefined();
    expect(step.env).toBeUndefined();
    expect(step.run ?? "").not.toMatch(/\|\|\s*true|exit\s+0|--passWithNoTests|--update|UPDATE_GOLDEN/u);
  }
  const checkout = steps.find((step) => step.uses === "actions/checkout@v4");
  expect(checkout?.with.ref).toBe("${{ github.event.pull_request.head.sha || github.sha }}");
  expect(steps.find((step) => step.uses === "actions/setup-python@v5")?.with["python-version"]).toBe("3.11");
  expect(steps.find((step) => step.uses === "moonrepo/setup-toolchain@v0")?.with).toEqual({ "auto-install": true, "auto-setup": true, "moon-version": "2.3.3" });
  for (const command of [
    "python scripts/test_setup_platform_renderer.py", "python scripts/setup-platform-renderer.py", "bun install --frozen-lockfile",
    "bun --config=bunfig.toml --no-env-file test scripts/motion-release-policy.test.ts scripts/motion-release-process.test.ts scripts/motion-release-preflight.test.ts",
    "bun x --no-install vitest run --config vitest.unit.config.ts tests/motion-release-cases.test.ts tests/motion-release-report.test.ts tests/motion-mcp-report.test.ts tests/motion-native-cache-trace.test.ts",
    "bun run apps/agent-bridge/scripts/run-motion-release.ts",
  ]) {
    const matches = steps.filter((step) => step.run === command);
    expect(matches).toHaveLength(1);
    expect(matches[0].if).toBeUndefined();
  }
  expect(steps.find((step) => step.name === "Windows FFmpeg")?.run).toBe("choco install ffmpeg --yes --no-progress --version=7.1.1");
  expect(steps.find((step) => step.name === "macOS FFmpeg")?.run).toContain("brew install ffmpeg@7");
  expect(steps.find((step) => step.name === "Linux FFmpeg")?.run).toContain("apt-get install -y ffmpeg");
  const upload = steps.find((step) => step.uses === "actions/upload-artifact@v4");
  expect(upload?.with.path).toBe("target/motion-release-evidence");
  expect(upload?.with["if-no-files-found"]).toBe("error");
};
it("requires actual complete scene release on all three platforms", () => validate(workflow));
for (const [before, after] of [
  ["ubuntu-latest, windows-latest, macos-latest", "ubuntu-latest, macos-latest"],
  ["fail-fast: false", "fail-fast: true"], ["timeout-minutes: 135", "timeout-minutes: 1"],
  ['python-version: "3.11"', 'python-version: "3.12"'], ["moon-version: 2.3.3", "moon-version: 2.0.0"],
  ["--version=7.1.1", "--version=6.0.0"], ["brew install ffmpeg@7", "brew install ffmpeg"],
  ["run: bun run apps/agent-bridge/scripts/run-motion-release.ts", "run: bun run apps/agent-bridge/scripts/run-motion-release.ts || true"],
  ["run: bun run apps/agent-bridge/scripts/run-motion-release.ts", "if: false\n        run: bun run apps/agent-bridge/scripts/run-motion-release.ts"],
  ["run: bun run apps/agent-bridge/scripts/run-motion-release.ts", "continue-on-error: true\n        run: bun run apps/agent-bridge/scripts/run-motion-release.ts"],
  ["run-motion-release.ts", "run-platform-renderer.ts"], ["if-no-files-found: error", "if-no-files-found: ignore"],
  ["run: bun run apps/agent-bridge/scripts/run-motion-release.ts", "run: bun run apps/agent-bridge/scripts/run-motion-release.ts --update"],
  ["OPENCUT_MOTION_EVIDENCE_DIR: ${{ github.workspace }}/target/motion-release-evidence", 'OPENCUT_MOTION_EVIDENCE_DIR: ${{ github.workspace }}/target/motion-release-evidence\n      OPENCUT_UPDATE_GOLDENS: "1"'],
  ["run: bun run apps/agent-bridge/scripts/run-motion-release.ts", 'env: { OPENCUT_UPDATE_GOLDENS: "1" }\n        run: bun run apps/agent-bridge/scripts/run-motion-release.ts'],
] as const) {
  it(`rejects release workflow weakening ${after}`, () => {
    expect(workflow).toContain(before);
    expect(() => validate(workflow.replace(before, after))).toThrow();
  });
}
const validateDriver = (text: string) => {
  for (const required of ["--no-default-features", "default_native_release_scene", "--ignored", "--exact", "OPENCUT_MOTION_RELEASE_REQUIRED", '"raster-cache-test-hooks"', "OPENCUT_REFERENCE_REQUIRED", "OPENCUT_TEST_HEADLESS_PATH", "requireMotionCoreReport", "requireMotionMcpReport", "requireExactRustCase", "requireReleaseInventory", "owned_release_process.rs", '"--reporter=json"', "accepted.json"]) {
    expect(text).toContain(required);
  }
  const defaultProof = text.lastIndexOf("requireMotionCoreReport(");
  expect(defaultProof).toBeGreaterThan(0);
  expect(defaultProof).toBeLessThan(text.indexOf('"raster-cache-test-hooks"'));
  expect(text).not.toContain("native_media_trace");
  expect(text).not.toMatch(/OPENCUT_(?:REFERENCE|MOTION_RELEASE)_REQUIRED:\s*"0"|OPENCUT_UPDATE_GOLDENS\s*:\s*"1"|--update|passWithNoTests/u);
};
it("requires distinct default and instrumented binaries with actual guarded reports", () => {
  validateDriver(driver);
  const pins = readFileSync(resolve(root, ".prototools"), "utf8");
  expect(pins).toContain('bun  = "1.4.0"');
  expect(pins).toContain('rust = "1.97.0"');
});
for (const required of ["--no-default-features", "raster-cache-test-hooks", "requireMotionCoreReport", "requireMotionMcpReport", "--reporter=json", "owned_release_process.rs", "accepted.json"]) {
  it(`rejects omitted required driver proof ${required}`, () => {
    expect(() => validateDriver(driver.replaceAll(required, "removed"))).toThrow();
  });
}

// Verified predecessor inputs remain frozen; no golden or protected-gate rewrite.
const FROZEN = [
  [".github/workflows/bun-ci.yml", "99578cf3bd44e1fc0a8ae3efa53af26c6a4d90d08f0615f26c80c4350edf9815"],
  [".github/workflows/windows-renderer-startup-diagnostic.yml", "c5e1738fb9c277e349bcc1ee4af59a9c77678e57b43bc24fd07d1c09fe6e63ba"],
  [".github/workflows/motion-platform-renderer.yml", "48868ce758bb0ec04d9bb57eac6f31b8536afa818a0ffa41e0afe810ac315959"],
  [".prototools", "4930d18e9affe3d9f28af39c04d4486ea4d91aa2dd4c22293196474fbf2cc8a6"],
  ["contracts/complete-reference-scene-v1.json", "41deb91f35ccbbf379e7d3ffd0e4d402077fbaa37bee3a9253363fe8221e9cc0"],
  ["crates/editor-core/tests/fixtures/render-golden/CURRENT", "4b948fda9e248c46a4544e4fbe11708544d6003c5ed11635fbfbb4ab3f1fc70b"],
  ["docs/verification/complete-reference-scene/frame650.png", "d745cbd7f717af55b51d081a0724a83b9671c763aa58a40983ddeafb2a086a85"],
  ["docs/verification/complete-reference-scene/reference.mp4", "e5915d227a6d860fecd09d06f30186478ae18b8b7868a1e233767cc5ca1f1b6a"],
  ["docs/verification/complete-reference-scene/source-native-evidence.json", "a6e0891b075c6d5aae4ec84e04be59324b1b3fecdb211d4e73f8c761a99c31bb"],
  ["docs/verification/complete-reference-scene/compiled-native-evidence.json", "ceb7f2c990e7ad38aa3722e707525f5efb145e860697b3b38ef7cc954f1ffd47"],
] as const;
for (const [name, hash] of FROZEN) {
  it(`retains verified predecessor ${name} and rejects altered bytes`, () => {
    const original = readFileSync(resolve(root, name));
    const digest = (bytes: Uint8Array) => new Bun.CryptoHasher("sha256").update(bytes).digest("hex");
    expect(digest(original)).toBe(hash);
    const changed = Buffer.from(original);
    changed[0] = (changed[0] ?? 0) ^ 1;
    expect(digest(changed)).not.toBe(hash);
  });
}
