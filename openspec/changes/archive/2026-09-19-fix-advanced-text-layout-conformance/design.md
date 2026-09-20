## Context

The issue-35 archive establishes the existing advanced layout contract. Review reproduced a 20-pixel box displacement from a -20-pixel shadow, accepted negative tracking in retained drafts, and MMMMM at size 30/tracking 0.1 wrapping or shrinking when given its own reported dimensions. These corrections remain owned by editor-core under ADR 0003. The original archive is immutable for this work.

## Goals / Non-Goals

**Goals:** Restore logical placement, canonical validation and exact-width consistency; independently test inclusive shared work accounting and all render intents.

**Non-Goals:** Public fields, capability identifiers, schema versions, encoding settings, new dependencies, altered numerical limits, epsilon comparisons, runtime configuration or stale-draft replay. The layout-absent path retains its exact arithmetic and geometry.

## Decisions

### Separate logical geometry from raster storage (G1, G2, R1)

Keep fractional logical padding-box dimensions and a signed raster-to-local translation in private evaluated text geometry. Reuse ShapedLayout.background: its width/height retain the logical dimensions and its raster-space x/y encode the negative raster-to-local offset, avoiding duplicate geometry fields. Raster sizing remains integer and includes glyph overhang and paint margins. If the raster pixel origin maps to local offset `o`, compose `M_logical * Translate(o)` before sampling; normalized anchors use fractional logical dimensions. Apply the same relation to legacy transform sampling, Transform2D, ancestor matrices, animation extrema and component/repeater occurrences. Bounds calculations still include the full raster, preserving overflow. A newline-only unbounded box may have zero logical width; permit nonnegative logical dimensions while retaining the existing positive raster-dimension validation. Ensure the offset is applied exactly once when prepared text is copied into the evaluated scene.

Alternative: compensating only the background or adding margins to position would leave anchors, animated sampling and transformed occurrences inconsistent. Using integer raster dimensions as logical dimensions loses fractional bounds. Preserve a separate unchanged legacy branch when layout is absent.

### Validate typed draft style payloads centrally (D1-D3)

Extend the canonical retained-draft validator with typed EditOperation traversal: AddText styles, present UpdateItem styles, and TimelineItem text styles in ComponentCreate/ComponentUpdate track payloads. Reuse validate_text_style. Retain current rich-document and paint traversal and validation rather than mistaking span styles or unrelated style objects for TextStyle. Validate during locked loading before any font publication or persistence transaction; staged in-memory work is not authoritative publication. Do not materialize operations against possibly stale base revisions.

Alternative: replaying drafts changes stale-draft compatibility; transport validation duplicates core ownership and misses persisted inputs. JSON-key guessing cannot reliably distinguish text styles. Invalid styles return existing INVALID_ARGUMENT without changing files, including when migration was otherwise necessary.

### One advanced-only width accumulator (W1, W2)

Represent accepted width and cluster count privately. Compute append as tracking before a subsequent cluster advance, then use that single candidate value for comparison and acceptance. Record width with each word-break opportunity and restore it on backtracking. Diagnostics and fit predicates use logical-order measured widths; the visual bidi pass uses its own placement cursor without replacing canonical line widths. Preserve full descending integer search and fixed pixel quantities. No epsilon or legacy arithmetic change.

Alternative: rounding diagnostic widths or loosening fit predicates masks inconsistent arithmetic and violates exact fitting. Measuring only in visual order can vary the sum with bidi reordering.

### Private checked budget with test injection (B1-B3, R2)

Encapsulate cumulative candidate-glyph accounting in a private budget object with checked addition and inclusive comparison to 16,777,216. Share one object across expanded-scene preflight. Provide a test-only constructor or internal test entry point with a smaller limit, routed through the actual preflight implementation. Production callers cannot override the ceiling. Direct unit tests establish the actual ceiling and overflow boundaries; small-budget tests exercise multiple occurrences and output ordering cheaply. Newline-only candidates continue charging their actual shaped glyph count, including zero.

Alternative: production-size expanded tests are unnecessarily expensive; seeding an arbitrary counter alone does not establish scene sharing or output ordering. A new line-work budget is outside the selected scope.

## Verification design

Write failing regressions before each fix. Geometry expectations come from authored logical coordinates and independently composed transforms, not the helper being tested. Include all supported anchors, fractional sizes, strokes, overhang, rotation, scaling, animated samples and expanded occurrences. Pixel comparisons supplement exact geometry assertions.

Migration tests snapshot every authoritative project/history/draft/font file before malformed retained styles are loaded, compare bytes and inventory after failure, and cover schema 20 and 21, valid/stale drafts and nested component payloads. Width tests combine the public dimension round trip with independent pinned-font metrics and exact/narrower boundary oracles for word, cluster, bidi and mixed-face cases. Budget tests assert actual inclusive equality and checked overflow, then observe preflight error precedence and unchanged destinations/artifact inventory through the real expanded-scene path.

Run Rust 1.97.0 formatting, strict workspace Clippy and workspace tests; native rendering/font suites with the verified temporary FFmpeg 7.1.1; bridge typecheck/lint/unit/contracts/MCP integration/packaged smoke; Python suites and strict OpenSpec validation. Builds and executable-dependent suites run sequentially because Windows shares the executables. Record commands, exits, complete log paths and limitations. All original issue-35 requirements and fourteen scenarios remain acceptance obligations alongside the corrective scenarios.

Run the unchanged protected Moon gate before archival and inspect its full result: only this active change may explain expected rejection. After all implementation checks pass, use openspec-verify-change, resolve mismatches, synchronize and archive with the corresponding skills, then require protected Moon and strict all-spec gates to pass. Failed or unavailable required checks block completion.

## Risks / Trade-offs

- Offset sign or duplicate composition can move transformed text: independent matrix and decoded-image regressions cover each path.
- Logical-order widths can differ by floating-point association from visual cursor endpoints: diagnostics and fitting use the defined logical measure, while glyph placement retains bidi behavior.
- Typed traversal can omit an operation variant: enumerate relevant variants and add creation/update fixtures for standalone and nested text.
- Font preparation or migration can publish too early: compare complete authoritative inventories and bytes on failure.
- Test-only budgets can diverge from production: inject only the numeric limit into the same private implementation; test production limit directly.

## Migration Plan

No schema migration or contract version bump is added. Existing schema-20 migration and schema-21 reopen gain validation before publication. Invalid retained data remains unchanged for explicit repair; valid legacy and stale drafts remain loadable. Roll back corrective code if necessary without downgrading schema 21 or rewriting user data. Preserve the issue-35 archive and archive this correction separately only after verification.

## Open Questions

No design decision is outstanding. Explicit artifact approval was received on 2026-09-19.
