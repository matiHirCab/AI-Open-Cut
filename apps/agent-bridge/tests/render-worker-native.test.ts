import type { spawn } from "node:child_process";
import { mkdtemp, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { expect, it, vi } from "vitest";
import { z } from "zod/v4";
import { loadBridgeConfig } from "../src/config";
import { HeadlessClient } from "../src/headless";

const observed = vi.hoisted(() => ({
  workers: [] as { pid: number | undefined; stderr: string }[],
}));
vi.mock("node:child_process", async (original) => {
  const actual = await original<typeof import("node:child_process")>();
  return {
    ...actual,
    spawn: vi.fn((...args: Parameters<typeof spawn>) => {
      const child = actual.spawn(...args);
      if (Array.isArray(args[1]) && args[1].includes("--render-worker")) {
        const record = { pid: child.pid, stderr: "" };
        observed.workers.push(record);
        child.stderr?.on("data", (chunk: Buffer) => {
          record.stderr += chunk.toString();
        });
      }
      return child;
    }),
  };
});

const required = <T>(value: T | undefined): T => {
  if (value === undefined) {
    throw new Error("Missing expected test evidence");
  }
  return value;
};

// This dedicated instrumented run is mandatory in the change verification index.
it.skipIf(process.env.OPENCUT_RASTER_CACHE_TESTS_REQUIRED !== "1")(
  "avoids native raster work through successive bridge calls and restarts cold",
  async () => {
    const root = await mkdtemp(join(tmpdir(), "opencut-native-worker-"));
    const config = loadBridgeConfig({
      ...process.env,
      OPENCUT_ALLOWED_MEDIA_DIRS: root,
      OPENCUT_EXPORTS_DIR: join(root, "exports"),
      OPENCUT_HEADLESS_PATH: resolve(
        import.meta.dirname,
        "../../../target/debug",
        process.platform === "win32"
          ? "opencut-headless.exe"
          : "opencut-headless"
      ),
      OPENCUT_PROJECTS_DIR: join(root, "projects"),
    });
    const client = new HeadlessClient(config);
    const fresh = new HeadlessClient(config);
    try {
      const created = await client.call(
        {
          name: "Native worker",
          operation: "create_project",
          settings: { fps: 10, height: 90, width: 160 },
        },
        z.object({ projectId: z.string() })
      );
      const state = await client.call(
        { operation: "get_state", projectId: created.projectId },
        z.object({
          project: z.object({ tracks: z.array(z.object({ id: z.string() })) }),
        })
      );
      const edited = await client.call(
        {
          expectedRevision: 0,
          operation: "edit_batch",
          operations: [
            {
              durationMs: 1000,
              operation: "add_svg",
              startMs: 0,
              svg: '<svg width="20" height="20"><rect width="20" height="20" fill="#f00"/></svg>',
              trackId: required(state.project.tracks[1]).id,
            },
          ],
          projectId: created.projectId,
        },
        z.object({ revision: z.number() })
      );
      const request = {
        expectedRevision: edited.revision,
        operation: "render_preview" as const,
        projectId: created.projectId,
        timeMs: 0,
      };
      const schema = z.object({ relativePath: z.string() });
      const cold = await client.call(request, schema, {
        requestId: "native-cold",
      });
      const warm = await client.call(request, schema, {
        requestId: "native-warm",
      });
      expect(
        await readFile(
          join(root, "projects", created.projectId, cold.relativePath)
        )
      ).toEqual(
        await readFile(
          join(root, "projects", created.projectId, warm.relativePath)
        )
      );
      await expect(
        client.call({ ...request, expectedRevision: 0 }, schema)
      ).rejects.toMatchObject({ code: "REVISION_CONFLICT" });
      await client.call(request, schema, { requestId: "native-after-error" });
      expect(observed.workers).toHaveLength(1);
      const restarted = await fresh.call(request, schema, {
        requestId: "native-fresh",
      });
      expect(
        await readFile(
          join(root, "projects", created.projectId, cold.relativePath)
        )
      ).toEqual(
        await readFile(
          join(root, "projects", created.projectId, restarted.relativePath)
        )
      );
      await vi.waitFor(() =>
        expect(required(observed.workers[1]).stderr).toContain('"native-fresh"')
      );
      const stats = observed.workers.map((w) =>
        w.stderr
          .split("\n")
          .filter(Boolean)
          .map((line) => JSON.parse(line).rasterCacheTest)
          .filter(Boolean)
      );
      expect(
        required(stats[0]).find((s) => s.requestId === "native-cold")
      ).toMatchObject({ hits: 0, misses: 1 });
      expect(
        required(stats[0]).find((s) => s.requestId === "native-warm")
      ).toMatchObject({ hits: 1, misses: 1 });
      expect(
        required(stats[0]).find((s) => s.requestId === "native-after-error")
      ).toMatchObject({ hits: 2, misses: 1 });
      expect(
        required(stats[1]).find((s) => s.requestId === "native-fresh")
      ).toMatchObject({ hits: 0, misses: 1 });
      expect(required(observed.workers[1]).pid).not.toBe(
        required(observed.workers[0]).pid
      );
    } finally {
      await Promise.all([client.close(), fresh.close()]);
      await rm(root, { force: true, recursive: true });
    }
  },
  30_000
);
