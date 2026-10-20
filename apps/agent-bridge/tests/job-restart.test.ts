import { type ChildProcess, spawn, spawnSync } from "node:child_process";
import {
  existsSync,
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
import {
  afterAll,
  afterEach,
  beforeAll,
  beforeEach,
  expect,
  it,
  vi,
} from "vitest";
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
import { removeFfmpegSpawnMcpAddition } from "./fixtures/ffmpeg-spawn-projection";
import {
  removeJobRestartOwnershipAddition,
  restoreJobRestartRaw,
} from "./fixtures/job-restart-projection";

import { removeRenderBudgetMcpAddition } from "./fixtures/render-budget-projection";
import { removeSolidGraphicMcpAdditions } from "./fixtures/solid-graphic-color-slots-projection";

const roots: string[] = [];
const root = () => {
  const directory = mkdtempSync(join(tmpdir(), "opencut-job-restart-"));
  roots.push(directory);
  return directory;
};
const fixtureRoot = mkdtempSync(join(tmpdir(), "opencut-crash-bundle-"));
const fixture = join(fixtureRoot, "job-crash-process.js");
const buildFixture = (entry: string, output: string) => {
  const built = spawnSync(
    "bun",
    ["build", entry, "--target=bun", "--outfile", output],
    { encoding: "utf8", timeout: 5000, windowsHide: true }
  );
  if (built.error || built.status !== 0 || !existsSync(output)) {
    throw new Error(
      `Restart fixture build failed: ${built.error?.message ?? built.stderr}`
    );
  }
};
beforeAll(() => {
  buildFixture(
    resolve(import.meta.dirname, "fixtures/job-crash-process.ts"),
    fixture
  );
}, 5000);
afterAll(() => rmSync(fixtureRoot, { force: true, recursive: true }));
const children = new Map<ChildProcess, Promise<void>>();
let closing = false;
beforeEach(() => {
  closing = false;
});
const closeFixtures = async () => {
  // A timed-out test must not launch a successor after its roots are deleted.
  closing = true;
  const settled = [...children.entries()].map(([child, closed]) => {
    child.kill("SIGKILL");
    return closed;
  });
  await Promise.all(settled);
};
afterEach(async () => {
  await closeFixtures();
  for (const directory of roots.splice(0)) {
    rmSync(directory, { force: true, recursive: true });
  }
});
const crashProcess = (args: string[], entry = fixture) =>
  new Promise<string>((resolveOutput, reject) => {
    if (closing) {
      reject(new Error("Restart fixture refused during teardown"));
      return;
    }
    const child = spawn("bun", [entry, ...args], {
      stdio: ["ignore", "pipe", "pipe"],
    });
    let output = "";
    let diagnostics = "";
    let failure: Error | undefined;
    const timer = setTimeout(() => {
      failure = new Error(
        `Restart fixture timed out: ${args[0]} pid=${child.pid}`
      );
      child.kill("SIGKILL");
    }, 10_000);
    // Register settlement before any event can end the child. Both normal
    // completion and teardown wait for close, including drained pipe handles.
    children.set(
      child,
      new Promise<void>((closed) => {
        child.once("close", () => {
          clearTimeout(timer);
          children.delete(child);
          closed();
          if (failure) {
            reject(failure);
          } else if (output.includes("\n") && !diagnostics) {
            resolveOutput(output.trim());
          } else {
            reject(
              new Error(
                `Restart fixture failed: ${args[0]} pid=${child.pid}: ${diagnostics}`
              )
            );
          }
        });
      })
    );
    child.once("error", (error) => {
      failure = error;
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
  });

it("refuses invalid fixture compilation without a source fallback", () => {
  const directory = root();
  const broken = join(directory, "broken.ts");
  const output = join(directory, "broken.js");
  writeFileSync(broken, "const = ;");
  expect(() => buildFixture(broken, output)).toThrow(
    "Restart fixture build failed"
  );
  expect(existsSync(output)).toBe(false);
});

it("refuses incomplete child records after settling the owned process", async () => {
  const directory = root();
  const partial = join(directory, "partial.js");
  const pidFile = join(directory, "pid");
  writeFileSync(
    partial,
    `import { writeFileSync } from "node:fs";
writeFileSync(process.argv[2], String(process.pid));
process.stdout.write("{");`
  );
  await expect(crashProcess([pidFile], partial)).rejects.toThrow(
    "Restart fixture failed"
  );
  const pid = Number(readFileSync(pidFile, "utf8"));
  expect(() => process.kill(pid, 0)).toThrow();
  expect(children.size).toBe(0);
});

it("settles a live fixture before teardown and refuses successor admission", async () => {
  const directory = root();
  const waiting = join(directory, "waiting.js");
  const pidFile = join(directory, "pid");
  writeFileSync(
    waiting,
    `import { writeFileSync } from "node:fs";
writeFileSync(process.argv[2], String(process.pid));
setInterval(() => undefined, 1000);`
  );
  const pending = crashProcess([pidFile], waiting).then(
    () => "unexpected success",
    (error: unknown) => error
  );
  await vi.waitFor(() => expect(existsSync(pidFile)).toBe(true));
  const pid = Number(readFileSync(pidFile, "utf8"));
  expect(() => process.kill(pid, 0)).not.toThrow();
  await closeFixtures();
  expect(await pending).toBeInstanceOf(Error);
  expect(children.size).toBe(0);
  expect(() => process.kill(pid, 0)).toThrow();
  await expect(crashProcess(["probe", directory])).rejects.toThrow(
    "Restart fixture refused during teardown"
  );
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
  const previous = removeSolidGraphicMcpAdditions(MCP_SURFACE);
  const source = JSON.stringify(previous);
  const restored = JSON.parse(
    restoreJobRestartRaw(source)
  ) as typeof MCP_SURFACE;
  const predecessor = removeFfmpegSpawnMcpAddition(
    removeRenderBudgetMcpAddition(previous)
  );
  expect(restored.toolDefinitions).toEqual(predecessor.toolDefinitions);
  expect(restored.capabilityIdentifiers).not.toContain(RESTART.capability);
  expect(restored.capabilityIdentifiers.length + 1).toBe(
    previous.capabilityIdentifiers.length
  );
  const changed = structuredClone(MCP_SURFACE);
  changed.version = 2;
  expect(() => restoreJobRestartRaw(JSON.stringify(changed))).toThrow();
});
