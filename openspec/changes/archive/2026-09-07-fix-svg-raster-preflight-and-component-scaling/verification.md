# Verification: SVG raster preflight and component scaling

## Approval and scope

User approved the proposal, full delta, design and tasks on 2026-09-07. Implementation stays in editor-core; headless/MCP changes are regression tests. Schema 15, document 1, protocol 1, public catalogs and persisted grammar are unchanged. Existing working-tree changes from issue #29 and the preceding follow-up were preserved. No golden references were regenerated.

## Completeness and correctness

The modified `Explicit SVG geometry and complexity` requirement is implemented by shared `svg_raster_path`, `svg_raster_stroke` and `validate_svg_raster_bounds` checks, complete occurrence `preflight_svg_documents`, deferred `pending_svg` representations and final `refine_scene` compilation. SVG rasterization uses the same conversions. Standalone shape coverage/encoding remains on its existing branch.

| Scenario | Automated evidence |
| --- | --- |
| Preserve viewport and drawing semantics | Existing geometry/viewBox oracles, SVG float fill/stroke/order tests and native SVG conformance |
| Enforce every boundary before expensive work | Existing numeric/work budget tests; mapped surface/precision/remaining-segment and raster-memory tests |
| Continue supported drawing after closepath | Existing `svg_close_continuations_and_normalized_budgets` and canonical SVG contract consumers |
| Count implicit continuation commands before expansion | Existing path/document exact and overflow command tests and atomic ingestion integration tests |
| Render large source geometry through a small viewport | Existing downscaled viewport/pixel regressions and native fixture |
| Enforce actual mapped surface and precision budgets | `svg_mapped_limits_precision_and_remaining_segments`, hidden/unused tests, backend integer/stroke tests and existing expanded-scene bounds |
| Reject finite but backend-unrepresentable mapping | `svg_rejects_backend_integer_overflow`, standalone headless `native_svg_numeric_rejection_preserves_state`, shared MCP workflow and native atomicity test |
| Preserve representable clipping and empty coverage | `svg_backend_bounds_and_stroke_envelopes` and `svg_representable_crossing_and_empty_coverage`: integer endpoints/dimensions, move-only bounds, negative/crossing/offscreen/degenerate fills, caps, joins and dashes |
| Compose cancelling component scales before sizing | `svg_component_scale_cancellation`: exact 100x100 reproduction, identity compiled-geometry comparison, nested groups/instances, legacy/Transform2D and different per-instance scales; native fixture preserves independent pixels with local 100 and outer 0.01 |
| Validate hidden and unreachable occurrence graphs | `svg_component_scale_cancellation`, `svg_hidden_and_unused_content_cannot_bypass_mapped_limits`, existing graph expansion/depth and aggregate limits |
| Preserve rendering lifecycle and failure atomicity | Native SVG frame/range/draft/export and undo/redo/reopen conformance; `svg_invalid_mapping_publishes_no_artifacts` snapshots all project/history/draft/output files; standalone headless and MCP checks preserve state; existing canonical stale-revision and alias/batch tests retained |

The two exact review reproductions failed before implementation and pass after it. An invalid test fixture combining non-default legacy transforms with Transform2D caused an intermediate workspace failure; resetting legacy properties before the explicit Transform2D test resolved it without changing production behavior.

Canonical request/response fixtures need no additions: ingestion acceptance and wire shapes are unchanged, while the changed rejection occurs at evaluated rendering. Existing canonical fixture consumers still pass. Runtime rendering regressions live in the native/headless/MCP suites rather than misclassifying render-invalid source as ingestion-invalid. No governed catalog or consumer declaration changes were introduced by this follow-up.

## Coherence review

Backend bounds are checked before saturating integer conversion, with representable outward-rounded dimensions and checked backend rectangle construction. Conservative stroke envelopes cover caps and miters; dash construction errors cannot select an undashed fallback. Numeric bounds are also checked for contours discarded by PathBuilder. Valid empty coverage remains accepted.

The occurrence walk composes group and outer instance matrices before magnification, checks hidden instances, freezes project reachability before checking every unreachable definition as a virtual root, and guards cycles/depth/occurrences and remaining segment/memory budgets. Intermediate flat evaluation holds only normalized SVG placeholders; no unused component-local surface is compiled or charged. Preflight and final scene compilation use separate passes, not duplicate charges to one scene allocation budget. Existing clocks and normalized validation remain in their owning core paths.

No outstanding implementation/spec/design mismatch was found. Final gate results are recorded below before completion.

## Validation evidence

- Rust formatting and strict workspace Clippy passed. Final `cargo test --workspace -- --test-threads=1` passed: 232 core unit tests (6 existing helper-process tests ignored), all Rust integration/desktop/headless tests, including 22 headless protocol tests. Final required-native focused SVG run passed all 12 tests in 15.34 seconds.
- Bridge `bun run typecheck`, `bun run lint`, `bun run test`: passed, 382 unit tests.
- `bun run contracts:check`: passed, including Rust/headless consumers and 318 TypeScript contract tests.
- `bun run test:integration`: passed, 10 tests; `bun run test:smoke`: passed, 5 packaged tests. Both execute the new failed-render job/state regression through the actual headless process.
- Hermetic Python runner: passed, 10 Kokoro tests and 5 transcription tests.
- Full required native golden render conformance passed in the initial native-configured workspace run. That run's only failure was the subsequently corrected test fixture described above. Production raster and occurrence fixes were already present. Final focused SVG conformance reruns cover the later move-only numeric guard.
- Native headless lifecycle, standalone SVG numeric rejection and all 13 Transform2D tests passed with configured FFmpeg/FFprobe 8.1.2 and DejaVuSans, required mode enabled.
- Strict OpenSpec validation: 22 items passed before archival. `git diff --check` passed; existing CRLF normalization warnings are informational.
- Local ignored logs: `local-data/svg-preflight-workspace.log`, `svg-preflight-workspace-final.log`, `svg-preflight-native-final.log`, `svg-preflight-contracts.log`, `svg-preflight-integration.log`, `svg-preflight-smoke.log`.

## Finalization

Completeness: 15/15 tasks complete. Correctness: the modified requirement and all 11 scenarios have automated evidence. Coherence: no outstanding mismatches or warnings. Accepted deltas were synchronized to the living SVG specification and the change archived on 2026-09-07. The protected `bunx @moonrepo/cli@2.3.3 run root:openspec-validate` gate passed with 21 specifications and CI parity policy validation. No required check is waived.
