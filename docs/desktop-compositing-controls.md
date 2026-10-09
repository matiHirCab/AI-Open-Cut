# Desktop compositing controls

Open a project explicitly with `opencut-desktop --project-store <projects directory> --project-id <project ID>`. Desktop reads authoritative snapshots and submits existing revisioned core operations. Component-local content is read-only; audio eligibility comes from imported asset metadata.

Root Image/Video media, Text, SolidColor, Rectangle, Shape, SVG and Grid expose masks, matte, matteOnly, blend and effects. Caption exposes blend only. Group and ComponentInstance expose effects and composition-bounds clip; Repeater, Transition and audio-only Media have no compositing mutations. Controlled owners composite contiguous children onto a transparent local surface, clip, apply ordered effects, then transform/attenuate outward. Other layers keep track/z-index/stable ordering. Tree indentation describes parentage.

Select masks/effects with previous/next controls. IDs are immutable and follow reordering; deleting selects the surviving record at the old index or the preceding/empty record. Add allocates the first unused `mask-N`/`effect-N` across the complete collection. A new mask is a fixed64local-pixel rectangle with white solid paint, alpha/add, default transform, no inversion/feather/expansion. This default does not guarantee coverage of every source. Core enforces the16-record limits and rejects deletion of animation-targeted records atomically.

Mask fields expose channel, four operations, inversion, signed expansion, feather, all transform scalars/position units, fill rule and separate full-precision RGBA components. Path navigation edits only the selected command's existing endpoint/control coordinates. Close and command variants are read-only. Gradient geometry and the selected stop offset/RGBA remain read-only; editing another property retains every undisplayed command/stop. There is no implicit conversion to solid paint.

Effects retain authored order and every unedited field, clock and animation target. Add defaults are Gaussian radius2; Glow radius2/intensity0.5/white; Tint white; Vignette0.5; Adjustment exposure0/contrast1/saturation1; Flash start0/duration200/intensity0.5/white; Particle count16/seed1/radius2/speed20/lifetime1000/white. Every parameter of these seven types is editable, including exact unsigned16 count and unsigned32 seed/timing values. The seed4294967295 endpoint is representable; representation parsing and core semantic bounds are separate. RGBA and other floating values use finite f64 without byte quantization or clamping.

A matte requires an explicit provider item ID and initially uses alpha; replacing an existing source retains its channel. Alpha/luma switching, explicit Clear matte and matteOnly use shared core semantics. Group/Instance clip can be set to `composition_bounds` or explicitly cleared while retaining effects. Unsupported aggregate leaf controls are absent.

Select a field, type a value and press Enter/Apply. Ctrl+A clears the input; Esc/Reset discards it without mutation. Drafts bind selection, revision, animation and compositing cursors and the exact field. Navigation, collection actions, refresh/history and successful edits discard obsolete input. Busy and refresh-required sessions guard authoring. Core failures retain their code/message/retryability; revision conflicts require explicit Refresh with no automatic retry. Undo/redo and reopening reconstruct committed state without saved UI cursors.

## Existing API examples

These use the unchanged `update_item` core/headless operation and `timeline_update_item` MCP tool. Collections are complete ordered replacements; omit a field to preserve it, use empty arrays to clear masks/effects and explicit null to clear matte/clip. Schema37, protocol1,78registered tools and all current/predecessor MCP pins remain unchanged.

The JSON snippets below are core edit bodies. For headless, put the body in an `edit` request alongside `projectId` and `expectedRevision`. For `timeline_update_item`, pass those revision fields and the edit fields without `operation`.

```json
{"operation":"update_item","itemId":"leaf","blendMode":"screen","matte":{"sourceId":"provider","channel":"luma"},"matteOnly":false}
```

```json
{"operation":"update_item","itemId":"group","clip":{"type":"composition_bounds"},"effects":[{"type":"screen_flash","id":"flash","startMs":0,"durationMs":200,"intensity":0.5,"color":{"r":1,"g":1,"b":1,"a":1}}]}
```

Creation IDs can be resolved within `timeline_batch_edit` aliases, including matte providers. Duplicate/missing IDs, cycles, locks, stale revisions and later-operation failures remain core-owned and preserve complete project/history/resource inventories. Existing preview, draft and export use the same evaluator. This change adds presentation and conformance evidence without changing rendering, persistence or public payloads.

## Desktop and native verification

Build the affected app and create a new disposable deterministic project through the actual headless API:

```sh
cargo build -p opencut-headless -p opencut-desktop
python3 scripts/create-desktop-compositing-fixture.py /tmp/new-compositing-fixture --headless target/debug/opencut-headless
target/debug/opencut-desktop --project-store /tmp/new-compositing-fixture/projects --project-id <printed project ID>
```

The fixture refuses an existing destination. It includes a visual leaf with solid and gradient masks, Move/Line/Quadratic/Cubic/Close commands, retained mask/effect animation targets, a provider, Group, Instance/local occurrence and actual imported audio. Use real mouse/keyboard controls, compare authoritative core states, and retain screenshots/commands/environment for every acceptance transition. Test defaults, every scalar/parameter family, order/delete, read-only content, matte/blend/clip, parsing/core feedback, reset, selection, history/reopen and external conflict/Refresh. Headless tests or a running window alone do not establish this mandatory observed GPUI acceptance.

The central panel now offers the core-backed controls in [Desktop review](desktop-review.md). Configured native core/headless/MCP parity remains the independent production-media oracle. Run all required Rust, bridge, contract, integration, packaged smoke, hermetic Python, native golden/report/three rules resolutions/cache/default restore and OpenSpec gates described in contributor and rendering documentation.

The canonical `contracts/desktop-compositing-controls-v1.json` is checked in and reviewed manually. `bun run contracts:check` retains every predecessor consumer and additionally runs `cargo test -p opencut-desktop desktop_compositing` and its fresh TypeScript/MCP catalog conformance. On Ubuntu its isolated CI step installs signed `libxcb1-dev libxkbcommon-dev libxkbcommon-x11-dev` before the pinned toolchain; GUI execution is a separate gate. Protected preflight requires the actual regular bridge package, enforces the exact complete command and tracks it in Moon inputs before Moon execution/attestation.
