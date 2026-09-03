## Why

Golden publication can currently make `CURRENT` durable before the installed generation's files and directory entries are durable. A crash in that window can leave the strict pointer selecting a partial or missing generation, defeating the recovery guarantees added by the previous pointer-commit fix.

## What Changes

- Make every validated golden generation durable before attempting the atomic `CURRENT` replacement.
- Treat content-sync, installation, and generation-directory-sync failures as pre-commit failures that preserve the prior pointer.
- Revalidate and resynchronize an already installed digest before selecting it.
- Add fault injection and reopen tests for each generation-durability boundary.
- Document the ordering between generation durability, pointer commit, and bounded cleanup.

Non-goals: recapturing golden media, changing the selected digest, changing render tolerances or performance schema 2, and changing any public or persisted application contract.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `render-regression-fixtures`: Require the immutable generation itself to be durably installed before the versioned pointer can commit to it.

## Impact

The change is confined to the test-only golden harness in `editor-core`, its unit tests, fixture documentation, and the existing render-regression-fixtures specification. It reuses the standard library on Unix and the existing Windows development dependency. There are no public API, project schema, headless protocol, MCP, fixture schema, dependency-edge, or migration changes.
