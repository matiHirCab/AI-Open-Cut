# Editable native motif pack v1

`contracts/native-motif-pack-v1.json` provides ordinary example data through the existing `create_motion_graphics` prompt. Read its JSON between `native-motif-pack-v1 recipe JSON:` and `END native-motif-pack-v1 recipe JSON`. The prompt retains one message and its original final hero recipe.

Five components measure 320 × 240 and last 1000 ms. Their original geometry uses existing native shapes, structured paths and a dot grid. The reference canvas is 1920 × 1080 at 30 fps.

| Motif | Editable roles and reference geometry | Color bindings |
| --- | --- | --- |
| Dot backdrop | `dots`: 280 × 200 viewport at (20,20), 40 px spacing, 4 px radius, 48 inclusive centres clipped to the viewport | Dot paint |
| Sticker contour | `paper`: original Bezier contour with 12 px outline; `accent`: 64 px disc | Paper, outline, accent |
| Sun rays | `rays`: eight literal contours at 45-degree intervals; `sun_disc`: 64 px disc | Rays, disc |
| Fireball | `flame_outer`, then `flame_core`: original closed vector paths | Outer, core |
| Speed lines | `speed_lines_shape`: six closed 180 × 6 rectangles at (20 + 12i, 50 + 24i), i = 0…5, in one path | Shared path fill |

The set has eight visual leaves, its paired palette gallery has 16, and one set around the existing two-card hero has 22. Six speed-line contours count as one visual leaf. Component track input excludes repeater items; this example preserves that schema. These are reference counts, and existing core validation and render admission remain authoritative.

Nine existing hex6 color bindings preserve paint alpha. An instance override changes that occurrence without modifying shared geometry or another instance. Replace its complete `slotValues` map with `{}` to restore defaults. Ordinary transforms control placement without automatic fitting. The pack adds no random generator, external artwork, AI asset, tracing, flattened plate or scene model.

Create the five empty seeds together as Phase A. Retain the actual IDs returned under each `phaseAAlias`; replace only exact JSON placeholder values, including an existing root `$overlay` track. With a fresh revision, submit all five populate operations and either gallery or around-hero placements together as Phase B. Each phase is atomic. Failed B preserves the complete pre-B project/history state; committed empty A definitions remain. Refresh after revision conflicts, preserve locks, and verify ordinary undo/redo/reopen. The data is not an executable factory.

Review frame 500 ms and silent range [500,600) ms, which contains three frames at 30 fps. Transparent gaps and fractional paint reveal the background. Native output clears an opaque canvas, so this example does not promise alpha-preserving export. Preexisting hero artwork remains managed media; motif geometry remains native.

PNG checks retain literal RGB points within the existing three-level bound. H264/yuv420p range checks retain hero, fill, order, gap and exterior points within the existing eight-level bound. Only thin dot/strip checks use fixed, fully covered interiors: the central 2 × 2 dot pixels and four interior rows across four interior strip columns. Unrounded channel means use those same bounds. Expected values follow authored paint and existing linear-light source-over with 8-bit alpha; cyan at alpha 0.5 is RGB (0,156,188), and at alpha 0.75 is (0,186,224) over black. They are not fitted to captured output. Blank, corrupted-interior, wrong-paint and wrong-order controls must reject.

Around-hero placements use scale 1: dot (40,-20), sun (900,0), sticker (850,840), fireball (1600,840), speed (180,840). The dot viewport spans y = 0…200, leaving 20 px before the cards at y = 220; other top paint also ends before the cards; bottom paint starts below their y = 820 edge. Sticker stroke spans approximately y = 854…1066. Both prior card PNG regions must remain byte-identical.

Transport checks and actual native media checks are separate evidence. This pack makes no installed-desktop or portable-performance claim.
