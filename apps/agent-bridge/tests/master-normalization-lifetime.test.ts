import {
  chmodSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readdirSync,
  readFileSync,
  writeFileSync,
} from "node:fs";
import { rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { afterEach, expect, it, vi } from "vitest";
import catalog from "../../../contracts/master-normalization-v1.json";
import { loadBridgeConfig } from "../src/config";
import { HeadlessClient } from "../src/headless";
import {
  audioAnalysisResultSchema,
  projectStateSchema,
  writeResultSchema,
} from "../src/schemas";

const resources: { root: string; client: HeadlessClient }[] = [];
const phases = [
  "capture",
  "original_measurement",
  "target_measurement",
  "processing",
  "ebu_verification",
  "truepeak_verification",
  "correction",
  "final_ebu_verification",
  "final_truepeak_verification",
] as const;
const create = async () => {
  const root = mkdtempSync(join(tmpdir(), "master-normalization-lifetime-"));
  const media = join(root, "media");
  mkdirSync(media);
  const script = resolve(
    import.meta.dirname,
    "fixtures/master-normalization-process.mjs"
  );
  const wrapper = (mode: "ffmpeg" | "ffprobe") => {
    const path = join(
      root,
      `${mode}${process.platform === "win32" ? ".cmd" : ""}`
    );
    writeFileSync(
      path,
      process.platform === "win32"
        ? `@bun "${script}" ${mode} %*\r\n`
        : `#!/bin/sh\nexec bun "${script}" ${mode} "$@"\n`
    );
    if (process.platform !== "win32") {
      chmodSync(path, 0o755);
    }
    return path;
  };
  const client = new HeadlessClient(
    loadBridgeConfig({
      ...process.env,
      OPENCUT_ALLOWED_MEDIA_DIRS: media,
      OPENCUT_EXPORTS_DIR: join(root, "exports"),
      OPENCUT_FFMPEG_PATH: wrapper("ffmpeg"),
      OPENCUT_FFPROBE_PATH: wrapper("ffprobe"),
      OPENCUT_HEADLESS_PATH:
        process.env.OPENCUT_TEST_HEADLESS_PATH ??
        resolve(
          import.meta.dirname,
          "../../../target/debug",
          process.platform === "win32"
            ? "opencut-headless.exe"
            : "opencut-headless"
        ),
      OPENCUT_PROJECTS_DIR: join(root, "projects"),
      OPENCUT_TEST_AUDIO_ANALYSIS_FILTERS: "1",
      OPENCUT_TEST_AUDIO_NORMALIZATION_FILTERS: "1",
    })
  );
  resources.push({ client, root });
  const { projectId } = await client.call(
    {
      name: "Normalization phases",
      operation: "create_project",
      settings: { fps: 24, height: 32, width: 32 },
    },
    writeResultSchema
  );
  const initial = await client.call(
    { operation: "get_state", projectId },
    projectStateSchema
  );
  await client.call(
    {
      expectedRevision: 0,
      operation: "edit_batch",
      operations: [
        {
          color: "#112233",
          durationMs: 1000,
          operation: "add_solid_color",
          startMs: 0,
          trackId: initial.project.tracks[1]?.id ?? "",
          transform: { opacity: 1, positionX: 0, positionY: 0, scale: 1 },
        },
        {
          normalization: catalog.settingsExample,
          operation: "audio_master_set_normalization",
        },
      ],
      projectId,
    },
    writeResultSchema
  );
  const dir = join(root, "projects", projectId);
  writeFileSync(
    join(dir, "previews/published.png"),
    "published unrelated file"
  );
  const request = {
    endMs: 1000,
    expectedRevision: 1,
    operation: "analyze_audio" as const,
    projectId,
    startMs: 0,
    waveformBins: 2,
  };
  const persisted = () => [
    readFileSync(join(dir, "project.json")),
    readFileSync(join(dir, "history.json")),
  ];
  return { client, dir, persisted, request, root };
};
afterEach(async (context) => {
  await Promise.all(
    resources.splice(0).map(async ({ root, client }) => {
      await client.close();
      if (context.task.result?.state === "fail") {
        console.warn(`failed normalization lifetime fixture retained: ${root}`);
        return;
      }
      await rm(root, {
        force: true,
        maxRetries: 10,
        recursive: true,
        retryDelay: 50,
      });
    })
  );
});

it.each(
  phases.flatMap((phase) =>
    ["cancel", "deadline", "shutdown"].map((mode) => ({ mode, phase }))
  )
)(
  "reaps actual normalization $phase backend and descendants for $mode",
  async ({ phase, mode }) => {
    const { dir, client, request, persisted } = await create();
    const before = persisted();
    const control = join(dir, ".normalization-test-phase");
    writeFileSync(control, phase);
    const controller = new AbortController();
    const pending = client.call(request, audioAnalysisResultSchema, {
      requestId: "normalization-owned",
      signal: controller.signal,
      ...(mode === "deadline" ? { timeoutMs: 2000 } : {}),
    });
    const outcome = pending.then(
      () => ({ code: "UNEXPECTED_SUCCESS" }),
      (error: unknown) => error
    );
    const pidFile = join(dir, `.normalization-test-${phase}.pid`);
    await vi.waitFor(() => expect(existsSync(pidFile)).toBe(true), {
      timeout: 1500,
    });
    const pids = JSON.parse(readFileSync(pidFile, "utf8")) as {
      backend: number;
      descendant: number;
    };
    const overlap =
      mode === "shutdown"
        ? null
        : client
            .call(
              { operation: "get_state", projectId: request.projectId },
              projectStateSchema
            )
            .then(
              (state) => ({ error: null, state }),
              (error: unknown) => ({ error, state: null })
            );
    if (mode === "cancel") {
      controller.abort();
    } else if (mode === "shutdown") {
      await client.close();
    }
    expect(await outcome).toMatchObject({
      code: mode === "deadline" ? "HEADLESS_TIMEOUT" : "JOB_CANCELLED",
    });
    await vi.waitFor(() => {
      expect(() => process.kill(pids.backend, 0)).toThrow();
      expect(() => process.kill(pids.descendant, 0)).toThrow();
    });
    expect(
      readdirSync(dir).filter((name) => name.startsWith(".opencut-work-"))
    ).toEqual([]);
    expect(readFileSync(join(dir, "previews/published.png"), "utf8")).toBe(
      "published unrelated file"
    );
    expect(readdirSync(join(dir, "previews"))).toEqual(["published.png"]);
    expect(persisted()).toEqual(before);
    if (overlap) {
      const overlappingResult = await overlap;
      expect(overlappingResult.error).toBeNull();
      expect(overlappingResult.state?.project.revision).toBe(1);
      writeFileSync(control, "");
      const recovered = await client.call(request, audioAnalysisResultSchema);
      expect(recovered.summary.frameCount).toBe(48_000);
      expect(persisted()).toEqual(before);
    }
  }
);

it("runs every measured/correction phase and reuses real request workers without persistent coefficients", async () => {
  const { dir, client, request, persisted } = await create();
  const before = persisted();
  const first = await client.call(request, audioAnalysisResultSchema);
  const second = await client.call(request, audioAnalysisResultSchema);
  expect(first.summary).toEqual(second.summary);
  const events = readFileSync(
    join(dir, ".normalization-phase-events.jsonl"),
    "utf8"
  )
    .trim()
    .split("\n")
    .map((line) => JSON.parse(line) as { phase: string });
  for (const phase of phases) {
    expect(events.filter((event) => event.phase === phase)).toHaveLength(2);
  }
  expect(persisted()).toEqual(before);
  expect(
    readdirSync(dir).filter((name) => name.startsWith(".opencut-work-"))
  ).toEqual([]);
});

it.each([
  ...phases.map((phase) => ({ kind: "exit", phase })),
  ...["capture", "processing", "correction"].flatMap((phase) =>
    ["partial", "short", "extra", "nonfinite"].map((kind) => ({ kind, phase }))
  ),
  { kind: "poison", phase: "target_measurement" },
  { kind: "poison", phase: "ebu_verification" },
  { kind: "poison", phase: "final_ebu_verification" },
  { kind: "infeasible", phase: "truepeak_verification" },
  { kind: "infeasible", phase: "final_truepeak_verification" },
])(
  "rejects actual $phase $kind faults without publication or changed state",
  async ({ phase, kind }) => {
    const { dir, client, request, persisted } = await create();
    const before = persisted();
    const fault = join(dir, ".normalization-test-fault");
    writeFileSync(fault, JSON.stringify({ kind, phase }));
    await expect(
      client.call(request, audioAnalysisResultSchema)
    ).rejects.toMatchObject({ code: "FFMPEG_FAILED" });
    expect(persisted()).toEqual(before);
    expect(
      readdirSync(dir).filter((name) => name.startsWith(".opencut-work-"))
    ).toEqual([]);
    expect(readdirSync(join(dir, "previews"))).toEqual(["published.png"]);
    expect(readFileSync(join(dir, "previews/published.png"), "utf8")).toBe(
      "published unrelated file"
    );
    writeFileSync(fault, JSON.stringify({ kind: "none", phase: "none" }));
    expect(
      (await client.call(request, audioAnalysisResultSchema)).summary.frameCount
    ).toBe(48_000);
    expect(persisted()).toEqual(before);
  }
);
