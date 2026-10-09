import { Client } from "@modelcontextprotocol/client";
import { AjvJsonSchemaValidator } from "@modelcontextprotocol/client/validators/ajv";
import { InMemoryTransport, McpServer } from "@modelcontextprotocol/server";
import { expect, it, vi } from "vitest";
import { z } from "zod/v4";
import { memoizedSdkValidator } from "./fixtures/memoized-sdk-validator";

const schema = {
  additionalProperties: false,
  properties: { email: { format: "email", type: "string" } },
  required: ["email"],
  type: "object",
};

it("interprets published-style nested unions and formats without weakening validation", () => {
  const nested = {
    $schema: "https://json-schema.org/draft/2020-12/schema",
    additionalProperties: false,
    properties: {
      email: { format: "email", type: "string" },
      entries: {
        items: {
          oneOf: [
            {
              additionalProperties: false,
              properties: { id: { format: "uuid", type: "string" } },
              required: ["id"],
              type: "object",
            },
            { maximum: 10, minimum: 0, type: "integer" },
          ],
        },
        maxItems: 2,
        minItems: 1,
        type: "array",
      },
    },
    required: ["email", "entries"],
    type: "object",
  };
  const interpreted = memoizedSdkValidator().getValidator(nested);
  const original = new AjvJsonSchemaValidator().getValidator(nested);
  for (const value of [
    {
      email: "a@example.com",
      entries: [0, { id: "123e4567-e89b-12d3-a456-426614174000" }],
    },
    {},
    { email: "invalid", entries: [1] },
    { email: "a@example.com", entries: [] },
    { email: "a@example.com", entries: [1, 2, 3] },
    { email: "a@example.com", entries: [-1] },
    { email: "a@example.com", entries: [1.5] },
    { email: "a@example.com", entries: [{ id: "invalid" }] },
    {
      email: "a@example.com",
      entries: [{ extra: 1, id: "123e4567-e89b-12d3-a456-426614174000" }],
    },
    { email: "a@example.com", entries: [1], extra: 1 },
  ]) {
    expect(interpreted(value).valid).toBe(original(value).valid);
  }
  expect(interpreted({ email: "a@example.com", entries: [1] }).valid).toBe(
    true
  );
  expect(interpreted({}).valid).toBe(false);
});

it("reuses only exact serialized successful default validators", () => {
  const provider = new AjvJsonSchemaValidator();
  const compile = vi.spyOn(provider, "getValidator");
  const memo = memoizedSdkValidator(provider);
  const first = memo.getValidator(schema);
  expect(memo.getValidator(structuredClone(schema))).toBe(first);
  memo.getValidator({ ...schema, title: "different" });
  const reordered = Object.fromEntries(Object.entries(schema).reverse());
  expect(JSON.stringify(reordered)).not.toBe(JSON.stringify(schema));
  memo.getValidator(reordered);
  expect(compile).toHaveBeenCalledTimes(3);
  expect(first({ email: "a@example.com" }).valid).toBe(true);
  expect(first({ email: "invalid" }).valid).toBe(false);
  expect(first({ email: "a@example.com", extra: 1 }).valid).toBe(false);
  expect(first({}).valid).toBe(false);
  expect(compile).toHaveBeenCalledTimes(3);
});

it("isolates clients, evicts the oldest of 78 entries and clears", () => {
  const provider = new AjvJsonSchemaValidator();
  const compile = vi.spyOn(provider, "getValidator");
  const memo = memoizedSdkValidator(provider);
  for (let i = 0; i < 78; i += 1) {
    memo.getValidator({ title: String(i), type: "string" });
  }
  memo.getValidator({ title: "0", type: "string" });
  expect(compile).toHaveBeenCalledTimes(78);
  memo.getValidator({ title: "78", type: "string" });
  memo.getValidator({ title: "1", type: "string" });
  expect(compile).toHaveBeenCalledTimes(79);
  memo.getValidator({ title: "0", type: "string" });
  expect(compile).toHaveBeenCalledTimes(80);
  memoizedSdkValidator(provider).getValidator({ title: "0", type: "string" });
  expect(compile).toHaveBeenCalledTimes(81);
  memo.clear();
  memo.getValidator({ title: "0", type: "string" });
  expect(compile).toHaveBeenCalledTimes(82);
});

it("retries thrown compilation errors including undefined without caching", () => {
  const provider = new AjvJsonSchemaValidator();
  const compile = vi.spyOn(provider, "getValidator");
  const memo = memoizedSdkValidator(provider);
  compile.mockImplementationOnce(() => {
    throw new Error("compile");
  });
  expect(() => memo.getValidator(schema)).toThrow("compile");
  compile.mockImplementationOnce(() => {
    const failure = undefined as unknown as Error;
    throw failure;
  });
  let caught = false;
  try {
    memo.getValidator(schema);
  } catch (error) {
    caught = true;
    expect(error).toBeUndefined();
  }
  expect(caught).toBe(true);
  memo.getValidator(schema);
  memo.getValidator(schema);
  expect(compile).toHaveBeenCalledTimes(3);
});

it("delegates serialization failure directly to the same provider", () => {
  const provider = new AjvJsonSchemaValidator();
  const compile = vi.spyOn(provider, "getValidator");
  const memo = memoizedSdkValidator(provider);
  const cyclic: Record<string, unknown> = { type: "string" };
  cyclic.self = cyclic;
  for (let i = 0; i < 2; i += 1) {
    try {
      memo.getValidator(cyclic);
    } catch {
      // Any default-provider error remains the provider's own result.
    }
  }
  expect(compile).toHaveBeenCalledTimes(2);
  expect(compile).toHaveBeenNthCalledWith(1, cyclic);
  expect(compile).toHaveBeenNthCalledWith(2, cyclic);
});

it("keeps real SDK output validation after live catalog refresh", async () => {
  const provider = new AjvJsonSchemaValidator();
  const compile = vi.spyOn(provider, "getValidator");
  const memo = memoizedSdkValidator(provider);
  const server = new McpServer({ name: "memo-server", version: "1" });
  let output: Record<string, unknown> = { email: "a@example.com" };
  server.registerTool(
    "email",
    {
      inputSchema: z.object({}),
      outputSchema: z.object({ email: z.email() }).strict(),
    },
    () => ({ content: [], structuredContent: { email: "a@example.com" } })
  );
  const client = new Client(
    { name: "memo-client", version: "1" },
    { jsonSchemaValidator: memo }
  );
  const [serverTransport, clientTransport] =
    InMemoryTransport.createLinkedPair();
  try {
    await server.connect(serverTransport);
    await client.connect(clientTransport);
    expect((await client.listTools()).tools.map((tool) => tool.name)).toEqual([
      "email",
    ]);
    expect(
      (await client.callTool({ arguments: {}, name: "email" }))
        .structuredContent
    ).toEqual(output);
    // Bypass the server's own result guard only in this negative control:
    // the real client must independently reject malformed wire output.
    const originalSend = serverTransport.send.bind(serverTransport);
    serverTransport.send = (message, options) => {
      if ("result" in message && "structuredContent" in message.result) {
        return originalSend(
          {
            ...message,
            result: { ...message.result, structuredContent: output },
          },
          options
        );
      }
      return originalSend(message, options);
    };
    output = { email: "invalid" };
    await expect(
      client.callTool({ arguments: {}, name: "email" })
    ).rejects.toMatchObject({
      code: -32_602,
      message: expect.stringContaining(
        "Structured content does not match the tool's output schema"
      ),
    });
    output = { email: "a@example.com", extra: 1 };
    await expect(
      client.callTool({ arguments: {}, name: "email" })
    ).rejects.toMatchObject({
      code: -32_602,
      message: expect.stringContaining(
        "Structured content does not match the tool's output schema"
      ),
    });
    output = { email: "b@example.com" };
    await client.listTools(undefined, { cacheMode: "refresh" });
    expect(
      (await client.callTool({ arguments: {}, name: "email" }))
        .structuredContent
    ).toEqual(output);
    expect(compile).toHaveBeenCalledTimes(1);
  } finally {
    try {
      await client.close();
      await server.close();
    } finally {
      memo.clear();
    }
  }
});
