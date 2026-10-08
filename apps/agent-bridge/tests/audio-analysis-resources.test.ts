import { randomUUID } from "node:crypto";
import { mkdir, mkdtemp, rm, symlink, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { Client } from "@modelcontextprotocol/client";
import { InMemoryTransport, McpServer } from "@modelcontextprotocol/server";
import { afterEach, expect, it, vi } from "vitest";
import catalog from "../../../contracts/audio-analysis-v1.json";
import { loadBridgeConfig } from "../src/config";
import { JobRegistry } from "../src/jobs";
import { audioAnalysisSummarySchema } from "../src/schemas";
import { artifactUri } from "../src/server/artifacts";
import { registerJobTools } from "../src/server/jobs";
import type { ServerDependencies } from "../src/server/shared";

const growth = vi.hoisted(() => ({ path: "", requests: [] as number[] }));
vi.mock("node:fs/promises", async (importOriginal) => {
  const fs = await importOriginal<typeof import("node:fs/promises")>();
  return {
    ...fs,
    open: async (...args: Parameters<typeof fs.open>) => {
      const handle = await fs.open(...args);
      if (
        growth.path &&
        (await fs.realpath(String(args[0]))) ===
          (await fs.realpath(growth.path))
      ) {
        const read = handle.read.bind(handle);
        let first = true;
        handle.read = ((...readArgs: unknown[]) => {
          growth.requests.push(Number(readArgs[2]));
          return (async () => {
            if (first) {
              first = false;
              await fs.writeFile(
                growth.path,
                Buffer.alloc(4 * 1024 * 1024 + 1)
              );
            }
            return Reflect.apply(read, handle, readArgs);
          })();
        }) as typeof handle.read;
      }
      return handle;
    },
  };
});
const cleanups: (() => Promise<unknown>)[] = [];
afterEach(async () => {
  growth.path = "";
  growth.requests = [];
  await Promise.all(cleanups.splice(0).map((f) => f()));
});
const fixture = async () => {
  const root = await mkdtemp(join(tmpdir(), "audio-analysis-resource-"));
  cleanups.push(() => rm(root, { force: true, recursive: true }));
  let now = 100;
  const jobs = new JobRegistry({ now: () => now, ttlMs: 10 });
  cleanups.push(() => jobs.close());
  const projectId = randomUUID();
  const summary = audioAnalysisSummarySchema.parse(catalog.silenceExample);
  const document = {
    bins: [
      {
        endFrame: 4800,
        left: { max: 0, min: 0, rms: 0 },
        right: { max: 0, min: 0, rms: 0 },
        startFrame: 0,
      },
    ],
    summary,
    version: 1,
  };
  const text = JSON.stringify(document);
  const config = loadBridgeConfig({
    OPENCUT_EXPORTS_DIR: join(root, "exports"),
    OPENCUT_PROJECTS_DIR: join(root, "projects"),
  });
  const dependencies = {
    config,
    jobs,
    session: { activeProjectId: null },
  } as unknown as ServerDependencies;
  const queued = jobs.startTask("audio_analysis", projectId, 7, () =>
    Promise.resolve({
      artifact: {
        mimeType: "application/json",
        relativePath: "previews/analysis.json",
        sizeBytes: Buffer.byteLength(text),
        warnings: [],
      },
      audioAnalysis: summary,
    })
  );
  await vi.waitFor(() =>
    expect(jobs.get(queued.jobId).status).toBe("completed")
  );
  const file = join(
    config.projectsDirectory ?? "",
    projectId,
    "previews/analysis.json"
  );
  await mkdir(join(file, ".."), { recursive: true });
  await writeFile(file, text);
  const server = new McpServer({ name: "analysis-resource", version: "1" });
  registerJobTools(server, dependencies);
  const client = new Client({ name: "test", version: "1" });
  const [s, c] = InMemoryTransport.createLinkedPair();
  await server.connect(s);
  await client.connect(c);
  cleanups.push(() => client.close());
  cleanups.push(() => server.close());
  return {
    client,
    dependencies,
    document,
    expire: () => {
      now = 111;
    },
    file,
    jobId: queued.jobId,
    text,
  };
};
it("returns explicit JSON text and metadata-first summary even after backing file disappears", async () => {
  const f = await fixture();
  const uri = artifactUri(f.jobId);
  expect((await f.client.readResource({ uri })).contents).toEqual([
    { mimeType: "application/json", text: f.text, uri },
  ]);
  await rm(f.file);
  for (const includeBinary of [false, true]) {
    // biome-ignore lint/performance/noAwaitInLoops: Verify both polling modes after the same file disappearance.
    const response = await f.client.callTool({
      arguments: { includeBinary, jobId: f.jobId },
      name: "job_get_status",
    });
    expect(response.isError).not.toBe(true);
    expect(response.structuredContent).toMatchObject({
      artifactResource: {
        mimeType: "application/json",
        name: "Audio analysis",
        uri,
      },
      audioAnalysis: catalog.silenceExample,
    });
    expect((response.content as { type: string }[]).map((c) => c.type)).toEqual(
      ["text", "resource_link"]
    );
  }
  await expect(f.client.readResource({ uri })).rejects.toThrow(
    "VALIDATION_FAILED"
  );
  expect(f.dependencies.session.activeProjectId).toBeNull();
});
it.each([
  "malformed",
  "nonUTF8",
  "oversized",
  "summary",
  "unknown",
  "symlink",
  "directory",
])(
  "rejects %s JSON without path disclosure or fabricated summary",
  async (mode) => {
    const f = await fixture();
    if (mode === "malformed") {
      await writeFile(f.file, "{");
    }
    if (mode === "nonUTF8") {
      await writeFile(f.file, Buffer.from([255]));
    }
    if (mode === "oversized") {
      await writeFile(f.file, Buffer.alloc(4 * 1024 * 1024 + 1));
    }
    if (mode === "summary") {
      await writeFile(
        f.file,
        JSON.stringify({
          ...f.document,
          summary: { ...f.document.summary, truePeakDbtp: 0 },
        })
      );
    }
    if (mode === "unknown") {
      await writeFile(
        f.file,
        JSON.stringify({ ...f.document, privatePath: f.file })
      );
    }
    if (mode === "symlink") {
      const other = join(f.file, "..", "foreign.json");
      await writeFile(other, f.text);
      await rm(f.file);
      await symlink(other, f.file);
    }
    if (mode === "directory") {
      await rm(f.file);
      await mkdir(f.file);
    }
    const error = await f.client
      .readResource({ uri: artifactUri(f.jobId) })
      .catch((e: unknown) => e);
    expect(String(error)).toContain("VALIDATION_FAILED");
    expect(String(error)).not.toContain(f.file);
    expect(f.dependencies.jobs.get(f.jobId).audioAnalysis).toEqual(
      catalog.silenceExample
    );
  }
);
it("bounds actual opened-handle reads when a file grows after stat admission", async () => {
  const f = await fixture();
  growth.path = f.file;
  await expect(
    f.client.readResource({ uri: artifactUri(f.jobId) })
  ).rejects.toThrow("VALIDATION_FAILED");
  expect(growth.requests[0]).toBe(4 * 1024 * 1024 + 1);
  expect(growth.requests.every((n) => n > 0 && n <= 4 * 1024 * 1024 + 1)).toBe(
    true
  );
});
it("retains UUID registry ownership and expiry across foreign and restarted sessions", async () => {
  const f = await fixture();
  for (const id of [randomUUID(), "../escape"]) {
    // biome-ignore lint/performance/noAwaitInLoops: Exercise each resource ownership response against the same registry.
    await expect(
      f.client.readResource({ uri: `opencut://jobs/${id}/artifact` })
    ).rejects.toThrow();
  }
  const restarted = new JobRegistry();
  expect(() => restarted.get(f.jobId)).toThrow("Job was not found");
  await restarted.close();
  f.expire();
  await expect(
    f.client.readResource({ uri: artifactUri(f.jobId) })
  ).rejects.toThrow("JOB_NOT_FOUND");
});
