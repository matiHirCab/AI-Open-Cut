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
import { basename, join } from "node:path";
import { expect } from "vitest";
import { loadBridgeConfig } from "../src/config";
import { JobRegistry } from "../src/jobs";
import { previewDisposer } from "../src/preview-disposal";
import type { Job } from "../src/schemas";
import {
  assertPreviewSentinel,
  cleanupPreviewFixture,
  sentinelContents,
  settlePreviewObservation,
  verifyPreviewFixtureControl,
} from "./preview-disposal-controls";

const artifact = () => ({
  mimeType: "image/png",
  relativePath: `previews/preview-${randomUUID()}.png`,
  sizeBytes: 4,
  warnings: [],
});

const verifyNativeControl = async (
  job: Job,
  headlessPath: string,
  environment: NodeJS.ProcessEnv,
  observe: (phase: string) => void
) => {
  const root = await mkdtemp(
    join(tmpdir(), "opencut-disposal-native-control-")
  );
  try {
    const projects = join(root, "projects");
    const previews = join(projects, job.projectId, "previews");
    const path = join(
      projects,
      job.projectId,
      job.artifact?.relativePath ?? ""
    );
    const victim = join(root, basename(path));
    await mkdir(previews, { recursive: true });
    await writeFile(path, "safe");
    await writeFile(victim, sentinelContents);
    const dispose = previewDisposer(
      loadBridgeConfig({
        ...environment,
        OPENCUT_HEADLESS_PATH: headlessPath,
        OPENCUT_PROJECTS_DIR: projects,
      })
    );
    await assertPreviewSentinel(victim, "native-only: before disposal");
    observe("native-only: before disposal");
    await dispose(job);
    await assertPreviewSentinel(victim, "native-only: after disposal");
    await expect(readFile(path)).rejects.toMatchObject({ code: "ENOENT" });
    await dispose(job);
    await assertPreviewSentinel(victim, "native-only: after missing disposal");
    observe("native-only: preserved");
  } finally {
    await rm(root, { force: true, recursive: true });
  }
};

export const verifyPreviewDisposal = async (
  headlessPath: string,
  environment: NodeJS.ProcessEnv = process.env,
  mode: "source" | "packaged" | "smoke" = "smoke"
) => {
  const observe = (phase: string, counts = {}) => {
    process.stdout.write(
      `${JSON.stringify({
        event: "preview-disposal.safety-evidence",
        headlessPath,
        mode,
        phase,
        platform: process.platform,
        runtime: { executable: process.execPath, versions: process.versions },
        ...counts,
      })}\n`
    );
  };
  observe("fixture-only: starting without native disposal");
  await verifyPreviewFixtureControl(observe);
  const root = await mkdtemp(join(tmpdir(), "opencut-preview-disposal-"));
  let failed = false;
  let linkPath: string | undefined;
  let finalPath: string | undefined;
  try {
    const projectId = randomUUID();
    const projects = join(root, "projects");
    const previews = join(projects, projectId, "previews");
    await mkdir(previews, { recursive: true });
    const output = artifact();
    const path = join(projects, projectId, output.relativePath);
    linkPath = previews;
    finalPath = path;
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
    await verifyNativeControl(job, headlessPath, environment, observe);
    const dispose = previewDisposer(
      loadBridgeConfig({
        ...environment,
        OPENCUT_HEADLESS_PATH: headlessPath,
        OPENCUT_PROJECTS_DIR: projects,
      })
    );
    // Exercise the actual source/package process while the configured ancestor changes.
    const moved = `${previews}-saved`;
    const victim = join(root, output.relativePath.split("/")[1] ?? "missing");
    await writeFile(victim, sentinelContents);
    await assertPreviewSentinel(victim, "race: before mutation");
    let stop = false;
    let replacements = 0;
    let sharingRefusals = 0;
    let invocations = 0;
    const race = (async () => {
      while (!stop) {
        try {
          // biome-ignore lint/performance/noAwaitInLoops: Real filesystem interleaving control.
          await rename(previews, moved);
        } catch (error) {
          if (
            process.platform !== "win32" ||
            !["EACCES", "EBUSY", "EPERM"].includes(
              (error as NodeJS.ErrnoException).code ?? ""
            )
          ) {
            throw error;
          }
          sharingRefusals += 1;
          // Windows sharing restrictions may refuse replacement while cleanup holds handles.
          await new Promise((resolve) => setTimeout(resolve, 1));
          continue;
        }
        let mutationFailure: unknown;
        try {
          await symlink(
            root,
            previews,
            process.platform === "win32" ? "junction" : "dir"
          );
          replacements += 1;
          await assertPreviewSentinel(victim, "race: after link creation");
          observe("race: before recursive link removal", {
            invocations,
            replacements,
            sharingRefusals,
          });
          await new Promise((resolve) => setTimeout(resolve, 1));
          await rm(previews, {
            maxRetries: 50,
            recursive: true,
            retryDelay: 2,
          });
          await assertPreviewSentinel(
            victim,
            "race: after recursive link removal"
          );
        } catch (error) {
          mutationFailure = error;
          throw error;
        } finally {
          await settlePreviewObservation(async () => {
            await rename(moved, previews);
            await assertPreviewSentinel(victim, "race: after restoration");
          }, mutationFailure);
        }
      }
    })();
    // Observe rejection immediately while retaining it for the awaited safety result.
    race.catch(() => {
      stop = true;
    });
    let nativeFailure: unknown;
    try {
      for (let attempt = 0; attempt < 12; attempt += 1) {
        invocations += 1;
        // biome-ignore lint/performance/noAwaitInLoops: Reproduce across actual private native invocations.
        await dispose(job).catch((error: unknown) => {
          expect(error).toMatchObject({ code: "VALIDATION_FAILED" });
        });
        await assertPreviewSentinel(
          victim,
          `race: after native call ${attempt + 1}`
        );
      }
    } catch (error) {
      nativeFailure = error;
      throw error;
    } finally {
      stop = true;
      try {
        await settlePreviewObservation(() => race, nativeFailure);
      } finally {
        observe("race: settled", {
          invocations,
          replacements,
          sharingRefusals,
        });
      }
    }
    expect(
      replacements,
      "race must exercise actual ancestor replacement"
    ).toBeGreaterThan(0);
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
    await assertPreviewSentinel(victim, "static: after final reparse removal");
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
    await assertPreviewSentinel(
      victim,
      "static: after ancestor reparse removal"
    );
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
    await assertPreviewSentinel(
      victim,
      "close debt: after ancestor reparse removal"
    );
    await rename(moved, previews);
    await jobs.close();
    await expect(readFile(path)).rejects.toMatchObject({ code: "ENOENT" });
    expect((await readFile(victim)).toString()).toBe("outside must survive");
  } catch (error) {
    failed = true;
    throw error;
  } finally {
    await cleanupPreviewFixture(
      root,
      [finalPath, linkPath].filter(
        (path): path is string => path !== undefined
      ),
      failed,
      observe
    );
  }
};
