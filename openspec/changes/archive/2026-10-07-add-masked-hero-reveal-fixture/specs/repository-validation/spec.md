## ADDED Requirements

### Requirement: Mandatory native masked hero CI conformance
The existing render-parity step MUST preserve every preceding native command and append immediately after group-compositing commands, before unchanged cache: exactly cargo test -p opencut-editor-core --test masked_hero_reveal_native -- --nocapture; cargo build -p opencut-headless; cargo test -p opencut-headless --test masked_hero_reveal_native -- --nocapture; bun run --cwd apps/agent-bridge test:unit --no-file-parallelism tests/masked-hero-reveal-native.test.ts in this order. Existing required dependencies/envkeys/pins/jobs/steps/timeouts/reports/benchmark/instrumentation/default restore/aggregate/duration constraints MUST remain. Configured core/headless/realMCP hero suites MUST execute actual media without required skips; additive omission/alteration/success-fallback controls for every newcommand and instrumented default-build substitution MUST reject while all predecessor assertions remain. New fixture120000ms test deadline MAY be used without extending any previous deadline.

#### Scenario: H16 Execute actual native hero artifacts
- **WHEN** unchanged configured render parity runs on the exact published commit
- **THEN** new core/default-headless/realMCP suites execute after all old suites and before cache with actual independent media/lifecycle/failure evidence and no required opt-in skips

#### Scenario: H17 Reject weakened native hero execution
- **WHEN** any newcommand is omitted/altered/failure-masked or default headless build is replaced with instrumented build
- **THEN** protected policy fails through additive controls and all preceding checks/pins/budgets/tolerances/deadlines remain enforced
