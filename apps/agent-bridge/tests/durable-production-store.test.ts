import { randomUUID } from "node:crypto";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readdirSync,
  readFileSync,
  renameSync,
  rmSync,
  statSync,
  symlinkSync,
  truncateSync,
  writeFileSync,
  writeSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, describe, expect, it, vi } from "vitest";
import fixture from "../../../contracts/durable-production-requests-v1.json";
import { JOB_RUNTIME_MARKER, JobRuntimeGuard } from "../src/job-runtime";
import {
  canonicalJson,
  DURABLE_FILES,
  DURABLE_LIMITS,
  intentDigest,
  parseAdmission,
  type StoredRecord,
  storedRecordSchema,
} from "../src/production-requests/contract";
import {
  DurableProductionStore,
  type StoreFaultPoint,
} from "../src/production-requests/store";

const ROOT_BINDING = /"rootBinding":"[a-f0-9]+"/;
const roots: string[] = [];
afterEach(() => {
  for (const root of roots.splice(0)) {
    rmSync(root, { force: true, recursive: true });
  }
});

const setup = (
  fault?: (point: StoreFaultPoint) => void,
  write?: (fd: number, bytes: Buffer, offset: number, length: number) => number
) => {
  const root = mkdtempSync(join(tmpdir(), "opencut-durable-store-"));
  roots.push(root);
  const guard = new JobRuntimeGuard(root);
  const store = new DurableProductionStore(root, {
    fault,
    runtimeGuard: guard,
    write,
  });
  const directory = join(root, DURABLE_FILES.directory);
  return { directory, guard, root, store };
};
const queued = (): StoredRecord =>
  storedRecordSchema.parse(
    fixture.independentCatalogExample.catalog.records[0]
  );

describe("durable production physical store", () => {
  it("reads an uninitialized store without creating roots or guards", () => {
    const { directory, guard, root, store } = setup();
    expect(store.snapshot()).toBeNull();
    expect(store.status()).toEqual({
      readable: true,
      reason: "not_initialized",
      writable: false,
    });
    expect(existsSync(directory)).toBe(false);
    expect(existsSync(join(root, JOB_RUNTIME_MARKER))).toBe(false);
    expect(guard.owned).toBe(false);
  });

  it("atomically retains a queued intent across a safe guarded reader restart", () => {
    const { directory, guard, root, store } = setup();
    const committed = store.update(() => [queued()]);
    expect(committed.records).toEqual([queued()]);
    expect(readFileSync(join(directory, DURABLE_FILES.catalog), "utf8")).toBe(
      canonicalJson(committed)
    );
    expect(existsSync(join(directory, DURABLE_FILES.stage))).toBe(false);
    const reader = new DurableProductionStore(root, {
      runtimeGuard: new JobRuntimeGuard(root),
    });
    expect(reader.snapshot()).toEqual(committed);
    expect(reader.status()).toEqual({
      readable: true,
      reason: "guard_occupied",
      writable: false,
    });
    expect(() => reader.update((records) => records)).toThrow();
    reader.close();
    expect(existsSync(join(directory, JOB_RUNTIME_MARKER))).toBe(true);
    store.close();
    guard.release();
    const freshGuard = new JobRuntimeGuard(root);
    const fresh = new DurableProductionStore(root, {
      runtimeGuard: freshGuard,
    });
    expect(fresh.snapshot()).toEqual(committed);
    fresh.close();
  });

  it("recognizes an entire committed successor after rename acknowledgement loss", () => {
    const { guard, store } = setup((point) => {
      if (point === "after_rename") {
        throw new Error("lost acknowledgement");
      }
    });
    expect(store.update(() => [queued()]).records).toEqual([queued()]);
    expect(store.status().writable).toBe(true);
    store.close();
    guard.release();
  });

  it("does not replay uncommitted admission during explicit debt recovery", () => {
    const faultState = new Set<StoreFaultPoint>(["before_rename"]);
    const { directory, guard, store } = setup((point) => {
      if (faultState.has(point)) {
        throw new Error("injected unavailable rename");
      }
    });
    expect(() => store.update(() => [queued()])).toThrow();
    expect(store.snapshot()).toBeNull();
    expect(store.status().reason).toBe("metadata_unconfirmed");
    expect(existsSync(join(directory, DURABLE_FILES.stage))).toBe(true);
    expect(() => store.close()).toThrow();
    faultState.clear();
    store.recover((catalog) => catalog?.records ?? []);
    expect(store.snapshot()).toBeNull();
    expect(existsSync(join(directory, DURABLE_FILES.catalog))).toBe(false);
    expect(existsSync(join(directory, DURABLE_FILES.stage))).toBe(false);
    store.close();
    guard.release();
  });

  it("preserves foreign stage bytes and blocks mutation before publication", () => {
    const { directory, guard, store } = setup();
    const initial = store.update(() => [queued()]);
    const stage = join(directory, DURABLE_FILES.stage);
    writeFileSync(stage, "foreign stage");
    expect(() => store.update(() => [])).toThrow();
    expect(store.snapshot()).toEqual(initial);
    expect(readFileSync(stage, "utf8")).toBe("foreign stage");
    expect(() => store.close()).toThrow();
    rmSync(stage);
    store.close();
    guard.release();
  });

  it("rejects noncanonical, oversized, invalid UTF8 and copied foreign catalogs", () => {
    for (const mutation of [
      (text: string) => Buffer.from(`${text}\n`),
      (text: string) =>
        Buffer.from(text.replace('"version":1', '"version":1,"version":1')),
      (text: string) => Buffer.from(text.replace('"version":1', '"version":2')),
      () => Buffer.alloc(1_048_577, 32),
      () => Buffer.from([0xff]),
      (text: string) =>
        Buffer.from(
          text.replace(ROOT_BINDING, `"rootBinding":"${"0".repeat(64)}"`)
        ),
    ]) {
      const { directory, guard, store } = setup();
      store.update(() => [queued()]);
      const path = join(directory, DURABLE_FILES.catalog);
      const original = readFileSync(path, "utf8");
      const bytes = mutation(original);
      writeFileSync(path, bytes);
      expect(() => store.snapshot()).toThrow();
      expect(readFileSync(path).equals(bytes)).toBe(true);
      expect(() => store.close()).toThrow();
      expect(guard.owned).toBe(true);
      writeFileSync(path, original);
      store.close();
      guard.release();
    }
  });

  it("enforces record count and prevents recovery from creating new identities", () => {
    const { guard, store } = setup();
    expect(() =>
      store.update(() =>
        Array.from({ length: 65 }, () => ({
          ...queued(),
          requestId: randomUUID(),
        }))
      )
    ).toThrow();
    const committed = store.update(() => [queued()]);
    expect(() =>
      store.recover(() => [{ ...queued(), requestId: randomUUID() }])
    ).toThrow();
    expect(store.snapshot()).toEqual(committed);
    store.close();
    guard.release();
  });

  it("refuses an unsafe configured work root without deleting its contents", () => {
    const { directory, guard, root, store } = setup();
    mkdirSync(directory);
    writeFileSync(join(directory, "foreign"), "keep");
    expect(() => store.update(() => [queued()])).toThrow();
    expect(readFileSync(join(directory, "foreign"), "utf8")).toBe("keep");
    expect(existsSync(join(root, JOB_RUNTIME_MARKER))).toBe(true);
    guard.release();
  });

  it("cleans only an owned partial stage without publishing failed admission", () => {
    const { directory, guard, store } = setup(
      undefined,
      (fd, bytes, offset) => {
        writeSync(fd, bytes, offset, 17);
        throw new Error("private path must not escape");
      }
    );
    expect(() => store.update(() => [queued()])).toThrow(
      "Durable production storage is unavailable"
    );
    expect(readFileSync(join(directory, DURABLE_FILES.stage)).length).toBe(17);
    expect(store.snapshot()).toBeNull();
    store.recover((catalog) => catalog?.records ?? []);
    expect(existsSync(join(directory, DURABLE_FILES.catalog))).toBe(false);
    expect(existsSync(join(directory, DURABLE_FILES.stage))).toBe(false);
    store.close();
    guard.release();
  });

  it.each(["child", "runtime"] as const)(
    "rejects debt confirmation after the %s guard is replaced",
    (kind) => {
      const failure = new Set<StoreFaultPoint>(["before_readback"]);
      const { directory, guard, root, store } = setup((point) => {
        if (failure.has(point)) {
          throw new Error("readback unavailable");
        }
      });
      expect(() => store.update(() => [queued()])).toThrow();
      const marker = join(
        kind === "child" ? directory : root,
        JOB_RUNTIME_MARKER
      );
      const backup = join(root, "original-guard");
      renameSync(marker, backup);
      writeFileSync(marker, "foreign guard");
      expect(() => store.snapshot()).toThrow();
      expect(store.status()).toEqual({
        readable: false,
        reason: "unsafe_storage",
        writable: false,
      });
      expect(readFileSync(marker, "utf8")).toBe("foreign guard");
      expect(() =>
        store.recover((catalog) => catalog?.records ?? [])
      ).toThrow();
      rmSync(marker);
      renameSync(backup, marker);
      failure.clear();
      expect(store.snapshot()?.records).toEqual([queued()]);
      store.recover((catalog) => catalog?.records ?? []);
      store.close();
      guard.release();
    }
  );

  it("preserves a same-byte foreign catalog inode instead of overwriting it", () => {
    const replacement = new Set<StoreFaultPoint>();
    const { directory, guard, root, store } = setup((point) => {
      if (replacement.has(point)) {
        const path = join(directory, DURABLE_FILES.catalog);
        const bytes = readFileSync(path);
        renameSync(path, join(root, "original-catalog"));
        writeFileSync(path, bytes);
      }
    });
    const initial = store.update(() => [queued()]);
    replacement.add("before_rename");
    expect(() => store.update(() => [])).toThrow();
    const path = join(directory, DURABLE_FILES.catalog);
    expect(readFileSync(path, "utf8")).toBe(canonicalJson(initial));
    expect(() => store.snapshot()).toThrow();
    rmSync(path);
    renameSync(join(root, "original-catalog"), path);
    replacement.clear();
    store.recover((catalog) => catalog?.records ?? []);
    expect(store.snapshot()).toEqual(initial);
    store.close();
    guard.release();
  });

  it("rejects growth between open and stat before allocating catalog bytes", () => {
    const growth = new Set<StoreFaultPoint>();
    const { directory, guard, store } = setup((point) => {
      if (growth.has(point)) {
        truncateSync(join(directory, DURABLE_FILES.catalog), 1_048_577);
      }
    });
    store.update(() => [queued()]);
    const path = join(directory, DURABLE_FILES.catalog);
    const original = readFileSync(path);
    growth.add("after_read_open");
    const allocation = vi.spyOn(Buffer, "alloc");
    try {
      expect(() => store.snapshot()).toThrow();
      expect(allocation.mock.calls.every(([size]) => size <= 1_048_577)).toBe(
        true
      );
    } finally {
      allocation.mockRestore();
      growth.clear();
      writeFileSync(path, original);
      store.close();
      guard.release();
    }
  });

  it("refuses a configured directory junction without touching its target", () => {
    const { root } = setup();
    const target = join(root, "target");
    const link = join(root, "link");
    mkdirSync(target);
    writeFileSync(join(target, "sentinel"), "keep");
    symlinkSync(target, link, "junction");
    const guard = new JobRuntimeGuard(link);
    const store = new DurableProductionStore(link, { runtimeGuard: guard });
    expect(() => store.snapshot()).toThrow();
    expect(() => store.update(() => [queued()])).toThrow();
    expect(readFileSync(join(target, "sentinel"), "utf8")).toBe("keep");
    expect(existsSync(join(target, JOB_RUNTIME_MARKER))).toBe(false);
    expect(existsSync(join(target, DURABLE_FILES.directory))).toBe(false);
  });

  it("rejects in-place generation drift while preserving the confirmed owned image", () => {
    const { directory, guard, store } = setup();
    const initial = store.update(() => [queued()]);
    const path = join(directory, DURABLE_FILES.catalog);
    const foreign = canonicalJson({
      ...initial,
      generation: initial.generation + 1,
    });
    writeFileSync(path, foreign);
    expect(() => store.snapshot()).toThrow();
    expect(() => store.update(() => [])).toThrow();
    expect(readFileSync(path, "utf8")).toBe(foreign);
    writeFileSync(path, canonicalJson(initial));
    store.close();
    guard.release();
  });

  it("admits the inclusive 64-record bound and preserves all rows when 65 is rejected", () => {
    const { guard, store } = setup();
    const records = Array.from({ length: 64 }, () => ({
      ...queued(),
      requestId: randomUUID(),
    }));
    const initial = store.update(() => records);
    expect(initial.records).toHaveLength(64);
    expect(() =>
      store.update((current) => [
        ...current,
        { ...queued(), requestId: randomUUID() },
      ])
    ).toThrow(expect.objectContaining({ code: "JOB_REGISTRY_FULL" }));
    expect(store.snapshot()).toEqual(initial);
    store.close();
    guard.release();
  });

  it.each([
    "before_stage_create",
    "after_stage_write",
    "after_stage_fsync",
    "before_rename",
    "before_readback",
  ] as const)(
    "preserves bounded admission evidence at %s and never recovers a missing identity",
    (phase) => {
      const faults = new Set<StoreFaultPoint>([phase]);
      const { directory, guard, store } = setup((point) => {
        if (faults.has(point)) {
          throw new Error("fault");
        }
      });
      expect(() => store.update(() => [queued()])).toThrow(
        expect.objectContaining({
          code: "DEPENDENCY_UNAVAILABLE",
          retryable: false,
        })
      );
      const snapshot = store.snapshot();
      expect(snapshot?.records ?? []).toEqual(
        phase === "before_readback" ? [queued()] : []
      );
      faults.clear();
      store.recover((catalog) => catalog?.records ?? []);
      expect(store.snapshot()?.records ?? []).toEqual(
        phase === "before_readback" ? [queued()] : []
      );
      expect(existsSync(join(directory, DURABLE_FILES.stage))).toBe(false);
      store.close();
      guard.release();
    }
  );

  it("maps occupied shared ownership to safe storage failure while isolated roots remain independent", () => {
    const first = setup(),
      separate = setup();
    const committed = first.store.update(() => [queued()]);
    const second = new DurableProductionStore(first.root, {
      runtimeGuard: new JobRuntimeGuard(first.root),
    });
    expect(second.snapshot()).toEqual(committed);
    expect(() => second.update(() => [])).toThrow(
      expect.objectContaining({
        code: "DEPENDENCY_UNAVAILABLE",
        message: "Durable production storage is unavailable",
        retryable: false,
      })
    );
    expect(separate.store.snapshot()).toBeNull();
    expect(separate.store.update(() => []).records).toEqual([]);
    second.close();
    first.store.close();
    first.guard.release();
    separate.store.close();
    separate.guard.release();
  });

  it("rejects generation overflow without rewriting the retained catalog", () => {
    const { directory, guard, root, store } = setup();
    const initial = store.update(() => [queued()]);
    store.close();
    guard.release();
    const path = join(directory, DURABLE_FILES.catalog);
    const bytes = canonicalJson({
      ...initial,
      generation: Number.MAX_SAFE_INTEGER,
    });
    writeFileSync(path, bytes);
    const freshGuard = new JobRuntimeGuard(root),
      fresh = new DurableProductionStore(root, { runtimeGuard: freshGuard });
    expect(fresh.snapshot()?.generation).toBe(Number.MAX_SAFE_INTEGER);
    expect(() => fresh.update(() => [])).toThrow(
      expect.objectContaining({ code: "DEPENDENCY_UNAVAILABLE" })
    );
    expect(readFileSync(path, "utf8")).toBe(bytes);
    fresh.close();
    freshGuard.release();
  });
  it("reserves terminal headroom for 64 maximum-size legal intents with a bounded fixed stage", () => {
    const base = {
      kind: "export_video",
      request: {
        expectedRevision: 4,
        format: "mp4",
        overwrite: false,
        projectId: "project-original",
        relativePath: ".mp4",
        resolution: "project",
      },
    };
    const remaining = 4096 - Buffer.byteLength(JSON.stringify(base));
    const { intent } = parseAdmission({
      ...fixture.input,
      intent: {
        ...base,
        request: {
          ...base.request,
          relativePath: `${"a".repeat(remaining)}.mp4`,
        },
      },
    });
    const sizes: number[] = [];
    const { directory, guard, store } = setup((point) => {
      if (point === "after_stage_fsync") {
        const bytes = readdirSync(directory).map(
          (name) => statSync(join(directory, name)).size
        );
        expect(
          bytes.every((value) => value <= DURABLE_LIMITS.catalogBytes)
        ).toBe(true);
        sizes.push(bytes.reduce((total, value) => total + value, 0));
      }
    });
    store.update(() =>
      Array.from({ length: 64 }, () => ({
        ...queued(),
        intent,
        intentSha256: intentDigest(intent),
        requestId: randomUUID(),
      }))
    );
    const completed = store.update((records) =>
      records.map((row) => ({
        ...row,
        outcome: {
          format: "mp4",
          kind: "rendered",
          sizeBytes: Number.MAX_SAFE_INTEGER,
        },
        runtimeId: "00000000-0000-4000-8000-000000000003",
        state: "completed",
      }))
    );
    expect(completed.records).toHaveLength(64);
    expect(
      completed.records.every(
        (row) => Buffer.byteLength(canonicalJson(row)) <= 8192
      )
    ).toBe(true);
    expect(sizes).toHaveLength(2);
    expect(sizes.every((value) => value <= 2_097_188)).toBe(true);
    expect(readdirSync(directory).sort()).toEqual(
      [DURABLE_FILES.catalog, JOB_RUNTIME_MARKER].sort()
    );
    store.close();
    guard.release();
  });
});
