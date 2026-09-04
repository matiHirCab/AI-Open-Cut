## 1. Approval and canonical fixtures

- [x] 1.1 Obtain explicit approval of proposal, design, and all delta specs before implementation; record approval evidence in proposal.md.
- [x] 1.1a Resolve and approve the text-measurement/source-box design correction in amendment.md, then synchronize its exact staged evaluation rules and verification scenarios into the delta specs before implementation.
- [x] 1.2 Add transform2d-v1 canonical runtime fixtures and ownership mapping, plus additive headless/MCP catalog changes. Cover every motion-graphics-contracts scenario with mirrored Rust/TypeScript parity cases; retain fixture-only roadmap status.
- [x] 1.3 Add schema-8 migration fixtures for all supported source versions and mixed retained history before updating consumers (project-persistence: both requirements, every scenario).

## 2. Core model and persistence

- [x] 2.1 Implement complete typed Transform2D and optional common-property ownership, switching rules, target restrictions, numeric validation, and animation incompatibility. Add tests for every timeline-editing scenario under Typed static Transform2D updates and Bounded Transform2D values.
- [x] 2.2 Implement deterministic schema-8 migration through the existing locked generation transaction. Test all project-persistence scenarios including invalid retained state, missing references, future/zero versions, omission, recovery at each injection phase, reopen, undo, and redo.
- [x] 2.3 Extend core item updates and batch behavior. Test every Transform2D transactional semantics scenario, including each error category, aliases, split/duplicate, changed IDs, lock failures, and complete rollback.

## 3. Evaluation and rendering

- [x] 3.1 Implement canonical source-box resolution and affine matrix evaluation in the owning core layer. Add independent oracle cases for every motion-graphics-architecture scenario, each numeric bound, and missing-asset precedence.
- [x] 3.2 Extend evaluated instructions and renderer lowering to execute the complete affine transform for each supported visual source, including caption rasterization, transparency, offsets, clipping, and opacity. Cover both rendering-export requirements and every scenario, including unavailable backend and unsafe input rejection.
- [x] 3.3 Add asymmetric visual golden fixtures and unchanged audio to frame/range/draft/export comparisons, verify documented tolerances, and rerun legacy transform/animation/caption/transition fixtures. Record fixture-to-scenario mappings in verification.md.

## 4. Typed transports and documentation

- [x] 4.1 Update headless typed requests/responses and capability reporting from canonical contracts. Add protocol tests for standalone Transform2D, null reset, invalid data, stable failures, and legacy requests (motion-graphics-contracts and timeline-editing scenarios).
- [x] 4.2 Update bridge Zod schemas, existing MCP update/batch registration, and capability reporting without duplicating core semantics. Add MCP integration and packaged-smoke cases for create-by-alias, update, rollback, undo/redo, reopen, and render parity.
- [x] 4.3 Document exact coordinates, source bounds, affine order, field switching, static-animation restriction, numeric limits, schema rollback, and capability discovery. Update ADR 0004's activation status and contract fixture guide. Obtain @matiHirCab review for governed contracts.

## 5. Verification and archive

- [x] 5.1 Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` from the repository root.
- [x] 5.2 In apps/agent-bridge run `bun run contracts:check`, `bun run typecheck`, `bun run lint`, `bun run test`, `bun run test:integration`, and `bun run test:smoke`. Record failures or unavailable prerequisites explicitly; none may be silently skipped.
- [x] 5.3 Run `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive` for focused authoring. Use `$openspec-verify-change` and create verification.md mapping every normative requirement/scenario to automated evidence and outcomes. Python providers have no changed surface in this scope; if shared provider contracts change, stop for scope approval and add their hermetic checks before continuing.
- [x] 5.4 Resolve every verification mismatch, synchronize and archive using `$openspec-archive-change`, then run `moon run root:openspec-validate`. The protected gate requires archive-only inventory, so any active change blocks final policy acceptance. Report all required failed/skipped checks and do not claim implementation complete until they pass.
