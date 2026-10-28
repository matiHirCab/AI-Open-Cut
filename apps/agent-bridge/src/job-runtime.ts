import { randomUUID } from "node:crypto";
import {
  closeSync,
  fstatSync,
  fsyncSync,
  lstatSync,
  mkdirSync,
  openSync,
  readSync,
  realpathSync,
  unlinkSync,
  writeSync,
} from "node:fs";
import { join } from "node:path";
import { BridgeError } from "./headless-events";

export const EPHEMERAL_JOBS_CAPABILITY = "ephemeral_jobs_v1";
export const JOB_RUNTIME_MARKER = ".opencut-job-runtime-v1";

const unavailable = () =>
  new BridgeError(
    "JOB_REGISTRY_FULL",
    "Job runtime is in use or requires operator crash reconciliation",
    true
  );

// Admission only: no durable jobs, private payloads, PID-based takeover or orphan adoption.
export class JobRuntimeGuard {
  readonly #root: string;
  #owned: { anchor: string; fd: number; nonce: string } | undefined;

  constructor(root: string) {
    this.#root = root;
  }

  get owned() {
    return this.#owned !== undefined;
  }

  acquire() {
    if (this.#owned) {
      this.#verify(this.#owned);
      return;
    }
    try {
      mkdirSync(this.#root, { recursive: true });
      const root = lstatSync(this.#root);
      if (!root.isDirectory() || root.isSymbolicLink()) {
        throw unavailable();
      }
      const anchor = realpathSync(this.#root);
      const fd = openSync(join(anchor, JOB_RUNTIME_MARKER), "wx+", 0o600);
      const nonce = randomUUID();
      // Keep ownership on failure: a possibly partial guard must still block production.
      this.#owned = { anchor, fd, nonce };
      writeSync(fd, nonce);
      fsyncSync(fd);
    } catch {
      // biome-ignore lint/style/useErrorCause: Never expose private filesystem failures.
      throw unavailable();
    }
  }

  release(cleanupSucceeded = true) {
    if (!cleanupSucceeded) {
      return;
    }
    const owned = this.#owned;
    if (!owned) {
      return;
    }
    this.#verify(owned);
    // Keep the held identity through unlink; failed removal remains owned.
    try {
      unlinkSync(join(owned.anchor, JOB_RUNTIME_MARKER));
      closeSync(owned.fd);
      this.#owned = undefined;
    } catch {
      // biome-ignore lint/style/useErrorCause: Never expose private filesystem failures.
      throw unavailable();
    }
  }

  #verify(owned: { anchor: string; fd: number; nonce: string }) {
    try {
      const root = lstatSync(this.#root);
      if (
        !root.isDirectory() ||
        root.isSymbolicLink() ||
        realpathSync(this.#root) !== owned.anchor
      ) {
        throw unavailable();
      }
      const path = lstatSync(join(owned.anchor, JOB_RUNTIME_MARKER));
      const held = fstatSync(owned.fd);
      const bytes = Buffer.alloc(owned.nonce.length);
      if (
        !path.isFile() ||
        path.isSymbolicLink() ||
        path.dev !== held.dev ||
        path.ino !== held.ino ||
        path.size !== bytes.length ||
        readSync(owned.fd, bytes, 0, bytes.length, 0) !== bytes.length ||
        bytes.toString() !== owned.nonce
      ) {
        throw unavailable();
      }
    } catch {
      // biome-ignore lint/style/useErrorCause: Never expose private filesystem failures.
      throw unavailable();
    }
  }
}
