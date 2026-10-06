# Issue52 mandatory native CI amendment

The actual mandatory Linux local five-test core matte suite and dedicated MCP matte artifact test SHALL also execute on the exact PR commit in the existing `render-parity` native conformance step. Current correctness workspace runs do not guarantee native dependency availability; authored opt-in skips are not native proof.

Add, after the unchanged existing font-resolution command, exactly:

```sh
cargo test -p opencut-editor-core --test track_mattes_native -- --nocapture
cargo build -p opencut-headless
bun run --cwd apps/agent-bridge test:unit --no-file-parallelism tests/track-matte-native.test.ts
```

The existing required FFmpeg/FFprobe/font and GoldenRequired/AnimationChannelRequired variables are inherited. Build the default headless binary before the MCP test; no instrumented-feature binary is used. Preserve every existing command, exact guard, job sequence, dependency, timeout, six-key native environment, report path, benchmark, raster-cache default restore, aggregate and duration budget. No workflow dispatch, timeout increase, suppression or optional failure.

Affected implementation ownership is `.github/workflows/bun-ci.yml`, the exact native command-body constant in `scripts/validate-ci-gates.ts`, additive tampering regression controls in `scripts/validate-ci-gates.test.ts`, and `docs/ci-parity-gates.md`. Extend the validator's authoritative exact body to require the three new commands: deleting, replacing, adding success fallback, or using an instrumented headless build SHALL reject. This strengthens the protected policy rather than loosening it. No domain/model/catalog or native oracle change.

Promote this exact amendment into the active change and add one repository-validation requirement with two scenarios: real mandatory core/MCP execution on the exact CI head; fail-closed deletion/alteration/default-binary guard. Update task6.2 traceability. Root and independent reviewer explicitly approve exact amendment before implementation.

Verify affected formatting/lint, complete CI-policy validator regressions, strict all-spec validation, pre-archive own-change-only protected rejection and post-archive protected pass. Existing exclusive final3 Rust pipeline remains source-valid because no Rust/Cargo/native-test input changes. Requalify pure bridge source hashes; native public and exact-head remote CI remain pending until actually run.
