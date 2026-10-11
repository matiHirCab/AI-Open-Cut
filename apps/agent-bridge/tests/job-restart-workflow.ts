import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { Client } from "@modelcontextprotocol/client";
import { StdioClientTransport } from "@modelcontextprotocol/client/stdio";
import { expect } from "vitest";
import type { ZodType } from "zod/v4";
import RESTART from "../../../contracts/job-restart-v1.json";
import {
  jobSchema,
  projectStateSchema,
  statusSchema,
  writeResultSchema,
} from "../src/schemas";
import { memoizedSdkValidator } from "./fixtures/memoized-sdk-validator";

type Parameters = ConstructorParameters<typeof StdioClientTransport>[0];
const call = async <T>(
  client: Client,
  name: string,
  args: Record<string, unknown>,
  schema: ZodType<T>
) => {
  const response = await client.callTool({ arguments: args, name });
  if (response.isError) {
    throw new Error(JSON.stringify(response.structuredContent));
  }
  return schema.parse(response.structuredContent);
};

export const verifyJobRestartWorkflow = async (
  client: Client,
  transport: StdioClientTransport,
  parameters: Parameters
) => {
  const project = await call(
    client,
    "project_create",
    { fps: 10, height: 90, name: "Crash reconciliation", width: 160 },
    writeResultSchema
  );
  const before = await call(
    client,
    "project_get_state",
    { projectId: project.projectId },
    projectStateSchema
  );
  const trackId = before.project.tracks.find(
    (track) => track.trackType === "audio"
  )?.id;
  const job = await call(
    client,
    "tts_generate_and_insert",
    {
      expectedRevision: project.revision,
      projectId: project.projectId,
      startMs: 0,
      text: "Crash acknowledgement",
      trackId,
    },
    jobSchema
  );
  let completed = job;
  for (
    let attempt = 0;
    attempt < 200 && completed.status !== "completed";
    attempt += 1
  ) {
    // biome-ignore lint/performance/noAwaitInLoops: Poll the real immutable job before killing its owner.
    await new Promise((resolve) => setTimeout(resolve, 25));
    completed = await call(
      client,
      "job_get_status",
      { jobId: job.jobId },
      jobSchema
    );
    expect(completed.status).not.toBe("failed");
  }
  expect(completed.status).toBe("completed");
  const committed = await call(
    client,
    "project_get_state",
    { projectId: project.projectId },
    projectStateSchema
  );
  const work = parameters.env?.OPENCUT_TTS_WORK_DIR;
  const { pid } = transport;
  if (!(work && pid)) {
    throw new Error("Missing controlled runtime identity");
  }
  const marker = join(work, RESTART.marker);
  const nonce = readFileSync(marker, "utf8");
  const closed = new Promise<void>((resolve) => {
    client.onclose = resolve;
  });
  process.kill(pid, "SIGKILL");
  await closed;
  const validator = memoizedSdkValidator();
  const successor = new Client(
    { name: "restart-successor", version: "1" },
    { jsonSchemaValidator: validator }
  );
  try {
    await successor.connect(new StdioClientTransport(parameters));
    const status = await call(successor, "editor_get_status", {}, statusSchema);
    expect(status.capabilities).toContain(RESTART.capability);
    const after = await call(
      successor,
      "project_get_state",
      { projectId: project.projectId },
      projectStateSchema
    );
    expect(after).toEqual(committed);
    expect(after.project.revision).toBeGreaterThan(before.project.revision);
    for (const name of ["job_get_status", "job_cancel"]) {
      // biome-ignore lint/performance/noAwaitInLoops: Check both public lookups independently.
      const result = await successor.callTool({
        arguments: { jobId: job.jobId },
        name,
      });
      expect(result.isError).toBe(true);
      expect(result.structuredContent).toMatchObject({
        error: { code: RESTART.coldLookupError },
      });
    }
    await expect(
      successor.readResource({ uri: `opencut://jobs/${job.jobId}/artifact` })
    ).rejects.toThrow(RESTART.coldLookupError);
    const blocked = await successor.callTool({
      arguments: {
        expectedRevision: after.project.revision,
        projectId: project.projectId,
        timeMs: 0,
      },
      name: "preview_render_frame",
    });
    expect(blocked.isError).toBe(true);
    expect(blocked.structuredContent).toMatchObject({
      error: { code: RESTART.admissionError, retryable: true },
    });
    expect(readFileSync(marker, "utf8")).toBe(nonce);
    const health = spawnSync(
      parameters.command,
      [...(parameters.args ?? []), "--health"],
      {
        cwd: parameters.cwd,
        encoding: "utf8",
        env: parameters.env,
        timeout: 30_000,
      }
    );
    const doctor = spawnSync(
      parameters.command,
      [...(parameters.args ?? []), "--doctor"],
      {
        cwd: parameters.cwd,
        encoding: "utf8",
        env: parameters.env,
        timeout: 30_000,
      }
    );
    expect(doctor.status).toBe(1);
    expect(doctor.stderr).toContain("operator crash reconciliation");
    expect(doctor.stderr).not.toContain(work);
    expect(readFileSync(marker, "utf8")).toBe(nonce);
    expect(health.status).toBe(0);
    expect(
      statusSchema.omit({ paths: true }).parse(JSON.parse(health.stdout))
        .capabilities
    ).toContain(RESTART.capability);
  } finally {
    await successor.close();
    validator.clear();
  }
  expect(readFileSync(marker, "utf8")).toBe(nonce);
};
