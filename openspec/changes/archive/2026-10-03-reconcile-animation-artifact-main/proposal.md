## Why

PR136's verified issue47 integration head `b372185178f44e4cbf78c37c0de470a80b8ae85c` conflicts with main after PR138/issue74 artifact resources merged at `4c2897e0`. The user explicitly requested conflict resolution; the combined branch must preserve animation clocks, issue59 speech timestamps and issue74 delivery semantics together before draft publication resumes.

## What Changes

- Normally merge the exact verified main head into the issue47 integration branch and resolve only actual shared bridge/schema/catalog/test overlaps.
- Preserve schema31 Scalar/Pack animation clocks and previously verified source/native behavior, plus imported speech timestamps and metadata-first artifact-resource delivery.
- Retain issue74's version-2 breaking content contract and explicit `includeBinary=true` PNG/WAV migration; add no further breaking change, new operation, schema or capability.
- Reconcile the canonical MCP surface and ownership catalog as the semantic union, deliberately update its expanded digest after conflict resolution, and retain all governed parity scenarios.
- Verify affected combined-tree checks and independent conformance; record precisely which unchanged Rust/native evidence remains reusable.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `agent-bridge`: explicitly require reconciled animation, speech and artifact delivery contract preservation and combined parity evidence.

## Impact

Incoming changes from merge base `480bd8d7` to main span 27 bridge, test, contract, documentation and specification files; they contain no Rust, headless, worker or native-render implementation change. The direct issue47-head-to-main diff also shows unmerged issue47 changes and MUST NOT be interpreted as upstream deletion instructions. Compatibility surfaces are shared bridge schemas, jobs/resources, status capabilities, MCP catalog/digest, artifact-delivery-v2 and contract ownership. Preserve imported archived/living specifications and all existing tests, including issue47 workflow partitions. Non-goals: new artifact policy, provider or animation behavior, schema changes, gate/timeout weakening, merging remote main, deployment or starting issue48.
