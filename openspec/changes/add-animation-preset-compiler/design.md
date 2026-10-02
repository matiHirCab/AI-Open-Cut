## Context

This issue-45 design is approved by the explicit user response recorded in `approval.md`; implementation and verification are in progress. Research base: main `90f7884f1b58deb1a4cd5992083520b516b56fe3`; published draft PR #133 head `618357054cd6d2d40275ec6f61163996852227a3`. The latter contains main and the archived exact-midpoint correction. Work is isolated on `codex/issue-45-proposal-20261002` in `/workspace/issue45-proposal`; PR #133 and the original checkout remain untouched. Dependencies #38, #39 and #11 were verified closed on 2026-10-02; #45 and the separate initial-pack issue #46 are open.

Current code has schema 28, protocol 1, `AnimationChannel` and deterministic `hold`, `linear`, `cubic_bezier`, `spring` sampling. `set_animation_channels` replaces the complete collection; core rejects collisions with legacy keyframes. `timeline` owns alias resolution and edit application; `validation` owns channel/curve rules; store owns project locking, revisions and atomic publication. Component-definition tracks and retained history contain the same item models. ADR 0004 already requires primitive-only evaluation and descriptive preset provenance.

Read instructions: root `AGENTS.md`, `docs/spec-driven-development.md`, `.codex/skills/openspec-propose/SKILL.md`, ADRs 0002/0003/0004, and applicable living specifications. `.agents/skills/workers-best-practices` was inspected; Workers are unaffected and that workflow is not applicable.

## Goals / Non-Goals

**Goals:** A useful, bounded proof of canonical versioned compilation; editable persisted output and source information; no catalog dependency after commit; explicit collisions; atomic revision/history behavior; typed agent parity; deterministic schema migration with unchanged legacy output.

**Non-Goals:** The full creative pack in #46, new interpolation/rendering semantics, external presets, effect/mask/audio-event synthesis, implicit static baselines, loops or marker expressions in preset parameters, version upgrades of existing instances, direct component-definition preset authoring, or preset application within draft operation lists. No new runtime dependencies or workflow/security changes are needed.

## Decisions

### 1. One minimal catalog entry proves the compiler

| Preset and version | Parameters | Resolved primitives |
| --- | --- | --- |
| `scalar_tween@1` | `property`, `startMs`, `durationMs`, `from`, `to`, optional `curve` defaulting to `"linear"` | One targetless `AnimationChannel`; two scalar keyframes; no loop |

Allowed `property` names and endpoint bounds:

| Property | Endpoint bounds | Compatible target |
| --- | --- | --- |
| `transform.position_x`, `transform.position_y` | [-1,000,000, 1,000,000] composition pixels | Existing legacy-transform visual kinds, groups and root component instances |
| `transform.scale_x`, `transform.scale_y` | (0, 100] absolute independent factors | Same legacy-transform visual kinds |
| `transform.opacity` | [0, 1] straight opacity | Same legacy-transform visual kinds |
| `audio.gain_db` | [-96, 12] decibels | Media whose managed asset has audio |

Compatibility exactly follows the current channel validator: five visual channels exclude `transform2d`, audio-only media, captions, transitions and repeaters; audio gain excludes non-media and assets without audio. Missing managed assets keep `ASSET_NOT_FOUND`. A group created with default identity `transform2d` must use the existing explicit clearing edit before applying a visual preset. Applying to a root component instance animates that instance; it does not edit its definition or other occurrences.

Timing is absolute item-local integer milliseconds. `startMs >= 0`, `durationMs >= 1`; each and their checked sum must be <= 9,007,199,254,740,991 for lossless cross-language representation. Both generated times must be in `[0,item.durationMs)`: an item of duration 1,000 accepts start 0/duration 999 and rejects start 0/duration 1,000. The first value holds before the start, the last afterward. No implicit percentage timing, rounding or endpoint clamping.

The starting keyframe receives the effective curve; the terminal keyframe always receives `"hold"`. Curves are the existing `AnimationCurve` records, with existing finite bounds and terminal-segment rules. Endpoints copy the submitted binary64 values without arithmetic; only checked integer addition derives the end time. Evaluation retains the current sampling/clamping rules and inherited fractional clocks. All defaults are materialized into persisted parameters.

Alternative considered: ship all five named #46 presets or invent styled starter presets. That expands a separately tracked pack and introduces creative choices unnecessary for the compiler. A single scalar seed exercises channel compilation, audio/visual compatibility, all existing curve types, edits and provenance. The owner must approve this seed choice explicitly. No promise is made that the initial seed is the creative pack.

### 2. Compile and publish through the existing core edit path

Request inside the existing headless `edit` envelope, or as a batch member:

```json
{
  "operation": "apply_animation_preset",
  "itemId": "rectangle-id",
  "presetId": "scalar_tween",
  "presetVersion": 1,
  "parameters": {
    "property": "transform.opacity",
    "startMs": 0,
    "durationMs": 500,
    "from": 0,
    "to": 1
  },
  "collisionPolicy": "reject"
}
```

`presetId` and `presetVersion` are mandatory: there is no `latest` or unversioned default. `collisionPolicy` defaults to `reject`; a missing curve defaults to linear. Unknown IDs/versions return `INVALID_ARGUMENT`. Structurally valid unknown identifiers reach core, so the bridge does not implement a second catalog dispatcher.

Under the existing project lock, load/recover and check the expected revision before compiling. Resolve the target item and aliases through existing timeline rules. Compile with no I/O, external resources or renderer access. Validate the resulting merged channel collection and the final candidate through existing channel, retained-state, inherited-scene and extended-work certification. Store then publishes project/history in one existing transaction. A success increments revision once, records one undo step, clears redo, and returns the existing `WriteResult` with the item ID; `get_state` exposes primitives/provenance. A batch records one revision/history entry for all ordered operations.

The pure compiler lives in a nested module of `timeline`, using model DTOs and existing `validation` helpers. Model and provenance shape checks stay in `validation`; persistence/migration in their current owners. This fits ADR 0003's existing edges. If implementation discovers that another edge is necessary, stop and amend the design/ADR/architecture test rather than create an unreviewed owner.

Alternative considered: MCP expansion or a preset-specific renderer. Both would split canonical behavior and make reopen/rendering depend on an outer layer. The existing evaluated channels already implement every seed output.

### 3. Collision and ordering rules are deliberately small

- Collision identity is the existing `(property, target)` pair. The seed always has no target.
- `reject` fails with `INVALID_ARGUMENT` if that typed channel already exists, even if keyframe intervals are disjoint or output would be identical.
- `replace` replaces that entire existing channel, including its loop, in its existing collection position; a new identity appends one channel. Unrelated channel order, values, targets, loops, static properties, effects and legacy keyframes remain unchanged.
- Overlapping legacy position, scale, opacity or volume keyframes fail with `INVALID_ARGUMENT` under either policy. Existing coupled-axis legacy collision rules remain authoritative. Conversion/removal requires a separate explicit low-level edit, optionally earlier in the same atomic batch.
- Preset application is not an ID creator. It can consume earlier creation aliases but cannot declare `resultAlias`; existing alias misuse remains `VALIDATION_FAILED`.

Alternative considered: append/splice keyframes, remove legacy keys or merge based on time overlap. Those require extra observable interpolation and destructive-edit rules. Exact-channel replacement is reviewable and matches current low-level ownership.

### 4. Persist a bounded sidecar, with channels as authoritative output

Schema-29 `VisualProperties` adds optional `animationPresetProvenance`, a map keyed by targetless channel property. Omission means an empty map; `null` is invalid; empty maps are omitted on serialization. It is not a field on channel input and does not change existing low-level channel payloads.

Illustrative state after the request above:

```json
{
  "animationChannels": [{
    "property": "transform.opacity",
    "keyframes": [
      {"timeMs": 0, "value": {"type": "scalar", "value": 0}, "curve": "linear"},
      {"timeMs": 500, "value": {"type": "scalar", "value": 1}, "curve": "hold"}
    ]
  }],
  "animationPresetProvenance": {
    "transform.opacity": {
      "presetId": "scalar_tween",
      "presetVersion": 1,
      "compilerVersion": 1,
      "parameters": {
        "property": "transform.opacity",
        "startMs": 0,
        "durationMs": 500,
        "from": 0,
        "to": 1,
        "curve": "linear"
      }
    }
  }
}
```

The stored `animationChannels` are the resolved primitives. There is no duplicate authoritative primitive snapshot, generated resource or stored expression. Each map entry must match an existing targetless channel identity and its parameter property. Parameters have the strict scalar-tween shape, with finite values and the established primitive bounds. Provenance describes authorship; it is not an integrity signature.

Persisted `presetId` accepts only `[a-z][a-z0-9_]{0,63}`; `presetVersion` and `compilerVersion` are positive u32. Historical identifiers and positive versions need not be present in the live compilation catalog. Reading validates the versioned descriptive shape and primitive/reference safety without looking up or rerunning a compiler or comparing output with today's expansion. Future source versions fitting this descriptive shape remain inert; a new parameter shape requires an explicit persisted-schema extension. Unknown project schemas still fail closed.

| Event | Provenance outcome |
| --- | --- |
| Successful apply/replace | Store the effective source record for the generated identity; preserve unrelated entries |
| `set_animation_channels`, including an identical replacement or empty collection | Clear all entries for that item because this operation replaces the complete authored collection |
| Other primitive removal/retiming/replacement | Remove entries for changed identities; never label modified output as freshly compiled |
| Full component track/document replacement | Ignore submitted source labels; retain only previously known labels for the same scope/item/identity with an unchanged channel; new raw definitions start without labels |
| Exact core duplicate/copy/component packaging | Copy labels with byte-equivalent local channels; remap ordinary references using current rules |
| Move, visibility, parenting or static/base-volume edit without channel changes | Retain labels; they describe local primitives, not a guaranteed final visual result |
| Duration edit or split | Preserve existing operation validity; if current rules reject channel times, reject atomically; clear labels only if an accepted operation changes the primitive; do not add auto-retiming |
| Delete | Remove labels with their owning item |
| Undo/redo/reopen/catalog evolution | Restore/read saved primitives and labels without compilation |

Reconciliation must cover all mutation paths, including component replacement and draft materialization of ordinary channel edits. Imported raw documents are structurally validated descriptive data; source labels never grant authority or execute code. No new user-supplied provenance setter is introduced.

Alternative considered: persist just preset references, per-channel embedded metadata, or an append-only application log. References would drift; embedded metadata would change the existing setter payload; unbounded logs would require another history model. One bounded sidecar per active generated identity is enough for the issue's provenance promise. Low-level replacement intentionally gives up labels; undo can recover them.

### 5. Named finite and complexity limits

| Limit | Value / behavior |
| --- | --- |
| Catalog entries initially | 1 exact ID/version pair |
| Generated channels/keyframes per apply | 1 channel / 2 keyframes |
| Effective provenance parameters | Closed six-field record; curve is the existing bounded tagged record or string |
| ID length | 64 ASCII bytes |
| Provenance entries per item | At most 64, and at most one per existing channel identity; the seed can occupy only its six allowed properties |
| Numeric timing representation | 0..9,007,199,254,740,991; duration positive; checked sum within the same bound and the item's half-open duration |
| Batch | Existing maximum 100 operations; at most 100 seed expansions / 200 generated keyframes before existing candidate limits |
| Existing primitive/scene/retained limits | Keep 64 channels per item, 1,000 keys per channel, history/evaluation/raster limits and 65,536-node extended certification unchanged |

Reapplication replaces the label instead of accumulating records. Reject any finite-value, closed-shape, arithmetic, parameter or candidate budget violation before publication; do not clamp input or silently truncate. Core owns semantic checks; transports enforce structural shape and translate core errors. No executable strings, preset files, paths, network resources, expressions or SVG parameters exist. Provenance is local project data and leaves the machine only through already-requested state responses.

### 6. Draft boundary avoids unversioned replay expansion

Existing draft documents persist edit intents and replay them during preview, rebase and commit; they do not freeze compiler results. For this milestone, `create_draft`/`update_draft` reject a list containing `apply_animation_preset` with core `INVALID_ARGUMENT` before any draft/resource write. Reading/materializing an injected unsupported preset draft also fails without changing files. This adds no restriction to previously accepted operations. Existing draft format/version remains 2 and valid old drafts stay unchanged.

A draft based on a committed preset project can render its saved channels, and can replace/clear them using existing low-level operations, with the provenance lifecycle above. A future approved draft extension can freeze compiled primitives plus provenance and migrate draft formats. Accepting a preset intent and recompiling it every reopen was rejected because retirement or compiler changes could alter an uncommitted draft.

Alternative considered: add frozen compiled draft steps now. That requires an additional persisted draft contract/migration and reconciliation with font steps, beyond issue #45's required standalone/batch surface. This explicit bounded exclusion needs owner approval.

### 7. Contract parity remains fixture-governed

Add `contracts/animation-presets-v1.json` as the canonical seed, limits, effective-parameter, primitive/provenance and invalid-input catalog. Extend `contracts/contract-ownership-v1.json` and `.github/CODEOWNERS` only for the new domain/contract consumer files, never for workflow gates. Synchronize `contracts/headless-protocol-v1.json` (nested edit examples, capability `animation_presets_v1`, project schema 29), `contracts/mcp-surface-v1.json` (tool, structural inputs/outputs and reviewed digest), and motion-graphics vocabulary adoption where required. Preserve existing request identifiers, protocol 1 and `contracts/error-codes-v1.json` semantics.

MCP `timeline_apply_animation_preset` uses the current project/revision envelope and `WriteResult` schema. Batch and exported TypeScript edit unions carry exactly the same typed fields. Project/component response schemas accept the optional sidecar. The shared draft operation union can structurally deserialize the new edit; core enforces the explicitly documented draft limitation, including direct headless/native calls.

Unknown preset/version/parameters, incompatible targets, collisions and complexity overflow use non-retryable `INVALID_ARGUMENT`; missing item uses `ITEM_NOT_FOUND`, missing assets `ASSET_NOT_FOUND`, locked tracks `TRACK_LOCKED`; stale project/draft revisions remain retryable `REVISION_CONFLICT`. Existing `PROJECT_NOT_FOUND`, alias `VALIDATION_FAILED`, parse, recovery and post-commit warning behavior is unchanged. No new error catalog entries are proposed. CODEOWNER: `@matiHirCab`.

Alternative considered: generate catalogs from runtime schemas or duplicate compile/validation code in Zod. Both would weaken independent parity evidence and violate existing ownership.

## Risks / Trade-offs

- [Small seed may not satisfy a creative-pack expectation] -> Explicitly approve one technical entry for #45; keep all five styled entries in #46.
- [Labels can become stale through secondary mutation paths] -> Sidecar reconciliation scenario coverage for raw setters, component replacement, duplicate/split, deletion, drafts, history and reopen; labels have no runtime authority.
- [Old applications cannot open schema 29] -> Forward-only locked migration, backups for rollback, fail-closed future-version tests; no format downgrade.
- [A valid new channel can make a coupled scene unsafe] -> Retain existing complete-candidate certification and history/resource checks rather than validating only the two endpoints.
- [Draft limitation is visible to new clients] -> Document it and test canonical failure with unchanged draft/project/history bytes; old draft operations remain compatible.
- [Published prerequisite must remain in the base] -> The proposal began on PR #133's exact published head. After its merge, main `51cd085253402bd0a2f1798f7a4bba418f83bb5d` was integrated without conflicts; retain the correction and refresh main/PR state before any separately authorized publication.

## Migration Plan

After approval, add schema-29 decoding/validation and migration in core. For every supported older current project and every undo/redo snapshot, traverse root and component tracks, initialize empty provenance, and upgrade through the existing schema ladder. Reject any provenance field in a source schema below 29, including an empty/null field. Do not infer authorship from existing channels, keyframes or other origins.

Perform validation and final retained-candidate preflight before the existing recoverable transaction publishes current state/history under the project lock. Preserve project IDs, revisions/timestamps, channel values/curves/loops, hierarchy, assets/fonts and managed bytes. A corrupted/future-version snapshot aborts the whole migration and retains the previous generation; existing recovery warning/commit-point rules apply. Already-schema-29 valid documents reopen without rewriting. Existing drafts need no new fields and no version migration.

There is no deployment in this task. Rollback of an implemented upgrade uses a backed-up pre-migration project/history generation with the older build; do not drop source fields or downgrade live schema-29 projects. A persisted historical source version missing from the runtime catalog is not a future project schema and must still reopen/render from saved primitives.

## Verification Plan

Automated coverage is required for every delta scenario; tasks include the exact checks. Tests must independently assert the fixed generated keys/provenance and sample values, rather than just compile then compare to the same compiler output. Cover every seed property/curve, bounds and boundaries, unknown versions, collision policies, untouched channels, revision/lock/missing/alias errors, complete batch rollback, metadata lifecycle, current/history/component migration, malformed retained state, idempotent reopen and catalog retirement. Require raw-channel versus preset-generated scene/frame/range/draft/export equality with the existing visual/audio tolerance. Run the existing exact-midpoint regression on this stacked base and do not modify PR #133's correction or goldens.

The initial proposal was prepared before executable files, contracts, migrations or tests were edited. Implementation began after the explicit approval recorded in `approval.md`; check results and remaining gates are recorded in `verification.md`. Archive-only protected policy rejection is expected while this approved change remains active and is not a passing gate.

## Open Questions

The owner approved this complete design as recorded in `approval.md`. Implementation must stay within the one-entry seed, raw-setter label clearing, reject/explicit-replace policy, schema 29 and draft/direct-definition exclusions. Any material expansion requires a new explicit decision. Final implementation acceptance and any external publication remain separate from design approval.
