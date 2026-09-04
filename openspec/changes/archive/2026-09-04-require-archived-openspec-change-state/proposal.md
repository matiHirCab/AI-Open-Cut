## Why

Repository validation currently accepts implemented OpenSpec changes that remain under `openspec/changes/`. This allows work to be presented as complete while its delta is not archived into the living specifications. One completed implementation change is still active because its Linux verification was previously unavailable.

## What Changes

- Make the protected CI policy reject every entry under `openspec/changes/` except the canonical `archive` directory.
- Include the active-change inventory in bootstrap preflight so no completion attestation can be emitted for an unarchived change.
- Add structural and filesystem regression coverage for empty, single, multiple, malformed, and archived-only change states.
- Document that active changes are permitted during local development but cannot pass the merge-ready protected gate.
- Complete the outstanding Linux verification for `harden-render-benchmark-migration-and-sampler-cleanup` and archive it before final validation.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `repository-validation`: Require the protected policy attestation to observe an archive-only OpenSpec change state.

## Impact

The change affects internal CI policy validation, its tests, OpenSpec workflow documentation, and the archival state of one completed change. It changes no application API, public contract, schema, renderer behavior, fixture, golden reference, status name, or branch-protection setting.
