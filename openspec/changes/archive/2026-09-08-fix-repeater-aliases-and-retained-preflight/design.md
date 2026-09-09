## Context

The current evaluator projects only visible source layers and skips hidden or clipped instances. Its global arithmetic preflight checks offset powers but not every retained occurrence's final geometry. Flat evaluation also invokes an obsolete repeater helper that clones all ordinary layers even though its callers exclude repeaters. Batch UpdateItem resolves only itemId despite accepting source aliases in its descriptor.

## Goals / Non-Goals

Goals: close all three review findings with measurable regression coverage; retain canonical ownership and stable rendering facts. Non-goals: new public types, dependencies, migrations, source kinds, or renderer semantics.

## Decisions

### Resolve replacement aliases in core

Give UpdateItem its own resolver branch. Resolve itemId first, then the optional repeater.source.id, using resolve_alias. Keep source.scope literal and retain existing VALIDATION_FAILED behavior for missing/forward aliases in batches. Drafts keep their existing EditOperation list and literal-ID semantics; test descriptor replacement with IDs already resolved from a committed batch, without introducing draft aliases. Transport-side substitution was rejected because it would split domain ownership and leave native callers inconsistent.

### Separate validation metadata from visible publication

Build a bounded retained-occurrence projection using source references and lightweight transform, interval, clock, binding, ownership and order metadata. Traverse hidden items/tracks, hidden or clipped instances, and nested component repeaters with the existing graph guards. Resolve effective slots in each owning scope before descending. Generated occurrences reference their source data instead of cloning evaluated geometry or rich documents.

Validate one complete root expansion and every component definition as a separate virtual composition with its own canvas and default slot resolution. Actual root instances use their effective overrides and composed transforms. Reset aggregate budgets between independent validation domains; never sum independent definitions into the root budget. Within each domain include ordinary and generated occurrences, even if hidden or inactive; interval intersection and visibility determine publication only. Validate finite arithmetic and parent-conjugated matrices even for empty output intervals.

Reusing visible expansion with hidden flags cleared was rejected: it would materialize invalid copies and risk changing output semantics. A second approximate geometry estimator was rejected because boundaries would drift.

### Share exact measurement and delay materialization

Adapt the current occurrence measurement and checked accumulators to consume lightweight occurrence views, using the same geometry, final matrices and resource-limit rules as refinement. Complete all domain checks before any generated EvaluatedVisualLayer clone. Retain only bounded metadata or recompute it during publication; do not retain one deep shape per projected copy. Publish the existing visible facts afterward, preserving IDs, numeric order, half-open intervals, opacity, clocks and RichText scope isolation.

Remove expand_flat_repeaters and its call entirely. The flat evaluator produces ordinary layers; the common repeater path owns all copy generation. A no-repeater fast guard inside the obsolete helper was rejected because it retains two semantic implementations.

### Record traceable evidence

Use test-only, thread-local materialization counting at the single copy-publication point to demonstrate zero generated clones on failed preflight. Verify output and no-I/O guarantees through existing facade tests. Add a dated follow-up note to the prior archived proposal/tasks explaining that its tests did not cover replacement aliases, retained geometry budgets or redundant snapshots; preserve historical commands and results.

## Risks / Trade-offs

- Retained traversal increases work: bound source closures first and stream shared measurements without retaining generated geometry.
- Independent definitions could be double-counted: explicit fresh budget domains and declaration-order tests.
- Slot resolution or interval clipping could alter visible semantics: keep publication filtering separate and compare nested RichText, transforms, identities and golden output.
- Alias error precedence could drift: itemId resolution precedes source resolution, with rollback and missing-alias tests.

## Migration Plan

No migration or public contract shape change. Preserve schema 17 and protocol 1. Rollback is an implementation revert; persisted projects remain compatible. Run all gates, verify, sync and archive before reporting completion.

## Open Questions

2026-09-08 inspection found that native draft creation/update accepts Vec<EditOperation>, not Vec<BatchEditOperation> (apps/headless/src/main.rs), and validation/materialization calls apply_operation without an alias map (timeline::validate_operations_against and EditorCore::get_draft_state). The previous promise of draft alias resolution was therefore incorrect. The user approved the corrected draft compatibility scope on 2026-09-08; all other approved decisions remain unchanged. No open questions remain.
