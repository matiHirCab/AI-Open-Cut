## Context

The reviewed native tests pass locally, but ordinary workspace tests lack native configuration, the worker test requires `raster-cache-test-hooks`, and the bridge test requires `OPENCUT_RASTER_CACHE_TESTS_REQUIRED=1`. The Linux render job currently selects neither cache suite. Its exact six-step sequence and command/environment allowlists are enforced by `validate-ci-gates.ts`; changing YAML alone must fail.

## Goals / Non-Goals

**Goals:** Make the existing core, native worker and real bridge reuse evidence mandatory within the existing required Render parity status; preserve all protected policy guarantees and restore the default binary before compatibility verification.

**Non-Goals:** Runtime changes, new dependencies or versions, golden changes, relaxed tests, new status names, platform expansion, commits, pushes or remote workflow dispatch.

## Decisions

1. Extend the existing render leaf rather than add an optional job. Its reviewed sequence becomes checkout, deterministic rendering dependencies, pinned toolchain, locked JavaScript dependency installation in `apps/agent-bridge`, existing native audiovisual/lifecycle parity, native raster-cache parity, strict report validation, report upload. The foundation aggregate remains unchanged. Separate unrequired CI was rejected because it would not close the review gap.

2. The new native cache step runs these commands serially from the repository root, propagating every failure:

   - `cargo test -p opencut-editor-core --lib raster_cach`
   - `cargo test -p opencut-headless --features raster-cache-test-hooks --test render_worker`
   - `cargo build -p opencut-headless --features raster-cache-test-hooks`
   - `bun run --cwd apps/agent-bridge test:unit --no-file-parallelism tests/render-worker-native.test.ts`
   - `cargo build -p opencut-headless`
   - `cargo test -p opencut-headless`

   Pinned Bun 1.4.0 supports the declared run --cwd syntax; the complete sequence is exercised during verification. Native cache environment is exactly `OPENCUT_FFMPEG_PATH=ffmpeg`, `OPENCUT_FFPROBE_PATH=ffprobe`, `OPENCUT_TEST_FONT_PATH=/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf`, `OPENCUT_GOLDEN_REQUIRED=1`, and `OPENCUT_RASTER_CACHE_TESTS_REQUIRED=1`. The report destination remains confined to existing report steps. Missing configuration/tools, missing instrumentation and failing assertions must fail this required step. Ordinary packaged integration/smoke retain their independent default build.

3. Keep exact allowlists. Extend the approved step sequence, validate locked dependency installation and its workspace, validate the complete native cache command body and its exact step-local environment. Continue rejecting custom shells, inherited environments, conditions, ignored exits, extra commands/steps, containers, golden mutation, and report publication before validation. Mutation tests remove or alter each new command and flag, omit the feature, move/remove default restoration, and inject bypasses. Update existing index-sensitive policy tests to reference the expanded sequence without weakening their assertions.

4. Documentation records the mandatory commands, local equivalents and evidence boundaries. No runtime/API/project contract changes occur; ADRs 0002/0003 require no new contract fixture, migration or owner edge. The policy remains a reviewed-code guard, not a new trust boundary. Preserve existing CODEOWNER ownership and prior archives.

## Risks / Trade-offs

- More CI work and build switches: run cache checks once in the existing Linux leaf, serially; restore the default build explicitly.
- Instrumented evidence accidentally skips: mandatory flags, feature build and the existing stats assertions make the dedicated run fail closed; mutation tests protect them.
- Exact policy tests drift: update workflow, validator and tests together and run the complete protected policy suite, including the real-Moon regression.
- Local host is Windows: execute native commands with verified local FFmpeg 7.1.1 and reviewed font; report that local success does not prove remote Linux execution. Do not modify runner dependency provisioning or golden baselines to hide incompatibility.

## Migration Plan

No data migration. Land the synchronized workflow/policy/documentation after verification and archival. A rollback must restore the previous reviewed policy and workflow together; do not bypass the guard to change ordering.

## Open Questions

The user approved these artifacts on 2026-09-21. No product decision is outstanding.
