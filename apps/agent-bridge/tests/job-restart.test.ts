import { spawn } from "node:child_process";
import {
  mkdtempSync,
  readdirSync,
  readFileSync,
  renameSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { afterEach, expect, it } from "vitest";
import OWNERSHIP from "../../../contracts/contract-ownership-v1.json";
import RESTART from "../../../contracts/job-restart-v1.json";
import MCP_SURFACE from "../../../contracts/mcp-surface-v1.json";
import { BridgeError } from "../src/headless";
import {
  EPHEMERAL_JOBS_CAPABILITY,
  JOB_RUNTIME_MARKER,
  JobRuntimeGuard,
} from "../src/job-runtime";
import { JobRegistry } from "../src/jobs";
import { jobSchema } from "../src/schemas";
import {
  removeJobRestartOwnershipAddition,
  restoreJobRestartRaw,
} from "./fixtures/job-restart-projection";

const roots: string[] = [];
const root = () => {
  const directory = mkdtempSync(join(tmpdir(), "opencut-job-restart-"));
  roots.push(directory);
  return directory;
};
afterEach(() => {
  for (const directory of roots.splice(0)) {
    rmSync(directory, { force: true, recursive: true });
  }
});
const fixture = resolve(import.meta.dirname, "fixtures/job-crash-process.ts");
const crashProcess = (args: string[]) =>
  new Promise<string>((resolveOutput, reject) => {
    const child = spawn("bun", [fixture, ...args], {
      stdio: ["ignore", "pipe", "pipe"],
    });
    let output = "";
    let diagnostics = "";
    const timer = setTimeout(() => {
      child.kill("SIGKILL");
      reject(new Error("Restart fixture timed out"));
    }, 10_000);
    child.once("error", (error) => {
      clearTimeout(timer);
      reject(error);
    });
    child.stderr.on("data", (chunk: Buffer) => {
      diagnostics += chunk.toString();
    });
    child.stdout.on("data", (chunk: Buffer) => {
      output += chunk.toString();
      if (output.includes("\n")) {
        child.kill("SIGKILL");
      }
    });
    child.once("close", () => {
      clearTimeout(timer);
      if (output.includes("\n") && !diagnostics) {
        resolveOutput(output.trim());
      } else {
        reject(new Error(`Restart fixture failed: ${diagnostics}`));
      }
    });
  });

it("loses every job phase on actual process death and bounds repeated crash admission", async () => {
  const directory = root();
  const snapshot = JSON.parse(await crashProcess(["produce", directory])) as {
    jobs: unknown[];
  };
  const jobs = snapshot.jobs.map((job) => jobSchema.parse(job));
  expect(jobs.map((job) => job.status)).toEqual([
    "running",
    "running",
    "completed",
    "failed",
    "cancelled",
  ]);
  expect(RESTART.restartStates).toEqual([
    "queued",
    "running",
    "completed",
    "failed",
    "cancelled",
  ]);
  const inventory = readdirSync(directory).sort();
  const marker = readFileSync(join(directory, JOB_RUNTIME_MARKER), "utf8");
  for (let attempt = 0; attempt < 3; attempt += 1) {
    const probe = JSON.parse(
      // biome-ignore lint/performance/noAwaitInLoops: Kill each predecessor before its successor.
      await crashProcess(["probe", directory, ...jobs.map((job) => job.jobId)])
    );
    expect(probe).toEqual({
      admission: RESTART.admissionError,
      missing: jobs.map(() => RESTART.coldLookupError),
    });
    expect(readdirSync(directory).sort()).toEqual(inventory);
    expect(readFileSync(join(directory, JOB_RUNTIME_MARKER), "utf8")).toBe(
      marker
    );
    expect(readFileSync(join(directory, "private-output"), "utf8")).toBe(
      "private bytes"
    );
  }
});

it("drops a genuinely queued admission killed before producer dispatch", async () => {
  const directory = root();
  const snapshot = JSON.parse(
    await crashProcess(["produce-queued", directory])
  ) as { jobs: unknown[] };
  const queued = jobSchema.parse(snapshot.jobs[0]);
  expect(queued.status).toBe("queued");
  const probe = JSON.parse(
    await crashProcess(["probe", directory, queued.jobId])
  );
  expect(probe).toEqual({
    admission: RESTART.admissionError,
    missing: [RESTART.coldLookupError],
  });
  expect(readdirSync(directory)).toEqual([JOB_RUNTIME_MARKER]);
});

it("keeps distinct private runtimes independent and releases only after graceful cleanup", async () => {
  const firstRoot = root();
  const secondRoot = root();
  const firstGuard = new JobRuntimeGuard(firstRoot);
  const secondGuard = new JobRuntimeGuard(secondRoot);
  const first = new JobRegistry({ beforeStart: () => firstGuard.acquire() });
  const second = new JobRegistry({ beforeStart: () => secondGuard.acquire() });
  const old = first.startTask("preview", "private", 4, () =>
    Promise.resolve({})
  );
  const fresh = second.startTask("preview", "private", 4, () =>
    Promise.resolve({})
  );
  expect(fresh.jobId).not.toBe(old.jobId);
  expect(fresh.persistence).toBe(RESTART.persistence);
  expect(() => second.get(old.jobId)).toThrowError(
    expect.objectContaining({ code: RESTART.coldLookupError })
  );
  expect(() => second.cancel(old.jobId)).toThrowError(
    expect.objectContaining({ code: RESTART.coldLookupError })
  );
  await first.close();
  firstGuard.release();
  const successorGuard = new JobRuntimeGuard(firstRoot);
  successorGuard.acquire();
  successorGuard.release();
  await second.close();
  secondGuard.release();
  expect(readdirSync(firstRoot)).toEqual([]);
  expect(readdirSync(secondRoot)).toEqual([]);
});

it("rejects occupied and replaced markers without invoking a producer or deleting foreign state", () => {
  const directory = root();
  const owner = new JobRuntimeGuard(directory);
  owner.acquire();
  const foreign = new JobRuntimeGuard(directory);
  const jobs = new JobRegistry({ beforeStart: () => foreign.acquire() });
  let invoked = false;
  expect(() =>
    jobs.startTask("tts", "private", 1, () => {
      invoked = true;
      return Promise.resolve({});
    })
  ).toThrowError(expect.objectContaining({ code: RESTART.admissionError }));
  expect(invoked).toBe(false);
  foreign.release();
  const path = join(directory, JOB_RUNTIME_MARKER);
  renameSync(path, `${path}.owned`);
  writeFileSync(path, "foreign guard");
  expect(() => owner.acquire()).toThrowError(
    expect.objectContaining({ code: RESTART.admissionError })
  );
  expect(() => owner.release()).toThrowError(
    expect.objectContaining({ code: RESTART.admissionError })
  );
  expect(readFileSync(path, "utf8")).toBe("foreign guard");
  // Restore ownership for handle cleanup (especially Windows).
  rmSync(path);
  renameSync(`${path}.owned`, path);
  owner.release();
});

it.runIf(process.platform !== "win32")(
  "refuses symlink roots and markers while preserving targets",
  () => {
    const directory = root();
    const target = root();
    const alias = join(directory, "alias");
    symlinkSync(target, alias, "dir");
    expect(() => new JobRuntimeGuard(alias).acquire()).toThrowError(
      expect.objectContaining({ code: RESTART.admissionError })
    );
    const privatePath = join(target, "private");
    writeFileSync(privatePath, "private bytes");
    symlinkSync(privatePath, join(directory, JOB_RUNTIME_MARKER));
    expect(() => new JobRuntimeGuard(directory).acquire()).toThrowError(
      expect.objectContaining({ code: RESTART.admissionError })
    );
    expect(readFileSync(privatePath, "utf8")).toBe("private bytes");
  }
);

it("keeps guard ownership through failed preview cleanup until close succeeds", async () => {
  const directory = root();
  const guard = new JobRuntimeGuard(directory);
  const cleanup = { fails: true };
  const jobs = new JobRegistry({
    beforeStart: () => guard.acquire(),
    disposePreview: () => {
      // biome-ignore lint/suspicious/noUnnecessaryConditions: This flag changes before the second close attempt.
      if (cleanup.fails) {
        return Promise.reject(
          new BridgeError("VALIDATION_FAILED", "safe failure")
        );
      }
      return Promise.resolve();
    },
  });
  jobs.startTask("preview", "private", 1, () =>
    Promise.resolve({
      artifact: {
        mimeType: "image/png",
        relativePath: "previews/preview.png",
        sizeBytes: 1,
        warnings: [],
      },
    })
  );
  await expect(jobs.close()).rejects.toMatchObject({
    code: "VALIDATION_FAILED",
  });
  expect(() => new JobRuntimeGuard(directory).acquire()).toThrowError(
    expect.objectContaining({ code: RESTART.admissionError })
  );
  cleanup.fails = false;
  await jobs.close();
  guard.release();
  expect(readdirSync(directory)).toEqual([]);
});

it("pins the canonical bridge-only capability and guarded runtime marker", () => {
  expect(EPHEMERAL_JOBS_CAPABILITY).toBe(RESTART.capability);
  expect(JOB_RUNTIME_MARKER).toBe(RESTART.marker);
  expect(RESTART.resume).toBe(false);
  expect(RESTART.automaticReplay).toBe(false);
  expect(RESTART.automaticOrphanAdoption).toBe(false);
});

it("restores only the reviewed ownership addition and refuses changed predecessor evidence", () => {
  const source = JSON.stringify(OWNERSHIP);
  expect(JSON.parse(restoreJobRestartRaw(source))).toEqual(
    removeJobRestartOwnershipAddition(OWNERSHIP)
  );
  const changed = structuredClone(OWNERSHIP);
  changed.categories.jobRestart.canonical = "foreign.json";
  expect(() => restoreJobRestartRaw(JSON.stringify(changed))).toThrow();
  const unrelated = structuredClone(OWNERSHIP);
  unrelated.categories.artifactDelivery.canonical = "foreign.json";
  expect(() => restoreJobRestartRaw(JSON.stringify(unrelated))).toThrow();
});

it("preserves all prior MCP schemas and capabilities except the single additive identifier", () => {
  const source = JSON.stringify(MCP_SURFACE);
  const restored = JSON.parse(
    restoreJobRestartRaw(source)
  ) as typeof MCP_SURFACE;
  expect(restored.toolDefinitions).toEqual(MCP_SURFACE.toolDefinitions);
  expect(restored.capabilityIdentifiers).not.toContain(RESTART.capability);
  expect(restored.capabilityIdentifiers.length + 1).toBe(
    MCP_SURFACE.capabilityIdentifiers.length
  );
  const changed = structuredClone(MCP_SURFACE);
  changed.version = 2;
  expect(() => restoreJobRestartRaw(JSON.stringify(changed))).toThrow();
});
