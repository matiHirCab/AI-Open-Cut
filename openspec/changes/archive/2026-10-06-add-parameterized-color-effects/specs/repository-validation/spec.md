## ADDED Requirements

### Requirement: Mandatory native parameterized-effect CI conformance
The existing render-parity step MUST execute actual native core parameterized effects suite and dedicated public MCP artifact witness on the exact approved head. Immediately after all unchanged ordered-effect commands it MUST run exactly cargo test -p opencut-editor-core --test parameterized_effects_native -- --nocapture, cargo build -p opencut-headless, and bun run --cwd apps/agent-bridge test:unit --no-file-parallelism tests/parameterized-effect-native.test.ts in order, before unchanged raster-cache controls. Default headless build MUST precede the public witness. All preceding commands/six required envkeys/values/steps/dependencies/timeouts/reports/benchmark/instrumentation/default restoration/aggregate/duration constraints MUST remain. Opt-in skips SHALL NOT substitute for mandatory configured execution. Exact command policy/additive mutation tests MUST reject omission/alteration/success fallback for each added command and instrumented-build substitution while preserving every prior negative.

#### Scenario: Execute actual parameterized native artifacts
- **WHEN** the required render-parity leaf runs with unchanged mandatory dependency/font/report environment
- **THEN** core and public parameterized-effect suites execute without native skips after all prior commands/default build and before existing cache checks

#### Scenario: Reject weakened parameterized native execution
- **WHEN** any added command is omitted/altered/success-masked or default build replaced with an instrumented build
- **THEN** protected validation fails and all prior sequence/environment/timeouts/reports/duration/aggregate constraints remain enforced
