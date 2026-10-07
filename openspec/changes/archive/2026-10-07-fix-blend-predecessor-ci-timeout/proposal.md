## Why

PR150 exact-head CI failed on Windows: the complete blend-predecessor negative-control test exceeded its unchanged 5000ms deadline. Each malformed blend field unnecessarily traverses and clones the enlarged schema37 predecessor chain before its small unchanged blend node is checked.

## What Changes

- Validate the unchanged approved blend addition nodes read-only before the complete historical predecessor transforms; retain the full valid projection and all historical checks.
- Retain all29 paths, three malformed values and missing-field controls, complete predecessor digest, source non-mutation assertions and the 5000ms deadline.
- Add an automated witness that malformed blend input is rejected before unrelated expensive schema subtrees are read.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- contract-governance: bounded early rejection in historical blend contract evidence without weakened conformance.

## Impact

Test-only blend predecessor projection and focused regression evidence, plus this OpenSpec lifecycle. No production behavior, contracts, catalog bytes/pins, schema37, protocol1, 78 tools, render semantics, CI settings or timeouts change. Non-goals: suppressing assertions, increasing deadlines, dropping malformed controls, altering the projection of valid input, or starting #57 before corrected #56 exact-head CI succeeds.
