# Editable native hero card

`native-hero-card-v1` is a versioned example returned by the existing
`create_motion_graphics` MCP prompt. List prompts, then request that prompt with
`projectId` and a `request` such as `Create native-hero-card-v1`. Retrieval is
read-only. The prompt includes the complete JSON recipe, including in a freshly
compiled bridge; the checked-in source is
[native-hero-card-v1.json](../apps/agent-bridge/src/recipes/native-hero-card-v1.json).

The reference project is 1920×1080 at 30 fps. Its two 800×600 cards occupy one
second and expand to fourteen visual leaves. Each shared component contains
named `background`, `device_frame`, `screen_panel`, `label_panel`, `accent`,
`artwork`, and `title` children. Five rectangles and editable text remain native;
only the artwork is a managed image. The initial navy, light gray, and gold paints
are fixed. This example adds no palette bindings or renderer recipe execution.

## Create and customize

1. Open the project and inspect its current revision, canvas, overlay track, and
   managed assets. Import approved original artwork with `asset_import` if needed.
   Imports are separate committed operations; a later recipe failure does not undo
   those imports. The synthetic reference artwork is 64×64 pixels, scaled four
   times. Other images retain their existing intrinsic dimensions and explicit
   transform; there is no automatic fitting.
2. Parse the JSON after the prompt's `native-hero-card-v1 recipe JSON:` marker.
   Replace exact string values `$overlay` and `$artwork` with the existing root
   overlay track ID and safe managed image asset ID. Do not evaluate the data or
   perform substitutions inside other strings.
3. Submit `recipe.operations` to `timeline_batch_edit`, with `projectId` and the
   current `expectedRevision`. The existing batch aliases `hero`, `left`, and
   `right` connect component creation, two instances, and one opacity channel in
   a single atomic revision. Read the returned project to obtain persistent IDs.
4. Use `component_instance_update` on one instance with its component ID, existing
   timing fields (`startMs`, `durationMs`, `trimStartMs`, `timeScale`), and
   typed slot values as shown below. Inspect the live tool schema for the complete
   request. Titles accept 1–24 Unicode scalars;
   artwork must reference an image managed by the same project. Preserve the
   instance's current timing. The update preserves its transform and animation;
   the other instance and shared definition remain unchanged.
5. Preview a frame with `preview_render_frame` at `timeMs: 500`, then use
   `preview_review_range` with `startMs: 0`, `endMs: 200`,
   `resolution: "project"`, and `includeAudio: false`. Poll each job to completion.
   The reference range contains six frames; the left card fades from hidden to
   visible over its first 100 ms. The left instance uses the existing legacy
   transform accepted by the scalar `transform.opacity` channel; its children and
   the right instance use ordinary Transform2D geometry.
6. Exercise `project_undo`, `project_redo`, and `project_open`, using each current
   revision. The project persists normal component geometry, slots, and channels,
   so subsequent previews do not retrieve or execute this recipe.

The customization's `slotValues` object is:

```json
{
  "title": { "type": "text", "value": "EDITED HERO" },
  "artwork": {
    "type": "asset",
    "value": { "kind": "asset", "scope": "project", "id": "replacementAssetId" }
  }
}
```

Replace `replacementAssetId` with the existing managed image ID.

For another canvas, explicitly adapt placement through existing operations after
inspecting the project. Prompt retrieval never changes project settings. A stale
revision fails with the existing retryable `REVISION_CONFLICT`. A late invalid
batch edit or absent component media asset fails with `ITEM_NOT_FOUND` and leaves
the previous project, history, and resource bytes unchanged.

## Verification and limits

The source integration and freshly assembled package smoke tests exercise real
MCP discovery, creation, slot replacement, alias rollback, revision conflicts,
history, and reopening. Their fake media tools do not establish native rendering.
The required default source and packaged native platform entry points additionally
render PNGs before and after customization and the animated range using actual
FFmpeg. Independent pixel checks establish geometry, palette, artwork replacement,
title changes, an unchanged second card, six-frame timing, and frame/range parity.
Blank-frame negative controls fail the same independent oracle.

The scene uses the canvas and native construction approach established by the
[illustrated scene benchmark](illustrated-scene-benchmark.md). This example has
its own bounds: two instances, fourteen leaves, one animation channel, and a
six-frame range. The benchmark's one-instance/static-slice caps and timing goals
do not constitute measurements or guarantees for this animated example.
These checks establish headless and packaged MCP behavior, not installation or
interaction in a separately running desktop application.
