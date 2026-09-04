# Transform2D review-fix verification

## Assessment

The four reviewed defects are corrected. No public DTO, persisted schema, canonical catalog, or private-owner dependency edge changed. Schema 8 and legacy rendering remain supported. The initial implementation archive is retained with an explicit correction note.

## Requirement and scenario trace

| Requirement | Scenarios and evidence |
| --- | --- |
| Oriented media source geometry | render_process::geometry_tests::display_orientation_extents_match_backend covers absent orientation, quarter turns, a mirrored matrix, non-quarter turns and fractional thresholds. invalid_metadata_fails_closed rejects unusable data. renderer::tests::oriented_geometry_is_probed_once_and_reused_without_writes checks canonical paths, shared-asset deduplication, typed probe failure, no mutation, and no re-probe during materialization. |
| Complete render preflight before destination inspection | renderer::tests::measured_overflow_precedes_collision_and_metadata_probes uses overflowing text alongside transformed media, both export-collision states, frame/range entry points, missing-reference precedence, zero metadata calls, and valid EXPORT_EXISTS preservation. Existing renderer adapter tests cover unavailable backends and path escapes. display_rotated_video_preserves_extent_and_all_render_intents checks 0/90/180/270/89.5/45-degree media, the full 800-pixel quarter-turn identity extent, asymmetric content, combined noncentral Transform2D, materialized drafts without project/history writes, frame/range/export SSIM >= 0.99, non-silent audio RMS <= 0.0001, and duration within one frame. |
| Portable Transform2D correctness and required native coverage | selected_font_metrics_determine_affine_anchor_before_writes now uses licensed repository DejaVuSans/Serif fixtures. native_configuration_is_explicit_and_required_mode_fails_closed tests absent/partial/required configurations and exact configured paths. The complete native integration binary was executed with FFmpeg removed from PATH and absolute environment paths. CI policy tests reject removed, bypassed, or substituted native Transform2D commands. The existing Windows/Linux/macOS correctness matrix is retained. |

## Implementation coherence

Pure scene preflight retains reference/value validation, while media affine dimensions wait for render preflight. Renderer orchestration resolves managed resource bindings, measures and validates text before any metadata process call, probes each transformed asset once, and gives only typed dimensions to affine finalization. Destination inspection and temporary/workspace allocation occur afterward. Materialization consumes the same scene and resource measurements.

The process adapter bounds FFprobe metadata output to 64 KiB and selects the first video stream. It reads the full display matrix because FFprobe's printed rotation truncates fractional degrees. Matrix angle extraction and rounding match FFmpeg's existing autorotation behavior before quarter-turn axis selection; orientation remains applied exactly once by FFmpeg. Malformed metadata is UNSUPPORTED_MEDIA, inability to start the probe is DEPENDENCY_UNAVAILABLE. No raw metadata, paths, or expressions enter public or persisted contracts.

## Executed verification

- cargo fmt --check --all: PASS.
- cargo clippy --workspace --all-targets -- -D warnings: PASS.
- cargo test --workspace, ordinary configuration: PASS on Windows after rerunning outside the sandbox for the existing process-memory sampler.
- Native Transform2D integration target: 12 PASS, including six display rotations and audiovisual/draft parity.
- Native integration with no FFmpeg on PATH and absolute configured tool paths: 12 PASS.
- Required legacy native golden and native headless lifecycle: PASS with the reviewed font; no golden references changed.
- Bridge typecheck, lint, unit tests: PASS (73 unit tests).
- Bridge contracts:check: PASS (Rust headless protocol and nine TypeScript contract tests).
- MCP integration: PASS (6 tests); packaged smoke: PASS (2 tests).
- Focused CI policy suite: PASS (223 tests, including three new command-protection cases).
- Pinned OpenSpec strict validation: PASS (15 items before archive).
- Final full workspace with required native mode: PASS, including 167 core unit tests, 20 architecture tests, 12 Transform2D tests, and nine headless protocol tests.
- Final Moon archive policy gate: PASS via pinned @moonrepo/cli@2.3.3; 231 policy tests and all 14 living specifications passed. Requirements synchronized and follow-up archived.

Native evidence used local FFmpeg/FFprobe 8.1.2 and checked-in DejaVuSans (the existing reviewed font identity). Ordinary runs intentionally omit native-only execution; the native suite was separately run with required mode and all dependencies. Five existing ignored helper entry points are exercised by their parent harnesses, except the opt-in external report validator, which is not invoked without a report.

## Limitations and resolved check failures

This host is Windows. Linux and macOS correctness jobs cannot be executed locally; their required CI matrix is unchanged and has not been claimed green. No remote push or CI dispatch was performed. Provider/Python surfaces are unchanged, so no provider-specific checks were added.

One initial policy run failed because a Windows edit introduced CRLF into the workflow fixture; the workflow was restored to LF and policy checks pass. One sandboxed workspace run failed in the existing Windows process-memory sampler; its unsandboxed rerun passed. Fractional-angle verification additionally caught FFprobe's truncated rotation output; full-matrix parsing and native 89.5-degree coverage resolve it.

## Image-orientation correction

The original video display-matrix evidence did not cover JPEG EXIF orientation carried on decoded frames. Review reproduced a 20x40 image clipped to 20x20 by identity Transform2D. The follow-up [transform2d-image-orientation](../2026-09-04-transform2d-image-orientation/verification.md) corrects this gap with bounded image-frame inspection and EXIF 1-8 regression coverage. This report remains historical evidence of the earlier checks.

## PR CI correction

PR #104 run 33926771556 passed macOS correctness, Linux native rendering, contracts, smoke, and OpenSpec gates, but Linux/Windows strict Clippy rejected four fixed-size chunks_exact calls in the Transform2D tests. The diagnostics came from Rust 1.98 Clippy; local verification used Rust 1.93. The test iterators now use as_chunks with the same chunk lengths, discarded remainder behavior, pixel classifications, and PCM decoding. No assertions, thresholds, native policy, or production behavior changed. This mechanical correction completes the already-approved strict-Clippy/native-test task; no new requirement or dependency is introduced.
