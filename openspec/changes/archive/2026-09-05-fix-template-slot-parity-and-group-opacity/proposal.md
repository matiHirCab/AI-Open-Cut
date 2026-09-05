## Why

Review of issue #23 reproduced two violations of the accepted template-slot requirements: the bridge silently drops the legal `__proto__` override key, and core rejects opacity slots on valid groups without explicit Transform2D. The existing suites pass without covering these cases.

## What Changes

- Preserve all legal slot IDs through bridge request parsing and returned state, validating every value without changing object prototypes or published JSON schemas.
- Apply group opacity through Transform2D in derived effective candidates, preserving stored tracks and all other transform fields.
- Add shared canonical regression evidence, native tests and source/packaged transport workflows for both defects.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `template-slots`: Explicit special-key round trips and group-opacity default/override scenarios.
- `agent-bridge`: Lossless validated override maps and unchanged advertised schemas through standalone and batch workflows.
- `motion-graphics-contracts`: Canonical regression evidence consumed by native and bridge tests.

## Impact

Changes belong to core effective-value validation, the bridge shared slot-value schema, canonical template-slot fixtures, tests and documentation. This is a compatibility-restoring defect fix: schema 12, protocol 1, slot ID grammar, fields, capabilities and stable errors remain unchanged. No persisted migration or dependency upgrade is needed. The designated public-contract consumer reviewer remains @matiHirCab.

## Non-goals

No renderer expansion, root component rendering, provider behavior, arbitrary property binding, new APIs, commits or pushes. Preserve the original add-typed-template-slots archive and all unrelated working-tree changes.

## Approval

The user explicitly requested implementation of the detailed fix plan on 2026-09-05. These concrete follow-up artifacts implement that plan; explicit artifact approval was received before implementation, as recorded below. Completed contract-owner review is a separate finalization task.

Artifact approval: the user explicitly approved the complete follow-up artifacts in this task on 2026-09-05 (Approve). Implementation is authorized through tasks.md.

Completed contract review: the user explicitly approved the completed canonical evidence and consumer corrections on 2026-09-05 in response to the designated CODEOWNER review request, authorizing synchronization, archival and the final Moon gate.
