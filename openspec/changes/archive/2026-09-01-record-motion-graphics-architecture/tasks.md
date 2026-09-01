## 1. Architecture Decision

- [x] 1.1 Add ADR 0004 with the six locked decisions, normative coordinate/timing/ordering/compositing rules, canonical ownership and security boundaries, alternatives, consequences, and the future-milestone compatibility/migration policy.
- [x] 1.2 Link ADR 0004 from the motion-graphics implementation plan and the editor-core architecture ADR without changing the user's existing roadmap content beyond the references/status needed for traceability.

## 2. Deterministic Documentation Evidence

- [x] 2.1 Add an editor-core architecture test that reads ADR 0004 and proves all six decisions and required observable semantics remain documented; include a deterministic failing fixture/case proving an omitted decision is detected.
- [x] 2.2 Confirm `contracts/contract-ownership-v1.json` and the versioned public fixtures require no update because this issue adds no runtime, persisted, headless, MCP, capability, or error surface; record that conclusion in ADR 0004.

## 3. Verification and Specification Sync

- [x] 3.1 Run `moon run openspec-validate`, `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`.
- [x] 3.2 Use `$openspec-verify-change` to confirm task, requirement, scenario, design, ADR, and test coherence; resolve every critical or warning mismatch.
- [x] 3.3 Sync the `motion-graphics-architecture` and `editor-core-architecture` deltas into the living specs, then archive `record-motion-graphics-architecture` with `$openspec-archive-change`.
