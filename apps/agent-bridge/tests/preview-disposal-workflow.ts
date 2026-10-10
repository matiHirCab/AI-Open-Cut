import { randomUUID } from "node:crypto";
import {
  mkdir,
  mkdtemp,
  readFile,
  rename,
  rm,
  symlink,
  writeFile,
} from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { expect } from "vitest";
import { loadBridgeConfig } from "../src/config";
import { JobRegistry } from "../src/jobs";
import { previewDisposer } from "../src/preview-disposal";
import type { Job } from "../src/schemas";

const artifact = () => ({
  mimeType: "image/png",
  relativePath: `previews/preview-${randomUUID()}.png`,
  sizeBytes: 4,
  warnings: [],
});
export const verifyPreviewDisposal = async (
  headlessPath: string,
  environment: NodeJS.ProcessEnv = process.env
) => {
  const root = await mkdtemp(join(tmpdir(), "opencut-preview-disposal-"));
  try {
    const projectId = randomUUID();
    const projects = join(root, "projects");
    const previews = join(projects, projectId, "previews");
    await mkdir(previews, { recursive: true });
    const output = artifact();
    const path = join(projects, projectId, output.relativePath);
    await writeFile(path, "safe");
    const job: Job = {
      artifact: output,
      createdAtMs: 0,
      expiresAtMs: 10,
      jobId: randomUUID(),
      kind: "preview",
      persistence: "process",
      progress: 1,
      projectId,
      revision: 1,
      status: "completed",
      updatedAtMs: 0,
    };
    const dispose = previewDisposer(
      loadBridgeConfig({
        ...environment,
        OPENCUT_HEADLESS_PATH: headlessPath,
        OPENCUT_PROJECTS_DIR: projects,
      })
    );
    // Exercise the actual source/package process while the configured ancestor changes.
    const moved = `${previews}-saved`;
    let stop = false;
    const race = (async () => {
      while (!stop) {
        try {
          // biome-ignore lint/performance/noAwaitInLoops: Real filesystem interleaving control.
          await rename(previews, moved);
        } catch {
          // Windows sharing restrictions may refuse replacement while cleanup holds handles.
          await new Promise((resolve) => setTimeout(resolve, 1));
          continue;
        }
        try {
          await symlink(
            root,
            previews,
            process.platform === "win32" ? "junction" : "dir"
          );
          await new Promise((resolve) => setTimeout(resolve, 1));
          await rm(previews, {
            maxRetries: 50,
            recursive: true,
            retryDelay: 2,
          });
        } finally {
          await rename(moved, previews);
        }
      }
    })();
    const victim = join(root, output.relativePath.split("/")[1] ?? "missing");
    await writeFile(victim, "outside must survive");
    try {
      for (let attempt = 0; attempt < 12; attempt += 1) {
        // biome-ignore lint/performance/noAwaitInLoops: Reproduce across actual private native invocations.
        await dispose(job).catch((error: unknown) => {
          expect(error).toMatchObject({ code: "VALIDATION_FAILED" });
        });
      }
    } finally {
      stop = true;
      await race;
    }
    expect((await readFile(victim)).toString()).toBe("outside must survive");
    await dispose(job);
    await expect(readFile(path)).rejects.toMatchObject({ code: "ENOENT" });
    await dispose(job);
    const foreign = join(root, "foreign.png");
    await writeFile(foreign, "preserve");
    for (const relativePath of [
      "../../foreign.png",
      "/foreign.png",
      "previews/foreign.png",
      "previews/preview-a.png",
    ]) {
      // biome-ignore lint/performance/noAwaitInLoops: Check each independent attack.
      await expect(
        dispose({ ...job, artifact: { ...output, relativePath } })
      ).rejects.toMatchObject({ code: "VALIDATION_FAILED" });
    }
    await symlink(
      process.platform === "win32" ? root : foreign,
      path,
      process.platform === "win32" ? "junction" : "file"
    );
    await expect(dispose(job)).rejects.toMatchObject({
      code: "VALIDATION_FAILED",
    });
    expect((await readFile(foreign)).toString()).toBe("preserve");
    await rm(path);
    await rm(previews, { recursive: true });
    await symlink(
      root,
      previews,
      process.platform === "win32" ? "junction" : "dir"
    );
    await expect(dispose(job)).rejects.toMatchObject({
      code: "VALIDATION_FAILED",
    });
    await rm(previews, { maxRetries: 50, recursive: true, retryDelay: 2 });
    await mkdir(previews);
    await writeFile(path, "safe");
    const jobs = new JobRegistry({ disposePreview: dispose });
    const owned = jobs.startTask("preview", projectId, 1, async () => ({
      artifact: output,
    }));
    await expect.poll(() => jobs.get(owned.jobId).status).toBe("completed");
    await rename(previews, moved);
    await symlink(
      root,
      previews,
      process.platform === "win32" ? "junction" : "dir"
    );
    await expect(jobs.close()).rejects.toMatchObject({
      code: "VALIDATION_FAILED",
    });
    expect(() =>
      jobs.startTask("preview", projectId, 1, async () => ({}))
    ).toThrowError(expect.objectContaining({ code: "BRIDGE_SHUTTING_DOWN" }));
    await rm(previews, { maxRetries: 50, recursive: true, retryDelay: 2 });
    await rename(moved, previews);
    await jobs.close();
    await expect(readFile(path)).rejects.toMatchObject({ code: "ENOENT" });
    expect((await readFile(victim)).toString()).toBe("outside must survive");
  } finally {
    await rm(root, { force: true, recursive: true });
  }
};
