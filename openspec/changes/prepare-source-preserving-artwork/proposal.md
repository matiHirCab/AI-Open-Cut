## Why

Issue #220 needs a reversible way to review simple artwork cleanup without overwriting approved managed originals. Existing masks and vector stickers do not produce source-linked raster derivatives.

## What Changes

- Add a bounded PNG-only artwork preparation workflow: retain existing alpha, optionally remove a caller-selected border-connected background within explicit RGB tolerance, then add an alpha-derived outline.
- Produce review PNGs showing the alpha and the result on light/dark backgrounds; require an explicit reviewed manifest for commit, or allow discard without project mutation.
- Persist derivative source identity/hash and exact parameters; protect originals through the canonical asset reference, integrity, deletion and history/GC owner.
- Expose thin headless operations and an opt-in bridge CLI, with no additional MCP tools, provider calls, model downloads or workflows.
- Migrate schema 46 and older supported projects/history deterministically to schema 47 without inventing derivations. Existing requests/aliases/errors retain meaning; old builds reject schema 47. Existing speech origin stays unchanged.

## Capabilities

### New Capabilities
- `artwork-preparation`: bounded source-preserving PNG cleanup, review and explicit commit/discard.

### Modified Capabilities
- `media-assets`: derivative source ownership in current state, retained history and drafts.
- `project-persistence`: deterministic current/history migration for optional artwork derivation metadata.

## Impact

Core model/assets/store/migrations and tests; additive headless requests; bridge CLI and tests; canonical artwork and headless/schema catalogs and additive MCP project-state asset output schema with its exact inverse proof, ownership inventory and CODEOWNERS; operator documentation. Shared high-risk files are model.rs, migrations.rs, assets.rs, store.rs, headless main.rs, headless-contract.ts and contract catalogs. This branch must reconcile another feature's schema allocation before landing; no unrelated branch is modified.

## Non-goals

No complex segmentation, perfect-cutout claim, JPEG/video/SVG cleanup, external inference, automatic timeline replacement, new renderer, publishing, paid API, remote upload, copyrighted bundle, batch production or changes to CI workflows. PNG-only and review files/CLI are explicit supported first-slice limits, not GUI readiness claims. Preserve all historical raw captures, digests, required tests, aliases and contract inverse proofs.
