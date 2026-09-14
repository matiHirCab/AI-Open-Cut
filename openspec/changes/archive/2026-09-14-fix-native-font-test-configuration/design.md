## Context

The two native tests in font_resolution.rs fall back to ambient ffmpeg/ffprobe. Ordinary correctness and contract jobs do not install those programs. Existing Transform2D tests distinguish optional unconfigured execution from required native execution. Local verification configured FFmpeg globally and missed the unconfigured case.

## Goals / Non-Goals

Make the two native font tests honor that explicit configuration contract. Preserve every rendering, source-removal, reopen, draft, range and export assertion. Extend the required render-parity command and exact-command enforcement to include this suite, as explicitly approved. Do not change production code, formats, contracts, dependencies or unrelated files.

## Decisions

Append `cargo test -p opencut-editor-core --test font_resolution` to the required native CI step and its exact-command validator. Add mutation tests rejecting removal, substitution and error masking of this command. The existing required environment applies. The initially passing native job did not execute these font tests; its success was not evidence for their CI coverage.

Use a small test-local pure configuration parser modeled on Transform2D. All of OPENCUT_FFMPEG_PATH, OPENCUT_FFPROBE_PATH and OPENCUT_TEST_FONT_PATH absent permits returning without native work only when OPENCUT_GOLDEN_REQUIRED is not 1. Any partial configuration or required mode without configuration fails. Complete configuration must pass readiness and font readability checks; bad paths must never silently skip. Continue using the existing bundled font fixtures for font-resolution assertions.

Run configuration checks before creating native-test projects. Parameterized parser tests avoid process-global environment mutation. Keep production resolution and error contracts unchanged.

Alternatives: installing FFmpeg in every correctness/contract job adds unnecessary platform setup; probing PATH and skipping errors could mask configured native failures; ignoring the tests would remove required coverage. None is selected.

## Risks / Trade-offs

Optional tests report success without executing native assertions when unconfigured; document this and require configured execution evidence separately. A test-local helper duplicates a small established policy, keeping this repair scoped.

## Migration and rollback

No migration, security or privacy impact. Revert this test-only repair to roll back. Schema 19, draft 2 and opencut-text-v2 remain unchanged.

## Verification

Use CI run 34871014552 as existing failing evidence. Add parser coverage first for absent, each partial combination, required missing, and complete configuration. Run font_resolution without native variables, then with Rust 1.97.0 and the established FFmpeg 7.1.1 required configuration. Confirm configured invalid paths fail. Rerun required repository checks and inspect actual CI after pushing. Map results to all delta scenarios in verification.md; do not claim completion while required checks fail.
