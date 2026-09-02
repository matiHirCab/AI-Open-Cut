## 1. Restore composition ownership

- [x] 1.1 Key invalid audio-event and marker definitions by exact `{ scope, id }` tuples in Rust and TypeScript.
- [x] 1.2 Group invalid audio events and markers by exact composition scope before applying their named inclusive limits.
- [x] 1.3 Preserve project-wide identity for assets, buses, and sound definitions.

## 2. Narrow ambiguity exemptions

- [x] 2.1 Require the standalone marker fixture's only duplicated name key to equal `{ scenario.scope, lookupName }`.
- [x] 2.2 Require bus, marker, sound-definition, and variant ambiguity fixtures to contain exactly one duplicated semantic key referenced by the appropriate event context.
- [x] 2.3 Reject unrelated or second duplicate keys before semantic classification while preserving canonical ambiguity results.

## 3. Add mirrored causal evidence

- [x] 3.1 Add cross-scope accepted and same-scope rejected ID mutations for audio events and markers.
- [x] 3.2 Add distributed-owner boundary and per-owner `limit + 1` mutations for event and marker limits.
- [x] 3.3 Add unrelated-duplicate mutations for standalone marker names and audio bus, marker, sound-definition, and variant ambiguity families.

## 4. Document, verify, and close

- [x] 4.1 Update fixture documentation and the living requirement through this change; leave ownership files unchanged.
- [x] 4.2 Run strict OpenSpec validation, `contracts:check`, Rust formatting/Clippy/workspace tests, TypeScript typecheck/lint/unit tests, bridge integration, packaged smoke, and `git diff --check`.
- [x] 4.3 Verify requirements, design, tasks, tests, and code agree; sync and archive only this corrective change after all checks pass.
