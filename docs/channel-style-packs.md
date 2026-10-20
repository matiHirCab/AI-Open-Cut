# Reusable channel style packs (CP1)

Channel packs save existing native components, their typography and typed slot defaults for explicit reuse in other projects. “MontiPKM” is an example workflow name; supply your own original scenes, palette, captions and authorized artwork.

The experimental interface is the source CLI in `apps/agent-bridge/src/channel-packs-cli.ts` and six additive native headless operations. It does not add installed desktop controls or MCP tools. Use existing manual native composition when this interface is unavailable.

## Capture and reuse

Create ordinary native components with the existing editor, including text/rich-text slots for editable caption content and native color slots for palette defaults. Open/migrate the source separately first: capture requires the current schema and no pending transaction recovery. It reads the source without modifying its project, history, drafts or managed media.

From `apps/agent-bridge`, with the usual headless/projects-root environment configured:

```sh
bun run src/channel-packs-cli.ts capture < capture.json > captured.json
bun run src/channel-packs-cli.ts list </dev/null
bun run src/channel-packs-cli.ts get < pack-id.json
bun run src/channel-packs-cli.ts export < pack-id.json > original-pack.json
bun run src/channel-packs-cli.ts import 'Independent copy' < original-pack.json
```

`pack-id.json` is `{ "packId": "the-saved-pack-id" }`. Capture input is:

```json
{
  "capture": {
    "projectId": "source-project-id",
    "expectedRevision": 3,
    "componentIds": ["existing-component-id"],
    "name": "MontiPKM configurable original",
    "captionRole": null,
    "narrationProfile": null
  }
}
```

Optionally set `captionRole` to `{ "recipeKey": "existing-component-id", "textSlotId": "caption" }`, pointing at an existing text/rich-text slot. A narration reference is `{ "profileId": "existing-profile-id", "expectedVersion": 1 }`. Capture stores that declaration without starting a provider. Exact managed font faces travel with the portable package; ambient font paths/families and source diagnostics are removed.

Artwork bytes do not travel with the package. Import each declared image into the target through the existing managed-asset workflow. Then bind every saved image key explicitly to its matching target asset id. Matching names or hashes never select a file automatically. Missing, mismatched or damaged content produces an actionable native error before pack publication in the project.

## Explicit application

`apply` reads a JSON object containing `projectId`, current `expectedRevision`, `packId`, exact `expectedVersion`, `images`, `narration` and `application`. Image bindings are `{ "dependencyKey": "saved-image-key", "assetId": "target-managed-asset-id" }`. Use `[]` only for a pack with no images.

Choose narration explicitly: `{ "mode": "saved" }`, `{ "mode": "disabled" }` or `{ "mode": "replacement", "reference": { "profileId": "chosen-id", "expectedVersion": 1 } }`. An unavailable or stale saved profile blocks application unless explicitly replaced/disabled. The result returns resolved defaults; the existing generation workflow still checks provider/voice readiness before inference.

Instantiation uses an existing unlocked overlay track and ordinary native instance fields:

```json
{
  "mode": "instantiate",
  "placements": [{
    "recipeKey": "saved-component-key",
    "trackId": "target-overlay-track-id",
    "startMs": 0,
    "trimStartMs": 0,
    "durationMs": 1000,
    "timeScale": 1,
    "staggerMs": 0,
    "slotValues": {},
    "transform": { "opacity": 1, "positionX": 0, "positionY": 0, "scale": 1 },
    "transform2d": null,
    "hidden": false,
    "zIndex": 0,
    "parent": null
  }]
}
```

Each target gets independent ordinary component identities and project-owned pinned fonts. Existing native canvas, timing, typed slot, lock and resource rules apply. The operation commits one project revision and one undo entry. Customize instances with existing native slot edits; no live link to the library is stored. Pack/profile updates do not change already applied content, reopening, undo/redo or rendering.

To publish a new pack version, `update` accepts `{ "packId": "id", "expectedVersion": 1, "capture": { ... } }` with a new source snapshot. To reapply, read the current project revision, select the desired pack version and use:

```json
{
  "mode": "reapply",
  "components": [{ "recipeKey": "saved-component-key", "componentId": "existing-target-component-id" }]
}
```

Map every recipe once to distinct target components. Reapply changes definition defaults, preserves per-instance overrides and reports before/after definition digests with `changed`. Those digests include ordinary identity-bearing data; they do not claim visual equivalence. Reapplying an identical definition reports unchanged while retaining normal successful native revision/undo semantics. Removing/narrowing slots that retained overrides require fails atomically. Locked content remains protected, including font-only changes.

## Limits and recovery

Portable format1 targets component schema46. Packages are closed data: no executable template, arbitrary resource path, network lookup or audio/global-resource dependency. Text-slot components provide caption styling; legacy generated caption items must first be authored as ordinary pinned text components for capture.

Each portable package is at most 8 MiB, with 16 recipes, 1024 items, 32 markers, 32 image dependencies, 32 font faces and JSON depth 64. Existing stricter native limits remain authoritative. The root library retains the latest version of at most 16 explicitly named packs and is bounded to 64 MiB. Application allows at most 32 placements and 100 generated native edit operations. Oversized fonts require a smaller explicit selection; no font subsetting or substitution occurs.

The CLI bounds UTF-8 input, serialized requests and output to 9 MiB. Import preserves raw package JSON so the core rejects repeated keys, including nested slot keys; ordinary literal names such as `__proto__` remain valid once. The output limit is checked before stdout emission. If an output/write failure follows a completed native effect, inspect current pack/project versions before retrying; the CLI does not claim rollback or automatically repeat effects.

Revision/version conflicts require a fresh explicit choice. Prepublication failures roll back only owned staged resources. A journal-published uncertain result retains normal native recovery ownership and its recovery-pending warning; explicitly reopen/recover before further work. No background application or automatic retry occurs.

On POSIX, SIGINT/SIGTERM interrupts the active command and closes its existing native process owner before normal exit. Interruption does not imply that an already published mutation was rolled back; inspect current versions before retrying.
