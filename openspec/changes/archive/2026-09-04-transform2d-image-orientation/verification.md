# Transform2D image-orientation verification

## Assessment

The approved private image-probe change is implemented. EXIF orientation no longer clips identity Transform2D: the red/yellow 40x20 encoded orientation-6 JPEG preserves its displayed 20x40 extent and all 800 colored pixels. Schema 8, public contracts, persisted probe facts, video stream probing, and FFmpeg autorotation are unchanged.

## Completeness and correctness

| Requirement/scenario | Evidence |
| --- | --- |
| Frame-derived static image geometry; preserve EXIF extent | `exif_images_preserve_extent_and_all_render_intents` generates temporary asymmetric JPEGs with absent orientation and EXIF 1-8. It asserts legacy/identity bounds and exact per-pixel red/yellow/black classification, including all 800 colored pixels. |
| Reject unusable image inspection | `image_frame_metadata_is_authoritative_and_fails_closed` covers frame authority over conflicting stream dimensions, absent/reflected matrices, missing/extra frames, invalid dimensions, malformed/missing matrix fields, and missing executable. `metadata_output_is_bounded_and_read_errors_fail_closed` covers the exact 64 KiB boundary, oversized output stopped at 65,537 bytes, and read failure. Existing video parser cases still pass. |
| Reuse inspected images | `oriented_geometry_is_probed_once_and_reused_without_writes` now exercises both image and video kinds, verifies the kind and canonical path passed to the private adapter, shared-asset deduplication, no materialization re-probe, unchanged project value, and typed probe failure without destination inspection or writes. |
| Preserve overflow precedence and valid collisions | `measured_overflow_precedes_collision_and_metadata_probes` now uses transformed image media alongside overflowing text. Fresh/existing export destinations and frame/range calls fail before probing/writes; missing assets and valid collisions retain their existing errors. |
| EXIF images across render intents | All nine image variants run combined noncentral transforms through materialized draft, frame, range, and export. Draft/frame decoded bytes match, project/history files remain unchanged, frame/range/export SSIM >= 0.99, and duration differs by at most one frame. Existing six video rotations retain their audio RMS <= 0.0001 and timing checks. |

## Coherence

The resolved media binding supplies `MediaType` to the private process port. Image queries use `v:0`, `-read_intervals %+#1`, and frame dimensions/display-matrix fields; video queries retain their stream fields. Only typed dimensions reach affine finalization. Missing orientation is identity, while unusable frame metadata never falls back to encoded geometry. Metadata remains limited to 64 KiB, with child kill/wait cleanup on read failure or excess output. Text validation still precedes any probe.

The native harness uses the existing configured executable/font helper and remains in the already-required Transform2D integration target. No workflow, dependency, or contract edits were needed for this follow-up. Tests inject EXIF using a small Rust helper and generate temporary JPEGs with configured FFmpeg.

Legacy YUV and affine RGBA rendering can round JPEG color conversion differently, so the identity geometry regression checks exact colored-pixel placement rather than byte equality. SSIM comparisons normalize both PNG and video to limited-range YUV420P before comparison, avoiding mismatched input color ranges. The required 0.99 threshold is unchanged. Initial comparisons without normalization failed; the final original red/yellow fixtures pass. No production color or encoding behavior was changed.

## Executed checks

- Rust formatting and strict workspace/all-target Clippy: PASS.
- Ordinary workspace tests: PASS on Windows, including 169 core unit tests, 20 architecture tests, 13 Transform2D tests, and nine headless protocol tests. Native-only cases intentionally skip without configuration; five existing ignored helper entry points retain existing harness policy.
- Required native Transform2D target: 13 PASS, with absent/1-8 EXIF image variants and six video rotation variants.
- Same native binary with PATH restricted to Windows System32 and absolute FFmpeg/FFprobe/font paths: 13 PASS.
- Required native golden conformance and headless lifecycle: PASS; existing golden references unchanged.
- Bridge typecheck/lint/unit: PASS, 73 unit tests.
- Bridge contracts: PASS, nine TypeScript contract tests and headless protocol tests.
- MCP integration and packaged smoke: PASS, six integration tests and two smoke tests.
- Pinned OpenSpec strict validation before archive: 15 items PASS.
- Final Moon policy gate: PASS via pinned @moonrepo/cli@2.3.3 after specification synchronization and archive; all 14 living specifications and CI policy checks passed.

Native checks used configured FFmpeg/FFprobe 8.1.2 and the checked-in DejaVuSans fixture. Python/provider surfaces were unchanged.

## Residual verification limitation

No implementation/spec mismatch remains in local verification. Remote Linux/macOS correctness and Linux native CI have not run for this uncommitted workspace; the three-platform CI matrix remains a pre-merge requirement. This report does not claim those jobs are green. No push, merge, or CI dispatch was performed.
