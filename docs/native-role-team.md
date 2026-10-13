# Editable role badges and bounded team layouts

Retrieve the existing `create_motion_graphics` MCP prompt with `projectId` and a request mentioning `native-role-team-v1`. Parse its JSON between `native-role-team-v1 recipe JSON:` and `END native-role-team-v1 recipe JSON`. Retrieval is read-only. The canonical data is [native-role-team-v1.json](../contracts/native-role-team-v1.json); existing motif and final hero blocks remain available to their original readers.

The fixed reference canvas is1920×1080 at30fps and lasts1000ms. One800×600 badge copies the seven editable native hero roles (title becomes `role_label`) and two original sun-motif roles translated480px horizontally. Its nine children have stable names and paint order: background, device_frame, screen_panel, label_panel, accent, artwork, role_label, rays, sun_disc. Geometry/text remain native; artwork alone is managed image media. Maximum four badges expand to36visual leaves, plus one nonvisual group and four opacity channels.

## Create and customize

1. Inspect the current project revision, canvas, root overlay track and managed assets. Import the fixed original64×64 split-white/red BMP (or approved original artwork) separately with `asset_import`. Imports are independent commits. Artwork retains intrinsic size and an explicit4× transform; arbitrary images are not automatically fitted.
2. Replace only exact JSON string values `$overlay` and `$artwork` with the existing overlay track ID and managed project image ID. Do not evaluate the data or replace text substrings.
3. Select exactly one `layouts.row["1".."4"]` or `layouts.grid["1".."4"]` array. Submit `[badgeCreate, badgeDefineSlots, ...selectedArray]` in one `timeline_batch_edit` with the current expectedRevision. The first operation creates geometry with empty slots; the next uses the earlier `@badge` alias to define slots. The selected array creates an ordinary parent group, adds ordered members and stores their channels. The largest batch has15operations.
4. Retain aliases `badge`, `team`, `member_1` through `member_4` from the result as real persistent IDs. These aliases identify members and their order; they do not add a persisted item-name field. For another canvas or count, explicitly author ordinary operations after inspecting project settings; this example offers no solver or fifth-member recipe and adds no core limit.
5. Update one occurrence using existing `component_instance_update`, its real item/component IDs, preserved startMs/durationMs/trimStartMs/timeScale and a complete `slotValues` map. Role label accepts1–24Unicode scalars, with existing resolved-text/layout validation still authoritative. The artwork slot references an image managed by the same project. Seven `shape.fill.color` slots (`background_color`, `device_frame_color`, `screen_panel_color`, `label_panel_color`, `accent_color`, `rays_color`, `sun_disc_color`) accept existing hex6 values and preserve paint alpha. Boolean `rays_hidden` hides only the rays. Shared geometry and other occurrences remain independent. Explicit `{}` restores defaults; an omitted map retains overrides.

For example:

```json
{
  "role_label": { "type": "text", "value": "TEAM LEAD" },
  "artwork": { "type": "asset", "value": { "kind": "asset", "scope": "project", "id": "replacementImageId" } },
  "background_color": { "type": "color", "value": "#00ff00" },
  "rays_hidden": { "type": "boolean", "value": true }
}
```

Replace replacementImageId with a real managed image. Missing images/invalid labels/types, unresolved aliases, locked tracks and invalid trailing edits retain existing typed errors and preserve the complete pre-request state. A stale revision yields retryable REVISION_CONFLICT: refresh and replan. Prior asset imports remain committed if construction fails.

## Placement and reveal

All members use0.5scale and ordinary parent membership. Row positions for member index i0..3 are(100+450i,390); grid positions are(480+560(i mod2),160+400 floor(i/2)). Counts1,2,3,4 each have a complete explicit operation list. Move the parent group with existing Transform2D edits to preserve relative arrangement.

Each member has an existing legacy-transform-compatible transform.opacity channel:0 at80i ms,1 at80i+100ms, linear between and clamped outside. The group adds no second stagger. These ordinary stored keys remain authorable and survive undo/redo/reopen; there is no baked video or executing recipe. Review frame500ms and silent range[0,400)ms (12frames at30fps). Exercise project_undo/project_redo/project_open with current revisions after construction/customization/reset.

## Evidence and limits

Core and typed fixture consumers cover supported counts/layouts, boundaries, independent values, atomic failures, locks/revisions and history. Actual source and freshly assembled default-package MCP clients retrieve the same data, create minimum/maximum teams and customize one instance. Required native tests use actual tools/font and independently authored interior/exterior, palette, image, glyph and stagger samples, preserving PNG3/encoded8 channel tolerances and prior hero/motif assertions. Synthetic negative controls establish verifier rejection and are not rendered-output evidence.

The64×64 reference artwork is original deterministic split white/red; the independent replacement is split white/blue. No external artwork, image generation, tracing, flattened native plate or random geometry is used. Fixed label/count/image/layout bounds do not imply arbitrary fitting, installed-desktop readiness, alpha-preserving exported plates or portable performance. Source/package/environment evidence is reported separately.

PNG checks cover every authored visual role. Lossy encoded range checks cover broad paint interiors and temporal states at tolerance8, with separate fixed-region yellow accent/ray and white glyph presence checks. Independent omission controls reject absent accents, rays and labels. These presence checks do not establish exact thin-feature or glyph RGB fidelity after lossy encoding.
