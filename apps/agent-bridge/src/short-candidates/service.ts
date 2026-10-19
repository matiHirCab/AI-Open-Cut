import type { HeadlessClient } from "../headless";
import { resolvedAssetInputSchema } from "../schemas";
import {
  PlanningError,
  type ShortPlanningSource,
  shortPlanningSourceSchema,
} from "./contract";
import { parsePlanningInput, planShortCandidates } from "./planner";

/** Uses the existing core boundary; the managed path never enters a plan. */
export class ShortCandidatePlanner {
  readonly #headless: Pick<HeadlessClient, "call">;

  constructor(headless: Pick<HeadlessClient, "call">) {
    this.#headless = headless;
  }

  async plan(value: unknown) {
    const input = parsePlanningInput(value);
    const resolve = async (): Promise<ShortPlanningSource> => {
      const asset = await this.#headless.call(
        {
          assetId: input.assetId,
          operation: "resolve_asset_input",
          projectId: input.projectId,
        },
        resolvedAssetInputSchema
      );
      if (!asset.probe?.hasAudio) {
        throw new PlanningError("source");
      }
      const source = shortPlanningSourceSchema.safeParse({
        assetId: asset.assetId,
        contentHash: asset.contentHash,
        durationMs: asset.probe.durationMs,
        projectId: asset.projectId,
        revision: asset.revision,
      });
      if (
        !source.success ||
        source.data.projectId !== input.projectId ||
        source.data.assetId !== input.assetId
      ) {
        throw new PlanningError("source");
      }
      if (source.data.revision !== input.expectedRevision) {
        throw new PlanningError("stale");
      }
      return source.data;
    };
    const before = await resolve();
    const plan = planShortCandidates(input, before);
    const after = await resolve();
    if (JSON.stringify(before) !== JSON.stringify(after)) {
      throw new PlanningError("stale");
    }
    return plan;
  }
}
