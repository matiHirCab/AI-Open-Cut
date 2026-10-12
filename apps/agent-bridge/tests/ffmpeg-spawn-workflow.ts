import { spawnSync } from "node:child_process";
import {
  chmodSync,
  mkdtempSync,
  readdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { Client } from "@modelcontextprotocol/client";
import { StdioClientTransport } from "@modelcontextprotocol/client/stdio";
import { expect } from "vitest";
import CATALOG from "../../../contracts/ffmpeg-spawn-diagnostics-v1.json";
import { eventSchema } from "../src/headless-events";
import {
  jobSchema,
  projectStateSchema,
  writeResultSchema,
} from "../src/schemas";
import { memoizedSdkValidator } from "./fixtures/memoized-sdk-validator";

type Parameters = ConstructorParameters<typeof StdioClientTransport>[0];

export const verifyFfmpegSpawnTransportWorkflow = async (base: Parameters) => {
  // The fake-provider smoke runners are Unix. Kernel startup evidence is separate.
  const root = mkdtempSync(join(tmpdir(), "opencut-spawn-transport-"));
  const marker = join(root, "execution-marker");
  const shim = join(root, "owned-headless");
  const fixture = resolve(
    import.meta.dirname,
    "fixtures/ffmpeg-spawn-headless.ts"
  );
  writeFileSync(
    shim,
    `#!/bin/sh\nexec '${process.execPath.replaceAll("'", "'\\''")}' '${fixture.replaceAll("'", "'\\''")}' "$@"\n`
  );
  chmodSync(shim, 0o755);
  const environment = {
    ...base.env,
    OPENCUT_EXPORTS_DIR: join(root, "exports"),
    OPENCUT_HEADLESS_PATH: shim,
    OPENCUT_PROJECTS_DIR: join(root, "projects"),
    OPENCUT_TEST_FFMPEG_SPAWN_MARKER: marker,
    OPENCUT_TEST_REAL_HEADLESS_PATH: base.env?.OPENCUT_HEADLESS_PATH ?? "",
    OPENCUT_TTS_WORK_DIR: join(root, "work"),
  };
  const client = new Client(
    { name: "spawn-transport-proof", version: "1" },
    {
      jsonSchemaValidator: memoizedSdkValidator(),
    }
  );
  const transport = new StdioClientTransport({ ...base, env: environment });
  try {
    const oneShot = spawnSync(shim, [], {
      encoding: "utf8",
      env: environment,
      input: JSON.stringify({
        expectedRevision: 0,
        operation: "render_preview",
        projectId: "transport-fixture",
        timeMs: 0,
      }),
    });
    expect(oneShot.status).toBe(0);
    expect(eventSchema.parse(JSON.parse(oneShot.stdout))).toEqual(
      CATALOG.headlessErrorExample
    );
    expect(readFileSync(marker, "utf8")).toBe("one-shot\n");
    await client.connect(transport);
    const createdResponse = await client.callTool({
      arguments: { name: "Startup transport fixture" },
      name: "project_create",
    });
    expect(createdResponse.isError).not.toBe(true);
    const { projectId } = writeResultSchema.parse(
      createdResponse.structuredContent
    );
    const opened = await client.callTool({
      arguments: { projectId },
      name: "project_open",
    });
    expect(opened.isError).not.toBe(true);
    const state = projectStateSchema.parse(opened.structuredContent);
    const trackId = state.project.tracks.find(
      (track) => track.trackType === "overlay"
    )?.id;
    expect(trackId).toBeDefined();
    const edited = await client.callTool({
      arguments: {
        color: "#112233",
        durationMs: 1000,
        expectedRevision: 0,
        projectId,
        startMs: 0,
        trackId,
        transform: { opacity: 1, positionX: 0, positionY: 0, scale: 1 },
      },
      name: "timeline_add_solid_color",
    });
    expect(edited.isError).not.toBe(true);
    expect(writeResultSchema.parse(edited.structuredContent).revision).toBe(1);
    const projectPath = join(root, "projects", projectId);
    const beforeProject = readFileSync(join(projectPath, "project.json"));
    const queued = await client.callTool({
      arguments: {
        endMs: 100,
        expectedRevision: 1,
        fps: 24,
        includeAudio: false,
        projectId,
        resolution: { height: 32, width: 32 },
        startMs: 0,
      },
      name: "preview_render_range",
    });
    expect(queued.isError).not.toBe(true);
    const job = jobSchema.parse(queued.structuredContent);
    const waitForTerminal = async (
      remaining: number
    ): Promise<ReturnType<typeof jobSchema.parse>> => {
      const response = await client.callTool({
        arguments: { jobId: job.jobId },
        name: "job_get_status",
      });
      expect(response.isError).not.toBe(true);
      const current = jobSchema.parse(response.structuredContent);
      if (current.status === "failed" || current.status === "completed") {
        return current;
      }
      if (remaining === 0) {
        throw new Error("Owned transport job did not reach a terminal result");
      }
      await new Promise((resolvePromise) => setTimeout(resolvePromise, 20));
      return waitForTerminal(remaining - 1);
    };
    const result = await waitForTerminal(100);
    expect(result.status).toBe("failed");
    expect(result.error).toEqual(CATALOG.headlessErrorExample.error);
    expect(result.artifact).toBeUndefined();
    expect(result.progress).toBe(0);
    expect(readFileSync(marker, "utf8")).toBe("one-shot\nworker\n");
    expect(readFileSync(join(projectPath, "project.json"))).toEqual(
      beforeProject
    );
    expect(readdirSync(join(root, "exports"))).toEqual([]);
    expect(readdirSync(join(projectPath, "previews"))).toEqual([]);
  } finally {
    await client.close();
    rmSync(root, { force: true, recursive: true });
  }
};
