import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { chmod, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import contract from "../../../contracts/channel-style-packs-v1.json";
import ownership from "../../../contracts/contract-ownership-v1.json";
import { runChannelCli } from "../src/channel-packs/cli";
import {
  CHANNEL_CLI_BYTES,
  channelApplyResultSchema,
  channelCaptureSchema,
  channelListSchema,
  channelPackSchema,
  channelProfileRefSchema,
  channelRequestSchema,
} from "../src/channel-packs/contract";
import type { HeadlessClient } from "../src/headless";
import {
  assertChannelContractConsumer,
  verifyChannelPackContract,
} from "./channel-packs-contract";
import {
  removeChannelPackAdditions,
  restoreChannelPackRaw,
} from "./fixtures/channel-packs-projection";
import { restoreShortPlanningOwnershipRaw } from "./fixtures/short-candidates-projection";

const stream = async function* (value: string | Uint8Array) {
  yield await Promise.resolve(
    typeof value === "string" ? Buffer.from(value) : value
  );
};
const saved = {
  id: "00000000-0000-4000-8000-000000000211",
  package: contract.package,
  version: 1,
};
const call = async (
  args: string[],
  input: string | Uint8Array,
  response: unknown = saved
) => {
  const requests: unknown[] = [];
  const stdout: string[] = [];
  const stderr: string[] = [];
  const headless: Pick<HeadlessClient, "call"> = {
    call: (request, schema) => {
      requests.push(request);
      return Promise.resolve(schema.parse(response));
    },
  };
  const status = await runChannelCli(
    args,
    stream(input),
    headless,
    (value) => {
      stdout.push(value);
      return Promise.resolve();
    },
    (value) => {
      stderr.push(value);
      return Promise.resolve();
    }
  );
  return { requests, status, stderr, stdout };
};

describe("channel packs", () => {
  it(
    "consumes manually frozen native wire/package shapes and exact raw predecessors",
    verifyChannelPackContract
  );

  it("rejects unknown own special keys throughout new closed records before native effects", async () => {
    const capture = contract.requests.find(
      (request) => request.operation === "channel_pack_capture"
    )?.capture;
    const apply = contract.requests.find(
      (request) => request.operation === "channel_pack_apply"
    );
    const profile = { expectedVersion: 1, profileId: saved.id };
    const withImage = {
      ...apply,
      images: [{ assetId: "managed_image", dependencyKey: "art" }],
    };
    const withProfile = {
      ...apply,
      narration: { mode: "replacement", reference: profile },
    };
    const withCaption = {
      ...capture,
      captionRole: { recipeKey: "cp1_card", textSlotId: "title" },
    };
    const appliedResult = {
      captionRole: { componentId: "component", textSlotId: "title" },
      components: [
        {
          afterDigest: "a".repeat(64),
          beforeDigest: null,
          changed: true,
          componentId: "component",
          recipeKey: "cp1_card",
        },
      ],
      narrationProfile: null,
      pack: { id: saved.id, name: "Original", version: 1 },
      write: {
        aliases: {},
        changedIds: [],
        projectId: "project",
        revision: 1,
        summary: "Applied",
        warnings: [],
      },
    };
    const metadata = {
      packs: [
        {
          fontCount: 0,
          hasCaptionRole: false,
          hasNarrationProfile: false,
          id: saved.id,
          imageCount: 0,
          name: "Original",
          packageBytes: 300,
          recipeCount: 1,
          version: 1,
        },
      ],
    };

    const cases = [
      { path: ["images", "0"], schema: channelRequestSchema, value: withImage },
      {
        path: ["narration", "reference"],
        schema: channelRequestSchema,
        value: withProfile,
      },
      {
        path: ["captionRole"],
        schema: channelCaptureSchema,
        value: withCaption,
      },
      { path: [], schema: channelApplyResultSchema, value: appliedResult },
      {
        path: ["components", "0"],
        schema: channelApplyResultSchema,
        value: appliedResult,
      },
      {
        path: ["captionRole"],
        schema: channelApplyResultSchema,
        value: appliedResult,
      },
      {
        path: ["pack"],
        schema: channelApplyResultSchema,
        value: appliedResult,
      },
      { path: [], schema: channelListSchema, value: metadata },
      { path: ["packs", "0"], schema: channelListSchema, value: metadata },
      { path: [], schema: channelPackSchema, value: saved },
      { path: ["package"], schema: channelPackSchema, value: saved },
      {
        path: ["package", "recipes", "0"],
        schema: channelPackSchema,
        value: saved,
      },
      { path: [], schema: channelCaptureSchema, value: capture },
      { path: [], schema: channelProfileRefSchema, value: profile },
      { path: [], schema: channelRequestSchema, value: apply },
      { path: ["narration"], schema: channelRequestSchema, value: apply },
      { path: ["application"], schema: channelRequestSchema, value: apply },
      {
        path: ["application", "placements", "0"],
        schema: channelRequestSchema,
        value: apply,
      },
    ];
    for (const key of ["__proto__", "constructor", "toString"]) {
      for (const example of cases) {
        expect(example.schema.safeParse(example.value).success).toBe(true);
        const value = JSON.parse(JSON.stringify(example.value));
        let selected = value as Record<string, unknown>;
        for (const part of example.path) {
          selected = selected[part] as Record<string, unknown>;
        }
        Object.defineProperty(selected, key, { enumerable: true, value: true });
        expect(
          example.schema.safeParse(JSON.parse(JSON.stringify(value))).success
        ).toBe(false);
      }
    }
    await Promise.all(
      ["__proto__", "constructor", "toString"].map(async (key) => {
        const result = await call(["list"], `{${JSON.stringify(key)}:true}`);
        expect(result.status).toBe(1);
        expect(result.requests).toEqual([]);
        expect(result.stdout).toEqual([]);
      })
    );
  });

  it("forwards inherited native structures and literal slot own keys without silent normalization", async () => {
    const request = contract.requests.find(
      (value) => value.operation === "channel_pack_apply"
    );
    const response = {
      captionRole: null,
      components: [],
      narrationProfile: null,
      pack: { id: saved.id, name: "Original", version: 1 },
      write: {
        changedIds: [],
        projectId: "target",
        revision: 1,
        summary: "Applied",
        warnings: [],
      },
    };
    await Promise.all(
      ["__proto__", "constructor", "toString"].map(async (key) => {
        const { operation: _operation, ...value } = JSON.parse(
          JSON.stringify(request)
        );
        const [placement] = value.application.placements;
        placement.parent = { id: "parent_item", scope: "project" };
        placement.transform2d = {
          anchor: { x: 0, y: 0 },
          opacity: 1,
          position: { unit: "pixels", x: 0, y: 0 },
          rotationDeg: 0,
          scaleX: 1,
          scaleY: 1,
          skewXDeg: 0,
          skewYDeg: 0,
        };
        for (const native of [
          placement.parent,
          placement.transform,
          placement.transform2d,
          placement.transform2d.anchor,
          placement.transform2d.position,
        ]) {
          Object.defineProperty(native, key, { enumerable: true, value: true });
        }
        Object.defineProperty(placement.slotValues, key, {
          enumerable: true,
          value: { type: "text", value: "Literal native slot" },
        });
        const result = await call(["apply"], JSON.stringify(value), response);
        if (key !== "__proto__") {
          expect(result.status).toBe(1);
          expect(result.requests).toEqual([]);
          return;
        }
        expect(result.status).toBe(0);
        // Structural adapters forward exact own data; actual Serde/native
        // validators reject unknown native fields, and retain legitimate maps.
        expect(result.requests).toEqual([
          { ...value, operation: "channel_pack_apply" },
        ]);
        const forwarded = JSON.parse(JSON.stringify(result.requests[0]));
        expect(
          Object.hasOwn(forwarded.application.placements[0].slotValues, key)
        ).toBe(true);
        expect(
          Object.hasOwn(
            forwarded.application.placements[0].transform2d.anchor,
            key
          )
        ).toBe(true);
      })
    );
  });

  it("preserves every legitimate literal native slot map key", () => {
    const request = contract.requests.find(
      (value) => value.operation === "channel_pack_apply"
    );
    for (const key of ["__proto__", "constructor", "toString"]) {
      const value = JSON.parse(JSON.stringify(request));
      const [placement] = value.application.placements;
      Object.defineProperty(placement.slotValues, key, {
        enumerable: true,
        value: { type: "text", value: "Native literal" },
      });
      const parsed = channelRequestSchema.parse(
        JSON.parse(JSON.stringify(value))
      );
      expect(parsed).toEqual(value);
      expect(
        Object.hasOwn(
          JSON.parse(JSON.stringify(parsed)).application.placements[0]
            .slotValues,
          key
        )
      ).toBe(true);
    }
  });

  it.runIf(process.platform !== "win32")(
    "interrupts the real source CLI and closes its detached native group and inherited producer",
    async () => {
      const root = await mkdtemp(join(tmpdir(), "opencut-channel-interrupt-"));
      const executable = join(root, "native-probe.mjs");
      const pidsPath = join(root, "owned-pids.json");
      await writeFile(
        executable,
        `#!${process.execPath}
import { spawn } from 'node:child_process';
import { writeFileSync } from 'node:fs';
const producer = spawn(process.execPath, ['-e', 'setInterval(() => {}, 1000)'], { stdio: 'ignore' });
writeFileSync(process.env.OPENCUT_CHANNEL_TEST_PIDS, JSON.stringify({ native: process.pid, producer: producer.pid }));
process.stdin.resume();
setInterval(() => {}, 1000);
`
      );
      await chmod(executable, 0o755);
      const child = spawn(
        "bun",
        [
          fileURLToPath(
            new URL("../src/channel-packs-cli.ts", import.meta.url)
          ),
          "get",
        ],
        {
          env: {
            ...process.env,
            OPENCUT_ALLOWED_MEDIA_DIRS: root,
            OPENCUT_CHANNEL_TEST_PIDS: pidsPath,
            OPENCUT_EXPORTS_DIR: join(root, "exports"),
            OPENCUT_HEADLESS_PATH: executable,
            OPENCUT_PROJECTS_DIR: join(root, "projects"),
          },
          stdio: "pipe",
        }
      );
      const closed = new Promise<void>((resolve) =>
        child.once("close", () => resolve())
      );
      child.stdin.end(JSON.stringify({ packId: saved.id }));
      child.stdout.resume();
      child.stderr.resume();
      let owned: { native: number; producer: number } | undefined;
      const alive = (pid: number) => {
        try {
          process.kill(pid, 0);
          return true;
        } catch {
          return false;
        }
      };
      try {
        await expect
          .poll(
            async () => {
              try {
                owned = JSON.parse(await readFile(pidsPath, "utf8"));
                return true;
              } catch {
                return false;
              }
            },
            { timeout: 10_000 }
          )
          .toBe(true);
        expect(owned).toBeDefined();
        const native = owned?.native;
        const producer = owned?.producer;
        expect(typeof native).toBe("number");
        expect(typeof producer).toBe("number");
        if (!(native && producer)) {
          throw new Error("Missing positively owned native topology");
        }
        expect(alive(native)).toBe(true);
        expect(alive(producer)).toBe(true);
        child.kill("SIGTERM");
        await expect.poll(() => child.exitCode, { timeout: 10_000 }).toBe(1);
        await closed;
        await expect
          .poll(() => alive(native) || alive(producer), { timeout: 10_000 })
          .toBe(false);
      } finally {
        // Failure cleanup targets only the native group explicitly recorded by
        // this test-owned probe, then its positively owned CLI handle.
        if (owned) {
          try {
            process.kill(-owned.native, "SIGKILL");
          } catch {
            /* owned group may outlive its leader */
          }
        }
        if (child.exitCode === null && child.signalCode === null) {
          child.kill("SIGKILL");
        }
        await closed;
        if (owned) {
          const recorded = owned;
          await expect
            .poll(() => alive(recorded.native) || alive(recorded.producer), {
              timeout: 10_000,
            })
            .toBe(false);
          await rm(root, { force: true, recursive: true });
        }
      }
    },
    30_000
  );

  it("preserves raw import JSON for duplicate-aware core validation and explicit naming", async () => {
    const raw = '{"name":"first","name":"second","script":"reject in core"}';
    const result = await call(["import", "Explicit"], raw);
    expect(result.status).toBe(0);
    expect(result.requests).toEqual([
      { name: "Explicit", operation: "channel_pack_import", packageJson: raw },
    ]);
    expect(JSON.parse(result.stdout[0] ?? "")).toEqual(saved);
  });

  it("exports only saved portable data without schema-default transformations", async () => {
    expect(channelPackSchema.parse(saved)).toEqual(saved);
    const result = await call(["export"], JSON.stringify({ packId: saved.id }));
    expect(result.status).toBe(0);
    expect(JSON.parse(result.stdout[0] ?? "")).toEqual(contract.package);
    expect(result.stderr).toEqual([]);
  });

  it("rejects invalid command/wrapper/UTF-8 and oversized input before native effects", async () => {
    await Promise.all(
      (
        [
          [["unknown"], "{}"],
          [["import"], "{}"],
          [["get"], "[]"],
          [["get"], '{"packId":"x","operation":"channel_pack_list"}'],
          [["list"], '{"script":"execute"}'],
          [["list"], new Uint8Array([255])],
          [["list"], " ".repeat(CHANNEL_CLI_BYTES + 1)],
        ] as const
      ).map(async ([args, raw]) => {
        const result = await call([...args], raw);
        expect(result.status).toBe(1);
        expect(result.requests).toEqual([]);
        expect(result.stdout).toEqual([]);
        expect(JSON.parse(result.stderr[0] ?? "").error.code).toBe(
          "INVALID_ARGUMENT"
        );
      })
    );
  });

  it("bounds output before emission without a false rollback claim or automatic retry", async () => {
    const response = {
      packs: [
        {
          fontCount: 0,
          hasCaptionRole: false,
          hasNarrationProfile: false,
          id: "x",
          imageCount: 0,
          name: "x".repeat(CHANNEL_CLI_BYTES),
          packageBytes: 1,
          recipeCount: 1,
          version: 1,
        },
      ],
    };
    const result = await call(["list"], "", response);
    expect(result.status).toBe(1);
    expect(result.requests).toHaveLength(1);
    expect(result.stdout).toEqual([]);
    expect(JSON.parse(result.stderr[0] ?? "").error.message).toContain(
      "effects may have completed"
    );
  });

  it("detects missing/skipped/comment-only/masked canonical callbacks", () => {
    const source =
      'import { verifyChannelPackContract } from "./channel-packs-contract";\nit("CP1", verifyChannelPackContract);';
    expect(() => assertChannelContractConsumer(source)).not.toThrow();
    for (const invalid of [
      "",
      `// ${source}`,
      source.replace('it("CP1"', 'it.skip("CP1"'),
      source.replace("verifyChannelPackContract);", "() => true);"),
    ]) {
      expect(() => assertChannelContractConsumer(invalid)).toThrow();
    }
  });

  it("preserves both independent ownership additions in either inverse order", () => {
    const current = readFileSync(
      new URL("../../../contracts/contract-ownership-v1.json", import.meta.url),
      "utf8"
    );
    const foundation = readFileSync(
      new URL(
        "./fixtures/short-candidates-predecessor-ownership.raw",
        import.meta.url
      ),
      "utf8"
    );
    for (const raw of [current, JSON.stringify(ownership)]) {
      const a = restoreShortPlanningOwnershipRaw(
        restoreChannelPackRaw(raw, "contract-ownership-v1")
      );
      const b = restoreChannelPackRaw(
        restoreShortPlanningOwnershipRaw(raw),
        "contract-ownership-v1"
      );
      expect(a).toBe(b);
      expect(a).toBe(
        raw === current ? foundation : JSON.stringify(JSON.parse(foundation))
      );
    }
  });

  it("preserves unrelated root/numeric/category bytes and rejects malformed owned additions", () => {
    const raw = readFileSync(
      new URL("../../../contracts/contract-ownership-v1.json", import.meta.url),
      "utf8"
    ).replace("{\n", '{\n  "unrelated": 1e2,\n');
    const restored = restoreChannelPackRaw(raw, "contract-ownership-v1");
    expect(restored).toContain('"unrelated": 1e2');
    const changed = structuredClone(ownership);
    changed.categories.channelStylePacks.consumers.pop();
    expect(() =>
      removeChannelPackAdditions(changed, "contract-ownership-v1")
    ).toThrow();
    const absent = { categories: { unrelated: null } };
    expect(removeChannelPackAdditions(absent, "contract-ownership-v1")).toBe(
      absent
    );
    const malformed = { categories: { channelStylePacks: null } };
    expect(() =>
      removeChannelPackAdditions(malformed, "contract-ownership-v1")
    ).toThrow();
    expect(
      createHash("sha256")
        .update(
          readFileSync(
            new URL(
              "./fixtures/channel-packs-predecessor-ownership.raw",
              import.meta.url
            )
          )
        )
        .digest("hex")
    ).toBe(contract.predecessorOwnershipSha256);
  });
});
