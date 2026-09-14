# Font Resolution Specification

## Purpose

Define deterministic content-addressed font selection, durable bindings and versioned text shaping owned by editor-core.

## Requirements

### Requirement: Durable bounded font bindings
Core MUST persist a closed font catalog and text bindings containing SHA-256 of exact font bytes, face index and layout profile `opencut-text-v2`. Each text item MUST bind regular, bold, italic and bold-italic faces for its selected family so later rich-text slot substitutions never trigger ambient lookup. Identical bytes MUST deduplicate. Root, nested, hidden and unused component text and retained draft text MUST receive the same rules. Font selectors MUST remain compatible: an allowed explicit path takes precedence, then normalized family lookup in configured root order with stable sorted candidates, then the configured or packaged default; unavailable requested selectors MUST produce a recorded fallback warning and pin the selected result. Missing required styled faces MUST return DEPENDENCY_UNAVAILABLE rather than synthesize or drop styling. New bindings MUST use static outline fonts with face index zero; collections, variable, bitmap-only and color-only fonts MUST fail with INVALID_ARGUMENT. Core MUST enforce inclusive bounds of 16 MiB per face file, 256 distinct face files and 256 MiB total font bytes per snapshot, 4096 candidates and directory depth 8 per configured root. Exceeding a bound or malformed font/binding MUST return INVALID_ARGUMENT before commit.

#### Scenario: Pin exact selected bytes
- **WHEN** plain or styled text is created with a permitted path, family or default selection
- **THEN** core stores validated managed faces and stable hashes with deterministic selection, deduplication and any fallback warning

#### Scenario: Reject unsafe and excessive resources
- **WHEN** a selector contains traversal, an unapproved absolute path, a symlink escape or a network resource, or a font violates format or inclusive complexity bounds
- **THEN** core rejects it with the established PATH_TRAVERSAL/PATH_NOT_ALLOWED or INVALID_ARGUMENT classification and no authoritative mutation

#### Scenario: Cover inclusive font limits and missing styles
- **WHEN** otherwise valid inputs meet each limit exactly, exceed it by one, or cannot supply a required styled face
- **THEN** exact limits succeed, excess fails with INVALID_ARGUMENT and missing styled dependencies fail with DEPENDENCY_UNAVAILABLE without silent substitution

### Requirement: Reversible stable font editing
Existing simple text/document add and update operations MUST resolve changed font selectors within the same atomic revisioned mutation. Updates omitting selectors, copies, splits, moves, trims, undo/redo and reopen MUST retain original hashes. Explicit selector changes or resets MUST resolve a new binding; edits to text, size, color or run styles MUST use existing bound faces. Standalone, timeline_batch_edit aliases and durable drafts MUST agree, and draft font bindings MUST be durable before a draft is returned. Revision conflicts MUST precede font side effects. Missing item references MUST retain ITEM_NOT_FOUND. Batch failure MUST leave current state, history and draft state unchanged and publish no owned font records. Draft preview MUST use its retained bindings without changing authoritative revision/history.

#### Scenario: Preserve bindings after source changes
- **WHEN** the original font files are removed/replaced and font roots or default settings change before reopen, copy, undo/redo or draft preview
- **THEN** all existing bound text uses exactly the retained font hashes and layout profile without resolving ambient fonts

#### Scenario: Resolve aliases and roll back
- **WHEN** a batch adds text under an alias and updates its selector followed by success or an invalid later edit
- **THEN** success commits exactly one revision/undo step with the final bindings and failure publishes none of the candidate state

#### Scenario: Reject stale and missing targets
- **WHEN** standalone, batch or draft commit supplies a stale revision or references a missing item
- **THEN** the existing stable typed failure preserves authoritative files, revision, history and managed ownership

### Requirement: Canonical versioned shaping
Core MUST shape effective text using only pinned faces and a pinned shaping/Unicode profile, without host font or locale lookup. Ordered text MUST remain unnormalized. Paragraph bidirectional resolution, script segmentation, default OpenType substitutions/positioning and cluster-safe line breaking MUST precede final glyph placement; absent language MUST use the profile's fixed default. Paint-only run changes MUST NOT break shaping context. Font/style changes MUST segment shaping; run boundaries MUST NOT introduce a line break. Newlines MUST force breaks; wrapping MUST use shaped cluster advances and MUST NOT split a shaping cluster, including a cluster wider than the line. Missing glyphs MUST use the selected face's glyph zero without searching other fonts. Positions MUST be finite project-canvas pixels with positive x rightward and y downward, scaled from font units using fontSize. Per effective text MUST be limited to 16384 shaped glyphs and 4096 lines, in addition to existing document and expanded-scene limits. Unsupported profiles, non-finite results and excess work MUST fail with INVALID_ARGUMENT before artifact work. Reopening or upgrading a binary MUST NOT silently reinterpret a stored profile, except for the explicitly approved opencut-text-v2 mandatory-separator conformance correction: CR, LF, CRLF, NEL, vertical tab, form feed, U+2028 and U+2029 MUST force lines without separator glyphs; CRLF MUST count once, byte clusters and actual Unicode paragraph bidi context MUST be preserved, and synthetic end-of-text MUST NOT add a line.

#### Scenario: Shape multilingual styled text deterministically
- **WHEN** fixtures contain kerning pairs, ligatures, combining marks, mixed-direction text, newlines, missing glyphs and adjacent runs with equal or differing shaping styles
- **THEN** exact glyph IDs, clusters, advances, offsets, line breaks and bounds match reviewed independent expectations and repeat unchanged after reopen

#### Scenario: Wrap at cluster boundaries
- **WHEN** wrap width crosses a shaped cluster boundary or a single cluster is wider than the width
- **THEN** complete clusters move to the next line as needed and an oversized cluster remains intact on its line without an infinite loop

#### Scenario: Bound shaping before output work
- **WHEN** effective text reaches or exceeds each glyph/line limit, including repeated component expansion, or names an unknown layout profile
- **THEN** inclusive limits pass and excess/unknown profiles fail with INVALID_ARGUMENT before destination inspection, artifact allocation or render execution
#### Scenario: Mandatory separator conformance
- **WHEN** effective text contains any supported mandatory separator, consecutive or trailing separators, CRLF split across runs, or mixed-direction paragraphs with a line separator
- **THEN** mandatory lines, original byte clusters and bidi context are correct regardless of wrap width, explicit empty lines remain, inclusive work limits hold and frame/range/draft/export agree

### Requirement: Stable font intent during draft replacement
Draft replacement MUST preserve already-pinned bindings across operation insertion, removal, reordering and non-font edits. Core MUST match structurally identical operations first, then operation kind, explicit target identity and font selector presence/value, ignoring content, typography, paint, timing and transforms. Component/local text identities MUST be scoped. Each prior operation MUST match at most once. Newly unresolved identities alone MUST contribute bindings; retained items sharing selectors MUST NOT overwrite them. New operations or changed selectors MUST resolve under current configuration. Matches with differing possible retained bindings MUST fail with INVALID_ARGUMENT before font publication or draft replacement; equivalent bindings MUST be deterministic. Existing revision, integrity and format guarantees remain unchanged.

#### Scenario: Reset without selector collision
- **WHEN** a draft resets the first of root or component texts sharing selectors after the default changes
- **THEN** only the reset text uses the new binding through preview, reopen and commit

#### Scenario: Preserve matched operation fonts
- **WHEN** a draft is updated with reordered, inserted or removed operations or changed text, size, color or rich-text styling after source removal or default changes
- **THEN** matched unchanged selectors retain their exact bindings, while new actions and changed selectors resolve normally

#### Scenario: Reject ambiguous replacement atomically
- **WHEN** structural or font-intent matching permits differing retained bindings
- **THEN** INVALID_ARGUMENT leaves draft, project, history and owned fonts unchanged; equivalent binding candidates are accepted deterministically
