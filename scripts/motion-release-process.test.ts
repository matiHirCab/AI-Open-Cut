import { afterAll, expect, it } from "bun:test";
import { existsSync, mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

const root = mkdtempSync(join(tmpdir(), "opencut-owned-release-control-"));
const repository = resolve(import.meta.dir, "..");
const suffix = process.platform === "win32" ? ".exe" : "";
const runner = join(root, `runner${suffix}`);
const child = join(root, `descendant${suffix}`);
for (const [source, output] of [["owned_release_process.rs", runner], ["release_descendant.rs", child]]) {
  const result = Bun.spawnSync(["rustc", "--edition=2024", "-D", "warnings", join(repository, "crates/editor-core/tests/fixtures", source!), "-o", output!]);
  if (result.exitCode !== 0) { throw new Error(result.stderr.toString()); }
}
afterAll(() => rmSync(root, { force: true, recursive: true }));
const alive = (pid: number) => {
  try {
    process.kill(pid, 0);
    // A dead orphan waiting for the executor's subreaper is not a running process.
    if (process.platform === "linux") {
      return !/\) Z /u.test(readFileSync(`/proc/${pid}/stat`, "utf8"));
    }
    return true;
  } catch { return false; }
};
for (const [mode, code] of [["hang", 124], ["failure", 7], ["success", 0]] as const) {
  it(`settles actual owned descendant on ${mode}`, async () => {
    const pidPath = join(root, `${mode}.pid`);
    const process = Bun.spawn([runner, "3000", child, mode, pidPath], { stdout: "pipe", stderr: "pipe" });
    const [exitCode, stderr] = await Promise.all([process.exited, new Response(process.stderr).text()]);
    expect(exitCode).toBe(code);
    expect(existsSync(pidPath)).toBe(true);
    const pid = Number(readFileSync(pidPath, "utf8"));
    expect(Number.isSafeInteger(pid) && pid > 0).toBe(true);
    const deadline = Date.now() + 5000;
    while (alive(pid) && Date.now() < deadline) { await Bun.sleep(10); }
    expect(alive(pid)).toBe(false);
    if (mode === "hang") { expect(stderr).toContain("timed out"); }
  }, 15_000);
}
it("refuses invalid deadline and missing executable without masking", async () => {
  for (const command of [[runner, "0", child], [runner, "7200001", child], [runner, "1000", join(root, "missing")]]) {
    const result = Bun.spawnSync(command);
    expect(result.exitCode).toBe(124);
    expect(result.stderr.length).toBeGreaterThan(0);
  }
});
