## Context

Issue #33 follows implemented rich-text documents (#32, schema 18) and centralized asset ownership (#82). `render_artifact.rs` currently resolves family/path/default selectors and styled sibling files during rendering. `measure_rich_text` advances one character at a time. These choices allow ambient fonts to change output and omit context-sensitive shaping. The user approved the proposal on 2026-09-12; the implementation and verification evidence are tracked below and in verification.md.

## Goals / Non-Goals

**Goals:** durable exact font ownership, bounded context-sensitive shaping, unchanged post-migration layout after reopen, shared render semantics, atomic current/history/draft migration, compatible editing requests and documented versioned rendering changes.

**Non-Goals:** see proposal.md, especially network/custom import services, new rich-text syntax, variable/color fonts and caption-only layout changes.

## Decisions

1. **Persist resources, not ambient paths.** Schema 19 adds a project font catalog keyed by lowercase SHA-256 with managed relative path, byte length and face index. Text bindings reference four style faces and `opencut-text-v2`; deduplicated immutable bytes live in a dedicated managed font namespace. Preserve existing requested fontFamily/fontPath as intent, never as reopen authority. Select once in stable root/candidate order; pin fallback selection and retain diagnostics. Use a redistributable packaged default family with all four styles and checked-in licenses/hashes. Existing DejaVu fixtures provide initial regular-face regression inputs but do not constitute the complete packaged family. Alternative: a hash without managed bytes cannot survive source removal; render-time caches cannot protect history or drafts.

2. **Separate safe resource ingestion from pure layout.** A new core `fonts` owner validates font metadata and performs pure shaping over supplied bytes; it has no environment, provider, process or filesystem imports. `assets` owns font discovery/ingestion, catalog integrity and retention through the existing persistence port. `store` orchestrates configuration, resolution and staged commits; `migrations` receives deterministic prepared binding data without doing I/O. The application supplies immutable approved font roots/default to core. Extend ADR 0003 and the architecture matrix for `assets -> fonts`, `validation -> fonts`, `evaluated_scene -> fonts` and `store -> fonts` if used, plus `render_artifact -> fonts` for prepared shaping/outline validation and the new owner. Audit exact edges before code and keep them synchronized; transports only translate typed configuration/results. Alternative: move resolver code into headless would duplicate core semantics and bypass draft/migration consumers.

3. **Shape once and rasterize glyph outlines.** Use pinned Rust shaping, bidi and Unicode line-break dependencies behind the versioned profile, together with existing ttf-parser outline extraction and tiny-skia rasterization. Bind verified bytes through a typed resource-preparation input, retain filesystem paths outside EvaluatedScene, and finalize shaped renderer-neutral glyph runs with face keys, clusters, advances/offsets and bounds before backend planning. Shape paragraph bidi/script segments, preserve context across paint-only changes, wrap complete clusters, and apply existing style boxes/ancestry afterward. Emit glyph outlines at canonical positions; FFmpeg composites prepared surfaces without reshaping. Cache by profile, face hashes, effective text and shaping/layout settings; cache hits never skip integrity checks. Exact glyph/cluster fixtures and independent placement assertions accompany image parity. Alternative: ttf-parser advances alone do not implement shaping; FFmpeg text layout can vary with backend build and would duplicate the canonical result.

4. **Retain bounded faces for future slot text.** Pin all four styles when resolving a family, even when base text is plain, so a later rich-text slot override can be evaluated without ambient lookup. A missing requested regular selector retains a warning and default fallback, but a selected family lacking required styles fails before commit. Missing glyphs render glyph zero from the pinned selected face; there is no OS fallback chain. Enforce the exact per-file/catalog/discovery/glyph/line bounds in font-resolution/spec.md as well as existing expanded-scene limits. Alternative: resolve styled faces lazily during preview would mutate or destabilize immutable drafts and reopen behavior. Cost: custom single-face families must supply compatible siblings before activation.

5. **Reuse one ownership and transaction policy.** Extend central reference discovery with typed font-content references instead of changing the media asset kind enum. Current text, definitions, slots, drafts and retained history protect bytes. New drafts persist bindings for their materialized text and keep them through rebase/commit, resolving only an explicit selector change. Stage validated immutable font files after all source snapshots pass validation; incorporate their publication and any draft binding migration in the recoverable envelope. Existing authoritative bytes remain intact until the durable commit point. Font content surviving a failed candidate is never owned and is cleaned by the same bounded managed GC. Alternative: separate font GC/transaction logic risks dangling history and crash-time mixed generations.

6. **Version the visual change explicitly.** Add a new major canonical text-layout contract (v2) and register its single owner/consumers; keep v1 fixtures as historical migration evidence. Schema 19 carries the profile and migration is the explicit transition from legacy layout. Report capability support without reusing a capability identifier with changed meaning; negotiate only supported layout versions and retain existing operation syntax/error codes. Update generated/public project response consumers and parity evidence together. Document that migrated kerning, clusters and wraps may differ once; subsequent reopen uses identical inputs. Future profile changes require another explicit migration rather than a silent dependency upgrade. Alternative: label required persistence fields or changed rendering semantics additive would violate ADR 0002.

## Risks / Trade-offs

- One-time legacy layout shifts -> approval explicitly includes this compatibility impact; fixtures retain old evidence and review the new expected result.
- Font licensing and packaging size -> ship only a reviewed redistributable family with hashes, licenses and packaged smoke coverage; never copy workstation fonts into repository fixtures without permission/license review.
- Complex scripts and styled boundaries -> exact pinned-profile fixtures for Latin kerning/ligatures, combining marks and bidi/scripts; missing-glyph behavior is explicit, not a claim of universal glyph coverage.
- Untrusted malformed fonts and expansion -> size/format limits before parsing, bounded shaping and existing expanded-scene budgets, plus malformed and overflow tests.
- Draft and migration atomicity -> extend existing injected-fault coverage to staged fonts and draft bindings; do not add a second independent journal.
- Existing public renderer callers -> preserve facade entry points where practical and translate missing schema-19 resources into stable failures; add facade tests, never recover via ambient resolution.

## Migration Plan

After explicit approval, update canonical v2 contracts and migration fixtures before consumers. Validate each source snapshot under its own version, run historical steps through schema 18, collect all current/history/draft font intents, resolve/validate them under one immutable configuration, and stage the complete schema-19 envelope. Publish under the existing lock and recovery protocol. Reopen verifies managed hashes without resolving original paths. Rollback before commit retains the original envelope; recovery after commit selects the complete new envelope. There is no lossy automatic downgrade; users needing an older binary must restore a complete pre-upgrade project backup. Run all required validation and conformance checks, use openspec-verify-change, and archive only after approval, implementation and verification are complete.

## Open Questions

None required to review this proposal. Exact dependency release pins and a distributable four-style default font must be verified and recorded during the implementation dependency task before introducing code; any change to specified layout, bounds, fallback or compatibility behavior requires updated artifacts and renewed approval. Approval of this proposal and designated contract CODEOWNER review remain required gates.

## Approval record

User explicitly approved this proposal in the task on 2026-09-12 with: Approve. Implementation is authorized within these artifacts.

## Implemented profile details

The direct pins are rustybuzz 0.20.1, unicode-bidi 0.3.18, unicode-linebreak 0.1.5 and unicode-script 0.5.8. Shaped-run caching is request-local and keyed by document, binding/profile and layout settings; verified byte preparation precedes every cache lookup. Retained font catalogs are included in the resource sidecar even for hidden and unused text. The scene frame rate is applied to glyph images before animation evaluation. Legacy schema-18 golden fixtures remain historical evidence; schema-19 conformance has independent glyph and native render tests.
