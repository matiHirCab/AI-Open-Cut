import { describe, expect, it, vi } from "vitest";
import FIXTURE from "../../../contracts/preview-review-v1.json";
import type { HeadlessRequest } from "../src/headless-contract";
import { RenderWorker } from "../src/render-worker";
import { schemas } from "../src/schemas";
import { registerRenderTools } from "../src/server/render";
import type { Server, ServerDependencies } from "../src/server/shared";

const input = {
  endMs: 1000,
  expectedRevision: 7,
  projectId: "project",
  startMs: 0,
};

describe("audio-enabled preset review contract", () => {
  it("accepts canonical selections and preserves audio and fps choices", () => {
    expect(schemas.previewReviewRange.parse(input)).toEqual({
      ...input,
      includeAudio: true,
      resolution: "project",
    });
    for (const resolution of FIXTURE.validResolutions) {
      expect(
        schemas.previewReviewRange.parse({
          ...input,
          fps: 10,
          includeAudio: false,
          resolution,
        })
      ).toEqual({ ...input, fps: 10, includeAudio: false, resolution });
    }
    for (const resolution of FIXTURE.invalidResolutions) {
      expect(
        schemas.previewReviewRange.safeParse({ ...input, resolution }).success
      ).toBe(false);
    }
    for (const extra of [
      { fps: 0 },
      { fps: 121 },
      { fps: Number.NaN },
      { endMs: 0 },
      { startMs: 1000 },
      { preset: "540p" },
      { resolution: { height: 720, width: 7681 } },
    ]) {
      expect(
        schemas.previewReviewRange.safeParse({ ...input, ...extra }).success
      ).toBe(false);
    }
    expect(
      schemas.previewRenderRange.parse({
        ...input,
        fps: 10,
        resolution: { height: 180, width: 320 },
      }).includeAudio
    ).toBe(false);
  });

  it("forwards typed review selection to the range job without computing settings", async () => {
    const handlers = new Map<string, (input: never) => Promise<unknown>>();
    const server = {
      registerTool: (
        name: string,
        _definition: unknown,
        handler: (input: never) => Promise<unknown>
      ) => handlers.set(name, handler),
    } as unknown as Server;
    const call = vi.fn().mockResolvedValue({
      project: {
        revision: 7,
        settings: { fps: 23, height: 555, width: 999 },
      },
    });
    const start = vi
      .fn()
      .mockReturnValue({ jobId: "review", kind: "preview_range" });
    registerRenderTools(server, {
      headless: { call },
      jobs: { start },
    } as unknown as ServerDependencies);
    await Promise.all(
      FIXTURE.validResolutions.map(async (resolution) => {
        const parsed = schemas.previewReviewRange.parse({
          ...input,
          resolution,
        });
        await handlers.get(FIXTURE.mcpTool)?.(parsed as never);
        expect(start).toHaveBeenCalledWith("preview_range", "project", 7, {
          ...parsed,
          fps: undefined,
          operation: FIXTURE.headlessOperation,
        });
      })
    );
    const parsed = schemas.previewReviewRange.parse({
      ...input,
      fps: 10,
      includeAudio: false,
    });
    await handlers.get(FIXTURE.mcpTool)?.(parsed as never);
    expect(start).toHaveBeenLastCalledWith("preview_range", "project", 7, {
      ...parsed,
      operation: FIXTURE.headlessOperation,
    });
    expect(
      RenderWorker.accepts({
        ...input,
        operation: "render_review_range",
      } satisfies HeadlessRequest)
    ).toBe(true);
    call.mockResolvedValue({ project: { revision: 8 } });
    start.mockClear();
    const conflict = await handlers.get(FIXTURE.mcpTool)?.(parsed as never);
    expect(conflict).toMatchObject({ isError: true });
    expect(start).not.toHaveBeenCalled();
  });
});
