## Why

Issue [#35](https://github.com/matiHirCab/AI-Open-Cut/issues/35) requires advanced text layout and deterministic auto-fit. Existing pinned shaping and styled text support wrapping, line spacing and backgrounds, but lack an explicit bounded layout box, tracking, vertical alignment and fit modes.

## What Changes

- Add an optional closed `TextStyle.layout` contract for tracking, line height, bounds, wrapping, vertical alignment, background corner radius and `none|shrink|fit_width|fit_box` fitting.
- Resolve layout from pinned fonts in editor-core with a bounded deterministic font-size search and report resolved size/overflow in render diagnostics.
- Preserve exact legacy rendering when layout is absent; migrate current state and retained history to schema 21 atomically without enabling layout on old text.
- Carry the same evaluated layout through frame, range, draft and export, standalone edits, batch aliases and component/slot/repeater evaluation.
- Synchronize canonical contracts, capability reporting, native consumers, documentation and scenario tests.

## Capabilities

### New Capabilities

- `advanced-text-layout`: Opt-in box layout, deterministic fit, reversible editing and migration semantics.

### Modified Capabilities

- `rendering-export`: Shared evaluated advanced text layout and diagnostics across render intents.

## Non-goals

New text operations, per-span layout overrides, paragraph spacing, layout animation channels, font discovery changes, variable fonts, clipping/ellipsis, arbitrary background paints, desktop editing controls and provider changes are outside this issue. Existing background color/opacity/padding remain the background paint contract.

## Impact

Core model/validation, fonts/shaping, evaluated scene, render artifact rasterization, migrations and timeline lifecycle tests; typed headless and MCP style/diagnostic schemas; public capability and schema-version catalogs and parity fixtures. Preserve ADR 0003 ownership without new dependency edges. Public fields are additive, with existing simple requests and aliases retained; persisted schema 21 requires forward migration and old binaries must reject it rather than downgrade. Contract review is required from @matiHirCab under ADR 0002.

## Approval

Approved by the user in this task on 2026-09-19 ("Approve"), covering proposal, design, delta specs and implementation tasks. Final contract conformance review remains part of verification.
