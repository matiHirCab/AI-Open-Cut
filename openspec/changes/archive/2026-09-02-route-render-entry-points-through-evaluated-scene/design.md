## Context

Issue #12 introduced a private, owned `EvaluatedSceneResult { scene, resource_bindings }` and hardened it as a pure, bounded editor-core derivation. Production rendering still calls `render_plan::evaluate_scene`, which evaluates and discards that envelope and then builds a borrowed `SceneEvaluation` containing `Project`, `Track`, `TimelineItem`, and `Asset` references. `build_render_plan` consequently re-resolves item ordering, transitions, transforms, audio automation, fades, roles, and ducking for every frame preview, range preview, draft preview, and export.

The headless operations already validate immutable project revisions before normal preview/export, while draft preview materializes a validated draft state and sends its uncommitted project snapshot through the same `Renderer::render_preview` facade. Resource paths, temporary workspaces, FFmpeg execution, progress, and publication are already isolated behind editor-core adapters. The change must replace only the semantic input seam, preserve these orchestration boundaries, and keep existing requests compatible.

## Goals / Non-Goals

**Goals:**

- Make every production render intent consume the same owned `EvaluatedScene` semantics.
- Make resource preparation consume the separate path-bearing binding sidecar without putting paths into `EvaluatedScene` or re-reading persisted records.
- Convert evaluated visual/audio instructions into the existing deterministic FFmpeg plan without semantic loss.
- Delete the borrowed project-backed scene representation and enforce that it cannot return.
- Preserve failures, immutable revisions, draft isolation, path safety, artifact atomicity, and simple output compatibility.
- Publish additive `evaluated_scene_rendering` capability evidence across the governed Rust, TypeScript/Zod, MCP, and canonical contract surfaces.

**Non-Goals:**

- Persisting or publicly serializing `EvaluatedScene` or its resource bindings.
- Adding project schema fields, migrations, timeline operations, creation aliases, masks, effects, hierarchy, components, executable SVG, network fetching, or arbitrary paths.
- Adding a new renderer backend or changing FFmpeg/AAC/H.264 output formats.
- Changing stable error identifiers, revision rules, history, or draft lifecycle behavior.

## Decisions

### Evaluate once, then carry the owned envelope through preparation

Each public `Renderer` entry point will call `evaluated_scene::evaluate_project` exactly once for the selected immutable project or draft snapshot. A read-only media preflight then validates binding closure, derives deterministic input indexes, canonicalizes requested media paths, and enforces project-root containment. Only after that preflight succeeds may the facade inspect export collisions, allocate temporary names, create a workspace, prepare text, or write filter files. The prepared media envelope is passed into `Renderer::prepare_render`, so paths and input indexes are not recalculated. `RenderIntent` remains a planning/execution concern so one full-scene evaluation supports frame, range, and export clipping without placing output destinations or backend concepts in the scene.

Alternative considered: evaluate a separate scene per intent. Rejected because intent-specific evaluation can reintroduce semantic drift and makes parity harder to prove. Alternative considered: keep the borrowed `SceneEvaluation` as an adapter. Rejected because it preserves the duplicate traversal issue #13 is required to remove.

### Derive backend input instances from evaluated instructions

The planner will assign deterministic FFmpeg input instances by walking evaluated visual and audio layers in stable first-use order and resolving each logical asset through `SceneResourceBindings`. Multiple timeline items may use one logical asset but retain their own item timing, trim, transform, and audio facts. Prepared filesystem paths remain outside the scene and are attached only to the backend plan.

Text preparation will consume evaluated text records and logical font IDs, then resolve those IDs through the font bindings and existing root-constrained font policy. It will not inspect `TextItem` or any project record. Missing or inconsistent internal bindings fail before FFmpeg execution or artifact publication.

Alternative considered: put prepared paths or FFmpeg input indexes into `EvaluatedScene`. Rejected because both are backend/resource-preparation facts and would violate the renderer-neutral contract. Alternative considered: recover paths by looking up project assets in the planner. Rejected because that recreates the forbidden persistence dependency.

### Translate only closed evaluated instructions

`build_render_plan` will match the closed evaluated visual and audio instruction types. Animation expressions may still be formatted as FFmpeg syntax downstream, but all timing, keyframe, transition, ordering, mute, fade, automation, role, and ducking inputs must come from evaluated records. The planner may clip those records for `Frame`, `Range`, or `Export`; it may not reinterpret project semantics.

Alternative considered: compare both planners in production and choose the old result on mismatch. Rejected because fallback would retain two canonical interpretations and could silently degrade output. Parity is established in tests before the old path is removed.

### Prove parity at semantic and artifact layers

Deterministic non-empty unit fixtures will assert exact `RenderPlan` equality across render intents after normalizing intent selection while covering media reuse, text/font selection, shapes, transforms, animation, transitions, fades, automation, roles, ducking, and stable ordering. Hermetic renderer-facade tests will prove every entry point invokes evaluation and canonical media preflight before mutating artifact/process adapters and shares failures. An FFmpeg integration fixture will compare sampled frame-preview, range-preview, and export frames at the same settings with SSIM at least `0.99`; aligned decoded float PCM over their shared interval must have RMS error at most `0.0001`, and stream timing may differ by no more than one output video frame. Linux CI installs FFmpeg, FFprobe, and a deterministic font, passes their explicit paths, and treats a configured dependency that cannot execute as a test failure. These tolerances cover codec/container effects while semantic plan assertions remain exact.

Alternative considered: require byte-identical PNG/MP4 artifacts. Rejected because container metadata and lossy H.264/AAC encoding are not a semantic difference. Alternative considered: test only filter strings. Rejected because resource binding, command construction, seeking, encoding, and publication could still diverge.

### Advertise support additively

The rendering subsystem and aggregate editor status will include `evaluated_scene_rendering` only when rendering is ready. Existing request operations and response shapes stay unchanged; capability arrays already accept additive string identifiers. The canonical headless catalog remains version 1 and is updated together with Rust status evidence, TypeScript/Zod validation, MCP status schema/catalog evidence, and contract parity tests.

Alternative considered: bump the protocol major version. Rejected because no field or operation is removed, narrowed, renamed, or reinterpreted incompatibly. Alternative considered: expose the internal scene over headless/MCP. Rejected because clients need feature detection, not backend-neutral internal records.

### Treat persistence and mutation acceptance criteria as preservation tests

This routing change performs no project mutation and no schema change. Reopen, undo/redo, retained drafts, optimistic revision conflict, and atomic batch alias behavior are covered by regression tests showing that rendering neither mutates nor migrates state and that existing mutation behavior remains unchanged. There is no migration or new standalone edit operation to manufacture.

Alternative considered: bump the project schema to record renderer support. Rejected because evaluated scenes are derived process-local state and a schema bump would create unnecessary migration risk.

## Risks / Trade-offs

- [Risk] The new adapter omits a legacy planner detail. → Port behavior by evaluated instruction category, retain focused legacy fixtures, assert exact semantic plans, and add end-to-end visual/audio tolerance evidence before deleting the borrowed path.
- [Risk] Reusing one logical asset for multiple items changes FFmpeg input ordering or trim behavior. → Derive backend input instances from ordered evaluated layers and test repeated assets with distinct source intervals and audio settings.
- [Risk] Font selection regresses when text no longer exposes `TextItem`. → Resolve logical font IDs exclusively through the binding sidecar and cover custom path, family, default, and rejected-path cases.
- [Risk] Capability reporting drifts across transports. → Update canonical catalogs and all governed consumers together and run the standalone contract parity gate.
- [Risk] Codec-based golden checks vary by FFmpeg build. → Keep exact semantic assertions authoritative and use deliberately tolerant SSIM/RMS integration thresholds with fixed synthetic sources and pinned command settings.
- [Risk] Canonical or symlink path escapes are discovered after temporary workspace creation. → Resolve media paths in a read-only pre-workspace phase, pass the prepared envelope forward, and assert no workspace, write, process, collision, or publication activity for unsafe paths.

## Migration Plan

1. Add failing scene-to-plan, resource-binding, entry-point, architecture, contract, and parity tests.
2. Adapt resource preparation and render planning to `EvaluatedSceneResult` while preserving current process and artifact interfaces.
3. Switch the renderer facade once, thereby routing normal frame/range/export and materialized draft preview through the same path.
4. Remove `SceneInput`, borrowed `SceneEvaluation`, and all project/timeline traversal from render planning; strengthen architecture enforcement.
5. Add the capability/catalog/documentation updates and run the full required verification matrix.

Rollback reverts the internal planner/resource adapter and additive capability/catalog changes together. No persisted project or history rollback is required because schema version 6 and on-disk data remain unchanged.

## Open Questions

None. The capability name and deterministic tolerance are fixed by this change for review before implementation.
