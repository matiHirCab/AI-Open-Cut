import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  mkdir,
  mkdtemp,
  readdir,
  readFile,
  rename,
  rm,
  symlink,
  writeFile,
} from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, expect, it, vi } from "vitest";
import type { z } from "zod/v4";
import fixture from "../../../contracts/artwork-preparation-v1.json";
import {
  artworkCliOutputSchema,
  runArtwork,
  runArtworkCli,
} from "../src/artwork/cli";
import { ARTWORK_LIMITS, artworkManifestSchema } from "../src/artwork/contract";
import { loadBridgeConfig } from "../src/config";
import { BridgeError } from "../src/headless";
import type { HeadlessRequest } from "../src/headless-contract";

async function* requestStream(value: unknown) {
  yield await Promise.resolve(Buffer.from(JSON.stringify(value)));
}

const roots: string[] = [];
const linuxIt = it.runIf(process.platform === "linux");
afterEach(async () => {
  await Promise.all(
    roots.splice(0).map((path) => rm(path, { force: true, recursive: true }))
  );
});
const setup = async () => {
  const root = await mkdtemp(join(tmpdir(), "opencut-artwork-cli-"));
  roots.push(root);
  const config = loadBridgeConfig({ OPENCUT_EXPORTS_DIR: root });
  const manifest = artworkManifestSchema.parse(
    structuredClone(fixture.manifest)
  );
  const files = { dark: "", light: "", mask: "", result: "" };
  for (const name of ["result", "mask", "light", "dark"] as const) {
    const bytes = Buffer.from(`independently reviewed ${name}`);
    files[name] = bytes.toString("base64");
    manifest.reviewHashes[name] = createHash("sha256")
      .update(bytes)
      .digest("hex");
  }
  manifest.resultHash = manifest.reviewHashes.result;
  const calls: HeadlessRequest[] = [];
  const committed = {
    changedIds: ["new-artwork"],
    projectId: manifest.projectId,
    revision: 3,
    summary: "Committed artwork",
    warnings: [],
  };
  const client = {
    call: <Output>(
      request: HeadlessRequest,
      schema: z.ZodType<Output>
    ): Promise<Output> => {
      calls.push(request);
      return Promise.resolve(
        schema.parse(
          request.operation === "prepare_artwork"
            ? { files, manifest }
            : committed
        )
      );
    },
  };
  const directory = join(root, "review");
  const input = {
    ...fixture.cli.validRequests[0],
    outputDirectory: directory,
  };
  return { calls, client, config, directory, files, input, manifest, root };
};

linuxIt(
  "writes exact bounded review files and commits only the explicitly approved captured manifest",
  async () => {
    const env = await setup();
    const emitted: string[] = [];
    const invoke = (request: unknown) =>
      runArtworkCli(
        requestStream(request),
        env.config,
        env.client,
        (value) => {
          emitted.push(value);
          return Promise.resolve();
        },
        (value) => Promise.reject(new Error(value))
      );
    expect(await invoke(env.input)).toBe(0);
    expect(
      artworkCliOutputSchema.parse(JSON.parse(emitted[0] ?? "null"))
    ).toEqual({
      ...fixture.cli.responses.preview,
      directory: env.directory,
      manifest: env.manifest,
    });
    expect((await readdir(env.directory)).sort()).toEqual([
      "dark.png",
      "light.png",
      "manifest.json",
      "mask.png",
      "result.png",
    ]);
    expect(await readFile(join(env.directory, "mask.png"), "utf8")).toBe(
      "independently reviewed mask"
    );
    await expect(
      runArtwork(
        { directory: env.directory, operation: "apply" },
        env.config,
        env.client
      )
    ).rejects.toThrow();
    expect(env.calls).toHaveLength(1);
    expect(
      await invoke({
        ...fixture.cli.validRequests[1],
        directory: env.directory,
      })
    ).toBe(0);
    const result = artworkCliOutputSchema.parse(
      JSON.parse(emitted[1] ?? "null")
    );
    expect(result).toEqual(fixture.cli.responses.apply);
    expect(env.calls[1]).toEqual({
      manifest: env.manifest,
      operation: "commit_artwork",
    });
    expect(await readdir(env.directory)).toHaveLength(5);
  }
);

linuxIt(
  "discards a verified package without dispatching any project mutation",
  async () => {
    const env = await setup();
    await runArtwork(env.input, env.config, env.client);
    const emitted: string[] = [];
    expect(
      await runArtworkCli(
        requestStream({
          ...fixture.cli.validRequests[2],
          directory: env.directory,
        }),
        env.config,
        env.client,
        (value) => {
          emitted.push(value);
          return Promise.resolve();
        },
        (value) => Promise.reject(new Error(value))
      )
    ).toBe(0);
    expect(
      artworkCliOutputSchema.parse(JSON.parse(emitted[0] ?? "null"))
    ).toEqual(fixture.cli.responses.discard);
    expect(env.calls).toHaveLength(1);
    expect(await readdir(env.root)).toEqual([]);
  }
);

linuxIt.each(["result", "mask", "light", "dark"])(
  "rejects altered %s review before apply or cleanup",
  async (name) => {
    const env = await setup();
    await runArtwork(env.input, env.config, env.client);
    await writeFile(join(env.directory, `${name}.png`), "tampered review");
    await expect(
      runArtwork(
        { approve: true, directory: env.directory, operation: "apply" },
        env.config,
        env.client
      )
    ).rejects.toThrow("changed");
    await expect(
      runArtwork(
        { directory: env.directory, operation: "discard" },
        env.config,
        env.client
      )
    ).rejects.toThrow("changed");
    expect(env.calls).toHaveLength(1);
    expect(await readFile(join(env.directory, `${name}.png`), "utf8")).toBe(
      "tampered review"
    );
  }
);

linuxIt("preserves all files when foreign content appears", async () => {
  const env = await setup();
  await runArtwork(env.input, env.config, env.client);
  await writeFile(join(env.directory, "foreign.txt"), "keep me");
  await expect(
    runArtwork(
      { directory: env.directory, operation: "discard" },
      env.config,
      env.client
    )
  ).rejects.toThrow("foreign");
  expect(await readdir(env.directory)).toHaveLength(6);
  expect(await readFile(join(env.directory, "foreign.txt"), "utf8")).toBe(
    "keep me"
  );
  expect(env.calls).toHaveLength(1);
});

linuxIt(
  "refuses missing explicit export permission, escapes, existing destinations and symlink traversal",
  async () => {
    const env = await setup();
    await expect(
      runArtwork(env.input, loadBridgeConfig({}), env.client)
    ).rejects.toThrow("explicit OPENCUT_EXPORTS_DIR");
    await expect(
      runArtwork(
        { ...env.input, outputDirectory: join(env.root, "..", "escape") },
        env.config,
        env.client
      )
    ).rejects.toThrow("outside");
    expect(env.calls).toHaveLength(0);
    await mkdir(env.directory);
    await writeFile(join(env.directory, "foreign.txt"), "untouched");
    await expect(
      runArtwork(env.input, env.config, env.client)
    ).rejects.toThrow();
    expect(await readFile(join(env.directory, "foreign.txt"), "utf8")).toBe(
      "untouched"
    );
    expect((await readdir(env.root)).sort()).toEqual(["review"]);
    await symlink(env.directory, join(env.root, "link"), "dir");
    await expect(
      runArtwork(
        { ...env.input, outputDirectory: join(env.root, "link", "child") },
        env.config,
        env.client
      )
    ).rejects.toThrow("real directories");
  }
);

linuxIt(
  "rejects symlinked reviewed files while preserving their external target",
  async () => {
    const env = await setup();
    await runArtwork(env.input, env.config, env.client);
    const external = join(env.root, "external");
    await writeFile(external, "keep outside package");
    await rm(join(env.directory, "mask.png"));
    await symlink(external, join(env.directory, "mask.png"));
    await expect(
      runArtwork(
        { approve: true, directory: env.directory, operation: "apply" },
        env.config,
        env.client
      )
    ).rejects.toThrow();
    await expect(
      runArtwork(
        { directory: env.directory, operation: "discard" },
        env.config,
        env.client
      )
    ).rejects.toThrow();
    expect(await readFile(external, "utf8")).toBe("keep outside package");
    expect(env.calls).toHaveLength(1);
  }
);

linuxIt(
  "cancelled calls and malformed/oversized CLI input never reach core",
  async () => {
    const env = await setup();
    const controller = new AbortController();
    controller.abort();
    await expect(
      runArtwork(env.input, env.config, env.client, controller.signal)
    ).rejects.toThrow("cancelled");
    const output = vi.fn(async (_value: string) => undefined);
    const errors = vi.fn(async (_value: string) => undefined);
    async function* input(bytes: Uint8Array) {
      yield await Promise.resolve(bytes);
    }
    for (const bytes of [
      Buffer.from("{"),
      Buffer.alloc(ARTWORK_LIMITS.manifestBytes + 1),
      Buffer.from([0xff]),
    ]) {
      expect(
        // biome-ignore lint/performance/noAwaitInLoops: Verify failures in series with accumulated dispatch evidence.
        await runArtworkCli(
          input(bytes),
          env.config,
          env.client,
          output,
          errors
        )
      ).toBe(1);
    }
    expect(output).not.toHaveBeenCalled();
    expect(errors).toHaveBeenCalledTimes(3);
    for (const [serialized] of errors.mock.calls) {
      expect(
        artworkCliOutputSchema.safeParse(JSON.parse(serialized)).success
      ).toBe(true);
    }
    expect(JSON.parse(errors.mock.calls[0]?.[0] ?? "null")).toEqual(
      fixture.cli.responses.invalid
    );
    expect(env.calls).toHaveLength(0);
  }
);

it("refuses unsupported file backends before any core call or mutation", async () => {
  const env = await setup();
  const descriptor = Object.getOwnPropertyDescriptor(process, "platform");
  try {
    Object.defineProperty(process, "platform", {
      configurable: true,
      value: "win32",
    });
    await expect(
      runArtwork(env.input, env.config, env.client)
    ).rejects.toMatchObject({ code: "DEPENDENCY_UNAVAILABLE" });
    expect(env.calls).toHaveLength(0);
    expect(await readdir(env.root)).toEqual([]);
  } finally {
    if (descriptor) {
      Object.defineProperty(process, "platform", descriptor);
    }
  }
});

linuxIt(
  "refuses a parent replaced by a symlink during preparation without writing outside exports",
  async () => {
    const env = await setup();
    const parent = join(env.root, "parent");
    const foreign = await mkdtemp(join(tmpdir(), "opencut-artwork-foreign-"));
    roots.push(foreign);
    await mkdir(parent);
    const client = {
      call: <Output>(request: HeadlessRequest, schema: z.ZodType<Output>) =>
        rename(parent, join(env.root, "captured-parent"))
          .then(() => symlink(foreign, parent, "dir"))
          .then(() => env.client.call(request, schema)),
    };
    await expect(
      runArtwork(
        { ...env.input, outputDirectory: join(parent, "review") },
        env.config,
        client
      )
    ).rejects.toThrow("parent changed");
    expect(await readdir(foreign)).toEqual([]);
    expect(await readdir(join(env.root, "captured-parent"))).toEqual([]);
  }
);

linuxIt(
  "rejects FIFO review inputs promptly without opening a producer or deleting files",
  async () => {
    const env = await setup();
    await runArtwork(env.input, env.config, env.client);
    await rm(join(env.directory, "mask.png"));
    const made = spawnSync("mkfifo", [join(env.directory, "mask.png")]);
    expect(made.status).toBe(0);
    await expect(
      runArtwork(
        { approve: true, directory: env.directory, operation: "apply" },
        env.config,
        env.client
      )
    ).rejects.toThrow("invalid");
    expect(env.calls).toHaveLength(1);
    expect(await readdir(env.directory)).toHaveLength(5);
  }
);

it("cancels while stdin remains idle and never waits for EOF or dispatches", async () => {
  const env = await setup();
  const controller = new AbortController();
  const idle: AsyncIterable<Uint8Array> = {
    [Symbol.asyncIterator]: () => ({
      next: () => new Promise(() => undefined),
    }),
  };
  const output = vi.fn((_value: string) => Promise.resolve());
  const errors = vi.fn((_value: string) => Promise.resolve());
  const running = runArtworkCli(
    idle,
    env.config,
    env.client,
    output,
    errors,
    controller.signal
  );
  controller.abort();
  expect(await running).toBe(1);
  expect(errors.mock.calls[0]?.[0]).toContain("JOB_CANCELLED");
  expect(output).not.toHaveBeenCalled();
  expect(env.calls).toHaveLength(0);
});

linuxIt(
  "cleans a partial staged preview when a later review hash fails",
  async () => {
    const env = await setup();
    env.files.dark = Buffer.from("wrong final image").toString("base64");
    await expect(runArtwork(env.input, env.config, env.client)).rejects.toThrow(
      "hash is invalid"
    );
    expect(await readdir(env.root)).toEqual([]);
    expect(env.calls.map((request) => request.operation)).toEqual([
      "prepare_artwork",
    ]);
  }
);

linuxIt(
  "discards late producer completion after cancellation without publishing output",
  async () => {
    const env = await setup();
    const controller = new AbortController();
    const client = {
      call: <Output>(request: HeadlessRequest, schema: z.ZodType<Output>) => {
        controller.abort();
        return env.client.call(request, schema);
      },
    };
    await expect(
      runArtwork(env.input, env.config, client, controller.signal)
    ).rejects.toThrow("cancelled");
    expect(await readdir(env.root)).toEqual([]);
    expect(env.calls.map((request) => request.operation)).toEqual([
      "prepare_artwork",
    ]);
  }
);

linuxIt(
  "rejects a missing reviewed image before commit or deletion of remaining files",
  async () => {
    const env = await setup();
    await runArtwork(env.input, env.config, env.client);
    await rm(join(env.directory, "mask.png"));
    await expect(
      runArtwork(
        { approve: true, directory: env.directory, operation: "apply" },
        env.config,
        env.client
      )
    ).rejects.toThrow("missing");
    await expect(
      runArtwork(
        { directory: env.directory, operation: "discard" },
        env.config,
        env.client
      )
    ).rejects.toThrow("missing");
    expect(await readdir(env.directory)).toHaveLength(4);
    expect(env.calls).toHaveLength(1);
  }
);

linuxIt(
  "serializes the canonical core failure envelope without retrying intent",
  async () => {
    const env = await setup();
    await runArtwork(env.input, env.config, env.client);
    const calls: HeadlessRequest[] = [];
    const client = {
      call: <Output>(
        request: HeadlessRequest,
        _schema: z.ZodType<Output>
      ): Promise<Output> => {
        calls.push(request);
        return Promise.reject(
          new BridgeError("REVISION_CONFLICT", "private core detail", true)
        );
      },
    };
    const emitted: string[] = [];
    const errors: string[] = [];
    expect(
      await runArtworkCli(
        requestStream({
          ...fixture.cli.validRequests[1],
          directory: env.directory,
        }),
        env.config,
        client,
        (value) => {
          emitted.push(value);
          return Promise.resolve();
        },
        (value) => {
          errors.push(value);
          return Promise.resolve();
        }
      )
    ).toBe(1);
    expect(emitted).toEqual([]);
    expect(errors).toHaveLength(1);
    expect(
      artworkCliOutputSchema.parse(JSON.parse(errors[0] ?? "null"))
    ).toEqual(fixture.cli.responses.coreFailure);
    expect(calls).toEqual([
      { manifest: env.manifest, operation: "commit_artwork" },
    ]);
    expect(await readdir(env.directory)).toHaveLength(5);
  }
);
