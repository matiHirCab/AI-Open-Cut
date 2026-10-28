import { createHash, randomUUID } from "node:crypto";
import { constants } from "node:fs";
import {
  type FileHandle,
  lstat,
  mkdir,
  open,
  readdir,
  realpath,
  rename,
  rmdir,
  unlink,
  writeFile,
} from "node:fs/promises";
import { dirname, isAbsolute, join, relative, resolve, sep } from "node:path";
import { z } from "zod/v4";
import type { BridgeConfig } from "../config";
import { BridgeError, type HeadlessClient } from "../headless";
import { writeResultSchema } from "../schemas";
import {
  ARTWORK_LIMITS,
  artworkManifestSchema,
  artworkParametersSchema,
  artworkPreviewSchema,
} from "./contract";

const names = ["result", "mask", "light", "dark"] as const;
const filenames = ["manifest.json", ...names.map((name) => `${name}.png`)];
const pathSchema = z.string().min(1).max(4096);
export const artworkCliInputSchema = z.discriminatedUnion("operation", [
  z
    .object({
      expectedRevision: z.number().int().nonnegative(),
      operation: z.literal("preview"),
      outputDirectory: pathSchema,
      parameters: artworkParametersSchema,
      projectId: z.string().min(1).max(128),
      sourceAssetId: z.string().min(1).max(128),
    })
    .strict(),
  z
    .object({
      approve: z.literal(true),
      directory: pathSchema,
      operation: z.literal("apply"),
    })
    .strict(),
  z.object({ directory: pathSchema, operation: z.literal("discard") }).strict(),
]);

/** Public JSON envelopes emitted by the existing CLI workflow. */
export const artworkCliOutputSchema = z.union([
  z.strictObject({
    directory: pathSchema,
    manifest: artworkManifestSchema,
    ok: z.literal(true),
    version: z.literal(1),
  }),
  z.strictObject({
    ok: z.literal(true),
    result: writeResultSchema,
    version: z.literal(1),
  }),
  z.strictObject({
    discarded: z.literal(true),
    ok: z.literal(true),
    version: z.literal(1),
  }),
  z.strictObject({
    error: z.strictObject({
      code: z.string().min(1),
      message: z.string(),
      retryable: z.boolean(),
    }),
    ok: z.literal(false),
    version: z.literal(1),
  }),
]);

type Client = Pick<HeadlessClient, "call">;
const hash = (bytes: Uint8Array) =>
  createHash("sha256").update(bytes).digest("hex");
const fail = (message: string): never => {
  throw new BridgeError("INVALID_ARGUMENT", message);
};
const within = (root: string, path: string) => {
  const tail = relative(root, path);
  return (
    tail !== "" &&
    tail !== ".." &&
    !tail.startsWith(`..${sep}`) &&
    !isAbsolute(tail)
  );
};
const checkSignal = (signal?: AbortSignal) => {
  if (signal?.aborted) {
    throw new BridgeError("JOB_CANCELLED", "Artwork operation cancelled");
  }
};

/** Require an explicit export root and reject every symlink in package traversal. */
const packagePath = async (
  config: BridgeConfig,
  value: string,
  existing: boolean
) => {
  if (process.platform !== "linux") {
    throw new BridgeError(
      "DEPENDENCY_UNAVAILABLE",
      "Artwork file review requires the Linux descriptor backend"
    );
  }
  if (!config.exportsDirectory) {
    throw new BridgeError(
      "INVALID_ARGUMENT",
      "Artwork review requires an explicit OPENCUT_EXPORTS_DIR"
    );
  }
  const root = resolve(config.exportsDirectory);
  const rootInfo = await lstat(root);
  if (
    rootInfo.isSymbolicLink() ||
    !rootInfo.isDirectory() ||
    (await realpath(root)) !== root
  ) {
    fail("Artwork review root must be a real directory");
  }
  const target = resolve(config.configRoot, value);
  if (!within(root, target)) {
    throw new BridgeError(
      "PATH_NOT_ALLOWED",
      "Artwork review directory is outside the configured export root"
    );
  }
  const parent = existing ? target : dirname(target);
  let current = root;
  for (const segment of relative(root, parent).split(sep).filter(Boolean)) {
    current = join(current, segment);
    // biome-ignore lint/performance/noAwaitInLoops: Validate each parent before descending into its child.
    const info = await lstat(current);
    if (info.isSymbolicLink() || !info.isDirectory()) {
      fail("Artwork review traversal requires real directories");
    }
  }
  return target;
};

const flagMask = (flags: number[]) =>
  flags.reduce((mask, flag) => {
    // biome-ignore lint/suspicious/noBitwiseOperators: POSIX descriptor flags are a bit mask.
    return mask | flag;
  }, 0);
const directoryFlags = () =>
  flagMask([
    constants.O_RDONLY,
    constants.O_DIRECTORY,
    constants.O_NOFOLLOW,
    constants.O_NONBLOCK,
  ]);
const anchor = (handle: FileHandle) => `/proc/self/fd/${handle.fd}`;
const holdParent = async (path: string) => {
  const handle = await open(dirname(path), directoryFlags());
  try {
    if ((await realpath(anchor(handle))) !== dirname(path)) {
      fail("Artwork parent changed or contains a symbolic link");
    }
    return handle;
  } catch (error) {
    await handle.close();
    throw error;
  }
};
const verifyParent = async (handle: FileHandle, original: string) => {
  const held = await handle.stat();
  const current = await lstat(dirname(original));
  if (
    current.isSymbolicLink() ||
    held.dev !== current.dev ||
    held.ino !== current.ino ||
    (await realpath(anchor(handle))) !== dirname(original)
  ) {
    fail("Artwork parent changed; review output was not published");
  }
};

const boundedRead = async (path: string, limit: number) => {
  const file = await open(
    path,
    flagMask([constants.O_RDONLY, constants.O_NOFOLLOW, constants.O_NONBLOCK])
  );
  try {
    const info = await file.stat();
    if (!info.isFile() || info.size > limit) {
      fail("Artwork review file is invalid or exceeds its byte limit");
    }
    // Read at most limit+1 even if another process grows the open inode.
    const bytes = Buffer.alloc(limit + 1);
    let bytesRead = 0;
    while (bytesRead <= limit) {
      // biome-ignore lint/performance/noAwaitInLoops: A short read determines the next bounded read offset.
      const chunk = await file.read(
        bytes,
        bytesRead,
        limit + 1 - bytesRead,
        bytesRead
      );
      if (chunk.bytesRead === 0) {
        break;
      }
      bytesRead += chunk.bytesRead;
    }
    if (bytesRead > limit) {
      fail("Artwork review file exceeds its byte limit");
    }
    return bytes.subarray(0, bytesRead);
  } finally {
    await file.close();
  }
};

const readPackage = async (directory: string) => {
  const entries = await readdir(directory);
  if (
    entries.length !== filenames.length ||
    entries.some((name) => !filenames.includes(name))
  ) {
    fail("Artwork review package has missing or foreign files");
  }
  const manifest = artworkManifestSchema.parse(
    JSON.parse(
      (
        await boundedRead(
          join(directory, "manifest.json"),
          ARTWORK_LIMITS.manifestBytes
        )
      ).toString("utf8")
    )
  );
  let total = 0;
  for (const name of names) {
    // biome-ignore lint/performance/noAwaitInLoops: Each file consumes the remaining aggregate byte budget.
    const bytes = await boundedRead(
      join(directory, `${name}.png`),
      ARTWORK_LIMITS.reviewBytes - total
    );
    total += bytes.length;
    if (hash(bytes) !== manifest.reviewHashes[name]) {
      fail("Artwork review file changed; prepare and review a new package");
    }
  }
  if (manifest.resultHash !== manifest.reviewHashes.result) {
    fail("Artwork result identity does not match its reviewed image");
  }
  return manifest;
};

const removeOwned = async (directory: string, removePath = directory) => {
  // Revalidate the complete package before deleting any file; never recursive rm.
  const entries = await readdir(directory);
  if (entries.some((name) => !filenames.includes(name))) {
    fail("Foreign files prevent artwork review cleanup");
  }
  for (const name of entries) {
    // biome-ignore lint/performance/noAwaitInLoops: Inspect each owned file before any destructive cleanup.
    const info = await lstat(join(directory, name));
    if (!info.isFile() || info.isSymbolicLink()) {
      fail("Unsafe file prevents artwork review cleanup");
    }
  }
  for (const name of entries) {
    // biome-ignore lint/performance/noAwaitInLoops: Stop cleanup immediately if a prior removal fails.
    await unlink(join(directory, name));
  }
  await rmdir(removePath);
};

const publishPreview = async (
  response: z.infer<typeof artworkPreviewSchema>,
  destination: string,
  parent: FileHandle,
  signal?: AbortSignal
) => {
  const staging = join(anchor(parent), `.opencut-artwork-${randomUUID()}`);
  const published = join(
    anchor(parent),
    relative(dirname(destination), destination)
  );
  let reserved = false;
  let staged = false;
  let stageHandle: FileHandle | undefined;
  try {
    await mkdir(staging, { mode: 0o700 });
    staged = true;
    stageHandle = await open(staging, directoryFlags());
    const stageAnchor = anchor(stageHandle);
    let total = 0;
    for (const name of names) {
      checkSignal(signal);
      const encoded = response.files[name];
      if (encoded.length > Math.ceil(ARTWORK_LIMITS.reviewBytes / 3) * 4) {
        fail("Artwork preview exceeds its byte limit");
      }
      const bytes = Buffer.from(encoded, "base64");
      total += bytes.length;
      if (
        bytes.toString("base64") !== encoded ||
        total > ARTWORK_LIMITS.reviewBytes ||
        hash(bytes) !== response.manifest.reviewHashes[name]
      ) {
        fail("Artwork preview content or hash is invalid");
      }
      // biome-ignore lint/performance/noAwaitInLoops: Validate aggregate bytes before publishing the next staged file.
      await writeFile(join(stageAnchor, `${name}.png`), bytes, {
        flag: "wx",
        mode: 0o600,
      });
    }
    const serialized = `${JSON.stringify(response.manifest, null, 2)}\n`;
    if (Buffer.byteLength(serialized) > ARTWORK_LIMITS.manifestBytes) {
      fail("Artwork manifest exceeds its byte limit");
    }
    await writeFile(join(stageAnchor, "manifest.json"), serialized, {
      flag: "wx",
      mode: 0o600,
    });
    await readPackage(stageAnchor);
    checkSignal(signal);
    // Exclusive reservation rejects an existing destination before publication.
    await verifyParent(parent, destination);
    await mkdir(published, { mode: 0o700 });
    reserved = true;
    await rename(staging, published);
    staged = false;
    reserved = false;
    return {
      directory: destination,
      manifest: response.manifest,
      ok: true,
      version: 1,
    };
  } finally {
    if (staged) {
      await removeOwned(stageHandle ? anchor(stageHandle) : staging, staging);
    }
    await stageHandle?.close();
    if (reserved) {
      await rmdir(published);
    }
  }
};

export const runArtwork = async (
  value: unknown,
  config: BridgeConfig,
  client: Client,
  signal?: AbortSignal
): Promise<unknown> => {
  const input = artworkCliInputSchema.parse(value);
  checkSignal(signal);
  if (input.operation === "preview") {
    const destination = await packagePath(config, input.outputDirectory, false);
    const parent = await holdParent(destination);
    try {
      const response = await client.call(
        {
          expectedRevision: input.expectedRevision,
          operation: "prepare_artwork",
          parameters: input.parameters,
          projectId: input.projectId,
          sourceAssetId: input.sourceAssetId,
        },
        artworkPreviewSchema,
        signal ? { signal } : {}
      );
      checkSignal(signal);
      await verifyParent(parent, destination);
      return await publishPreview(response, destination, parent, signal);
    } finally {
      await parent.close();
    }
  }
  const directory = await packagePath(config, input.directory, true);
  const parent = await holdParent(directory);
  let packageHandle: FileHandle | undefined;
  try {
    await verifyParent(parent, directory);
    const located = join(
      anchor(parent),
      relative(dirname(directory), directory)
    );
    packageHandle = await open(located, directoryFlags());
    const packageAnchor = anchor(packageHandle);
    const manifest = await readPackage(packageAnchor);
    checkSignal(signal);
    if (input.operation === "discard") {
      await verifyParent(parent, directory);
      const held = await packageHandle.stat();
      const current = await lstat(located);
      if (held.dev !== current.dev || held.ino !== current.ino) {
        fail("Artwork review directory changed before discard");
      }
      await removeOwned(packageAnchor, located);
      return { discarded: true, ok: true, version: 1 };
    }
    const result = await client.call(
      { manifest, operation: "commit_artwork" },
      writeResultSchema,
      signal ? { signal } : {}
    );
    return { ok: true, result, version: 1 };
  } finally {
    await packageHandle?.close();
    await parent.close();
  }
};

const readNext = (
  iterator: AsyncIterator<Uint8Array>,
  signal?: AbortSignal
): Promise<IteratorResult<Uint8Array>> =>
  new Promise((resolveNext, rejectNext) => {
    const cancelled = () => {
      signal?.removeEventListener("abort", cancelled);
      rejectNext(new BridgeError("JOB_CANCELLED", "Artwork input cancelled"));
    };
    signal?.addEventListener("abort", cancelled, { once: true });
    if (signal?.aborted) {
      cancelled();
      return;
    }
    Promise.resolve()
      .then(() => iterator.next())
      .then(
        (chunk) => {
          signal?.removeEventListener("abort", cancelled);
          resolveNext(chunk);
        },
        (error: unknown) => {
          signal?.removeEventListener("abort", cancelled);
          rejectNext(error);
        }
      );
  });

export const runArtworkCli = async (
  stream: AsyncIterable<Uint8Array>,
  config: BridgeConfig,
  client: Client,
  output: (value: string) => Promise<void>,
  errors: (value: string) => Promise<void>,
  signal?: AbortSignal
) => {
  try {
    const chunks: Uint8Array[] = [];
    let length = 0;
    const iterator = stream[Symbol.asyncIterator]();
    let complete = false;
    while (!complete) {
      // biome-ignore lint/performance/noAwaitInLoops: EOF, cancellation and accumulated input bounds determine admission.
      const next = await readNext(iterator, signal);
      complete = next.done === true;
      if (next.done) {
        break;
      }
      const chunk = next.value;
      length += chunk.byteLength;
      if (length > ARTWORK_LIMITS.manifestBytes) {
        fail("Artwork CLI input exceeds its byte limit");
      }
      chunks.push(chunk);
    }
    const input = JSON.parse(
      new TextDecoder("utf-8", { fatal: true }).decode(Buffer.concat(chunks))
    );
    const result = await runArtwork(input, config, client, signal);
    await output(`${JSON.stringify(result)}\n`);
    return 0;
  } catch (error) {
    await errors(
      `${JSON.stringify({
        error:
          error instanceof BridgeError
            ? {
                code: error.code,
                message:
                  "Artwork operation failed; inspect request, review package and current project before retry",
                retryable: error.retryable,
              }
            : {
                code: "INVALID_ARGUMENT",
                message:
                  "Artwork request or review package is invalid or unavailable",
                retryable: false,
              },
        ok: false,
        version: 1,
      })}\n`
    );
    return 1;
  }
};
