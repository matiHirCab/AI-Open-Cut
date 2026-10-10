import { afterAll, expect, it } from "bun:test";
import { copyFileSync, existsSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

const repository = resolve(import.meta.dir, "..");
const root = mkdtempSync(join(tmpdir(), "opencut-release-preflight-"));
afterAll(() => rmSync(root, { force: true, recursive: true }));
const fonts = join(root, "fonts");
mkdirSync(fonts);
for (const name of ["DejaVuSans.ttf", "DejaVuSans-Bold.ttf", "DejaVuSans-Oblique.ttf", "DejaVuSans-BoldOblique.ttf"]) {
  copyFileSync(join(repository, "crates/editor-core/resources/fonts", name), join(fonts, name));
}
const environment = () => ({
  ...process.env,
  OPENCUT_FFMPEG_PATH: process.env.OPENCUT_FFMPEG_PATH ?? Bun.which("ffmpeg") ?? "",
  OPENCUT_FFPROBE_PATH: process.env.OPENCUT_FFPROBE_PATH ?? Bun.which("ffprobe") ?? "",
  OPENCUT_TEST_PYTHON: process.env.OPENCUT_TEST_PYTHON ?? Bun.which("python") ?? "",
  OPENCUT_TEST_FONT_PATH: join(fonts, "DejaVuSans.ttf"),
  OPENCUT_MOTION_EVIDENCE_DIR: join(root, "evidence"),
});
const refuse = (env: NodeJS.ProcessEnv, message: string) => {
  const result = Bun.spawnSync([process.execPath, "run", "apps/agent-bridge/scripts/run-motion-release.ts"], { cwd: repository, env, timeout: 10_000 });
  expect(result.exitCode).not.toBe(0);
  expect(result.stderr.toString()).toContain(message);
  expect(existsSync(join(root, "evidence"))).toBe(false);
};
for (const key of ["OPENCUT_FFMPEG_PATH", "OPENCUT_FFPROBE_PATH", "OPENCUT_TEST_FONT_PATH", "OPENCUT_TEST_PYTHON"]) {
  it(`refuses absent configured ${key} before any native/evidence output`, () => {
    const env: NodeJS.ProcessEnv = environment();
    delete env[key];
    refuse(env, key);
  });
}
it("refuses relative dependency and evidence paths", () => {
  refuse({ ...environment(), OPENCUT_FFMPEG_PATH: "ffmpeg" }, "OPENCUT_FFMPEG_PATH");
  refuse({ ...environment(), OPENCUT_MOTION_EVIDENCE_DIR: "relative-evidence" }, "absolute release evidence");
});
it("refuses an actual ambient golden-update request", () => {
  refuse({ ...environment(), OPENCUT_UPDATE_GOLDENS: "1" }, "Golden updates are forbidden");
});
it("refuses a changed font rather than silently falling back", () => {
  writeFileSync(join(fonts, "DejaVuSans.ttf"), "deliberately altered font control");
  refuse(environment(), "Required fixed font differs");
});
