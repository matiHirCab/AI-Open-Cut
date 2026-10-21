import {
  type ChildProcessWithoutNullStreams,
  spawn,
  spawnSync,
} from "node:child_process";
import { existsSync } from "node:fs";
import { readFile } from "node:fs/promises";
import { createInterface } from "node:readline";
import { setTimeout as pause } from "node:timers/promises";

export async function boundedNarrationWait<T>(
  work: Promise<T>,
  milliseconds: number,
  message: string
) {
  let timer: ReturnType<typeof setTimeout> | undefined;
  try {
    return await Promise.race([
      work,
      new Promise<never>((_resolve, reject) => {
        timer = setTimeout(() => reject(new Error(message)), milliseconds);
      }),
    ]);
  } finally {
    clearTimeout(timer);
  }
}

const gone = (error: unknown) =>
  error instanceof Error && "code" in error && error.code === "ESRCH";
const alive = (pid: number) => {
  try {
    process.kill(pid, 0);
    return true;
  } catch (error) {
    if (gone(error)) {
      return false;
    }
    throw error;
  }
};
export class NarrationCliCleanupError extends AggregateError {
  constructor(errors: unknown[]) {
    super(
      errors,
      "Test-owned narration processes did not retire gracefully; preserve the fixture root"
    );
  }
}

// Own a separate process group on POSIX. Emergency termination addresses only
// the spawned CLI tree; normal acceptance requires its graceful close first.
export class NarrationCliProcess {
  readonly child: ChildProcessWithoutNullStreams;
  readonly finished: Promise<{
    code: number | null;
    signal: NodeJS.Signals | null;
  }>;
  readonly #lines: AsyncIterator<string>;
  readonly #workerPidFile: string | undefined;
  #closed = false;
  #stderr = "";
  #stderrBytes = 0;
  #bytes = 0;
  #overflow = false;
  constructor(entrypoint: string, environment: NodeJS.ProcessEnv) {
    this.#workerPidFile = environment.OPENCUT_TEST_WORKER_PID_FILE;
    if (this.#workerPidFile && existsSync(this.#workerPidFile)) {
      throw new Error("Refusing to adopt an existing worker PID fixture");
    }
    this.child = spawn("bun", ["run", entrypoint], {
      detached: process.platform !== "win32",
      env: environment,
      stdio: "pipe",
    });
    this.finished = new Promise((resolve, reject) => {
      this.child.once("error", reject);
      this.child.once("close", (code, signal) => {
        this.#closed = true;
        resolve({ code, signal });
      });
    });
    this.child.stdout.on("data", (chunk: Buffer) => {
      this.#bytes += chunk.length;
      if (this.#bytes > 1_048_576) {
        this.#overflow = true;
        this.child.stdout.destroy();
        this.child.kill();
      }
    });
    this.child.stderr.on("data", (chunk: Buffer) => {
      this.#stderrBytes += chunk.length;
      if (this.#stderrBytes > 1_048_576) {
        this.#overflow = true;
        this.child.stderr.destroy();
        this.child.kill();
        return;
      }
      this.#stderr += chunk.toString();
    });
    this.#lines = createInterface({ input: this.child.stdout })[
      Symbol.asyncIterator
    ]();
  }
  async command(input: unknown): Promise<unknown> {
    this.child.stdin.write(`${JSON.stringify(input)}\n`);
    const line = await boundedNarrationWait(
      Promise.race([
        this.#lines.next(),
        this.finished.then(() => {
          throw new Error(`Source CLI closed before response: ${this.#stderr}`);
        }),
      ]),
      15_000,
      "Source CLI command did not settle"
    );
    if (line.done || this.#overflow) {
      throw new Error("Source CLI response missing or oversized");
    }
    const value: unknown = JSON.parse(line.value);
    if (!value || typeof value !== "object" || !("result" in value)) {
      throw new Error(
        `Source CLI error response: ${JSON.stringify(value)}; ${this.#stderr}`
      );
    }
    return value.result;
  }
  async finish() {
    const result = await boundedNarrationWait(
      this.finished,
      5000,
      "Source CLI did not close owned descriptors"
    );
    // biome-ignore lint/suspicious/noUnnecessaryConditions: Owned stream callbacks update the overflow flag.
    if (this.#overflow) {
      throw new Error("Source CLI output exceeded its acceptance budget");
    }
    return result;
  }
  async #retireLeader() {
    // biome-ignore lint/suspicious/noUnnecessaryConditions: Child close callbacks update lifecycle state.
    if (this.#isClosed()) {
      return;
    }
    this.child.kill("SIGTERM");
    try {
      await this.finish();
    } catch (error) {
      const { pid } = this.child;
      if (pid && !this.#isClosed()) {
        if (process.platform === "win32") {
          spawnSync("taskkill", ["/PID", String(pid), "/T", "/F"], {
            timeout: 5000,
          });
        } else {
          try {
            process.kill(-pid, "SIGKILL");
          } catch (cause) {
            if (!gone(cause)) {
              throw cause;
            }
          }
        }
      }
      await boundedNarrationWait(
        this.finished,
        5000,
        "Owned CLI tree retirement unconfirmed"
      );
      throw error;
    }
  }
  async #retireWorker() {
    if (!this.#workerPidFile) {
      return;
    }
    let text: string;
    try {
      text = await readFile(this.#workerPidFile, "utf8");
    } catch (error) {
      if (
        error instanceof Error &&
        "code" in error &&
        error.code === "ENOENT"
      ) {
        return;
      }
      throw error;
    }
    const pid = Number(text);
    if (!Number.isSafeInteger(pid) || pid <= 0) {
      throw new Error("Invalid test-owned worker identity; root preserved");
    }
    if (!alive(pid)) {
      return;
    }
    process.kill(pid, "SIGKILL");
    const until = Date.now() + 5000;
    while (alive(pid) && Date.now() < until) {
      // biome-ignore lint/performance/noAwaitInLoops: Poll only this test-owned worker until its bounded retirement deadline.
      await pause(25);
    }
    if (alive(pid)) {
      throw new Error("Owned worker retirement unconfirmed; root preserved");
    }
  }
  async dispose() {
    const errors: unknown[] = [];
    try {
      await this.#retireLeader();
    } catch (error) {
      errors.push(error);
    }
    try {
      await this.#retireWorker();
    } catch (error) {
      errors.push(error);
    }
    if (errors.length > 0) {
      throw new NarrationCliCleanupError(errors);
    }
  }
  #isClosed() {
    return this.#closed;
  }
}

export const narrationCleanupUnconfirmed = (error: unknown): boolean =>
  error instanceof NarrationCliCleanupError ||
  (error instanceof AggregateError &&
    error.errors.some(narrationCleanupUnconfirmed));
export async function usingNarrationCli<T>(
  session: NarrationCliProcess,
  run: () => Promise<T>
) {
  const work = await run().then(
    (value) => ({ ok: true, value }) as const,
    (error) => ({ error, ok: false }) as const
  );
  const cleanup = await session.dispose().then(
    () => ({ ok: true }) as const,
    (error) => ({ error, ok: false }) as const
  );
  if (!(work.ok || cleanup.ok)) {
    throw new AggregateError(
      [work.error, cleanup.error],
      "Narration acceptance and cleanup failed"
    );
  }
  if (!work.ok) {
    throw work.error;
  }
  if (!cleanup.ok) {
    throw cleanup.error;
  }
  return work.value;
}
