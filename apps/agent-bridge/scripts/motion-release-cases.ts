export const RELEASE_AREAS = [
  "depth",
  "svg",
  "repeater",
  "particle",
  "render-plan",
  "preview",
  "memory",
  "timing",
] as const;
export interface ReleaseCase {
  area: (typeof RELEASE_AREAS)[number];
  ignored: boolean;
  selector: string;
  target: string;
}
const lib = (
  area: ReleaseCase["area"],
  selector: string,
  ignored = false
): ReleaseCase => ({ area, ignored, selector, target: "lib" });
const integration = (
  area: ReleaseCase["area"],
  target: string,
  selector: string
): ReleaseCase => ({ area, ignored: false, selector, target });

/** Private verification inventory; project limits remain in EditorCore. */
export const RELEASE_CASES: readonly ReleaseCase[] = [
  integration(
    "depth",
    "components",
    "full_graph_scope_identity_and_longest_path_boundaries"
  ),
  lib(
    "depth",
    "evaluated_scene::mattes::tests::continuous_duration_one_depth_32_preserves_weighted_samples_without_provenance_product"
  ),
  lib(
    "svg",
    "validation::svg::tests::every_numeric_and_work_budget_is_bounded"
  ),
  lib(
    "svg",
    "validation::svg::tests::hostile_xml_and_unsupported_features_never_normalize"
  ),
  integration(
    "repeater",
    "repeaters",
    "strict_and_semantic_descriptor_failures"
  ),
  lib(
    "repeater",
    "evaluated_scene::instance_tests::repeater_preflight_accepts_exact_occurrence_limit_and_rejects_one_over"
  ),
  lib(
    "particle",
    "validation::extended_visual::release_limit_tests::particle_parameters_accept_inclusive_limits_and_reject_excess_or_non_finite"
  ),
  lib(
    "particle",
    "evaluated_scene::extended_certification::tests::scene_effect_work_is_cumulative_and_overflow_fails_closed"
  ),
  lib(
    "render-plan",
    "render_plan::tests::non_empty_semantic_plan_is_identical_across_render_intents"
  ),
  lib(
    "render-plan",
    "evaluated_scene::temporal_fixture_tests::epic6_source_limits_fail_before_source_callbacks_or_output"
  ),
  integration(
    "render-plan",
    "reference_scene",
    "complete_reference_coverage_rejects_removed_groups_operations_overrides_and_bindings"
  ),
  lib(
    "preview",
    "render_artifact::preview_cache::tests::inclusive_limits_lru_accounting_oversized_and_replacement"
  ),
  lib(
    "preview",
    "render_artifact::preview_cache::tests::clones_concurrent_inserts_and_poison_keep_bounded_immutable_payloads"
  ),
  lib(
    "memory",
    "evaluated_scene::mattes::tests::normal_matte_metadata_exact_shared_limit_succeeds_and_excess_refuses_before_reservation"
  ),
  lib("memory", "renderer::golden::process_tree_sampler_isolated_helper", true),
  lib(
    "timing",
    "evaluated_scene::tests::missing_non_finite_and_invalid_timing_fail_closed"
  ),
  lib(
    "timing",
    "renderer::golden::process_tree_sampler_observation_timeout_reaps_child"
  ),
  lib(
    "timing",
    "renderer::golden::process_tree_sampler_child_exit_and_readiness_timeout_are_bounded"
  ),
];

export const requireReleaseInventory = (value: unknown) => {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new Error("Missing release case inventory");
  }
  const inventory = value as Record<string, unknown>;
  if (
    Object.keys(inventory).sort().join(",") !== "cases,version" ||
    inventory.version !== 1 ||
    !Array.isArray(inventory.cases) ||
    inventory.cases.length !== RELEASE_CASES.length
  ) {
    throw new Error("Incomplete release case inventory");
  }
  inventory.cases.forEach((candidate: unknown, index: number) => {
    if (
      typeof candidate !== "object" ||
      candidate === null ||
      Array.isArray(candidate)
    ) {
      throw new Error("Malformed release case");
    }
    const row = candidate as Record<string, unknown>;
    const expected = RELEASE_CASES[index];
    if (
      !expected ||
      Object.keys(row).sort().join(",") !== "area,ignored,selector,target" ||
      !Object.entries(expected).every(([key, field]) => row[key] === field)
    ) {
      throw new Error("Unknown, duplicate or substituted release case");
    }
  });
};

export const releaseCaseArguments = (row: ReleaseCase) => [
  "cargo",
  "test",
  "--release",
  "--no-default-features",
  "-p",
  "opencut-editor-core",
  ...(row.target === "lib" ? ["--lib"] : ["--test", row.target]),
  row.selector,
  "--",
  "--exact",
  "--color",
  "never",
  ...(row.ignored ? ["--ignored"] : []),
];

const SUCCESS_RESULT = /test result: ok\. 1 passed; 0 failed; 0 ignored;/u;
const FAILURE_RESULT =
  /test result: FAILED|\b[1-9][0-9]* failed;|\b[1-9][0-9]* ignored;/u;
/** Named execution and result are both required; Cargo zero-test success is refusal. */
export const requireExactRustCase = (
  selector: string,
  output: string,
  exitCode: number
) => {
  const escaped = selector.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
  const passed = new RegExp(`^test ${escaped} \\.\\.\\. ok\\r?$`, "gmu");
  if (
    exitCode !== 0 ||
    [...output.matchAll(passed)].length !== 1 ||
    !SUCCESS_RESULT.test(output) ||
    FAILURE_RESULT.test(output)
  ) {
    throw new Error(
      `Required release case did not execute and pass: ${selector}`
    );
  }
};
