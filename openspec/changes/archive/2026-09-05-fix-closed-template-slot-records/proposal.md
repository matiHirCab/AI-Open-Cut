## Why

Zod can discard an own enumerable `__proto__` field before strict object validation, accepting malformed template-slot records that Rust rejects. This violates the existing closed-record contract and can silently alter requests before they reach core.

## What Changes

- Validate unknown own enumerable record fields before existing Zod value parsing, using a shared public-API wrapper with preserved parsed types and nested issue paths.
- Cover definitions, bindings, constraints, all eight value envelopes, rich-text documents/runs and managed-asset references in requests and responses.
- Keep override maps open to arbitrary string slot IDs and retain safe reconstruction, including `__proto__`, `constructor` and `toString`.
- Add raw-JSON canonical negative fixtures, native/bridge parity and real source/packaged atomicity evidence.
- Preserve published input/output JSON schemas through metadata derived from existing declarations and verify the actual MCP catalog without changing it.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `template-slots`: Explicitly reject unknown own fields at every closed slot-record location while preserving valid special slot IDs.
- `agent-bridge`: Validate record structure before stripping and preserve nested errors and failed-request atomicity across standalone and batch workflows.
- `motion-graphics-contracts`: Govern canonical raw-JSON negatives, structural schema parity and designated completed contract review.

## Impact

Changes are confined to bridge slot schemas, canonical slot fixtures, affected Rust/TypeScript consumers, shared integration/smoke evidence and documentation. This corrects acceptance to the already advertised closed contract; schema 12, protocol 1, identifiers, stable errors, group-opacity behavior and dependencies remain unchanged. Core continues to own semantic validation and persistence.

## Non-goals

No repository-wide schema rewrite, renderer expansion, migration, dependency upgrade, new field/capability/error, MCP catalog adjustment, commit or push. Both previous archives remain intact.

## Approval

The user explicitly approved this proposal, design, delta specifications and tasks on 2026-09-05 with "Approve". Implementation is authorized. Completed designated contract-owner review was explicitly approved by the user on 2026-09-05 after implementation and verification.
