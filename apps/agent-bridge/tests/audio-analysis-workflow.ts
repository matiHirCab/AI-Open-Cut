import {
  existsSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  renameSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import type { Client } from "@modelcontextprotocol/client";
import { expect, vi } from "vitest";
import type { ZodType } from "zod/v4";
import catalog from "../../../contracts/audio-analysis-v1.json";
import current from "../../../contracts/master-normalization-v1.json";
import {
  audioAnalysisArtifactSchema,
  editDraftSchema,
  jobSchema,
  projectStateSchema,
  statusSchema,
  writeResultSchema,
} from "../src/schemas";

type Call = <T>(
  name: string,
  input: Record<string, unknown>,
  schema: ZodType<T>
) => Promise<T>;
export const verifyAudioAnalysisWorkflow = async (
  client: Client,
  call: Call
) => {
  const status = await call("editor_get_status", {}, statusSchema);
  expect(status.projectSchemaVersion).toBe(current.projectSchemaVersion);
  expect(status.protocolVersion).toBe(1);
  expect(status.subsystems.rendering.capabilities).toContain(
    catalog.capability
  );
  const { tools } = await client.request({ method: "tools/list" });
  expect(tools.filter((t) => t.name === catalog.tool)).toHaveLength(1);
  const { projectId } = await call(
    "project_create",
    { height: 32, name: "Audio analysis workflow", width: 32 },
    writeResultSchema
  );
  const read = () => call("project_open", { projectId }, projectStateSchema);
  const initial = await read();
  const added = await call(
    "timeline_add_solid_color",
    {
      color: "#112233",
      durationMs: 1000,
      expectedRevision: 0,
      projectId,
      startMs: 0,
      trackId: initial.project.tracks[1]?.id,
      transform: { opacity: 1, positionX: 0, positionY: 0, scale: 1 },
    },
    writeResultSchema
  );
  await call(
    "draft_create",
    {
      expectedRevision: 1,
      operations: [
        {
          color: "#445566",
          itemId: added.changedIds[0],
          operation: "update_item",
        },
      ],
      projectId,
    },
    editDraftSchema
  );
  const dir = join(status.paths.projectsDirectory.resolvedPath, projectId);
  const persisted = () =>
    Object.fromEntries(
      [
        "project.json",
        "history.json",
        ...readdirSync(join(dir, "drafts"))
          .filter((n) => n.endsWith(".json"))
          .map((n) => `drafts/${n}`),
      ].map((n) => [n, readFileSync(join(dir, n), "utf8")])
    );
  const before = await read();
  const bytes = persisted();
  const terminal = async (
    jobId: string,
    remaining = 300
  ): Promise<ReturnType<typeof jobSchema.parse>> => {
    const job = await call("job_get_status", { jobId }, jobSchema);
    if (["completed", "failed", "cancelled"].includes(job.status)) {
      return job;
    }
    if (remaining === 0) {
      throw new Error("analysis job did not terminate");
    }
    await new Promise((resolve) => setTimeout(resolve, 25));
    return terminal(jobId, remaining - 1);
  };
  const queue = (extra: Record<string, unknown> = {}) =>
    call(
      catalog.tool,
      {
        endMs: 1000,
        expectedRevision: 1,
        projectId,
        startMs: 0,
        waveformBins: 7,
        ...extra,
      },
      jobSchema
    );
  // Concurrent actual bridge/headless producers exercise reusable plus one-shot.
  const queued = await Promise.all([
    queue(),
    queue({ endMs: 200, startMs: 100, waveformBins: 2 }),
  ]);
  for (const pending of queued) {
    // biome-ignore lint/performance/noAwaitInLoops: Inspect both already concurrent producers and their owned resources.
    const job = await terminal(pending.jobId);
    expect(job).toMatchObject({
      artifact: { mimeType: "application/json" },
      audioAnalysis: {
        channels: 2,
        integratedLufs: null,
        linearSamplePeak: 0,
        samplePeakDbfs: null,
        sampleRateHz: 48_000,
        truePeakDbtp: null,
      },
      kind: "audio_analysis",
      revision: 1,
      status: "completed",
    });
    const response = await client.callTool({
      arguments: { includeBinary: true, jobId: job.jobId },
      name: "job_get_status",
    });
    expect((response.content as { type: string }[]).map((c) => c.type)).toEqual(
      ["text", "resource_link"]
    );
    const uri = job.artifactResource?.uri ?? "";
    const resource = await client.readResource({ uri });
    const [block] = resource.contents;
    expect(block?.mimeType).toBe("application/json");
    expect(block).not.toHaveProperty("blob");
    const text = block && "text" in block ? block.text : "";
    const document = audioAnalysisArtifactSchema.parse(JSON.parse(text));
    expect(document.summary).toEqual(job.audioAnalysis);
    expect(document.bins).toHaveLength(job.audioAnalysis?.actualBinCount ?? 0);
    expect(document.bins[0]?.startFrame).toBe(0);
    expect(document.bins.at(-1)?.endFrame).toBe(job.audioAnalysis?.frameCount);
    expect(text).toBe(
      readFileSync(join(dir, job.artifact?.relativePath ?? ""), "utf8")
    );
    const count = document.summary.frameCount;
    const bins = document.bins.length;
    for (const [index, bin] of document.bins.entries()) {
      expect(bin.startFrame).toBe(Math.floor((index * count) / bins));
      expect(bin.endFrame).toBe(Math.floor(((index + 1) * count) / bins));
      expect(bin.left).toEqual({ max: 0, min: 0, rms: 0 });
      expect(bin.right).toEqual({ max: 0, min: 0, rms: 0 });
    }
  }
  const previews = join(dir, "previews");
  const preservedPreviews = join(dir, "previews-before-confinement");
  const outside = join(
    status.paths.projectsDirectory.resolvedPath,
    `${projectId}-outside-output`
  );
  const inside = join(dir, "alternate-output");
  mkdirSync(outside);
  mkdirSync(inside);
  renameSync(previews, preservedPreviews);
  const unsafeOutputs =
    process.platform === "win32"
      ? ["file"]
      : ["file", "external-link", "internal-link", "dangling-link"];
  try {
    for (const unsafe of unsafeOutputs) {
      if (unsafe === "file") {
        writeFileSync(previews, "not a directory");
      } else {
        const targets: Record<string, string> = {
          "dangling-link": join(dir, "absent-output"),
          "external-link": outside,
          "internal-link": inside,
        };
        symlinkSync(targets[unsafe] ?? "", previews, "dir");
      }
      // biome-ignore lint/performance/noAwaitInLoops: Actual reusable headless requests independently reject each unsafe destination.
      const failed = await terminal((await queue()).jobId);
      expect(failed).toMatchObject({
        error: { code: "PATH_NOT_ALLOWED", retryable: false },
        status: "failed",
      });
      expect(failed.artifact).toBeUndefined();
      expect(failed.audioAnalysis).toBeUndefined();
      expect(readdirSync(outside)).toEqual([]);
      expect(readdirSync(inside)).toEqual([]);
      expect(
        readdirSync(dir).some((name) => name.startsWith(".opencut-"))
      ).toBe(false);
      expect(persisted()).toEqual(bytes);
      rmSync(previews);
    }
  } finally {
    rmSync(previews, { force: true });
    renameSync(preservedPreviews, previews);
  }
  for (const [selection, code] of [
    [{ endMs: 1001, expectedRevision: 0 }, "REVISION_CONFLICT"],
    [{ endMs: 1001 }, "INVALID_ARGUMENT"],
    [{ endMs: 100, startMs: 100 }, "INVALID_ARGUMENT"],
  ] as const) {
    // biome-ignore lint/performance/noAwaitInLoops: Verify explicit per-request failure precedence.
    const failed = await terminal((await queue(selection)).jobId);
    expect(failed).toMatchObject({
      error: { code, retryable: code === "REVISION_CONFLICT" },
      status: "failed",
    });
    expect(failed.audioAnalysis).toBeUndefined();
    expect(failed.artifact).toBeUndefined();
  }
  const cancelled = await queue();
  const cancel = await call(
    "job_cancel",
    { jobId: cancelled.jobId },
    jobSchema
  );
  expect(cancel.status).toBe("cancelled");
  expect((await terminal(cancelled.jobId)).audioAnalysis).toBeUndefined();
  for (const phase of ["pcm", "measurement"]) {
    const control = join(dir, ".analysis-test-phase");
    const pidFile = join(dir, `.analysis-test-${phase}.pid`);
    writeFileSync(control, phase);
    // biome-ignore lint/performance/noAwaitInLoops: Separately reach and cancel each actual analysis subprocess pass.
    const hung = await queue();
    await vi.waitFor(() => expect(existsSync(pidFile)).toBe(true), {
      timeout: 5000,
    });
    const pids = JSON.parse(readFileSync(pidFile, "utf8")) as {
      backend: number;
      descendant: number;
    };
    expect(
      (await call("job_cancel", { jobId: hung.jobId }, jobSchema)).status
    ).toBe("cancelled");
    await vi.waitFor(
      () => {
        expect(() => process.kill(pids.backend, 0)).toThrow();
        expect(() => process.kill(pids.descendant, 0)).toThrow();
        expect(
          readdirSync(dir).some((name) => name.startsWith(".opencut-work-"))
        ).toBe(false);
      },
      { timeout: 5000 }
    );
    rmSync(control);
    rmSync(pidFile);
    const ended = await terminal(hung.jobId);
    expect(ended.audioAnalysis).toBeUndefined();
    expect(ended.artifact).toBeUndefined();
  }
  for (const name of ["timeline_batch_edit", "draft_create"]) {
    // biome-ignore lint/performance/noAwaitInLoops: Verify each unsupported mutation surface independently.
    const response = await client.callTool({
      arguments: {
        expectedRevision: 1,
        operations: [
          {
            endMs: 1000,
            operation: "analyze_audio",
            startMs: 0,
            waveformBins: 1,
          },
        ],
        projectId,
      },
      name,
    });
    expect(response.isError).toBe(true);
  }
  const unknown = await client.callTool({
    arguments: {
      endMs: 1000,
      expectedRevision: 1,
      projectId,
      rawFilter: "volume=2",
      startMs: 0,
      waveformBins: 1,
    },
    name: catalog.tool,
  });
  expect(unknown.isError).toBe(true);
  expect((await read()).project).toEqual(before.project);
  expect(persisted()).toEqual(bytes);
  await call(
    "project_undo",
    { expectedRevision: 1, projectId },
    writeResultSchema
  );
  expect((await read()).project.tracks).toEqual(initial.project.tracks);
  await call(
    "project_redo",
    { expectedRevision: 2, projectId },
    writeResultSchema
  );
  expect((await read()).project.tracks).toEqual(before.project.tracks);
};
