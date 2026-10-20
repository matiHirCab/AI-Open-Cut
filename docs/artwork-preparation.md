# Source-preserving artwork preparation

Issue #220 adds an explicit local file-review workflow for managed PNG artwork. It creates a separate image asset; the original image and timeline placement remain unchanged. No provider, downloaded model or remote service is used. The file-review CLI initially requires Linux and `/proc/self/fd` directory handles; other platforms return `DEPENDENCY_UNAVAILABLE` before invoking core or changing files. Core PNG preparation/commit remain platform-neutral. Existing manual managed artwork/mask authoring is the fallback where this backend is unavailable. This workflow uses the bridge source CLI; it does not add an MCP tool or claim installed GUI availability.

Use a static PNG already imported as an Image asset with probed width/height matching its decoded PNG. Very old assets lacking those facts remain usable by existing editing/rendering operations; this new workflow refuses them with `ASSET_INTEGRITY_FAILED`, which in that case means missing required facts, not proven damaged bytes. Explicitly reimport the authorized original via the existing import workflow before preparing it. Ordinary first-open project/history migration remains unchanged; preview does not introduce artwork edits and preserves already-normalized current project bytes. Inputs are limited to 4 MiB and 512×512 pixels; animated PNG and other formats are rejected. PNG palettes, transparency and grayscale are expanded to RGBA8; 16-bit channels use their high byte. Gamma and ICC metadata are not applied. A caller must configure `OPENCUT_EXPORTS_DIR` explicitly and create that real directory. Review packages must be new directories inside it, with an existing real parent. Links and overwrite are refused.

From `apps/agent-bridge`, supply one JSON request on stdin:

```sh
bun run src/artwork-cli.ts < artwork-preview-request.json
```

Example preview request (substitute actual IDs and current revision):

```json
{
  "operation": "preview",
  "projectId": "00000000-0000-4000-8000-000000000220",
  "expectedRevision": 2,
  "sourceAssetId": "00000000-0000-4000-8000-000000000221",
  "parameters": {
    "cleanup": {"color": [255, 255, 255], "tolerance": 0},
    "outline": {"radius": 2, "color": [20, 30, 40, 255]}
  },
  "outputDirectory": "/absolute/configured/exports/review-artwork-1"
}
```

Omit `cleanup` to retain existing alpha. Cleanup removes only pixels connected to the image boundary through four-neighbor pixels whose RGB channels differ from the selected color by at most the inclusive tolerance (0–255); fully transparent pixels can connect regions. This deliberately simple rule can remove a border-connected part of the subject. It is not automatic subject identification. Inspect before applying.

The outline uses a square/Chebyshev dilation of cleaned alpha, radius 0–16, composited behind the original with the selected RGBA color. Radius zero disables it entirely. The image is padded by the radius on all four sides to avoid clipping the outline. `contentOffset` records the original image's top-left within that padding. No timeline size or position is silently adjusted.

A package contains exactly:

- `result.png`: the transparent derived image.
- `mask.png`: cleaned alpha as opaque grayscale, before outline.
- `light.png` and `dark.png`: result flattened over white and RGB32.
- `manifest.json`: captured project/revision, source ID/hash, parameters, dimensions, padding and hashes of all four PNGs.

Open all four PNGs in a local image viewer. For example, on a desktop Linux session use `xdg-open /absolute/configured/exports/review-artwork-1/light.png` and repeat for `dark.png`, `mask.png` and `result.png`; use a local viewer available in your Linux session. A successful file write alone does not establish visual approval. Inspect white subject regions, fine edges, holes and transparent halos. Each invocation emits at most 8 MiB of PNG content plus a manifest up to 32 KiB. Review directories are explicit user-owned output: discard them when no longer needed; there is no hidden retention service or automatic deletion of unrelated files.

After review, explicitly approve the exact package:

```json
{"operation":"apply","directory":"/absolute/configured/exports/review-artwork-1","approve":true}
```

Run that JSON through the same CLI. Apply verifies every reviewed PNG hash, then core recomputes the requested derivative and compares source/result identity under the existing project lock. It creates one managed Image asset and one normal history revision. Use the returned asset ID in existing typed timeline edits only when you want to replace or insert artwork. Undo/redo and reopen retain the derivative and its source provenance.

To reject the result instead:

```json
{"operation":"discard","directory":"/absolute/configured/exports/review-artwork-1"}
```

Discard never calls a project mutation. It verifies the complete package and removes only the verified selected package files. Missing, changed, linked or foreign files cause refusal and are preserved for manual inspection. Apply leaves the reviewed package intact so it remains available for audit/discard.

A stale revision fails with `REVISION_CONFLICT`; prepare and review a fresh package explicitly. Altered packages, invalid controls, source hash differences and unsupported input fail without a partial asset. Never edit manifest fields to force a retry. A repeated apply at its captured revision conflicts. If the process or connection fails during apply, inspect the current project and derivative provenance before retrying: metadata is not a resumable process capability and no exactly-once execution is promised. Cancellation before the durable commit point leaves project state intact; a commit already published remains authoritative.

Schema47 adds optional `artworkDerivation` to assets, separate from speech origin. Current state and retained supported history migrate atomically through the existing core owner, with no fabricated derivations on old assets. Sources needed by current/draft derivatives block logical deletion; history retains owned bytes until its normal eviction. Unknown future schemas are rejected. Back up the entire managed project before runtime upgrades; older builds cannot open schema47 and there is no schema-number-edit downgrade path.

Concurrent #211 integration touches the headless inventory and contract catalogs. Its package v1 pins `componentSchemaVersion: 46`; that package format marker must remain independent of project schema47. Existing v1 packages need an explicit supported conversion at import, with no current-version substitution or old-package rewrite. Packaging a new derived artwork asset must preserve its transitive source/provenance; a v1 path that cannot represent those facts must reject that new case explicitly until a reviewed extension exists. This branch does not contain or claim tested interoperability with #211. Preserve both six #211 and two #220 headless operations at integration. This feature has no functional dependency on #211 and does not change workflows.
