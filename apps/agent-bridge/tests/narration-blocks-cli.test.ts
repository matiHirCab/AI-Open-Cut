import { expect, it, vi } from "vitest";
import { runNarrationCli } from "../src/narration-blocks/cli";

const setup = () => {
  const workflow = {
    commit: vi.fn(),
    discard: vi.fn(),
    plan: vi.fn(),
    preview: vi.fn(),
    publicationStatus: vi.fn(),
    reconcile: vi.fn(),
  };
  const output: string[] = [];
  const run = async (chunks: Uint8Array[]) => {
    async function* stream() {
      await Promise.resolve(); // Model the asynchronous provider/transport boundary.

      yield* chunks;
    }
    const status = await runNarrationCli(
      stream(),
      workflow,
      async (line) => {
        await Promise.resolve(); // Model the asynchronous provider/transport boundary.

        output.push(line);
      },
      () => ({
        markNonCancellable: vi.fn(),
        onProgress: vi.fn(),
        signal: new AbortController().signal,
      })
    );
    return { lines: output.map((line) => JSON.parse(line)), status };
  };
  return { run, workflow };
};
const encode = (value: string) => new TextEncoder().encode(value);

it("retains a session across split UTF8 chunks and dispatches each command once", async () => {
  const f = setup();
  f.workflow.discard.mockResolvedValue({ discarded: true });
  const bytes = encode(
    '{"command":"discard","groupId":"😀"}\n{"command":"abandon","groupId":"other"}'
  );
  const result = await f.run([
    bytes.subarray(0, 34),
    bytes.subarray(34, 35),
    bytes.subarray(35),
  ]);
  expect(result.status).toBe(0);
  expect(result.lines).toEqual([
    { result: { discarded: true } },
    { result: { discarded: true } },
  ]);
  expect(f.workflow.discard.mock.calls).toEqual([["😀"], ["other", true]]);
});

it.each([
  '{"command":"discard","groupId":"x","extra":true}',
  '{"command":"discard","groupId":"x","__proto__":{"polluted":true}}',
  '{"command":"discard","groupId":null}',
  '{"command":"preview","plan":null}',
  '{"command":"commit","groupId":"x","projectId":"p","expectedRevision":0,"trackId":"t","startMs":0,"markerPolicy":null}',
  '{"command":"unknown"}',
  "null",
])(
  "rejects malformed or extra command shape before workflow: %s",
  async (raw) => {
    const f = setup();
    const result = await f.run([encode(`${raw}\n`)]);
    expect(result.status).toBe(1);
    expect(result.lines[0]).toMatchObject({
      error: { code: "INVALID_ARGUMENT" },
    });
    for (const operation of Object.values(f.workflow)) {
      expect(operation).not.toHaveBeenCalled();
    }
  }
);

it("rejects invalid UTF8 but processes the next valid line", async () => {
  const f = setup();
  f.workflow.discard.mockResolvedValue({ discarded: true });
  const result = await f.run([
    new Uint8Array([255, 10]),
    encode('{"command":"discard","groupId":"ok"}\n'),
  ]);
  expect(result.status).toBe(1);
  expect(result.lines).toHaveLength(2);
  expect(f.workflow.discard).toHaveBeenCalledOnce();
});

it("bounds an unterminated input line before parsing or dispatch", async () => {
  const f = setup();
  const result = await f.run([
    new Uint8Array(16 * 1024 * 1024).fill(32),
    encode(" "),
  ]);
  expect(result.status).toBe(1);
  expect(result.lines).toHaveLength(1);
  expect(f.workflow.plan).not.toHaveBeenCalled();
});

it("bounds a workflow result before serialization and reports no private payload", async () => {
  const f = setup();
  f.workflow.discard.mockResolvedValue({
    oversized: "x".repeat(1024 * 1024),
    privatePath: "/private/owned.wav",
  });
  const result = await f.run([encode('{"command":"discard","groupId":"x"}\n')]);
  expect(result.status).toBe(1);
  expect(result.lines).toHaveLength(1);
  expect(JSON.stringify(result.lines)).not.toContain("/private");
  expect(result.lines[0]).toHaveProperty("error");
});

it("attempts native transport closure after failed provider retirement and retains crash admission", async () => {
  const { closeNarrationSession } = await import(
    "../src/narration-blocks/lifecycle"
  );
  const failure = new Error("shutdown unconfirmed");
  const workflow = { close: vi.fn(() => Promise.reject(failure)) };
  const headless = { close: vi.fn(() => Promise.resolve()) };
  const guard = { release: vi.fn() };
  const result = await closeNarrationSession({ guard, headless, workflow });
  expect(result).toEqual({
    cleanupConfirmed: false,
    errors: [failure],
    publicationUnknown: false,
  });
  expect(headless.close).toHaveBeenCalledOnce();
  expect(guard.release).toHaveBeenCalledWith(false);
});

it("settles preview ownership before native close and confirms runtime release only after both", async () => {
  const { closeNarrationSession } = await import(
    "../src/narration-blocks/lifecycle"
  );
  const settled = Promise.withResolvers<{ publicationUnknown: boolean }>();
  const order: string[] = [];
  const closing = closeNarrationSession({
    guard: {
      release: (confirmed) => {
        order.push(`guard:${confirmed}`);
      },
    },
    headless: {
      close: () => {
        order.push("native");
        return Promise.resolve();
      },
    },
    workflow: { close: () => settled.promise },
  });
  expect(order).toEqual([]);
  settled.resolve({ publicationUnknown: true });
  expect((await closing).publicationUnknown).toBe(true);
  expect(order).toEqual(["native", "guard:true"]);
});
