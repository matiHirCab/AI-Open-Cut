import { spawnSync } from "node:child_process";
import {
  existsSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { afterEach, expect, it } from "vitest";
import { JOB_RUNTIME_MARKER, JobRuntimeGuard } from "../src/job-runtime";
import { JobRegistry } from "../src/jobs";

const roots: string[] = [];
afterEach(() => {
  for (const directory of roots.splice(0)) {
    rmSync(directory, { force: true, recursive: true });
  }
});
const source = resolve(import.meta.dirname, "../src/index.ts");
const preload = resolve(import.meta.dirname, "fixtures/doctor-cli-preload.mjs");
const filesystemPreload = resolve(
  import.meta.dirname,
  "fixtures/doctor-filesystem-preload.mjs"
);
const doctor = (mode?: string) => {
  const directory = mkdtempSync(join(tmpdir(), "opencut-doctor-cli-"));
  roots.push(directory);
  const work = join(directory, "work");
  const marker = join(work, JOB_RUNTIME_MARKER);
  const run = () =>
    spawnSync(
      "bun",
      [
        ...(mode
          ? [
              "--preload",
              mode === "writable-cleanup" ? filesystemPreload : preload,
            ]
          : []),
        source,
        "--doctor",
      ],
      {
        encoding: "utf8",
        env: {
          ...process.env,
          OPENCUT_CONFIG_ROOT: directory,
          OPENCUT_HEADLESS_PATH: join(directory, "missing-headless"),
          OPENCUT_KOKORO_MODEL_DIR: join(directory, "missing-speech-model"),
          OPENCUT_KOKORO_PYTHON: process.execPath,
          OPENCUT_LOG_LEVEL: "error",
          OPENCUT_PROJECTS_DIR: join(directory, "projects"),
          OPENCUT_TEST_DOCTOR_MODE: mode,
          OPENCUT_TRANSCRIPTION_MODEL_DIR: join(
            directory,
            "missing-transcription-model"
          ),
          OPENCUT_TRANSCRIPTION_PYTHON: process.execPath,
          OPENCUT_TRANSPORT: "stdio",
          OPENCUT_TTS_WORK_DIR: work,
        },
        timeout: 10_000,
      }
    );
  return { directory, marker, run, work };
};

const admitFreshJob = async (work: string) => {
  const guard = new JobRuntimeGuard(work);
  const jobs = new JobRegistry({ beforeStart: () => guard.acquire() });
  let invoked = false;
  try {
    const job = jobs.startTask("preview", "project", 1, () => {
      invoked = true;
      return Promise.resolve({});
    });
    await jobs.close();
    expect(invoked).toBe(true);
    expect(job.persistence).toBe("process");
  } finally {
    await jobs.close();
    guard.release();
  }
};

it("releases the guard after actual unready diagnostics with clean shutdown", async () => {
  const fixture = doctor();
  const result = fixture.run();
  expect(result.error).toBeUndefined();
  expect(result.status).toBe(1);
  const report = JSON.parse(result.stdout);
  expect(report.ready).toBe(false);
  expect(report.checks).toContainEqual(
    expect.objectContaining({ id: "speech", status: "error" })
  );
  expect(existsSync(fixture.marker)).toBe(false);
  await admitFreshJob(fixture.work);
  // An ordinary diagnostic failure can also be inspected repeatedly.
  expect(fixture.run().status).toBe(1);
  expect(existsSync(fixture.marker)).toBe(false);
});

it.each(["ready", "unready"])(
  "releases its guard after completed %s diagnostics and successful close",
  async (mode) => {
    const fixture = doctor(mode);
    const result = fixture.run();
    expect(result.error).toBeUndefined();
    expect(result.status).toBe(mode === "ready" ? 0 : 1);
    expect(JSON.parse(result.stdout).ready).toBe(mode === "ready");
    expect(existsSync(fixture.marker)).toBe(false);
    await admitFreshJob(fixture.work);
  }
);

it("releases its guard after an ordinary directory diagnostic failure", async () => {
  const fixture = doctor();
  writeFileSync(join(fixture.directory, "projects"), "not a directory");
  const result = fixture.run();
  expect(result.error).toBeUndefined();
  expect(result.status).toBe(1);
  expect(JSON.parse(result.stdout).checks).toContainEqual(
    expect.objectContaining({ id: "writeability", status: "error" })
  );
  expect(existsSync(fixture.marker)).toBe(false);
  await admitFreshJob(fixture.work);
});

it.each([
  "doctor-cleanup",
  "headless-cleanup",
  "speech-cleanup",
  "transcription-cleanup",
])("retains the guard after %s fails", (mode) => {
  const fixture = doctor(mode);
  const result = fixture.run();
  expect(result.error).toBeUndefined();
  expect(result.status).toBe(1);
  expect(result.stderr).toContain("cleanup failed");
  expect(result.stderr).not.toContain(fixture.directory);
  const nonce = readFileSync(fixture.marker, "utf8");
  expect(() => new JobRuntimeGuard(fixture.work).acquire()).toThrowError(
    expect.objectContaining({ code: "JOB_REGISTRY_FULL", retryable: true })
  );
  expect(readFileSync(fixture.marker, "utf8")).toBe(nonce);
});

it.each([false, true])(
  "retains its guard after actual probe deletion failure (earlier directory failure: %s)",
  (earlierFailure) => {
    const fixture = doctor("writable-cleanup");
    if (earlierFailure) {
      // The first probe rejects immediately; other probes still own cleanup.
      writeFileSync(join(fixture.directory, "projects"), "not a directory");
    }
    const result = fixture.run();
    expect(result.error).toBeUndefined();
    expect(result.status).toBe(1);
    expect(result.stderr).toContain("Doctor artifact cleanup failed");
    expect(result.stderr).not.toContain(fixture.directory);
    expect(existsSync(fixture.marker)).toBe(true);
    expect(() => new JobRuntimeGuard(fixture.work).acquire()).toThrowError(
      expect.objectContaining({ code: "JOB_REGISTRY_FULL" })
    );
  }
);

it("preserves an occupied guard and foreign private output", () => {
  const fixture = doctor("ready");
  const owner = new JobRuntimeGuard(fixture.work);
  owner.acquire();
  const nonce = readFileSync(fixture.marker, "utf8");
  const output = join(fixture.work, "foreign-private-output");
  writeFileSync(output, "private bytes");
  try {
    const result = fixture.run();
    expect(result.error).toBeUndefined();
    expect(result.status).toBe(1);
    expect(result.stderr).toContain("operator crash reconciliation");
    expect(result.stderr).not.toContain(fixture.directory);
    expect(readFileSync(fixture.marker, "utf8")).toBe(nonce);
    expect(readFileSync(output, "utf8")).toBe("private bytes");
  } finally {
    owner.release();
  }
});
