import { chmod, mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { afterEach, describe, expect, it } from "vitest";

import { runtimePathDiagnostics } from "../src/diagnostics";

const roots: string[] = [];

afterEach(async () => {
  await Promise.all(
    roots
      .splice(0)
      .map(async (path) => await rm(path, { force: true, recursive: true }))
  );
});

describe("runtime path diagnostics", () => {
  it("reports resolved readiness facts for configured files and directories", async () => {
    const root = await mkdtemp(join(tmpdir(), "opencut-diagnostics-"));
    roots.push(root);
    const media = join(root, "media");
    const model = join(root, "model");
    const work = join(root, "work");
    const worker = join(root, "worker.py");
    const executable = join(
      root,
      process.platform === "win32" ? "python.exe" : "python"
    );
    await Promise.all([mkdir(media), mkdir(model), mkdir(work)]);
    await Promise.all([
      writeFile(worker, "# worker\n"),
      writeFile(executable, ""),
    ]);

    const paths = await runtimePathDiagnostics({
      allowedMediaDirectories: [media],
      configRoot: root,
      exportsDirectory: work,
      ffmpegPath: executable,
      ffprobePath: executable,
      generatedMediaDirectories: [media],
      projectsDirectory: work,
      ttsModelDirectory: model,
      ttsPythonPath: executable,
      ttsWorkDirectory: work,
      ttsWorkerPath: worker,
    });

    expect(paths.allowedMediaDirectories[0]).toMatchObject({
      error: null,
      exists: true,
      readable: true,
      ready: true,
      writable: true,
    });
    expect(paths.ffmpeg).toMatchObject({
      error: null,
      executable: true,
      exists: true,
      ready: true,
    });
    expect(paths.kokoro.worker).toMatchObject({
      error: null,
      exists: true,
      readable: true,
      ready: true,
    });
  });

  it("uses actionable generic errors without leaking path-adjacent secrets", async () => {
    const root = await mkdtemp(join(tmpdir(), "opencut-diagnostics-secret-"));
    roots.push(root);
    const secret = "token-super-secret";
    const paths = await runtimePathDiagnostics({
      configRoot: root,
      ffmpegPath: join(root, secret, "ffmpeg"),
    });

    expect(paths.ffmpeg).toMatchObject({
      error: "File is missing or inaccessible",
      exists: false,
      ready: false,
    });
    expect(paths.ffmpeg.error).not.toContain(secret);
  });

  it("resolves bare executables from the configured child environment", async () => {
    const root = await mkdtemp(join(tmpdir(), "opencut-diagnostics-path-"));
    roots.push(root);
    const executable = join(
      root,
      process.platform === "win32" ? "customffmpeg.exe" : "customffmpeg"
    );
    await writeFile(executable, "");
    if (process.platform !== "win32") {
      await chmod(executable, 0o755);
    }
    const environment =
      process.platform === "win32"
        ? { Path: root, PathExt: ".EXE" }
        : { PATH: root };

    const paths = await runtimePathDiagnostics({
      configRoot: root,
      environment,
      ffmpegPath: "customffmpeg",
    });

    expect(paths.ffmpeg).toMatchObject({
      error: null,
      executable: true,
      exists: true,
      ready: true,
      resolvedPath: executable,
    });
  });
});
