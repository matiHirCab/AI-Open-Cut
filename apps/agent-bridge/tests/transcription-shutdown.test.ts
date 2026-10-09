import { execFileSync, spawn } from "node:child_process";
import { existsSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { Client } from "@modelcontextprotocol/client";
import { StdioServerTransport } from "@modelcontextprotocol/server/stdio";
import { expect, it, vi } from "vitest";
import { memoizedSdkValidator } from "./fixtures/memoized-sdk-validator";

it.each(
  process.platform === "win32" ? ["stdin EOF"] : ["SIGTERM", "stdin EOF"]
)(
  "actual bridge %s aborts alignment before graceful provider disposal",
  async (shutdownEntry) => {
    const root = mkdtempSync(join(tmpdir(), "transcription-shutdown-"));
    const pidFile = join(root, "provider.pid");
    const source = join(root, "audio.wav");
    const fixture = join(root, "headless.mjs");
    writeFileSync(
      source,
      "test source; domain validation is outside this lifecycle fixture"
    );
    writeFileSync(
      fixture,
      `import { readFileSync } from "node:fs";
const r = JSON.parse(readFileSync(0, "utf8"));
process.stdout.write(JSON.stringify({type:"result",result:{
 assetId:r.assetId, projectId:r.projectId, path:${JSON.stringify(source)},
 contentHash:null, probe:{durationMs:1000,hasAudio:true,hasVideo:false,
 audioChannels:1,audioCodec:null,audioSampleRateHz:24000,formatName:null,
 videoCodec:null,videoHeight:null,videoWidth:null}, revision:0
}})+"\\n");
`
    );
    const headless = join(
      root,
      process.platform === "win32" ? "headless.exe" : "headless"
    );
    // The adapter directly spawns executables without a shell. A .cmd launcher
    // does not provide that boundary on Windows; compile the identical fixture.
    execFileSync(
      "bun",
      ["build", fixture, "--compile", "--outfile", headless],
      {
        stdio: "pipe",
      }
    );
    const validator = memoizedSdkValidator();
    const client = new Client(
      { name: "shutdown-test", version: "1" },
      { jsonSchemaValidator: validator }
    );
    const bridge = spawn(
      "bun",
      ["run", resolve(import.meta.dirname, "../src/index.ts")],
      {
        env: {
          ...Object.fromEntries(
            Object.entries(process.env).filter(
              (entry): entry is [string, string] => entry[1] !== undefined
            )
          ),
          OPENCUT_ALLOWED_MEDIA_DIRS: root,
          OPENCUT_EXPORTS_DIR: join(root, "exports"),
          OPENCUT_HEADLESS_PATH: headless,
          OPENCUT_PROJECTS_DIR: root,
          OPENCUT_TEST_TRANSCRIPTION_PID_PATH: pidFile,
          OPENCUT_TRANSCRIPTION_PYTHON:
            process.env.OPENCUT_TEST_PYTHON ?? "python",
          OPENCUT_TRANSCRIPTION_TIMEOUT_MS: "10000",
          OPENCUT_TRANSCRIPTION_WORKER: resolve(
            import.meta.dirname,
            "fixtures/fake_transcription_worker.py"
          ),
        },
        stdio: ["pipe", "pipe", "pipe"],
        windowsHide: true,
      }
    );
    // Public SDK JSON-RPC framing over owned streams lets EOF reach the real
    // bridge without the client transport's process-kill disposal fallback.
    const transport = new StdioServerTransport(bridge.stdout, bridge.stdin);
    let bridgeStderr = "";
    bridge.stderr.on("data", (chunk: Buffer | string) => {
      bridgeStderr = (bridgeStderr + chunk.toString()).slice(-16_384);
    });
    let providerPid: number | undefined;
    try {
      await client.connect(transport);
      const job = await client.callTool({
        arguments: {
          assetId: "asset",
          knownText: "__shutdown_alignment__",
          projectId: "project",
        },
        name: "transcription_preview",
      });
      expect(job.isError).not.toBe(true);
      await vi.waitFor(async () => {
        const state = await client.callTool({
          arguments: {
            jobId: (job.structuredContent as { jobId: string }).jobId,
          },
          name: "job_get_status",
        });
        expect(
          state.structuredContent,
          `Readiness job: ${JSON.stringify(state.structuredContent)}\nBridge stderr:\n${bridgeStderr}`
        ).not.toMatchObject({ status: "failed" });
        expect(existsSync(pidFile)).toBe(true);
      });
      const workerPid = Number(readFileSync(pidFile, "utf8"));
      providerPid = workerPid;
      const bridgePid = bridge.pid;
      if (bridgePid === undefined) {
        throw new Error("Bridge exited before shutdown signal");
      }
      if (shutdownEntry === "SIGTERM") {
        process.kill(bridgePid, "SIGTERM");
      } else {
        bridge.stdin.end();
      }
      // The worker sleeps 60s and inference timeout is 10s. This checks the
      // real shutdown entry's jobs-first order, rather than provider.close().
      await vi.waitFor(
        () => {
          expect(() => process.kill(bridgePid, 0)).toThrow();
          expect(() => process.kill(workerPid, 0)).toThrow();
        },
        { timeout: 3000 }
      );
    } finally {
      await client.close();
      bridge.stdin.end();
      if (bridge.exitCode === null) {
        bridge.kill();
      }
      providerPid ??= existsSync(pidFile)
        ? Number(readFileSync(pidFile, "utf8"))
        : undefined;
      if (providerPid) {
        try {
          process.kill(providerPid, "SIGKILL");
        } catch {
          /* already reaped */
        }
      }
      validator.clear();
      await rm(root, { force: true, recursive: true });
    }
  },
  10_000
);
