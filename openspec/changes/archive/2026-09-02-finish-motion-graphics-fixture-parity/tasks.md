## 1. Reject pre-normalization duplicates

- [x] 1.1 Reject repeated Rust layer and component-local layer definitions at insertion time before metadata comparison or aggregate counting.
- [x] 1.2 Reject repeated Rust mask and effect definitions at insertion time and preserve TypeScript parity.

## 2. Strictly preflight invalid scenarios

- [x] 2.1 Add mirrored common-field validation and fixture-ID-specific intentional exemptions for component, layer, slot, marker, mask, effect, audio, transform, and curve invalid scenarios.
- [x] 2.2 Align scenario identifiers, legal scopes, safe integers, finite ranges, required collections, names, defaults, and constraints without changing canonical classifications or reasons.
- [x] 2.3 Count constrained negative-scenario names by Unicode scalar values in Rust and TypeScript.

## 3. Prove causal parity

- [x] 3.1 Add mirrored duplicate-definition mutations for layers, component layers, masks, and effects.
- [x] 3.2 Add a table-driven malformed invalid-envelope matrix for IDs, scopes, names, Unicode boundaries, integers, numbers, ranges, collections, and constraints.
- [x] 3.3 Make failure helpers assert the expected invariant for aggregate limits, duplicates, managed resources, reference closure, malformed envelopes, and swapped payloads.
- [x] 3.4 Update fixture documentation and the living requirement where necessary; leave ownership files unchanged unless helpers move.

## 4. Verify and close

- [x] 4.1 Run strict OpenSpec validation and `bun run contracts:check` from `apps/agent-bridge`.
- [x] 4.2 Run Rust formatting, workspace Clippy with warnings denied, and workspace tests.
- [x] 4.3 Run TypeScript typecheck, lint, unit tests, bridge integration tests, and packaged smoke tests.
- [x] 4.4 Run `git diff --check`, verify no runtime or prior archive changed, sync the living specification, and archive only this corrective change after every check passes.
