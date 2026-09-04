## Why

Issue #18 requires editable Transform2D semantics across the core, agent API, and renderer. Schema v7 currently provides common visual ownership but only legacy pixel position, uniform scale, and opacity; the richer vocabulary remains fixture-only.

## What Changes

- Add an optional, complete `transform2d` value to common visual properties while retaining the legacy `transform` shape and behavior. Support explicit position units, normalized anchor, independent positive scale, rotation, skew, and opacity.
- Define one bounded affine evaluation and render path with normative anchor, scale, skew, rotation, position ordering.
- Migrate schemas 1–7 and all retained history atomically to schema 8, with `transform2d` absent by default so legacy rendering remains unchanged.
- Extend existing item updates and batch updates additively, advertise `transform2d`, and synchronize governed contracts and regression evidence.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `timeline-editing`: Transform2D ownership, validation, standalone and batch mutation semantics.
- `project-persistence`: Schema-v8 migration, deterministic history and reopen behavior.
- `motion-graphics-architecture`: Canonical affine evaluation and explicit milestone boundary.
- `motion-graphics-contracts`: Typed runtime transform adoption alongside remaining fixture-only concepts.
- `rendering-export`: Complete affine rendering shared by preview, draft, and export.

## Impact

Core models, migrations, validation, timeline mutation, evaluated scenes, renderer planning/preparation; typed headless and bridge schemas/capabilities; canonical catalogs and consumer parity tests; ADR 0004 and coordinate documentation. Contract review belongs to @matiHirCab under the ownership catalog. Public extensions are additive; schema 8 requires a compatible reader and older readers must reject it. No removal or reinterpretation of legacy fields is proposed.

## Non-goals

Hierarchy, components, masks, effects, z-order changes, new animation channels, negative/zero scale, new desktop controls, provider protocols, GPU compositors, and global changes to legacy color/compositing behavior are excluded. Existing animation remains supported with legacy transforms; Transform2D and legacy transform animation on the same item are rejected explicitly in this milestone.

## Approval

Approved by the user in this task with the message "Approve" on 2026-09-04, covering the proposal, design, delta specifications, and implementation tasks.
