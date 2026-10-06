## MODIFIED Requirements

### Requirement: Mandatory native track-matte CI conformance
The existing render-parity native conformance step MUST execute the actual native core track-matte suite and dedicated MCP artifact witness on the exact CI head, inheriting its unchanged six-key required dependency/font/report environment. Immediately after the unchanged font-resolution command it MUST run exactly `cargo test -p opencut-editor-core --test track_mattes_native -- --nocapture`, `cargo build -p opencut-headless`, and `bun run --cwd apps/agent-bridge test:unit --no-file-parallelism tests/track-matte-native.test.ts` in that order. The default headless build MUST precede MCP execution. All existing commands, environment values, steps, dependencies, timeouts, guards, report paths, benchmark, raster-cache default restoration, aggregate and duration budget MUST remain unchanged. Ordinary opt-in skips SHALL NOT substitute for actual mandatory execution. The exact command-body validator and additive regressions MUST reject omission, alteration, success fallback and an instrumented headless build; no model, native oracle or public contract change is authorized.

#### Scenario: Execute actual native core and MCP conformance on the CI head
- **WHEN** the required render-parity leaf runs on the exact PR commit with its configured tools/font and required flags
- **THEN** the complete native core matte suite executes, the default headless binary builds and the dedicated MCP matte artifact witness executes with no native opt-in skip before the unchanged raster-cache step

#### Scenario: Reject weakened mandatory matte execution
- **WHEN** any new command is omitted, altered, success-masked or replaced with an instrumented headless build
- **THEN** exact policy validation fails while every prior protected command, environment, sequence, timeout, report and aggregate constraint remains enforced

### Requirement: Mandatory native blend-mode CI conformance
The existing render-parity native conformance step MUST additionally execute the actual native core blend-mode suite `crates/editor-core/tests/blend_modes_native.rs` and dedicated MCP artifact witness `apps/agent-bridge/tests/blend-mode-native.test.ts` on the exact future blend-mode CI head. Immediately after the unchanged three track-matte commands it MUST run exactly `cargo test -p opencut-editor-core --test blend_modes_native -- --nocapture`, `cargo build -p opencut-headless`, and `bun run --cwd apps/agent-bridge test:unit --no-file-parallelism tests/blend-mode-native.test.ts` in that order, before the unchanged raster-cache step. The default headless build MUST precede the blend MCP witness. It MUST inherit the existing unchanged six-key required dependency/font/report environment (`OPENCUT_FFMPEG_PATH`, `OPENCUT_FFPROBE_PATH`, `OPENCUT_ANIMATION_CHANNEL_RENDER_REQUIRED`, `OPENCUT_GOLDEN_REPORT_PATH`, `OPENCUT_GOLDEN_REQUIRED`, `OPENCUT_TEST_FONT_PATH`), including the existing required values; no new opt-in flag or environment entry is authorized. Every preceding native command, track-matte command, step, dependency, timeout, guard, report path, benchmark, raster-cache instrumentation/default restoration, aggregate constraint and duration budget MUST remain unchanged. Ordinary unit-suite opt-in skips MUST NOT substitute for actual mandatory execution. Exact command-body policy validation and additive regressions MUST reject omission, alteration or success fallback for each new command and replacement of the new default headless build with an instrumented build, while preserving all existing negative controls. This requirement specifies future implementation intent only; these named suites and commands are not claimed to exist or have executed during external proposal preparation.

#### Scenario: Execute actual native core and public blend conformance
- **WHEN** the required render-parity leaf runs on the exact approved blend-mode PR commit with its existing required tools/font/report environment
- **THEN** the complete native blend suite executes, the default headless binary builds and the dedicated public blend artifact witness executes without a native opt-in skip after all unchanged track-matte commands and before the unchanged raster-cache step

#### Scenario: Reject weakened mandatory blend execution
- **WHEN** any added blend command is omitted, altered, success-masked or its default headless build is replaced with an instrumented build
- **THEN** exact policy validation fails and all prior protected commands, six environment entries, step ordering, timeouts, report, duration and aggregate constraints remain enforced

## ADDED Requirements

### Requirement: Mandatory native ordered-effect CI conformance
The existing render-parity native conformance step MUST additionally execute the actual native core ordered-effect suite `crates/editor-core/tests/ordered_effects_native.rs` and dedicated MCP artifact witness `apps/agent-bridge/tests/ordered-effect-native.test.ts` on the exact approved ordered-effect CI head. Immediately after the unchanged three blend commands it MUST run exactly `cargo test -p opencut-editor-core --test ordered_effects_native -- --nocapture`, `cargo build -p opencut-headless`, and `bun run --cwd apps/agent-bridge test:unit --no-file-parallelism tests/ordered-effect-native.test.ts` in that order before the unchanged raster-cache step. Default headless build MUST precede the public witness. Existing six required dependency/font/report environment keys and values, all mask/matte/blend commands, steps/dependencies/timeouts/reports/benchmark/cache instrumentation/default restoration/aggregate/duration budget MUST remain unchanged. Ordinary native opt-in skips SHALL NOT substitute for actual execution. Exact command-body policy validation and additive mutation regressions MUST reject omission/alteration/success fallback for each new command and instrumented-build substitution, preserving every prior negative control. This CI evidence requirement SHALL introduce no additional production behavior/public contract/schema/version/budget; the separately approved atomic-adoption selector correction remains governed by project-persistence. The named suites are implemented and execute under the existing required native configuration; local evidence and exact-head remote CI evidence MUST be recorded separately.

#### Scenario: Execute actual core and public ordered-effect evidence
- **WHEN** the required render-parity leaf executes the exact ordered-effect PR head with its existing six-key required tool/font/report environment
- **THEN** full core and public ordered-effect witnesses execute without native opt-in skips after all unchanged blend commands and default build, before the unchanged cache controls

#### Scenario: Reject weakened ordered-effect native execution
- **WHEN** an added effect command is omitted, altered, success-masked or its default build replaced with an instrumented build
- **THEN** exact protected policy validation fails with all prior commands/environment/step ordering/timeouts/report/duration/aggregate and negative controls preserved
