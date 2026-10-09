## Implementation
- [x] 1. Replace shell/batch launcher with compiled host-native fixture and add bounded failure diagnostics.
- [x] 2. Run focused shutdown, full unit, typecheck/lint and complete MCP source/package checks; retain unchanged passing Rust/Python evidence for unchanged inputs.
- [x] 3. Verify requirement/scenario traceability, strict specs and pre-archive protected gate before archival.

## Delivery follow-through
After implementation checks and conformance verification pass, synchronize/archive, require the final protected gate to pass, then commit/push to the existing draft PR. Remote Windows outcome remains a verification obligation.
