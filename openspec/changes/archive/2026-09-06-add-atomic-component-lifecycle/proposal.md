## Why

Issue #25 (MG-M1-09) requires an atomic, agent-addressable component lifecycle. Existing component_create, component_define_slots, add_component_instance and component_instance_update implement creation, slot definition and instantiation, but duplicate_items cannot apply overrides or expose a single creation alias for subsequent batch edits.

## What Changes

- Add component_instance_duplicate with a source itemId, nonnegative offsetMs, optional typed slotValues and optional batch resultAlias.
- Preserve the shared component definition and source instance; clone the root instance with a fresh ID and optionally replace its complete override map.
- Verify creation, slot definition, instantiation, duplication and subsequent edits together through standalone and aliased atomic batches.
- Advertise additive component_lifecycle capability and synchronize canonical contracts, native declarations, MCP schemas, tests and documentation.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `component-evaluation`: atomic root instance duplication with optional override replacement and a creation alias.
- `agent-bridge`: complete lifecycle discovery and standalone/batch transport workflows.
- `motion-graphics-contracts`: additive lifecycle contract fixtures and compatibility evidence.

## Impact

Core model/timeline own the new edit and alias resolution; existing validation/store own validity and atomic publication. Headless and bridge expose typed adapters. Canonical lifecycle, protocol, MCP and ownership catalogs and all governed consumers require synchronized updates and @matiHirCab review.

Compatibility is additive under protocol 1 and persisted schema 13: no new stored fields, migration, stable errors or renderer semantics. Existing duplicate_items and instance-update requests retain their meaning. Existing migration tests for current state and retained history remain required regression evidence.

## Non-goals

No deep-copy of definitions, component-local editing API, arbitrary property overrides, new template registry, redundant instantiation operation, renderer changes, new dependency edge, provider changes or UI work.

## Approval

Approved by the user on 2026-09-06 with the message "Approve" after reviewing the proposal, design, specifications and tasks.
