import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdir, readdir, readFile, rename, writeFile } from "node:fs/promises";
import { join, resolve } from "node:path";
import { expect, vi } from "vitest";
import { z } from "zod/v4";
import narrationBlocks from "../../../contracts/narration-blocks-v1.json";
import { loadBridgeConfig } from "../src/config";
import { HeadlessClient } from "../src/headless";
import type { JobTaskContext } from "../src/jobs";
import { closeNarrationSession } from "../src/narration-blocks/lifecycle";
import { NarrationBlocksWorkflow } from "../src/narration-blocks/workflow";
import {
  narrationPlanSchema,
  narrationProfileSchema,
  projectStateSchema,
  writeResultSchema,
} from "../src/schemas";
import { SpeechApplicationService } from "../src/speech";
import { createNarrationTestProvider } from "./fixtures/narration-blocks-provider";
import {
  NarrationCliProcess,
  usingNarrationCli,
} from "./fixtures/narration-cli-process";

async function assertOwnedWorkerRetired(pidFile: string) {
  const pid = Number(await readFile(pidFile, "utf8"));
  expect(Number.isSafeInteger(pid) && pid > 0).toBe(true);
  expect(() => process.kill(pid, 0)).toThrow();
}

async function verifyInterruptedCli(
  environment: NodeJS.ProcessEnv,
  native: HeadlessClient,
  profileId: string,
  projectId: string
) {
  const repository = resolve(import.meta.dirname, "../../..");
  const work = join(environment.OPENCUT_TTS_WORK_DIR ?? "", "signal-cli");
  const model = join(work, "model");
  await mkdir(model, { recursive: true });
  const pidFile = join(work, "owned-worker.pid");
  const session = new NarrationCliProcess(
    join(repository, "apps/agent-bridge/src/narration-blocks-cli.ts"),
    {
      ...environment,
      OPENCUT_KOKORO_MODEL_DIR: model,
      OPENCUT_KOKORO_PYTHON: environment.OPENCUT_TEST_PYTHON ?? "python3",
      OPENCUT_KOKORO_WORKER: join(
        repository,
        "apps/agent-bridge/tests/fixtures/fake_tts_worker.py"
      ),
      OPENCUT_TEST_IGNORE_TERM: "1",
      OPENCUT_TEST_WORKER_PID_FILE: pidFile,
      OPENCUT_TTS_WORK_DIR: work,
    }
  );
  await usingNarrationCli(session, async () => {
    const before = await native.call(
      { operation: "get_state", projectId },
      projectStateSchema
    );
    const plan = narrationPlanSchema.parse(
      await session.command({
        command: "plan",
        input: {
          boundaries: [4],
          expectedVersion: 1,
          maxTextCharacters: 4096,
          profileId,
          sourceText: "hang",
        },
      })
    );
    const pending = session.command({ command: "preview", plan }).then(
      (value) => ({ value }),
      (error) => ({ error })
    );
    await vi.waitFor(
      async () => {
        expect(await readFile(join(work, "synthesis-started"), "utf8")).toBe(
          "started"
        );
      },
      { interval: 25, timeout: 10_000 }
    );
    session.child.kill("SIGTERM");
    const outcome = await pending;
    if ("value" in outcome) {
      expect(outcome.value).toMatchObject({
        error: { code: "JOB_CANCELLED" },
        ready: false,
      });
    }
    expect((await session.finish()).code).toBe(1);
    await assertOwnedWorkerRetired(pidFile);
    expect(
      await native.call(
        { operation: "get_state", projectId },
        projectStateSchema
      )
    ).toEqual(before);
    expect(
      (await readdir(work)).filter((name) => name.endsWith(".wav"))
    ).toEqual([]);
    await assertOwnedWorkerRetired(pidFile);
  });
}

async function verifySourceCli(
  environment: NodeJS.ProcessEnv,
  native: HeadlessClient,
  projectId: string,
  trackId: string
) {
  const repository = resolve(import.meta.dirname, "../../..");
  const work = join(environment.OPENCUT_TTS_WORK_DIR ?? "", "source-cli");
  await mkdir(work, { recursive: true });
  const profile = await native.call(
    {
      operation: "narration_profile_create",
      settings: {
        language: "en-US",
        name: "Source CLI acceptance",
        speed: 1,
        textOptions: {
          chunking: "sentence",
          normalization: "basic",
          pronunciations: [],
          sentencePauseMs: 120,
        },
        voiceId: "test_voice",
      },
    },
    narrationProfileSchema
  );
  const model = join(work, "model");
  const pidFile = join(work, "owned-worker.pid");
  await mkdir(model, { recursive: true });
  const session = new NarrationCliProcess(
    join(repository, "apps/agent-bridge/src/narration-blocks-cli.ts"),
    {
      ...environment,
      OPENCUT_KOKORO_MODEL_DIR: model,
      OPENCUT_KOKORO_PYTHON: environment.OPENCUT_TEST_PYTHON ?? "python3",
      OPENCUT_KOKORO_WORKER: join(
        repository,
        "apps/agent-bridge/tests/fixtures/fake_tts_worker.py"
      ),
      OPENCUT_TEST_WORKER_PID_FILE: pidFile,
      OPENCUT_TTS_WORK_DIR: work,
    }
  );
  const command = (input: unknown) => session.command(input);
  await usingNarrationCli(session, async () => {
    const plan = narrationPlanSchema.parse(
      await command({
        command: "plan",
        input: {
          boundaries: [3, 5],
          expectedVersion: 1,
          maxTextCharacters: 4096,
          profileId: profile.id,
          sourceText: "A. B.",
        },
      })
    );
    const preview = z
      .object({ groupId: z.string(), ready: z.literal(true) })
      .parse(await command({ command: "preview", plan }));
    const current = await native.call(
      { operation: "get_state", projectId },
      projectStateSchema
    );
    const result = await command({
      command: "commit",
      expectedRevision: current.project.revision,
      groupId: preview.groupId,
      projectId,
      startMs: 0,
      trackId,
    });
    expect(result).toMatchObject({
      manifest: { id: plan.id },
      publicationUnknown: false,
    });
    session.child.stdin.end();
    expect((await session.finish()).code).toBe(0);
    const stored = await native.call(
      { operation: "open_project", projectId },
      projectStateSchema
    );
    expect(
      stored.project.narrationManifests[0]?.blocks.map((block) => [
        block.binding.startMs,
        block.binding.durationMs,
      ])
    ).toEqual([
      [0, 100],
      [100, 100],
    ]);
    expect(
      (await readdir(work)).filter((name) => name.endsWith(".wav"))
    ).toEqual([]);
  });
  if (process.platform !== "win32") {
    await verifyInterruptedCli(environment, native, profile.id, projectId);
  }
}

// Simulated inference; real WAV bytes, process transport, core and FFprobe.
export async function verifyNarrationBlocksWorkflow(
  environment: NodeJS.ProcessEnv
) {
  const output = join(
    environment.OPENCUT_TTS_WORK_DIR ?? "",
    "narration-acceptance"
  );
  const config = loadBridgeConfig({
    ...environment,
    OPENCUT_TTS_WORK_DIR: output,
  });
  if (!config.projectsDirectory) {
    throw new Error("Acceptance requires an explicit projects directory");
  }
  await mkdir(output, { recursive: true });
  const fake = createNarrationTestProvider(output);
  const native = new HeadlessClient(config);
  const speech = new SpeechApplicationService(fake.provider, native, 60_000);
  const workflow = new NarrationBlocksWorkflow(native, speech);
  const context = (): JobTaskContext => ({
    markNonCancellable: vi.fn(),
    onProgress: vi.fn(),
    signal: new AbortController().signal,
  });
  const read = (projectId: string) =>
    native.call({ operation: "get_state", projectId }, projectStateSchema);
  const failures: unknown[] = [];
  let evidenceResult: Record<string, unknown> | undefined;
  const create = async (name: string) => {
    const result = await native.call(
      { name, operation: "create_project" },
      writeResultSchema
    );
    return await read(result.projectId);
  };
  try {
    const settings = {
      language: "en-US",
      name: "Native sentence acceptance",
      speed: 1,
      textOptions: {
        chunking: "sentence" as const,
        normalization: "basic" as const,
        pronunciations: [],
        sentencePauseMs: 120,
      },
      voiceId: "contract_voice",
    };
    const profile = await native.call(
      { operation: "narration_profile_create", settings },
      narrationProfileSchema
    );
    const planInput = {
      boundaries: [3, 5],
      expectedVersion: 1,
      maxTextCharacters: 4096,
      profileId: profile.id,
      sourceText: "A. B.",
    };
    for (const maxTextCharacters of [0, 4097, Number.MAX_SAFE_INTEGER]) {
      // biome-ignore lint/performance/noAwaitInLoops: Invalid budgets must reach the real core unchanged, before any inference.
      await expect(
        workflow.plan({ ...planInput, maxTextCharacters })
      ).rejects.toMatchObject({ code: "VALIDATION_FAILED" });
      expect(fake.requests).toHaveLength(0);
    }
    const plan = await workflow.plan(planInput);
    const first = await create("Narration blocks one");
    const track = first.project.tracks.find(
      (value) => value.trackType === "audio"
    );
    if (!track) {
      throw new Error("Native project missing default audio track");
    }
    const preview = await workflow.preview(plan, context());
    expect(preview.ready).toBe(true);
    const originalHashes = await Promise.all(
      preview.completed.map(async (_, index) =>
        createHash("sha256")
          .update(await readFile(join(output, `owned-${index + 1}.wav`)))
          .digest("hex")
      )
    );
    await native.call(
      {
        expectedVersion: 1,
        operation: "narration_profile_update",
        profileId: profile.id,
        settings: { ...settings, speed: 1.2 },
      },
      narrationProfileSchema
    );
    const unrelated = await native.call(
      {
        edit: {
          name: "Unrelated edit",
          operation: "update_track",
          trackId: track.id,
        },
        expectedRevision: 0,
        operation: "edit",
        projectId: first.project.id,
      },
      writeResultSchema
    );
    const commit = {
      expectedRevision: 0,
      groupId: preview.groupId,
      projectId: first.project.id,
      startMs: 100,
      trackId: track.id,
    };
    await expect(workflow.commit(commit, context())).rejects.toMatchObject({
      code: "REVISION_CONFLICT",
    });
    expect((await read(first.project.id)).project.narrationManifests).toEqual(
      []
    );
    const result = await workflow.commit(
      { ...commit, expectedRevision: unrelated.revision },
      context()
    );
    expect(fake.requests).toHaveLength(2);
    expect(result.write.revision).toBe(unrelated.revision + 1);
    expect(
      result.manifest.blocks.map((block) => [
        block.binding.startMs,
        block.binding.durationMs,
      ])
    ).toEqual([
      [100, 400],
      [500, 700],
    ]);
    expect(
      result.manifest.blocks.map(
        (block) => block.request.narrationProfile.version
      )
    ).toEqual([1, 1]);
    const saved = await read(first.project.id);
    for (const [index, block] of result.manifest.blocks.entries()) {
      const asset = saved.project.assets.find(
        (value) => value.id === block.binding.assetId
      );
      if (!asset) {
        throw new Error("Native manifest asset absent");
      }
      const path = join(
        config.projectsDirectory,
        first.project.id,
        asset.projectRelativePath
      );
      // biome-ignore lint/performance/noAwaitInLoops: Verify each actual managed asset against its precommit digest.
      const bytes = await readFile(path);
      expect(createHash("sha256").update(bytes).digest("hex")).toBe(
        originalHashes[index]
      );
      const probe = spawnSync(config.ffprobePath, [
        "-v",
        "error",
        "-show_entries",
        "format=duration",
        "-of",
        "json",
        path,
      ]);
      expect(probe.status, probe.stderr.toString()).toBe(0);
      const facts: { format: { duration: string } } = JSON.parse(
        probe.stdout.toString()
      );
      expect(Math.round(Number(facts.format.duration) * 1000)).toBe(
        index === 0 ? 400 : 700
      );
    }
    const second = await create("Narration blocks two");
    const secondPlan = await workflow.plan({
      ...planInput,
      expectedVersion: 2,
    });
    expect(
      secondPlan.blocks.map((block) => block.request.narrationProfile.version)
    ).toEqual([2, 2]);
    expect(
      secondPlan.blocks.some((block) =>
        plan.blocks.some((old) => old.id === block.id)
      )
    ).toBe(false);
    expect((await read(second.project.id)).project.narrationManifests).toEqual(
      []
    );
    expect((await read(first.project.id)).project.narrationManifests).toEqual(
      saved.project.narrationManifests
    );
    const library = join(config.projectsDirectory, "narration-profiles.json");
    const backup = join(output, "owned-library.json");
    await rename(library, backup);
    try {
      const undone = await native.call(
        {
          expectedRevision: saved.project.revision,
          operation: "undo",
          projectId: first.project.id,
        },
        writeResultSchema
      );
      expect((await read(first.project.id)).project.narrationManifests).toEqual(
        []
      );
      await native.call(
        {
          expectedRevision: undone.revision,
          operation: "redo",
          projectId: first.project.id,
        },
        writeResultSchema
      );
      const reopened = await native.call(
        { operation: "open_project", projectId: first.project.id },
        projectStateSchema
      );
      expect(reopened.project.narrationManifests).toEqual(
        saved.project.narrationManifests
      );
      const [block] = result.manifest.blocks;
      if (!block) {
        throw new Error("Expected first native block");
      }
      await native.call(
        {
          blockId: block.id,
          expectedRevision: reopened.project.revision,
          manifestId: plan.id,
          operation: "detach_narration_block",
          projectId: first.project.id,
        },
        writeResultSchema
      );
      const detached = await read(first.project.id);
      expect(detached.project.narrationManifests[0]?.blocks).toEqual(
        result.manifest.blocks.slice(1)
      );
      expect(detached.project.tracks).toEqual(reopened.project.tracks);
      expect(detached.project.assets).toEqual(reopened.project.assets);
    } finally {
      await rename(backup, library);
    }
    fake.control.failAt = 4;
    const partial = await workflow.preview(secondPlan, context());
    expect(partial.ready).toBe(false);
    expect(partial.completed).toHaveLength(1);
    expect((await read(second.project.id)).project.narrationManifests).toEqual(
      []
    );
    await workflow.discard(partial.groupId);
    const cancelled = new AbortController();
    cancelled.abort();
    await expect(
      workflow.preview(secondPlan, { ...context(), signal: cancelled.signal })
    ).rejects.toMatchObject({ code: "JOB_CANCELLED" });
    expect((await read(second.project.id)).project.narrationManifests).toEqual(
      []
    );
    for (const example of narrationBlocks.planningCases.filter((value) =>
      value.name.startsWith("ecmascript-")
    )) {
      // biome-ignore lint/performance/noAwaitInLoops: Compare each literal Unicode plan through the real core and existing resolver.
      const unicode = await workflow.plan({
        ...planInput,
        boundaries: example.boundaries,
        expectedVersion: 2,
        sourceText: example.text,
      });
      expect(unicode.blocks.map((block) => block.request.text)).toEqual(
        example.requests
      );
      const generated = await workflow.preview(unicode, context());
      expect(generated.ready).toBe(true);
      await workflow.discard(generated.groupId);
    }
    const cliTrack = second.project.tracks.find(
      (value) => value.trackType === "audio"
    );
    if (!cliTrack) {
      throw new Error("Missing CLI audio track");
    }
    await verifySourceCli(environment, native, second.project.id, cliTrack.id);
    const evidence = {
      durationsMs: [400, 700],
      hashes: originalHashes,
      headless: "actual",
      kind: "narration-blocks",
      probe: "actual",
      profileVersions: [1, 2],
      startsMs: [100, 500],
      synthesis: "simulated",
    };
    if (environment.OPENCUT_PLATFORM_EVIDENCE_DIR) {
      await writeFile(
        join(
          environment.OPENCUT_PLATFORM_EVIDENCE_DIR,
          `narration-blocks-${environment.OPENCUT_NARRATION_TEST_MODE ?? "native"}.json`
        ),
        JSON.stringify(evidence, null, 2)
      );
    }
    evidenceResult = evidence;
  } catch (error) {
    failures.push(error);
  } finally {
    const closed = await closeNarrationSession({ headless: native, workflow });
    failures.push(...closed.errors);
  }
  if (failures.length > 0) {
    throw new AggregateError(failures, "Native narration acceptance failed");
  }
  if (!evidenceResult) {
    throw new Error("Native narration evidence missing");
  }
  return evidenceResult;
}
