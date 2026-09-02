## 1. Close invalid-audio reference preflight

- [x] 1.1 Require every Rust invalid-audio sound-definition bus ID to exist in the declared project bus set before failure derivation.
- [x] 1.2 Mirror the same membership check and exact invariant in TypeScript.
- [x] 1.3 Preserve ordinary bus uniqueness and the one exact `audio_event.ambiguous_bus` duplicate-key exemption.

## 2. Add causal parity evidence

- [x] 2.1 Add table-driven Rust mutations for every invalid audio fixture that assert the exact missing sound-definition bus invariant.
- [x] 2.2 Add the identical TypeScript mutation matrix.
- [x] 2.3 Add restored-bus controls proving every mutated fixture returns to its canonical exact concept, classification, and reason.

## 3. Document, verify, and close

- [x] 3.1 Update fixture documentation and the living requirement through this change; leave the catalog and ownership files unchanged.
- [x] 3.2 Run strict OpenSpec validation, `contracts:check`, Rust formatting/Clippy/workspace tests, TypeScript typecheck/lint/unit tests, bridge integration, packaged smoke, and `git diff --check`.
- [x] 3.3 Verify requirements, design, tasks, tests, and code agree; sync and archive only this corrective change after all checks pass.
