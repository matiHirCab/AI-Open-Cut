## 1. Complete invalid-envelope preflight

- [x] 1.1 Add mirrored uniqueness validation for every invalid scenario definition and context collection, with fixture-ID-specific ambiguity exemptions only where required.
- [x] 1.2 Pass the complete catalog limits through both invalid classifiers and enforce each represented collection's exact named inclusive limit before semantic classification.
- [x] 1.3 Make all mutation helpers assert the intended duplicate or named-limit invariant.

## 2. Preserve branching component graphs

- [x] 2.1 Replace single-successor component dependency maps with ordered adjacency lists in Rust and TypeScript.
- [x] 2.2 Validate entry IDs, duplicate component IDs, duplicate directed edges, and missing endpoints before traversal.
- [x] 2.3 Add deterministic active-path cycle detection and longest-reachable-depth validation across every branch.

## 3. Add mirrored causal evidence

- [x] 3.1 Add Rust and TypeScript duplicate mutations for component IDs/edges, slot target layers, marker IDs, mask context layers, and audio assets/events/context definitions, while retaining intentional ambiguity fixtures.
- [x] 3.2 Add inclusive-boundary and `limit + 1` invalid-envelope mutations for every represented named collection limit.
- [x] 3.3 Add branching direct/indirect cycle, longest-depth, missing-endpoint, and duplicate-edge component regressions with exact expected failures.

## 4. Document, verify, and close

- [x] 4.1 Update fixture documentation and the living requirement through this change; leave ownership files unchanged unless helpers move.
- [x] 4.2 Run strict OpenSpec validation, `contracts:check`, Rust formatting/Clippy/workspace tests, TypeScript typecheck/lint/unit tests, bridge integration, packaged smoke, and `git diff --check`.
- [x] 4.3 Verify requirements, design, tasks, tests, and code agree; sync and archive only this corrective change after all checks pass.
