## ADDED Requirements

### Requirement: Mandatory native track-matte CI conformance
The existing render-parity native conformance step MUST execute the actual native core track-matte suite and dedicated MCP artifact witness on the exact CI head, inheriting its unchanged six-key required dependency/font/report environment. Immediately after the unchanged font-resolution command it MUST run exactly `cargo test -p opencut-editor-core --test track_mattes_native -- --nocapture`, `cargo build -p opencut-headless`, and `bun run --cwd apps/agent-bridge test:unit --no-file-parallelism tests/track-matte-native.test.ts` in that order. The default headless build MUST precede MCP execution. All existing commands, environment values, steps, dependencies, timeouts, guards, report paths, benchmark, raster-cache default restoration, aggregate and duration budget MUST remain unchanged. Ordinary opt-in skips SHALL NOT substitute for actual mandatory execution. The exact command-body validator and additive regressions MUST reject omission, alteration, success fallback and an instrumented headless build; no model, native oracle or public contract change is authorized.

#### Scenario: Execute actual native core and MCP conformance on the CI head
- **WHEN** the required render-parity leaf runs on the exact PR commit with its configured tools/font and required flags
- **THEN** the complete native core matte suite executes, the default headless binary builds and the dedicated MCP matte artifact witness executes with no native opt-in skip before the unchanged raster-cache step

#### Scenario: Reject weakened mandatory matte execution
- **WHEN** any new command is omitted, altered, success-masked or replaced with an instrumented headless build
- **THEN** exact policy validation fails while every prior protected command, environment, sequence, timeout, report and aggregate constraint remains enforced
