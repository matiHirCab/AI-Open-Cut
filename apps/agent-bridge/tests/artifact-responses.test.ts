import { randomUUID } from "node:crypto";
import { mkdir, mkdtemp, rm, symlink, writeFile } from "node:fs/promises";
import { createServer as createTcpServer } from "node:net";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { Client } from "@modelcontextprotocol/client";
import { InMemoryTransport, McpServer } from "@modelcontextprotocol/server";
import { afterEach, expect, it, vi } from "vitest";
import DELIVERY from "../../../contracts/artifact-delivery-v2.json";
import { loadBridgeConfig } from "../src/config";
import { BridgeError } from "../src/headless";
import { serveHttp } from "../src/http";
import { JobRegistry } from "../src/jobs";
import { type Job, jobSchema } from "../src/schemas";
import {
  artifactUri,
  jobWithArtifactResource,
  readJobArtifact,
} from "../src/server/artifacts";
import { registerContextResources } from "../src/server/context";
import { registerJobTools } from "../src/server/jobs";
import {
  previewJobResponse,
  type ServerDependencies,
} from "../src/server/shared";

const cleanups: (() => Promise<unknown>)[] = [];
afterEach(async () => {
  await Promise.all(cleanups.splice(0).map((cleanup) => cleanup()));
});
const fixture = async () => {
  const root = await mkdtemp(join(tmpdir(), "artifact-delivery-"));
  cleanups.push(() => rm(root, { force: true, recursive: true }));
  const config = loadBridgeConfig({
    OPENCUT_EXPORTS_DIR: join(root, "exports"),
    OPENCUT_GENERATED_MEDIA_DIRS: join(root, "generated"),
    OPENCUT_PROJECTS_DIR: join(root, "projects"),
    OPENCUT_TTS_WORK_DIR: join(root, "speech"),
  });
  const jobs = new JobRegistry();
  cleanups.push(() => jobs.close());
  const dependencies = {
    config,
    jobs,
    session: { activeProjectId: null },
    speech: { previewAudio: vi.fn() },
  } as unknown as ServerDependencies;
  return { dependencies, root };
};
const complete = async (
  dependencies: ServerDependencies,
  kind: Job["kind"] = "preview",
  path = "previews/frame.png",
  projectId = randomUUID()
) => {
  const { jobId } = dependencies.jobs.startTask(kind, projectId, 7, () =>
    Promise.resolve({
      artifact: {
        mimeType: kind === "preview" ? "image/png" : "video/mp4",
        relativePath: path,
        sizeBytes: 3,
        warnings: [],
      },
    })
  );
  await vi.waitFor(() =>
    expect(dependencies.jobs.get(jobId).status).toBe("completed")
  );
  return dependencies.jobs.get(jobId);
};
const clientFor = async (dependencies: ServerDependencies) => {
  const server = new McpServer({ name: "artifact-test", version: "1" });
  registerJobTools(server, dependencies);
  registerContextResources(server, dependencies);
  const client = new Client({ name: "test", version: "1" });
  const [serverTransport, clientTransport] =
    InMemoryTransport.createLinkedPair();
  await server.connect(serverTransport);
  await client.connect(clientTransport);
  cleanups.push(() => client.close());
  cleanups.push(() => server.close());
  return client;
};

it.each(DELIVERY.fixtures)(
  "defaults $kind to canonical metadata and a safe resource link",
  async (example) => {
    const { dependencies } = await fixture();
    let job: Job;
    if (example.kind === "speech_preview") {
      const queued = dependencies.jobs.startTask(
        "speech_preview",
        "speech-preview",
        0,
        () =>
          Promise.resolve({
            speechPreview: {
              durationMs: 20,
              expiresAtMs: Date.now() + 1000,
              language: "en-US",
              modelId: "model",
              modelVersion: null,
              providerId: "provider",
              sampleRateHz: 24_000,
              token: "private-preview-token",
              voice: "voice",
            },
          })
      );
      await vi.waitFor(() =>
        expect(dependencies.jobs.get(queued.jobId).status).toBe("completed")
      );
      job = dependencies.jobs.get(queued.jobId);
    } else {
      job = await complete(
        dependencies,
        example.kind as Job["kind"],
        example.relativePath
      );
    }
    const client = await clientFor(dependencies);
    await Promise.all(
      [undefined, false].map(async (includeBinary) => {
        const result = await client.callTool({
          arguments: {
            jobId: job.jobId,
            ...(includeBinary === undefined ? {} : { includeBinary }),
          },
          name: "job_get_status",
        });
        expect(result.isError).not.toBe(true);
        const content = result.content as { type: string; text?: string }[];
        expect(content.map((block) => block.type)).toEqual(
          DELIVERY.defaultContentTypes
        );
        const output = jobSchema.parse(result.structuredContent);
        expect(JSON.parse(content[0]?.text ?? "")).toEqual(output);
        expect(output.artifactResource).toMatchObject({
          mimeType: example.mimeType,
          name: example.name,
          uri: artifactUri(job.jobId),
        });
        expect(content[1]).not.toHaveProperty("data");
        expect(JSON.stringify(content[1])).not.toContain(
          "private-preview-token"
        );
        expect(output.artifactResource?.expiresAtMs).toBe(
          job.speechPreview?.expiresAtMs ?? job.expiresAtMs
        );
        const jsonResource = await client.readResource({
          uri: `opencut://jobs/${job.jobId}`,
        });
        const [first] = jsonResource.contents;
        expect(
          first && "text" in first ? JSON.parse(first.text) : null
        ).toEqual(output);
      })
    );
    expect(dependencies.speech.previewAudio).not.toHaveBeenCalled();
  }
);

it.each(["preview", "preview_range", "export"] as const)(
  "reads only explicit owned $kind bytes",
  async (kind) => {
    const { dependencies } = await fixture();
    const path =
      kind === "export"
        ? "nested/video.mp4"
        : `previews/file.${kind === "preview" ? "png" : "mp4"}`;
    const job = await complete(dependencies, kind, path);
    const root =
      kind === "export"
        ? dependencies.config.exportsDirectory
        : join(dependencies.config.projectsDirectory ?? "", job.projectId);
    const file = join(root ?? "", path);
    await mkdir(join(file, ".."), { recursive: true });
    await writeFile(file, Buffer.from([1, 2, 3]));
    const client = await clientFor(dependencies);
    const resource = await client.readResource({ uri: artifactUri(job.jobId) });
    expect(resource.contents).toEqual([
      {
        blob: "AQID",
        mimeType: job.artifact?.mimeType,
        uri: artifactUri(job.jobId),
      },
    ]);
    const inline = await client.callTool({
      arguments: { includeBinary: true, jobId: job.jobId },
      name: "job_get_status",
    });
    expect(
      (inline.content as { type: string }[]).map((block) => block.type)
    ).toEqual(
      kind === "preview"
        ? ["text", "resource_link", "image"]
        : DELIVERY.defaultContentTypes
    );
    expect(dependencies.session.activeProjectId).toBe(null);
    expect(dependencies.jobs.get(job.jobId)).toEqual(job);
    await rm(file);
    expect(
      (await previewJobResponse(dependencies, job.jobId)).structuredContent
    ).toMatchObject({ revision: 7 });
    await expect(
      client.readResource({ uri: artifactUri(job.jobId) })
    ).rejects.toThrow("VALIDATION_FAILED");
    if (kind === "preview") {
      const missing = await client.callTool({
        arguments: { includeBinary: true, jobId: job.jobId },
        name: "job_get_status",
      });
      expect(missing.isError).toBe(true);
      expect(JSON.stringify(missing)).not.toContain(file);
      expect(missing.structuredContent).toMatchObject({
        error: {
          code: DELIVERY.failures.missingUnsafeOrUnreadableFile,
          retryable: false,
        },
      });
    }
  }
);

it("reads WAV explicitly, preserves token retention, and sanitizes expired speech errors", async () => {
  const { dependencies, root } = await fixture();
  const path = join(root, "generated", "private-name.wav");
  await mkdir(join(path, ".."), { recursive: true });
  await writeFile(path, Buffer.from([1, 2, 3]));
  const job = {
    ...(await complete(dependencies)),
    artifact: undefined,
    kind: "speech_preview" as const,
    speechPreview: {
      durationMs: 20,
      expiresAtMs: Date.now() + 1000,
      language: "en-US",
      modelId: "model",
      modelVersion: null,
      providerId: "provider",
      sampleRateHz: 24_000,
      token: "private-token",
      voice: "voice",
    },
  };
  vi.spyOn(dependencies.jobs, "get").mockReturnValue(job);
  vi.mocked(dependencies.speech.previewAudio).mockReturnValue({
    mimeType: "audio/wav",
    path,
  });
  const client = await clientFor(dependencies);
  expect(
    (await client.readResource({ uri: artifactUri(job.jobId) })).contents[0]
  ).toMatchObject({ blob: "AQID", mimeType: "audio/wav" });
  expect(
    (await previewJobResponse(dependencies, job.jobId, true)).content[2]
  ).toMatchObject({ data: "AQID", type: "audio" });
  const workPath = join(
    dependencies.config.ttsWorkDirectory,
    "default-root.wav"
  );
  await mkdir(join(workPath, ".."), { recursive: true });
  await writeFile(workPath, Buffer.from([1, 2, 3]));
  vi.mocked(dependencies.speech.previewAudio).mockReturnValue({
    mimeType: "audio/wav",
    path: workPath,
  });
  expect(
    (await client.readResource({ uri: artifactUri(job.jobId) })).contents[0]
  ).toMatchObject({ blob: "AQID" });
  vi.mocked(dependencies.speech.previewAudio).mockImplementation(() => {
    throw new BridgeError(
      "GENERATED_ARTIFACT_NOT_FOUND",
      `missing ${path} private-token`
    );
  });
  const result = await client.callTool({
    arguments: { includeBinary: true, jobId: job.jobId },
    name: "job_get_status",
  });
  expect(result.structuredContent).toMatchObject({
    error: { code: "GENERATED_ARTIFACT_NOT_FOUND" },
  });
  expect(JSON.stringify(result)).not.toContain("private-token");
  expect(JSON.stringify(result)).not.toContain(path);
  await expect(
    client.readResource({ uri: artifactUri(job.jobId) })
  ).rejects.toThrow("GENERATED_ARTIFACT_NOT_FOUND");
  expect(
    (await previewJobResponse(dependencies, job.jobId)).content
  ).toHaveLength(2);
  vi.mocked(dependencies.speech.previewAudio).mockReturnValue({
    mimeType: "audio/wav",
    path: join(root, "foreign.wav"),
  });
  await expect(readJobArtifact(dependencies, job)).rejects.toMatchObject({
    code: "VALIDATION_FAILED",
  });
});

it.each([
  "../secret",
  "/secret",
  "C:/secret",
  "\\\\host\\secret",
  "https://host/file",
  "previews/../secret",
  "previews/%2e%2e/file",
  "previews//file",
  "previews/./file",
  "assets/file",
  "previews/file\u0000",
])("rejects unsafe preview metadata %s without leaking it", async (path) => {
  const { dependencies } = await fixture();
  const job = await complete(dependencies, "preview", path);
  const client = await clientFor(dependencies);
  const result = await client.callTool({
    arguments: { jobId: job.jobId },
    name: "job_get_status",
  });
  expect(result.isError).toBe(true);
  expect(result.structuredContent).toMatchObject({
    error: { code: "VALIDATION_FAILED" },
  });
  expect(JSON.stringify(result)).not.toContain(path);
});

it("reads an owned root beneath a trusted parent alias", async () => {
  const { dependencies, root } = await fixture();
  const job = await complete(dependencies);
  const projects = dependencies.config.projectsDirectory ?? "";
  const file = join(projects, job.projectId, "previews", "frame.png");
  await mkdir(join(projects, job.projectId, "previews"), { recursive: true });
  await writeFile(file, "owned preview");
  const alias = join(root, "parent-alias");
  await symlink(root, alias, "junction");
  const aliasedDependencies = {
    ...dependencies,
    config: {
      ...dependencies.config,
      projectsDirectory: join(alias, "projects"),
    },
  };
  expect((await readJobArtifact(aliasedDependencies, job)).data).toEqual(
    Buffer.from("owned preview")
  );
  const outside = join(root, "outside-alias");
  await mkdir(outside);
  await writeFile(join(outside, "frame.png"), "foreign preview");
  const previews = join(projects, job.projectId, "previews");
  await rm(previews, { recursive: true });
  await symlink(outside, previews, "junction");
  await expect(readJobArtifact(aliasedDependencies, job)).rejects.toMatchObject(
    {
      code: "VALIDATION_FAILED",
    }
  );
});

it.each(["root", "project", "directory", "file"])(
  "rejects %s symlinks and directories",
  async (target) => {
    const { dependencies, root } = await fixture();
    const job = await complete(dependencies);
    const projects = dependencies.config.projectsDirectory ?? "";
    const project = join(projects, job.projectId);
    const previews = join(project, "previews");
    const file = join(previews, "frame.png");
    const outside = join(root, "outside");
    await mkdir(outside, { recursive: true });
    await writeFile(join(outside, "frame.png"), "secret");
    if (target === "root") {
      await symlink(outside, projects, "junction");
    }
    if (target === "project") {
      await mkdir(projects);
      await symlink(outside, project, "junction");
    }
    if (target === "directory") {
      await mkdir(project, { recursive: true });
      await symlink(outside, previews, "junction");
    }
    if (target === "file") {
      await mkdir(previews, { recursive: true });
      await symlink(join(outside, "frame.png"), file);
    }
    await expect(readJobArtifact(dependencies, job)).rejects.toMatchObject({
      code: "VALIDATION_FAILED",
    });
    if (target === "file") {
      await rm(file);
      await mkdir(file);
      await expect(readJobArtifact(dependencies, job)).rejects.toMatchObject({
        code: "VALIDATION_FAILED",
      });
    }
  }
);

it("keeps non-output states and conflict metadata unchanged", async () => {
  const { dependencies } = await fixture();
  const job = await complete(dependencies);
  for (const status of ["queued", "running", "failed", "cancelled"] as const) {
    const state = {
      ...job,
      error: {
        code: "REVISION_CONFLICT",
        failedStage: null,
        ffmpegExitCode: null,
        ffmpegStderrExcerpt: null,
        message: "conflict",
        retryable: false,
      },
      generatedArtifact: { expiresAtMs: 99, token: "commit-token" },
      status,
    };
    expect(jobWithArtifactResource(state)).toEqual(state);
  }
  expect(
    jobWithArtifactResource({ ...job, artifact: undefined, kind: "tts" })
  ).not.toHaveProperty("artifactResource");
  if (!job.artifact) {
    throw new Error("artifact missing");
  }
  await expect(
    readJobArtifact(dependencies, {
      ...job,
      artifact: { ...job.artifact, mimeType: "text/plain" },
    })
  ).rejects.toMatchObject({ code: "VALIDATION_FAILED" });
});

it("rejects invalid input, foreign, expired, evicted and restarted jobs", async () => {
  const { dependencies } = await fixture();
  const job = await complete(dependencies);
  const client = await clientFor(dependencies);
  expect(
    (
      await client.callTool({
        arguments: { includeBinary: "true", jobId: job.jobId },
        name: "job_get_status",
      })
    ).isError
  ).toBe(true);
  for (const uri of [
    `opencut://jobs/${randomUUID()}/artifact`,
    `${artifactUri(job.jobId)}?token=private`,
    `${artifactUri(job.jobId)}#private`,
    "opencut://jobs/..%2Fsecret/artifact",
    `opencut://jobs/${job.jobId.toUpperCase()}/artifact`,
  ]) {
    // biome-ignore lint/performance/noAwaitInLoops: Exercise invalid addresses independently through a single MCP client.
    await expect(client.readResource({ uri })).rejects.toThrow();
  }
  const other = await clientFor({ ...dependencies, jobs: new JobRegistry() });
  await expect(
    other.readResource({ uri: artifactUri(job.jobId) })
  ).rejects.toThrow("JOB_NOT_FOUND");
  let now = 10;
  dependencies.jobs = new JobRegistry({
    maxCount: 1,
    now: () => now,
    ttlMs: 10,
  });
  const expiring = await complete(dependencies);
  now = 20;
  const timed = await clientFor(dependencies);
  await expect(
    timed.readResource({ uri: artifactUri(expiring.jobId) })
  ).rejects.toThrow("JOB_NOT_FOUND");
  now = 21;
  const evicted = await complete(dependencies);
  await complete(dependencies);
  await expect(
    timed.readResource({ uri: artifactUri(evicted.jobId) })
  ).rejects.toThrow("JOB_NOT_FOUND");
});

it("accepts SDK-normalized aliases only for the same owned UUID resource", async () => {
  const { dependencies } = await fixture();
  const job = await complete(dependencies);
  const file = join(
    dependencies.config.projectsDirectory ?? "",
    job.projectId,
    "previews/frame.png"
  );
  await mkdir(join(file, ".."), { recursive: true });
  await writeFile(file, Buffer.from([1, 2, 3]));
  const client = await clientFor(dependencies);
  for (const uri of [
    `OPENCUT://jobs/${job.jobId}/artifact`,
    `opencut://jobs/${job.jobId}/x/../artifact`,
    `opencut://jobs/${job.jobId}/x/%2e%2e/artifact`,
  ]) {
    // biome-ignore lint/performance/noAwaitInLoops: Verify each accepted alias selects the exact same owned output.
    expect((await client.readResource({ uri })).contents[0]).toMatchObject({
      blob: "AQID",
      uri: artifactUri(job.jobId),
    });
  }
  for (const uri of [
    `opencut://jobs/${randomUUID()}/x/../artifact`,
    `opencut://jobs/%${job.jobId.charCodeAt(0).toString(16)}${job.jobId.slice(1)}/artifact`,
    `${artifactUri(job.jobId)}/suffix`,
    `opencut://jobs/${job.jobId}%2Fother/artifact`,
  ]) {
    // biome-ignore lint/performance/noAwaitInLoops: Exercise rejected normalized destinations separately.
    await expect(client.readResource({ uri })).rejects.toThrow();
  }
});

it("requires the same HTTP bearer authorization for resource reads", async () => {
  const { dependencies } = await fixture();
  const job = await complete(dependencies);
  const port = await new Promise<number>((resolvePromise, reject) => {
    const reservation = createTcpServer();
    reservation.listen(0, "127.0.0.1", () => {
      const address = reservation.address();
      if (!address || typeof address === "string") {
        reject(new Error("HTTP address missing"));
        return;
      }
      reservation.close(() => resolvePromise(address.port));
    });
  });
  const config = {
    ...dependencies.config,
    httpAuthToken: "private-auth-token",
    httpPort: port,
  };
  const server = serveHttp(
    dependencies.speech,
    dependencies.transcription,
    dependencies.headless,
    dependencies.jobs,
    config,
    () => Promise.resolve({})
  );
  cleanups.push(() => server.close());
  const response = await fetch(`http://127.0.0.1:${port}/mcp`, {
    body: JSON.stringify({
      id: 1,
      jsonrpc: "2.0",
      method: "resources/read",
      params: { uri: artifactUri(job.jobId) },
    }),
    headers: { "content-type": "application/json" },
    method: "POST",
  });
  expect(response.status).toBe(401);
  expect(await response.text()).toBe('{"error":"unauthorized"}');
});
