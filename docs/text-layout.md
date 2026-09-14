# Content-addressed text layout

Schema 19 pins four static outline faces for every text item, including hidden
tracks and unused component definitions. The project catalog maps lowercase
SHA-256 keys to `fonts/<sha256>.font`, byte length and face index zero. Text
bindings contain regular, bold, italic and boldItalic hashes, the
`opencut-text-v2` profile and persisted fallback warnings. Identical bytes share
one managed file. Keep the project directory, its fonts, history and drafts
together when copying or backing up a project.

At creation or an explicit selector update, core tries an allowed `fontPath`,
then the normalized `fontFamily` in configured root order and sorted filename
order, then the configured default or embedded DejaVu Sans 2.37. Normalization
lowercases and removes spaces, hyphens and underscores. Missing requested paths
or families record a warning. A selected family must provide all four styles;
missing styles fail with `DEPENDENCY_UNAVAILABLE`. Fonts must stay within
configured roots. Traversal, network locations and symlink escapes are rejected.
The embedded family and its license are in `crates/editor-core/resources/fonts`.
Its reviewed byte hashes are in `contracts/text-layout-v2.json`.

Limits are inclusive: 16 MiB per file, 256 distinct files and 256 MiB per
catalog, 4096 directory entries and depth 8 per discovery root. Collections,
variable fonts and fonts without static TrueType or CFF outlines are rejected.
No platform fallback or synthetic styling occurs after a binding is stored.
Content, paint, size and rich-text style updates preserve existing bindings;
explicit selector changes or resets resolve again. Copies, undo/redo and draft
preview retain the bound bytes. Draft version 2 persists the catalog and the
bindings selected by each operation; preview does not mutate project history.
Replacing a draft's operations retains font bindings across reordering and
non-font edits. Exact operations match first, followed by operation targets and
font selectors. Indistinguishable matches with different retained bindings return
`INVALID_ARGUMENT` without replacing the draft; new operations and changed
selectors resolve under the current configuration.
Matching considers alternatives in both directions: one old action cannot
arbitrarily donate its pinned font to one of several edited replacements.
Retained versus newly unresolved outcomes are ambiguous even if the current
default happens to match. Equivalent complete retention outcomes are paired in
original order. Component draft replacements retain fonts per scoped local text
ID, including when other children change selectors, are added or removed, or
move between tracks. New children never inherit a sibling's binding by selector.
Draft version 2 stores one binding per selector within each operation. If a
replacement requires different bindings for that selector, it fails with
`INVALID_ARGUMENT` before publishing fonts or replacing the draft. Identical
bindings can share the persisted entry.
When replacing a matched component action, changed selectors and reintroduced
local IDs explicitly resolve with current configuration, even if applying the
action inherited a base-project binding. Resolution intent is distinct from
retention or inheritance during matching. If the complete resolved binding
differs from that inherited binding, draft version 2 cannot replay the result
and the replacement fails atomically. Equal bindings succeed; artificial
clearing during preparation never adds a persisted selector entry. All four
faces and binding metadata participate in this comparison.
Matching uses the actual prepared prefix, including fonts supplied by earlier
draft actions. It maximizes structural matches before total compatible matches
and checks every globally optimal alternative against each operation's inherited
bindings. Choosing an equivalent canonical pair cannot hide later ambiguity.
Resolution and validation proceed in memory; a failure in a later operation
publishes none of the fonts staged by its prefix.
Full component replacement also retains bindings for existing text IDs when
selectors are unchanged, including when binding metadata is omitted. Change
selectors to select another font; retained binding metadata cannot be replaced
directly through a component update.

The layout profile pins rustybuzz 0.20.1, unicode-bidi 0.3.18,
unicode-linebreak 0.1.5 and unicode-script 0.5.8. Text stays unnormalized.
Their crate licenses are MIT, MIT OR Apache-2.0, Apache-2.0, and
MIT OR Apache-2.0 respectively, verified against the pinned crate manifests.
Paragraph bidi resolution, script/style segmentation and default OpenType
shaping use language `und`. Paint boundaries preserve shaping context; a
ligature uses the paint at its cluster's first logical character. Missing
characters use glyph zero of the selected face. Wrapping uses complete shaped
clusters and Unicode break opportunities; an oversized cluster stays intact.
Newlines force lines. Effective text is bounded by the existing 4096 UTF-8
bytes/256 runs, plus 16384 glyphs and 4096 lines. Positions are project-canvas
pixels, x right and y down, scaled by fontSize/unitsPerEm. Glyph outline bounds,
padding, alignment, outline and shadow determine the raster bounds. Existing
component timing, ancestry, animation and Transform2D apply to that same raster
for frame, range, draft and export rendering. Caption layout is unchanged.

The approved v2 conformance correction recognizes CR, LF, CRLF, NEL, vertical
tab, form feed, U+2028 and U+2029 as mandatory breaks. CRLF counts once, even
across runs; separators produce no glyphs. Explicit empty/trailing lines and
original UTF-8 cluster positions are retained. U+2028 preserves paragraph bidi
context; paragraph separators begin a new paragraph. Existing text containing
these separators can change from the previous incorrect single-line output.
This narrow correction keeps schema 19, draft version 2 and `opencut-text-v2`;
it does not authorize other silent profile changes.

Opening older projects migrates current state, retained undo/redo and drafts
under the project lock. Font content is published before a recovery journal can
reference it; journal replay publishes project, history and draft updates as
one recoverable generation. IDs, revisions, text and media are preserved.
**This migration can change text kerning, ligatures and wrapping once.** Keep a
backup before upgrading if the legacy layout is needed. There is no downgrade.
Subsequent opens use the stored profile and bytes, even when source fonts or
machine configuration change. Unknown profiles and future schemas fail closed.
Missing or changed managed bytes return `ASSET_INTEGRITY_FAILED`; restore the
exact matching file from backup. Replacing it with a similarly named font is
not a valid repair.

Headless/MCP status advertises `content_addressed_text_layout_v2`,
`projectSchemaVersion: 19` and `textLayoutVersion: 2`. Clients can request
`textLayoutVersion: 2` on status; unsupported versions return
`INVALID_ARGUMENT`. The JSON-lines protocol version remains 1.
