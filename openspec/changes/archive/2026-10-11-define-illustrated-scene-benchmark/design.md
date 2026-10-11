## Context

Main `431a5e38` contains all seven completed foundations named by #181. Existing public EditorCore and Renderer facades support native shapes, text and managed images; color slots currently target text only. No active changes existed before this proposal. The existing native golden entry point is executed by the protected Linux render job at the exact published head.

## Goals / Non-Goals

Define and measure a small useful illustrated workload; establish three bounded later targets. Exclude new product presets, palette APIs, desktop edits, optimizations, precompositions and infrastructure. #182 owns the reusable hero-card product, #183 graphic slots, #184 motifs, #185 team layout, #186 diagnostics and #187 the measured rendering decision. #188/#189 remain conditional; #190 owns desktop and #191 end-to-end acceptance.

## Decisions

1. Keep test-only authoring/measurement in editor-core's existing renderer golden area, with a reviewed JSON recipe beside test fixtures. Use existing native edit/component/input semantics; no production dependency edge or public catalog. Alternative: a new benchmark service or bridge API would broaden scope and duplicate ownership.
2. Small benchmark: 1920×1080, 30 fps, 1000 ms static hero card. Named background, panel/frame, screen/artwork, accent and label roles; maximum 16 visual leaf items, one component instance, 64 text scalars, one original 64×64 opaque managed image, 128 total authored path commands, no effects/masks/repeaters or animation in the executable initial slice. Samples 0, 500 and 966 ms; measured frame 500 ms and range [0,100) ms (three frames). Three later composition inventories retain 1920×1080/30 fps, at most 3000 ms and explicit role/complexity caps, but do not require their implementation here.
3. Review independent rectangle/interior/exterior samples in recipe coordinates before execution. Test typed construction and named roles, retain hashes of the verified frame/range (optional local artifact copy), and decode actual output to check those samples and dimensions/frame count; do not derive expectations from renderer output. Text presence/layout is checked separately from antialiased glyph interiors. Synthetic artwork has an explicit analytic color pattern. Existing goldens, digest pins and codec tolerances are untouched. Alternative: captured output as its own oracle would miss coordinated drift.
4. Include the fixture's native evidence from the existing native golden conformance test after configured tools resolve. Do not change protected workflow commands or timing limits. A separate focused fixture test provides hermetic inventory validation; native measurement emits a clearly separate report from the canonical golden schema-3 report. All existing golden conformance remains in place.
5. Record three fresh Renderer frame and range calls plus three repeated calls using the same warmed Renderer. Timer covers facade request through returned artifact, excludes independent decoding; reused outputs remain pixel-verified. Report exact Git revision plus fixture/driver hashes and dirty source identity where applicable, executing test binary SHA256, FFmpeg/FFprobe versions, deterministic font hash and reference host identity. Fresh does not flush OS caches. Repeated encoded preview/cache hits are explicitly included. Choose rounded reference targets with documented headroom after measurements; do not assert elapsed timing across CI machines. Existing limits (including sampled composition output-frame pixel work 268435456, surface area 16777216 and composition memory 1073741824 bytes) stay unchanged. A typed rejection is recorded honestly, not solved by raising limits.

## Risks / Trade-offs

- Full-HD native rendering adds bounded CI work → only one tiny static card, one short range, finite sample counts, existing timeout/duration protections.
- Host/build/cache variance → exact identities, raw samples and environment-scoped targets, no universal threshold.
- Small static card cannot prove dense animated pack readiness → future targets explicitly unmeasured; #187 decides only after later fixtures exist.
- Managed artwork/text has font/media dependencies → original generated inputs and hash-pinned font, no network/private media.

## Compatibility, failure and rollback

No production behavior, persisted schema, contract, revision, lock, migration or failure semantics change. Author through existing atomic APIs and compare authoritative state before/after rendering. Temp roots confine generated media/output; cleanup remains test-owned. Missing configured native dependencies and oracle drift fail required evidence. Revert fixture/test/docs only to roll back. No deployment or merge is authorized.
