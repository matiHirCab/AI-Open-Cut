import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";
import { expect } from "vitest";
import type { ZodType } from "zod/v4";
import { eventSchema } from "../src/headless-events";
import {
  jobSchema,
  projectStateSchema,
  writeResultSchema,
} from "../src/schemas";

type Call = <T>(
  name: string,
  input: Record<string, unknown>,
  schema: ZodType<T>
) => Promise<T>;
const snapshot = (root: string) => {
  const files: Record<string, string> = {};
  const visit = (directory: string) => {
    for (const name of readdirSync(directory).sort()) {
      if (name === ".lock") {
        continue;
      }
      const path = join(directory, name);
      if (statSync(path).isDirectory()) {
        visit(path);
      } else {
        files[path] = createHash("sha256")
          .update(readFileSync(path))
          .digest("hex");
      }
    }
  };
  visit(root);
  return files;
};

export const verifyRenderBudgetWorkflow = async (
  call: Call,
  environment: Record<string, string>,
  projects: string,
  exportsDirectory: string,
  marker: string
) => {
  const { projectId } = await call(
    "project_create",
    { name: "Private /source/path --command" },
    writeResultSchema
  );
  const state = await call("project_open", { projectId }, projectStateSchema);
  const trackId = state.project.tracks.find(
    (track) => track.trackType === "overlay"
  )?.id;
  if (!trackId) {
    throw new Error("Missing diagnostic fixture overlay");
  }
  await call(
    "timeline_add_shape",
    {
      durationMs: 1000,
      expectedRevision: 0,
      fill: { color: { a: 1, b: 0, g: 0, r: 1 }, type: "solid" },
      geometry: { height: 1024, type: "rectangle", width: 1024 },
      projectId,
      startMs: 0,
      stroke: null,
      trackId,
    },
    writeResultSchema
  );
  // An owned low-resolution positive control proves the fake execution marker
  // works through the same worker/configuration before checking its absence.
  const healthy = await call(
    "preview_render_range",
    {
      endMs: 100,
      expectedRevision: 1,
      fps: 30,
      includeAudio: false,
      projectId,
      resolution: { height: 180, width: 320 },
      startMs: 0,
    },
    jobSchema
  );
  let completed = false;
  for (let attempt = 0; attempt < 300; attempt += 1) {
    // biome-ignore lint/performance/noAwaitInLoops: Observe this owned positive-control job.
    const job = await call(
      "job_get_status",
      { jobId: healthy.jobId },
      jobSchema
    );
    if (["completed", "failed", "cancelled"].includes(job.status)) {
      expect(job.status, JSON.stringify(job.error)).toBe("completed");
      expect(job.artifact).toBeDefined();
      completed = true;
      break;
    }
    await new Promise((resolve) => setTimeout(resolve, 25));
  }
  expect(completed).toBe(true);
  expect(existsSync(marker)).toBe(true);
  expect(readFileSync(marker, "utf8")).toContain("render\n");
  const before = snapshot(join(projects, projectId));
  const exportsBefore = snapshot(exportsDirectory);
  const markerBefore = existsSync(marker) ? readFileSync(marker, "utf8") : "";
  const headless = environment.OPENCUT_HEADLESS_PATH;
  if (!headless) {
    throw new Error("Missing diagnostic fixture headless executable");
  }
  const native = spawnSync(headless, {
    encoding: "utf8",
    env: { ...process.env, ...environment },
    input: JSON.stringify({
      endMs: 100,
      expectedRevision: 1,
      fps: 30,
      height: 4096,
      includeAudio: false,
      operation: "render_preview_range",
      projectId,
      startMs: 0,
      width: 4096,
    }),
  });
  const event = eventSchema.parse(JSON.parse(native.stdout));
  if (event.type !== "error" || !event.error.renderBudget) {
    throw new Error("Expected native admission diagnostic");
  }
  expect(event.error.code).toBe("INVALID_ARGUMENT");
  expect(event.error.retryable).toBe(false);
  expect(event.error.renderBudget.budget).toBe("composition_bytes");
  expect(event.error.renderBudget.measurement).toBe("conservative");
  expect(event.error.renderBudget.limit).toBe("1073741824");
  expect(BigInt(event.error.renderBudget.work ?? "0")).toBeGreaterThan(
    1_073_741_824n
  );
  const queued = await call(
    "preview_render_range",
    {
      endMs: 100,
      expectedRevision: 1,
      fps: 30,
      includeAudio: false,
      projectId,
      resolution: { height: 4096, width: 4096 },
      startMs: 0,
    },
    jobSchema
  );
  let terminal = false;
  for (let attempt = 0; attempt < 300; attempt += 1) {
    // biome-ignore lint/performance/noAwaitInLoops: Poll the owned request to its existing terminal state.
    const job = await call(
      "job_get_status",
      { jobId: queued.jobId },
      jobSchema
    );
    if (["completed", "failed", "cancelled"].includes(job.status)) {
      expect(job.status).toBe("failed");
      expect(job.error).toEqual(event.error);
      expect(job.artifact).toBeUndefined();
      terminal = true;
      break;
    }
    await new Promise((resolve) => setTimeout(resolve, 20));
  }
  expect(terminal).toBe(true);
  expect(snapshot(join(projects, projectId))).toEqual(before);
  expect(snapshot(exportsDirectory)).toEqual(exportsBefore);
  expect(existsSync(marker) ? readFileSync(marker, "utf8") : "").toBe(
    markerBefore
  );
  const diagnostic = JSON.stringify(event.error.renderBudget);
  expect(diagnostic).not.toContain("Private");
  expect(diagnostic).not.toContain("/source");
  expect(diagnostic).not.toContain("command");
};
