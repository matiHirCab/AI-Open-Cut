## Why

Historical follow-up (2026-09-08): later review found missing coverage for update-item source aliases, complete retained-content geometry budgets and the obsolete flat snapshot. Original commands and tasks.md results remain historical evidence, not proof that those cases conformed. The approved [follow-up correction](../2026-09-08-fix-repeater-aliases-and-retained-preflight/proposal.md) addresses these gaps with fresh verification tasks.

The schema-17 repeater implementation can omit component-instance occurrences when a repeater targets an ancestor group, and its generated ordering can compare textual occurrence identifiers instead of the source subtree's numeric order. The migration suite also does not exercise schema 16 across every persistence fault phase even though the archived change records that coverage as complete. Subsequent conformance reviews additionally found that nested group controllers are undercounted during occurrence preflight, component-local repeater copies can retain pre-override text, recursive component/group preflights can run before canonical cycle validation, and generated visual work can be cloned before all expanded scene budgets are known.

## What Changes

- Evaluate each composition scope's ordinary shape and fully resolved component occurrences before expanding repeaters, so a group copy includes its complete transitive visual occurrence set.
- Preserve numeric source-subtree order inside every generated copy, including subtrees with ten or more equal-z siblings and mixed shape/component descendants.
- Extend native render parity coverage to a group that actually contains a component instance.
- Exercise schema 16-to-17 migration before journal creation and after every publication phase, including current state, undo/redo history, generation recovery, and transaction cleanup.
- Count complete transitive group source closures during the existing 65,536-occurrence preflight, including hidden/inactive exact-boundary cases.
- Apply evaluated rich-text slot documents before a component scope freezes its ordinary-source snapshot so local and outer repeater copies preserve identical text runs and styles.
- Validate all project parent/reference/cycle invariants before recursive SVG, component, repeater, or audio preflight, while retaining active-node guards in every recursive traversal as defense in depth.
- Project the complete expanded layer, segment, composed-matrix, raster-surface, and aggregate-memory work for all repeaters in a scope before cloning or publishing any generated occurrence.
- Replace the deep ordinary-layer snapshot with bounded lightweight source indices and metadata so rejected work cannot allocate its would-be copies first.
- Add a dated completion note to the archived `add-lazy-repeaters` approval evidence without erasing its original chronology.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `repeaters`: Clarify complete group occurrence expansion and numeric preservation of source-subtree order with explicit regression scenarios.
- `rendering-export`: Require parity evidence for a repeated group containing resolved component descendants.
- `project-persistence`: Require schema 16-to-17 coverage at every existing persistence fault boundary.

## Impact

The implementation is confined to `editor-core` validation/evaluated-scene assembly, its native renderer fixtures, persistence tests, and OpenSpec evidence. Public request/response types, MCP schemas, canonical contract catalogs, protocol identifiers, error semantics, and project schema version remain unchanged. The correction is behavior-compatible for conforming inputs: previously missing occurrences become visible, incorrectly ordered generated layers assume their specified order, excessive or invalid recursive graphs fail deterministically at canonical preflight, expanded scene budgets fail before generated-layer materialization, and repeated rich text receives the already-selected instance binding.

## Non-goals

This change does not add source kinds, time offsets, authoring controls, persisted expansion, schema 18, migrations, downgrade behavior, new capabilities, or provider/renderer-specific repeater logic. It preserves existing validation outcomes, preflight limits, public aliases, and contract ownership while changing evaluation order and allocation timing to enforce them safely.

## Approval Status

The original revision was approved explicitly by the user in this task on 2026-09-07 with “Approve.” The nested-group preflight and rich-text amendments discovered by subsequent review were separately approved explicitly by the user on 2026-09-07 with “Approve,” covering that amended proposal, repeater delta, design, and tasks before further executable code edits. The safe-recursion and complete-budget-preflight amendments added after the latest review were approved explicitly by the user on 2026-09-07 with “Approve,” before executable implementation edits for this revision.
