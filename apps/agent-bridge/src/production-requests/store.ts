import { randomUUID } from "node:crypto";
import {
  closeSync,
  constants,
  fstatSync,
  fsyncSync,
  lstatSync,
  mkdirSync,
  opendirSync,
  openSync,
  readSync,
  realpathSync,
  renameSync,
  unlinkSync,
  writeSync,
} from "node:fs";
import { join, resolve } from "node:path";
import { TextDecoder } from "node:util";
import { BridgeError } from "../headless";
import {
  JOB_RUNTIME_MARKER,
  type JobRuntimeGuard,
  JobRuntimeGuard as RuntimeGuard,
} from "../job-runtime";
import {
  canonicalJson,
  catalogSchema,
  DURABLE_FILES,
  DURABLE_LIMITS,
  type DurableCatalog,
  type StorageStatus,
  type StoredRecord,
  sha256,
} from "./contract";

export type StoreFaultPoint =
  | "before_stage_create"
  | "after_stage_write"
  | "after_stage_fsync"
  | "before_rename"
  | "after_rename"
  | "before_readback"
  | "after_read_open";

interface Identity {
  dev: bigint;
  ino: bigint;
}
interface Anchor extends Identity {
  path: string;
}
interface StoreOptions {
  fault?: ((point: StoreFaultPoint) => void) | undefined;
  runtimeGuard: JobRuntimeGuard;
  storeId?: () => string;
  write?:
    | ((fd: number, bytes: Buffer, offset: number, length: number) => number)
    | undefined;
}
interface Debt {
  base: DurableCatalog | null;
  baseIdentity: Identity | undefined;
  next: DurableCatalog;
  nextIdentity: Identity | undefined;
}

const unavailable = (cause?: unknown) => {
  let options: ErrorOptions | undefined;
  if (cause !== undefined) {
    options = {
      cause: new Error(
        cause instanceof BridgeError ? cause.code : "Storage operation failed"
      ),
    };
  }
  return new BridgeError(
    "DEPENDENCY_UNAVAILABLE",
    "Durable production storage is unavailable",
    false,
    options
  );
};
const sameIdentity = (left: Identity, right: Identity) =>
  left.dev === right.dev && left.ino === right.ino;
const hasCode = (error: unknown, code: string) =>
  error !== null &&
  typeof error === "object" &&
  "code" in error &&
  error.code === code;
const absent = (error: unknown) => hasCode(error, "ENOENT");
const sameCatalog = (
  left: DurableCatalog | null,
  right: DurableCatalog | null
) => canonicalJson(left) === canonicalJson(right);

/** Application metadata only: native project persistence and artifacts remain core-owned. */
export class DurableProductionStore {
  readonly #work: string;
  readonly #root: string;
  readonly #options: StoreOptions;
  readonly #guard: JobRuntimeGuard;
  #workAnchor: Anchor | undefined;
  #rootAnchor: Anchor | undefined;
  #storeId: string | undefined;
  #catalogIdentity: Identity | undefined;
  #confirmed: DurableCatalog | null | undefined;
  #emptyAtAcquire = false;
  #debt: Debt | undefined;
  #stage: { identity: Identity; expected: Buffer } | undefined;

  constructor(workRoot: string, options: StoreOptions) {
    this.#work = resolve(workRoot);
    this.#root = join(this.#work, DURABLE_FILES.directory);
    this.#options = options;
    this.#guard = new RuntimeGuard(this.#root);
  }

  get hasDebt() {
    return this.#debt !== undefined;
  }

  acquire() {
    try {
      this.#acquireOwned();
    } catch (error) {
      throw unavailable(error);
    }
  }

  #acquireOwned() {
    this.#pinWork();
    this.#options.runtimeGuard.acquire();
    this.#pinWork();
    if (!this.#rootAnchor) {
      try {
        mkdirSync(this.#root, { mode: 0o700 });
      } catch (error) {
        if (!hasCode(error, "EEXIST")) {
          throw unavailable();
        }
      }
    }
    this.#pinRoot();
    const inventory = this.#inventory();
    if (inventory.foreign) {
      throw unavailable();
    }
    this.#emptyAtAcquire ||= inventory.names.length === 0;
    if (!this.#guard.owned) {
      this.#catalogIdentity = undefined;
      this.#confirmed = undefined;
    }
    this.#guard.acquire();
    this.#verify();
  }

  snapshot(): DurableCatalog | null {
    try {
      if (this.#debt) {
        this.#verify();
      }
      const catalog = this.#readCatalog();
      if (
        this.#debt &&
        !sameCatalog(catalog, this.#debt.base) &&
        !sameCatalog(catalog, this.#debt.next)
      ) {
        throw unavailable();
      }
      if (this.#debt && catalog) {
        const expected = sameCatalog(catalog, this.#debt.base)
          ? this.#debt.baseIdentity
          : this.#debt.nextIdentity;
        if (
          !(
            expected &&
            this.#catalogIdentity &&
            sameIdentity(expected, this.#catalogIdentity)
          )
        ) {
          throw unavailable();
        }
      }
      if (this.#debt) {
        this.#verify();
      }
      return catalog;
    } catch (error) {
      throw unavailable(error);
    }
  }

  status(): StorageStatus {
    try {
      const catalog = this.snapshot();
      if (this.#debt) {
        return {
          readable: true,
          reason: "metadata_unconfirmed",
          writable: false,
        };
      }
      if (!catalog) {
        return { readable: true, reason: "not_initialized", writable: false };
      }
      for (const [guard, path] of [
        [this.#options.runtimeGuard, this.#work],
        [this.#guard, this.#root],
      ] as const) {
        if (guard.owned) {
          guard.acquire();
        } else if (this.#exists(join(path, JOB_RUNTIME_MARKER))) {
          return { readable: true, reason: "guard_occupied", writable: false };
        }
      }
      const inventory = this.#inventory();
      if (inventory.foreign || inventory.names.includes(DURABLE_FILES.stage)) {
        return { readable: true, reason: "unsafe_storage", writable: false };
      }
      return { readable: true, reason: null, writable: true };
    } catch {
      return { readable: false, reason: "unsafe_storage", writable: false };
    }
  }

  update(
    change: (records: StoredRecord[]) => readonly StoredRecord[]
  ): DurableCatalog {
    if (this.#debt) {
      throw unavailable();
    }
    this.acquire();
    const base = this.snapshot();
    const records = change(structuredClone(base?.records ?? []));
    if (records.length > DURABLE_LIMITS.records) {
      throw new BridgeError(
        "JOB_REGISTRY_FULL",
        "Durable production registry is full",
        true
      );
    }
    if (base?.generation === Number.MAX_SAFE_INTEGER) {
      throw unavailable();
    }
    const next = catalogSchema.safeParse({
      generation: (base?.generation ?? 0) + 1,
      records,
      rootBinding: sha256(this.#requiredRoot().path),
      storeId: base?.storeId ?? this.#options.storeId?.() ?? randomUUID(),
      version: 1,
    });
    if (!next.success) {
      throw unavailable();
    }
    return this.#commit(base, next.data);
  }

  /** One explicit metadata-only transaction; never creates a missing identity. */
  recover(change: (catalog: DurableCatalog | null) => readonly StoredRecord[]) {
    this.acquire();
    const actual = this.snapshot();
    const records = change(structuredClone(actual));
    const identities = new Set(
      actual?.records.map((record) => record.requestId)
    );
    if (records.some((record) => !identities.has(record.requestId))) {
      throw unavailable();
    }
    this.#cleanOwnedStage();
    const debt = this.#debt;
    this.#debt = undefined;
    this.#confirmed = actual;
    try {
      if (canonicalJson(records) === canonicalJson(actual?.records ?? [])) {
        return actual;
      }
      return this.update(() => records);
    } catch (error) {
      this.#debt ??= debt;
      throw error;
    }
  }

  close() {
    if (!this.#guard.owned) {
      return;
    }
    if (this.#debt) {
      throw unavailable();
    }
    this.#verify();
    this.snapshot();
    const inventory = this.#inventory();
    if (inventory.foreign || inventory.names.includes(DURABLE_FILES.stage)) {
      throw unavailable();
    }
    try {
      this.#guard.release();
    } catch (error) {
      throw unavailable(error);
    }
  }

  #commit(base: DurableCatalog | null, next: DurableCatalog) {
    const baseIdentity = this.#catalogIdentity;
    const bytes = Buffer.from(canonicalJson(next));
    if (
      bytes.length > DURABLE_LIMITS.catalogBytes ||
      next.records.length * DURABLE_LIMITS.recordBytes + 1024 >
        DURABLE_LIMITS.catalogBytes
    ) {
      throw new BridgeError(
        "JOB_REGISTRY_FULL",
        "Durable production registry is full",
        true
      );
    }
    const stage = join(this.#root, DURABLE_FILES.stage);
    if (this.#exists(stage) || this.#inventory().foreign) {
      throw unavailable();
    }
    try {
      this.#options.fault?.("before_stage_create");
      this.#verify();
      this.#writeStage(stage, bytes);
      this.#options.fault?.("before_rename");
      this.#verify();
      if (
        !sameCatalog(
          this.#readCatalog(baseIdentity ? [baseIdentity] : []),
          base
        )
      ) {
        throw unavailable();
      }
      this.#verifyStage(true);
      renameSync(stage, join(this.#root, DURABLE_FILES.catalog));
      this.#options.fault?.("after_rename");
      this.#options.fault?.("before_readback");
      this.#verify();
      if (
        !(
          this.#stage &&
          sameCatalog(this.#readCatalog([this.#stage.identity]), next)
        )
      ) {
        throw unavailable();
      }
      this.#verify();
      this.#stage = undefined;
      this.#confirmed = next;
      return next;
    } catch (error) {
      try {
        this.#options.fault?.("before_readback");
        this.#verify();
        if (
          this.#stage &&
          sameCatalog(this.#readCatalog([this.#stage.identity]), next)
        ) {
          this.#verify();
          this.#stage = undefined;
          this.#confirmed = next;
          return next;
        }
      } catch {
        // The exact write is unconfirmed; preserve bounded evidence and guards.
      }
      this.#debt = {
        base,
        baseIdentity,
        next,
        nextIdentity: this.#stage?.identity,
      };
      throw unavailable(error);
    }
  }

  #writeStage(path: string, bytes: Buffer) {
    const fd = openSync(path, "wx", 0o600);
    try {
      this.#stage = {
        expected: Buffer.from(bytes),
        identity: fstatSync(fd, { bigint: true }),
      };
      let offset = 0;
      while (offset < bytes.length) {
        const written = this.#options.write
          ? this.#options.write(fd, bytes, offset, bytes.length - offset)
          : writeSync(fd, bytes, offset, bytes.length - offset);
        if (written <= 0 || written > bytes.length - offset) {
          throw unavailable();
        }
        offset += written;
      }
      this.#options.fault?.("after_stage_write");
      fsyncSync(fd);
      this.#options.fault?.("after_stage_fsync");
    } finally {
      closeSync(fd);
    }
  }

  #readCatalog(expected?: readonly Identity[]): DurableCatalog | null {
    if (!(this.#pinWork() && this.#pinRoot())) {
      return null;
    }
    const path = join(this.#root, DURABLE_FILES.catalog);
    if (!this.#exists(path)) {
      return this.#missingCatalog();
    }
    let identities = expected;
    if (!identities) {
      if (this.#debt) {
        identities = [this.#debt.baseIdentity, this.#debt.nextIdentity].filter(
          (candidate): candidate is Identity => candidate !== undefined
        );
      } else if (this.#guard.owned && this.#catalogIdentity) {
        identities = [this.#catalogIdentity];
      }
    }
    const { bytes, identity } = this.#readBytes(
      path,
      DURABLE_LIMITS.catalogBytes,
      identities
    );
    const text = new TextDecoder("utf-8", { fatal: true }).decode(bytes);
    const result = catalogSchema.safeParse(JSON.parse(text));
    if (
      !result.success ||
      canonicalJson(result.data) !== text ||
      result.data.rootBinding !== sha256(this.#requiredRoot().path) ||
      (this.#storeId && this.#storeId !== result.data.storeId)
    ) {
      throw unavailable();
    }
    if (this.#guard.owned && !expected && !this.#debt) {
      if (
        this.#confirmed !== undefined &&
        !sameCatalog(result.data, this.#confirmed)
      ) {
        throw unavailable();
      }
      this.#confirmed = result.data;
    }
    this.#storeId = result.data.storeId;
    this.#catalogIdentity = identity;
    return result.data;
  }

  #missingCatalog(): null {
    const inventory = this.#inventory();
    if (this.#storeId || inventory.foreign) {
      throw unavailable();
    }
    const ownedEmpty =
      this.#guard.owned &&
      this.#emptyAtAcquire &&
      inventory.names.every(
        (name) =>
          name === JOB_RUNTIME_MARKER ||
          (name === DURABLE_FILES.stage && this.#stage !== undefined)
      );
    if (inventory.names.length !== 0 && !ownedEmpty) {
      throw unavailable();
    }
    if (this.#guard.owned && this.#confirmed === undefined) {
      this.#confirmed = null;
    }
    return null;
  }

  #readBytes(path: string, limit: number, expected?: readonly Identity[]) {
    try {
      return this.#readVerifiedBytes(path, limit, expected);
    } catch (error) {
      throw unavailable(error);
    }
  }

  #readVerifiedBytes(
    path: string,
    limit: number,
    expected?: readonly Identity[]
  ) {
    this.#verifyAnchors();
    const before = lstatSync(path, { bigint: true });
    if (
      !before.isFile() ||
      before.isSymbolicLink() ||
      before.size > BigInt(limit)
    ) {
      throw unavailable();
    }
    const fd = openSync(path, constants.O_NOFOLLOW ?? constants.O_RDONLY);
    try {
      this.#options.fault?.("after_read_open");
      const held = fstatSync(fd, { bigint: true });
      if (
        !sameIdentity(before, held) ||
        held.size > BigInt(limit) ||
        held.size !== before.size ||
        (expected && !expected.some((identity) => sameIdentity(identity, held)))
      ) {
        throw unavailable();
      }
      this.#verifyAnchors();
      const current = lstatSync(path, { bigint: true });
      if (
        !current.isFile() ||
        current.isSymbolicLink() ||
        !sameIdentity(held, current)
      ) {
        throw unavailable();
      }
      const bytes = Buffer.alloc(Number(held.size) + 1);
      let offset = 0;
      while (offset < bytes.length) {
        const count = readSync(
          fd,
          bytes,
          offset,
          bytes.length - offset,
          offset
        );
        if (count === 0) {
          break;
        }
        offset += count;
      }
      const after = fstatSync(fd, { bigint: true });
      const named = lstatSync(path, { bigint: true });
      this.#verifyAnchors();
      if (
        offset > limit ||
        BigInt(offset) !== held.size ||
        after.size !== held.size ||
        named.size !== held.size ||
        !named.isFile() ||
        named.isSymbolicLink() ||
        !sameIdentity(named, held)
      ) {
        throw unavailable();
      }
      return {
        bytes: bytes.subarray(0, offset),
        identity: { dev: held.dev, ino: held.ino },
      };
    } finally {
      closeSync(fd);
    }
  }

  #verifyStage(complete: boolean) {
    if (!this.#stage) {
      throw unavailable();
    }
    const { bytes } = this.#readBytes(
      join(this.#root, DURABLE_FILES.stage),
      DURABLE_LIMITS.catalogBytes,
      [this.#stage.identity]
    );
    if (
      (complete && bytes.length !== this.#stage.expected.length) ||
      !bytes.equals(this.#stage.expected.subarray(0, bytes.length))
    ) {
      throw unavailable();
    }
  }

  #cleanOwnedStage() {
    const stage = join(this.#root, DURABLE_FILES.stage);
    if (!this.#exists(stage)) {
      this.#stage = undefined;
      return;
    }
    this.#verify();
    this.#verifyStage(false);
    try {
      unlinkSync(stage);
    } catch (error) {
      throw unavailable(error);
    }
    this.#stage = undefined;
  }

  #pinWork() {
    const anchor = this.#directory(this.#work, this.#workAnchor);
    if (anchor) {
      this.#workAnchor = anchor;
    }
    return anchor;
  }

  #pinRoot() {
    const anchor = this.#directory(this.#root, this.#rootAnchor);
    if (anchor) {
      this.#rootAnchor = anchor;
    }
    return anchor;
  }

  #directory(path: string, previous?: Anchor): Anchor | null {
    try {
      const stat = lstatSync(path, { bigint: true });
      if (!stat.isDirectory() || stat.isSymbolicLink()) {
        throw unavailable();
      }
      const anchor = { dev: stat.dev, ino: stat.ino, path: realpathSync(path) };
      if (
        previous &&
        (!sameIdentity(anchor, previous) || anchor.path !== previous.path)
      ) {
        throw unavailable();
      }
      return anchor;
    } catch (error) {
      if (absent(error) && !previous) {
        return null;
      }
      throw unavailable();
    }
  }

  #requiredRoot() {
    if (!this.#rootAnchor) {
      throw unavailable();
    }
    return this.#rootAnchor;
  }

  #verifyAnchors() {
    if (!(this.#pinWork() && this.#pinRoot())) {
      throw unavailable();
    }
  }

  #verify() {
    this.#verifyAnchors();
    if (!(this.#guard.owned && this.#options.runtimeGuard.owned)) {
      throw unavailable();
    }
    try {
      this.#guard.acquire();
      this.#options.runtimeGuard.acquire();
    } catch (error) {
      throw unavailable(error);
    }
  }

  #exists(path: string) {
    try {
      lstatSync(path);
      return true;
    } catch (error) {
      if (absent(error)) {
        return false;
      }
      throw unavailable();
    }
  }

  #inventory() {
    try {
      return this.#readInventory();
    } catch (error) {
      throw unavailable(error);
    }
  }

  #readInventory() {
    const names: string[] = [];
    const allowed = new Set<string>([
      DURABLE_FILES.catalog,
      DURABLE_FILES.stage,
      JOB_RUNTIME_MARKER,
    ]);
    const directory = opendirSync(this.#root);
    try {
      for (
        let entry = directory.readSync();
        entry;
        entry = directory.readSync()
      ) {
        if (!allowed.has(entry.name)) {
          return { foreign: true, names };
        }
        names.push(entry.name);
      }
      return { foreign: false, names };
    } finally {
      directory.closeSync();
    }
  }
}
