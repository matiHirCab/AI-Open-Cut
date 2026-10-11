## Context

Verified main 50ec842c contains #181's bounded illustrated benchmark and the independent doctor fix. Existing component_create/component_define_slots, add_component_instance/component_instance_update, update_item animation channels, timeline_batch_edit, job rendering and history/reopen support the requested behavior. Existing create_motion_graphics is discoverable and returns deterministic read-only guidance. ADR0003/0004 and living component/template/animation/render requirements remain authoritative.

## Goals / Non-Goals

Goals: one concrete versioned native recipe discoverable through current MCP, two independently editable card instances, and actual native/lifecycle evidence. Non-goals follow the proposal; no renderer or schema work.

## Decisions

1. Include a static versioned JSON recipe in existing prompt text and document the same data. This is example data, not a new interpreted execution engine: clients resolve explicit placeholders to an existing root overlay and safe managed image identifiers before submitting ordinary operations. New tool/resource registrations were considered but existing prompt retrieval and atomic edits are sufficient; their structural catalogs/pins stay unchanged. Prompt retrieval cannot edit/import/render or invoke providers. Bun includes recipe data in the compiled bridge; runtime packaging needs no external recipe file.
2. Use one 800x600 component, seven named leaves (background, device_frame, screen_panel, label_panel, accent, artwork, title), fixed solid native geometry/paints and existing title (1–24 Unicode scalars) and artwork(image) slots. Example canvas 1920x1080, 30fps, duration 1000ms, two card instances contained in it. Original test artwork is 64x64 with analytic colors. Existing image replacement uses native dimensions/transform; no automatic fitting or new image-dimension validator is advertised. Preview remains necessary for arbitrary supported replacements.
3. Create definition and instances with existing creation aliases in one revision-checked batch; compile one simple compatible transform.opacity entrance channel into ordinary stored keyframes on the first instance, using its compatible legacy static transform (typed visual channels and Transform2D cannot coexist). The second instance and native children may retain Transform2D. Renderers never execute recipe/preset identifiers. Shared tracks are unchanged by instance overrides. No core validation is duplicated in the bridge; tests resolve only literal recipe placeholders and exercise real core failures.
4. Share MCP workflow assertions across source and freshly assembled packaged smoke. Reuse existing source/packaged platform-native entry points for full-resolution PNG and short range [0,200)ms (six frames) with actual dependencies. Verify independent solid/interior/artwork/glyph samples, native before/after title-region change with unchanged right-card pixels, initial-hidden versus final-visible animated states, frame/range parity within existing codec tolerance, state immutability and reopen. Existing platform/canonical report schemas, goldens, guards and mandatory commands remain unchanged; additional evidence is separate.

## Risks / Trade-offs

Placeholder misuse -> explicit instructions plus real missing-asset and late-invalid batch rollback controls. Existing artwork sizing is manual -> document the 64x64 reference and preview/ordinary transform adjustment. Native tests cost -> bounded two instances/fourteen leaves, six frames, no dense/full-duration output or universal timing claim. Point/glyph/layout checks are narrower than full-image goldens -> name limits and retain existing conformance evidence. Public prompt content evolves -> preserve identifiers/arguments, side-effect-free deterministic retrieval and unchanged structural catalog digests.

## Migration Plan

No migration or deployment. Removing the recipe guidance restores prior prompt content; ordinary created projects remain readable by current supported native models.

## Open Questions

None blocking this slice. Later palettes, packs, renderer strategy and desktop controls belong to their own issues.
