## Why

Independent review of issue #43 reproduced three defects despite passing regression suites: eviction of a stale draft's base can make its project inaccessible, extended visuals lose inherited transition timing, and sampled-video encoder failures expose raw private paths and oversized diagnostics. This correction makes those failure cases explicit and verifies them independently.

## What Changes

- Preserve project access and stale-draft revision-conflict behavior when retained history no longer contains the draft's base; never replay it against an unrelated current revision.
- Apply canonical inherited transition clocks to extended visual preparation across frame, range, draft, and export intents.
- Sanitize and bound sampled encoder diagnostics using the existing core process policy, preserving typed failures and cleanup.
- Add focused regressions, synchronize living requirements, and qualify previous archived evidence with this correction's results.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `project-persistence`: atomic schema-27 adoption and retained draft validation explicitly cover unavailable historical bases.
- `rendering-export`: shared extended rendering explicitly covers inherited transition timing and safe sampled preparation failures.

## Impact

The owning implementation is `crates/editor-core` persistence, render planning/evaluation, and process diagnostics, with core and headless regression evidence. Headless and MCP remain adapters; no parallel domain validation or dependency edge is introduced. Public operations, aliases, canonical catalogs, stable error codes/retryability, schema version 27, and CODEOWNERS remain compatible. No new request or persisted shape is proposed; any unexpected contract change requires a separate approved amendment and parity review.

## Non-goals

No new channels, effects, migration version, renderer intent, or transition feature. No protected CI changes, golden regeneration, commits, pushes, or PR creation. Preserve existing uncommitted work. Previous green checks establish only their tested scenarios, not correctness of these three reproduced cases.

## Approval

Approved by the user in this conversation on 2026-09-30 with the explicit message "approve", after the proposal, design, two delta requirements and tasks were presented. This approval covers D1-D3 and the listed verification work.
