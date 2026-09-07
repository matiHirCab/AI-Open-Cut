# Verification: SVG coordinate conversion precision

## Approval and scope

User explicitly approved the proposal, complete delta, design and tasks on 2026-09-07. Production changes are restricted to editor-core's shared SVG raster-point conversion. Tests and SVG documentation were updated. No API, schema, migration, canonical ingestion fixture, dependency edge, standalone shape encoder or curve-subdivision behavior changed. Existing issue #29 working-tree changes were preserved; no golden references were regenerated.

## Requirement and scenario traceability

The modified `Explicit SVG geometry and complexity` requirement is implemented by `svg_raster_point` and its call in `EvaluatedShape::svg_raster_path`, before path insertion and bounds accumulation. Evaluation and artifact preparation already share this path. The named conversion threshold is distinct from the unchanged curve tolerance.

| New scenario | Automated evidence |
| --- | --- |
| Reject diagonal geometry collapsed by float conversion | `svg_rejects_diagonal_conversion_loss`, both public rejection suites and native artifact atomicity; the exact reproduction failed before implementation |
| Accept equivalent representable diagonal geometry | `svg_precision_control_has_independent_band_pixels` checks exact PAM pixels at (50,55), (50,45), (50,65); `svg_precision_control_native` checks decoded native pixels within two channel values |
| Enforce inclusive Euclidean conversion error | `svg_conversion_precision_boundaries_and_sampling`: signed endpoints, exact 0.25, next f64 above it, combined X/Y displacement, non-finite inputs and conversion overflow, offscreen/move-only contours and exactly representable large points |
| Measure conversion error in raster-space units | ViewBox/density cases in the conversion test and `svg_component_sampling_precision` through actual nested component evaluation, including hidden instances |
| Preserve public state and artifacts on precision rejection | `native_svg_numeric_rejection_preserves_state`; shared MCP workflow in integration and packaged smoke; `svg_invalid_mapping_publishes_no_artifacts` snapshots current/history/draft/output files for frame/range/draft/export failures |

The delta retains the preceding 11 scenarios unchanged. Existing geometry/viewBox, path/document command boundaries, opacity, closepath, mapped surface/segment/memory limits, backend integer/stroke bounds, clipping/empty coverage, component scale/ancestry, hidden/unreachable content, canonical atomic edits and native lifecycle suites remain their coverage. The older large-bound test now uses exactly representable coordinates and zero raster origin so it tests integer/stroke bounds without violating the new conversion rule.

Each public rejection source is tested independently. Native/headless helpers create separate projects. MCP undoes the inserted invalid SVG after each render assertion so the next case cannot be masked by the preceding invalid document. Existing stale-revision and alias/batch evidence remains intact. No governed fixture/declaration update or additional CODEOWNER review is required because canonical ingestion and wire shapes are unchanged.

## Correctness and coherence review

Finite f64 raster coordinates are retained while the f32 round trip is measured with f64 hypot. Non-finite original/converted coordinates are explicitly rejected; the displacement comparison is strictly greater than 0.25, preserving equality. Both coordinates contribute to one Euclidean displacement. No path or bounds update precedes the check, including contours the backend might discard. Existing integer/stroke/dash/component/resource checks still run. No extra per-document buffer is introduced.

The approved choice is a conversion-error threshold, not exact representability or a combined flattening-plus-conversion guarantee. This limitation is stated in code, design and public documentation. No outstanding implementation/design/specification mismatch was identified.

## Verification evidence

- The exact diagonal rejection regression failed before the implementation and passed afterward.
- Rust formatting and strict workspace Clippy passed. Final workspace results are recorded at finalization below.
- Required native configuration used FFmpeg/FFprobe 8.1.2 and the pinned DejaVu font. Full native golden conformance passed within the native-configured workspace run (equivalent coverage to its exact test selector). The final focused SVG run passed all 17 tests; both exact headless native lifecycle/rejection checks passed; all 13 Transform2D integration tests passed with native rendering enabled.
- A concurrent workspace rebuild encountered Windows LNK1104 because the preceding native test executable was still running. The retry waits for that executable to exit; no code or build configuration change was needed.
- Bridge typecheck passed; lint passed after applying the repository formatter to the edited workflow; all 382 unit tests passed.
- Contract parity passed, including governed Rust/headless consumers and 318 TypeScript contract tests.
- MCP integration passed all 10 tests; packaged smoke passed all 5 tests. Both run both numeric rejection cases through the actual headless process.
- Hermetic Python runner passed 10 Kokoro tests and 5 transcription tests.
- The new native control test initially lacked the preview directory and used a decoder for a different canvas size. Its setup was corrected to the existing 240x120 native canvas while preserving the SVG's 100x100 viewport; the independent pixel check then passed. These were test setup corrections, not production changes.
- Strict OpenSpec validation passed all 22 pre-archive items; `git diff --check` passed with only existing informational CRLF normalization warnings.
- Ignored local logs: `local-data/svg-precision-workspace-native.log`, `svg-precision-workspace-final.log`, `svg-precision-focused-native.log`, `svg-precision-contracts.log`, `svg-precision-integration.log`, `svg-precision-smoke.log`.

## Finalization

Final workspace retry passed with 237 editor-core unit tests, 12 desktop tests, 22 headless protocol tests and all workspace integration suites. Six existing helper tests remain intentionally ignored in the general runner; required native conformance was explicitly exercised with native configuration. The initial native workspace run passed full golden conformance and all other executed tests except the subsequently corrected control-test setup; the final native SVG run passed all 17 tests after that correction. No required check is waived.

| Dimension | Result |
| --- | --- |
| Completeness | 13/13 tasks complete, including synchronization, archival and the protected gate |
| Correctness | Modified requirement covered; all 16 scenarios mapped to new or retained automated coverage |
| Coherence | Approved design followed; no critical issues, warnings or suggestions remain |

Verification is complete. The modified requirement and five new scenarios were synchronized into openspec/specs/svg-ingestion/spec.md, preserving existing requirements and scenarios. The change was archived on 2026-09-07. The unchanged protected Moon gate passed: 231 policy tests, 21 strict specification checks and CI parity policy validation. All required checks passed; no mismatches remain.
