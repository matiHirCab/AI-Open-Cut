## Why

The 2026-09-08 review reproduced acceptance of 4,369 transition facts from a component with 17 transitions and 256 additional copies, exceeding the existing 4,096 limit. It also reproduced a repeated component whose media slot substitutes audible video for silent video: canonical validation accepts it and evaluation emits 257 visuals and one audio layer despite the visual-only source restriction.

## What Changes

- Count transition facts over the complete retained occurrence projection before generated-layer materialization, with independent root and definition domains.
- Validate repeater source audio against effective defaults and overrides, including nested components and local repeaters, through canonical core validation.
- Separate non-recursive slot application from full component validation to prevent circular validation calls; retain graph guards and effective-value-aware reuse.
- Add boundary, isolation, rollback, transport and no-artifact regressions and a dated qualification of the previous archived verification.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `repeaters`: Complete transition budgets and effective visual-only closures, independent of visibility.
- `timeline-editing`: Atomic rejection of effective-audio repeater sources in edits and drafts.
- `agent-bridge`: Transport parity for effective-audio rejection and unchanged transaction state.

## Impact

Core owns validation, slot resolution and scene projection changes. Headless and bridge receive regression tests, not duplicate domain logic. Schema 17, protocol 1, public request/response shapes, error catalogs, deterministic IDs and persisted formats remain unchanged. No new dependency, migration or breaking public contract change is proposed.

## Non-goals

No audio repetition, new slot kinds, changes to visible transition behavior, or cleanup of unrelated issue-31 work. Preserve every existing uncommitted change.

## Approval Status

The user explicitly approved these concrete proposal, design, three deltas and tasks on 2026-09-08 ("Approve"), before executable edits.
