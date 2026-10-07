# Group compositing and overlays

Schema 37 adds `screen_flash`, `particle_overlay`, and composition-bounds clipping.
The existing timeline edits, batch aliases, drafts, history, frame/range previews,
and exports carry these fields. No new MCP operation is required.

Groups and component instances activate a transparent local aggregate when they
have a nonempty effect stack or `clip: { "type": "composition_bounds" }`.
Their children finish their own masks, effects, transforms, mattes, opacity, and
shutters before blending into that aggregate. The owner clips pixel centers to
its half-open composition bounds, runs its effects in stored order, then applies
its outward transform and opacity once. A group uses its containing composition;
an instance uses its definition's dimensions. Anchors and particle emission use
that fixed basis even when the children occupy a signed or offcanvas union.
An owner remains one contiguous stacking block; a child's local z cannot escape.

Omitting `clip` in an edit preserves it. Setting it to `null` clears it. Stored
clips are closed, non-null records and are eligible only on groups and instances.
An empty effect stack does not itself activate compositing. Uncontrolled owners
retain their existing flattening and audio behavior. Aggregate masks, mattes,
non-normal blend modes, authored motion blur, and aggregate effect channels remain
unsupported. Existing leaf animation channels retain their behavior.

At a controlled boundary, outward factors stop before the next enclosing
controlled owner and freeze at the requested output sample. Each descendant retains its
own shutter before local blending. A private named leaf matte provider separately
samples its full-world ancestry, gain, visibility, and shutter on the query grid.
That provider excludes ancestor aggregate clipping/effects and destination blend
background. `matteOnly` removes direct source and halo contribution while retaining
private coverage and audio. Groups and instances are not matte providers.

`screen_flash` accepts `id`, `startMs`, `durationMs`, `intensity`, and `color`.
Its half-open envelope decays linearly from the start. In premultiplied linear
light it moves each channel toward the alpha ceiling while preserving alpha;
transparent pixels remain transparent. It does not create an opaque canvas wash.

`particle_overlay` accepts `id`, `count`, `seed`, `radiusPx`,
`speedPxPerSecond`, `lifetimeMs`, and `color`. Its unsigned wrapping hash gives
fixed positions and phases. Particles move vertically on the original emission
domain's torus, use sixteen closed-circle coverage samples per pixel, and blend
in ascending particle order. Integer clock modulo occurs before floating-point
conversion. A zero-area domain or identity parameters produce no visible change
while retaining admission costs. Particles can emit from empty positive aggregates.

The canonical [catalog](../contracts/group-compositing-v1.json) defines all closed
fields, ranges, equations, work charges, and fixtures. Existing occurrence,
surface, work, request, and live-memory limits remain in force. Admission covers
signed domains, source/provider/query/shutter overlap, and continuous animation
before candidate publication. Executors validate the complete destination and
prepare every fallible source and plane before committing pixels.

Migration from authentic schema 36 changes only the schema marker in current
state and retained history. Older sources carrying these new fields, overlay
kinds, or nonempty group/instance effects are rejected before adoption, including
unused definitions and source-matched drafts. Existing lock, journal, integrity,
and resource-adoption guarantees remain applicable.

Status exposes `group_compositing_models_v1` for the model and
`group_compositing_v1` only when rendering dependencies are ready. Protocol version
1 and all 78 MCP tools are preserved. Structural parity retains the verified
schema-36 predecessor and every older pinned predecessor.

Configured native CI runs `group_compositing_native`, rebuilds the default
headless executable, then runs `group-compositing-native.test.ts` after the
parameterized-effects witnesses and before the unchanged raster-cache checks.
Independent complete plates exercise positive Gaussian/flash/particle ordering,
signed private mattes, nested instances, owner opacity, and persisted workflows.
Lossless byte, encoded SSIM, PCM RMS, and timing tolerances remain unchanged.
