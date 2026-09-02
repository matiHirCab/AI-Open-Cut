## 1. Correct aggregate and wrapper validation

- [x] 1.1 Accumulate payload-derived component, layer, marker, slot, and audio-event counts by their specified project or composition owner and enforce inclusive catalog limits in Rust and TypeScript.
- [x] 1.2 Reject fixture IDs duplicated within valid fixtures, within invalid fixtures, or across both arrays before map/result insertion.
- [x] 1.3 Parse managed resources as unique project-scoped asset tuples and preserve managed-reference closure.

## 2. Complete strict field parity

- [x] 2.1 Add Rust checks matching Zod for component track IDs and slot-value keys, non-empty animation channels, marker ID/name/safe-integer time, and non-empty curve collections.
- [x] 2.2 Audit adjacent fields in the affected Rust declarations against Zod and close every remaining identifier, required-value, collection, numeric, or length mismatch without weakening TypeScript.

## 3. Add corrective evidence and documentation

- [x] 3.1 Add mirrored boundary tests for aggregate limits spread across multiple fixtures, including independent root and component owners at the limit and at limit plus one.
- [x] 3.2 Add mirrored duplicate-ID, managed-resource kind/scope/duplicate/closure, and malformed-field regressions for every corrected parity constraint.
- [x] 3.3 Clarify aggregate ownership, global fixture identity, and managed project assets in the living specification and fixture documentation; update ownership files only if helpers move.

## 4. Verify and close

- [x] 4.1 Run strict OpenSpec validation and `bun run contracts:check` from `apps/agent-bridge`.
- [x] 4.2 Run `cargo fmt --check --all`, workspace Clippy with warnings denied, and workspace Rust tests.
- [x] 4.3 Run TypeScript typecheck, lint, unit tests, bridge integration tests, and packaged smoke tests.
- [x] 4.4 Run `git diff --check`, verify no runtime or archived surface changed, sync the living specification, and archive only this corrective change after every check passes.
