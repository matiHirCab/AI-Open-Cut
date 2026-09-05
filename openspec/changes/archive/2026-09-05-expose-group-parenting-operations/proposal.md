## Why

Issue #21 requires a complete agent-addressable group editing workflow. Issues #19 and #20 already supply typed creation, parenting, z-index, aliases, schema-10 persistence and ancestor evaluation, but there is no atomic ungroup operation or capability identifying its availability.

## What Changes

- Add `group_ungroup` with `groupId`, standalone and inside `timeline_batch_edit`. Remove only the named group and promote its immediate children to its parent (or root), preserving their local properties and flat ordering.
- Reuse `add_group`, `item_set_parent`, and `item_set_z_index`; verify creation, reparenting, z-index and ungroup together through real headless and MCP flows.
- Resolve a prior creation alias in `groupId`, enforce all affected track locks in core, and publish one revision/undo step or nothing.
- Add discovery through `group_ungroup`, governed contract fixtures, typed schemas, documentation and lifecycle/failure regression evidence.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `timeline-editing`: Atomic ungroup semantics, locks, aliases, changed IDs and history.
- `motion-graphics-contracts`: Additive ungroup wire contract, capability and cross-language parity.
- `agent-bridge`: End-to-end standalone and batch group workflows through typed transports.

## Impact

Core model operation union and timeline dispatch; headless typed requests and capability reporting; bridge schemas, registration and typed requests; group-parent, headless, MCP and ownership catalogs; tests and group-operation documentation. No new dependency edges or provider behavior.

Public operation/capability additions preserve protocol major, stable errors and existing operation meanings. Persisted schema remains 10: no new persisted fields or migration are necessary. Existing schema/history migration regressions remain required. Renderer evaluation is unchanged; removing a group intentionally removes its inherited visual contribution, consistently with existing local-preserving detachment.

## Non-goals

Keep-world transform baking, recursive deletion or recursive ungroup, new aliases for existing operation names, nested compositing, group animation, UI authoring, new persisted fields, and renderer changes.

## Approval

The user explicitly replied “Approve” in this task on 2026-09-05, approving the proposal, design, delta scenarios and tasks, including local-preserving ungroup promotion. Implementation is authorized through tasks.md. Designated contract-owner review remains required after implementation.

The user explicitly approved implementation of the null-resultAlias correction plan on 2026-09-05. This enforces the existing prohibition without changing other operations.

Final CODEOWNER approval: on 2026-09-05, the user replied "I approve" to the explicit request to approve the final implementation and contracts as CODEOWNER and authorize synchronization, archive, validation and push for PR #107. This satisfies the designated owner review gate for the implementation, canonical contracts, consumers and parity evidence.
