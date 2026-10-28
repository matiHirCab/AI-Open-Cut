import { createHash } from "node:crypto";
import { describe, expect, it } from "vitest";
import { z } from "zod/v4";
import fixture from "../../../contracts/durable-production-requests-v1.json";
import type { HeadlessClient } from "../src/headless";
import {
  admissionInputSchema,
  canonicalJson,
  catalogSchema,
  parseAdmission,
  publicRecordSchema,
  storedRecordSchema,
} from "../src/production-requests/contract";
import { prepareProduction } from "../src/production-requests/prepare";
import { schemas } from "../src/schemas";

describe("independent durable production v1 contract", () => {
  it("normalizes all four existing render intents from reviewed examples", () => {
    for (const example of fixture.validIntentNormalization) {
      expect(
        parseAdmission({ ...fixture.input, intent: example.input }).intent
      ).toEqual(example.expected);
    }
  });

  it("matches exact independent canonical catalog bytes and digest", () => {
    const example = fixture.independentCatalogExample;
    const catalog = catalogSchema.parse(example.catalog);
    expect(canonicalJson(catalog)).toBe(example.canonicalBytes);
    expect(
      createHash("sha256").update(canonicalJson(catalog)).digest("hex")
    ).toBe(example.canonicalSha256);
  });

  it("accepts every complete independently authored public state", () => {
    for (const record of Object.values(fixture.completePublicSnapshots)) {
      expect(publicRecordSchema.parse(record)).toEqual(record);
    }
  });

  it("rejects raw special keys before they can be discarded", () => {
    for (const key of ["__proto__", "constructor", "prototype", "extra"]) {
      const input = JSON.parse(JSON.stringify(fixture.input));
      Object.defineProperty(input, key, { enumerable: true, value: 1 });
      expect(() => parseAdmission(input)).toThrow();
      const nested = JSON.parse(JSON.stringify(fixture.input));
      Object.defineProperty(nested.intent.request, key, {
        enumerable: true,
        value: 1,
      });
      expect(() => parseAdmission(nested)).toThrow();
      const rangeInput = fixture.validIntentNormalization[1]?.input;
      const resolution = JSON.parse(`{"width":320,"height":180,"${key}":1}`);
      expect(() =>
        parseAdmission({
          ...fixture.input,
          intent: {
            ...rangeInput,
            request: { ...rangeInput?.request, resolution },
          },
        })
      ).toThrow();
    }
  });

  it("advertises existing review defaults as optional input fields", () => {
    const schema = z
      .object({
        properties: z.object({
          intent: z.object({
            anyOf: z.array(
              z.object({
                properties: z.object({
                  kind: z.object({ const: z.string() }),
                  request: z.object({ required: z.array(z.string()) }),
                }),
              })
            ),
          }),
        }),
      })
      .parse(z.toJSONSchema(admissionInputSchema));
    const review = schema.properties.intent.anyOf.find(
      (item) => item.properties.kind.const === "review_range"
    );
    expect(review).toBeDefined();
    expect(new Set(review?.properties.request.required)).toEqual(
      new Set(["projectId", "expectedRevision", "endMs", "startMs"])
    );
  });

  it("rejects incorrect hashes, duplicate identities and future catalogs", () => {
    const original = fixture.independentCatalogExample.catalog;
    const [record] = original.records;
    expect(() =>
      storedRecordSchema.parse({ ...record, intentSha256: "0".repeat(64) })
    ).toThrow();
    expect(() =>
      catalogSchema.parse({ ...original, records: [record, record] })
    ).toThrow();
    expect(() => catalogSchema.parse({ ...original, version: 2 })).toThrow();
  });

  it("does not permit fabricated confirmed outcomes or execution handles", () => {
    const base = fixture.completePublicSnapshots.queued;
    for (const patch of [
      { state: "completed" },
      { state: "uncertain", uncertainty: "previous_runtime" },
      { jobId: "predecessor" },
      { outcome: { format: "png", kind: "rendered", sizeBytes: 123 } },
      { expiresAtMs: base.expiresAtMs + 1 },
    ]) {
      expect(() => publicRecordSchema.parse({ ...base, ...patch })).toThrow();
    }
  });

  it("bounds persisted intent before native work and rejects unknown versions", () => {
    const input = fixture.validIntentNormalization[3]?.input;
    expect(input).toBeDefined();
    const oversized = {
      ...input,
      request: { ...input?.request, relativePath: `${"x".repeat(4096)}.mp4` },
    };
    expect(() =>
      parseAdmission({ ...fixture.input, intent: oversized })
    ).toThrow();
    expect(() => parseAdmission({ ...fixture.input, version: 2 })).toThrow();
  });

  it("accepts exactly 4096 UTF8 bytes and rejects the next byte and multibyte overflow", () => {
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
    const exact = {
      ...base,
      request: {
        ...base.request,
        relativePath: `${"a".repeat(remaining)}.mp4`,
      },
    };
    expect(Buffer.byteLength(JSON.stringify(exact))).toBe(4096);
    expect(parseAdmission({ ...fixture.input, intent: exact }).intent).toEqual(
      exact
    );
    expect(() =>
      parseAdmission({
        ...fixture.input,
        intent: {
          ...exact,
          request: {
            ...exact.request,
            relativePath: `a${exact.request.relativePath}`,
          },
        },
      })
    ).toThrow();
    const unicode = {
      ...base,
      request: {
        ...base.request,
        relativePath: `${"🎬".repeat(Math.floor(remaining / 4) + 1)}.mp4`,
      },
    };
    expect(Buffer.byteLength(JSON.stringify(unicode))).toBeGreaterThan(4096);
    expect(() =>
      parseAdmission({ ...fixture.input, intent: unicode })
    ).toThrow();
  });
  it("keeps legacy export preparation outside the new durable intent byte limit", async () => {
    const relativePath = `${"x".repeat(5000)}.mp4`;
    const request = schemas.projectExportVideo.parse({
      expectedRevision: 4,
      format: "mp4",
      projectId: "project-original",
      relativePath,
      resolution: "720p",
    });
    const headless = {
      call: () => Promise.resolve({ project: { revision: 4 } }),
    } as unknown as HeadlessClient;
    expect(
      await prepareProduction(headless, { kind: "export_video", request })
    ).toEqual({
      kind: "export",
      request: {
        expectedRevision: 4,
        height: 720,
        operation: "export_video",
        overwrite: false,
        projectId: "project-original",
        relativePath,
        width: 1280,
      },
    });
  });
});
