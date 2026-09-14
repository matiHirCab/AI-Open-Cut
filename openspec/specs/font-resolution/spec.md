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

### Requirement: Complete draft matching ambiguity detection
Core MUST prioritize structural matches before font-intent matches and evaluate one-to-one alternatives in both directions with bounded analysis. Different complete retained bindings or retained versus newly unresolved outcomes MUST fail with INVALID_ARGUMENT before font publication or authoritative mutation. Equivalent retention outcomes MUST be paired deterministically in original order. Existing revision, catalog, integrity and work-limit checks MUST remain effective.

#### Scenario: One old action has several replacements
- **WHEN** one old font action can match either of two non-exact replacements
- **THEN** both replacement orders fail atomically with an ambiguity message and unchanged draft, project, history and owned font bytes

#### Scenario: Exact and equivalent assignments
- **WHEN** operations are inserted, removed, reordered or repeatedly edited
- **THEN** exact matches take priority, differing alternative bindings fail, and identical retention outcomes succeed deterministically

### Requirement: Component child font retention
Paired component actions MUST retain complete bindings per scoped local text ID with unchanged selector presence/value. Pairing MUST use operation kind and explicit component identity, or overlapping local text IDs for anonymous creation, subject to ambiguity detection. Non-font edits, track movement and ordering MUST NOT reset bindings. New IDs and changed selectors MUST resolve with current configuration; removed IDs MUST NOT contribute retention. Retention MUST NOT leak to sibling identities or other components sharing local IDs.

#### Scenario: Partial component font edit
- **WHEN** a component-create or component-update draft changes one child's selector after defaults change or source removal
- **THEN** untouched siblings retain all four font hashes through preview, reopen and commit while the changed child resolves normally

#### Scenario: Local identity changes
- **WHEN** children are inserted, removed, reordered, moved between tracks or edited without selector changes
- **THEN** unchanged identities retain bindings, new identities resolve normally and other components remain independent

### Requirement: Representable draft font steps
Core MUST preserve project schema 19, draft version 2 and opencut-text-v2 without new public fields. Each persisted selector entry MUST represent the complete binding of every newly unresolved identity sharing that selector in its step. Differing bindings MUST fail atomically with descriptive INVALID_ARGUMENT before font publication; identical bindings MUST succeed.

#### Scenario: Conflicting selector outcomes
- **WHEN** a retained component child and a new or changed child share a selector but require different bindings
- **THEN** replacement fails with unchanged draft, project, history and owned font bytes

#### Scenario: Equivalent selector outcomes
- **WHEN** multiple component children sharing a selector resolve or retain the same complete binding
- **THEN** the draft remains readable as version 2 and reproduces those bindings after reopen and commit

#### Scenario: Inherited binding conflict
- **WHEN** inserted actions cause a component child to inherit a different binding that selector-keyed draft replay cannot replace with the retained binding
- **THEN** replacement fails atomically with descriptive INVALID_ARGUMENT and publishes no staged fonts

### Requirement: Explicit component selector resolution
Matched component actions MUST explicitly resolve changed selector presence or values and newly introduced local text IDs using current configuration, even when operation application inherits a base binding. Internal resolution markers MUST participate in ambiguity comparisons and MUST NOT equal inherited or retained outcomes solely because hashes coincide. Core MUST capture and clear marked inherited bindings in memory before resolution. If the complete resolved binding differs from the captured binding and normal v2 replay would retain that binding, core MUST reject atomically with descriptive INVALID_ARGUMENT before publication. Equal complete bindings MUST succeed. Selector steps MUST only describe identities normal operation replay leaves unresolved. Markers MUST NOT alter stored operations or public fields.

#### Scenario: Selector reversion conflicts with base binding
- **WHEN** a matched replacement reverts a family or path selector, including null resets, from draft font B to a base selector whose inherited font A differs from current resolution C
- **THEN** replacement rejects with unchanged draft, project, history and font bytes, including differences confined to styled faces

#### Scenario: Equivalent selector reversion
- **WHEN** current resolution and the inherited complete binding agree after a selector reversion
- **THEN** replacement succeeds identically through preview, reopen, repeated replacement and commit while unchanged siblings retain bindings

#### Scenario: Reintroduced local identity
- **WHEN** a matched replacement reintroduces a local text ID missing from the previous operation but present in the base component
- **THEN** it explicitly resolves and accepts only representable results under the same inherited-binding comparison

#### Scenario: Resolution outcome ambiguity
- **WHEN** alternative component matches would explicitly resolve a local ID or retain or inherit its binding
- **THEN** the differing intent outcomes reject as ambiguous even when current hashes could coincide

### Requirement: Matching against prepared draft prefixes
Core MUST compare matching outcomes against actual complete bindings supplied by preceding replacement operations, including retained and freshly resolved fonts. Bounded weighted assignment MUST maximize structural matches first and total compatible matches second, retaining every globally optimal candidate and unmatched possibility. Canonical original-order assignment MUST NOT narrow later ambiguity checks. Core MUST advance chronological in-memory preparation only after all alternatives agree for a step; any later error MUST leave authoritative and owned font bytes unchanged. No assignment permutation enumeration or public format change is permitted.

#### Scenario: Equivalent inheritance from a preceding action
- **WHEN** an earlier exact or font-intent-matched action supplies a pinned font and a later non-font action is replaced by equivalent actions
- **THEN** both replacement orders succeed through preview, reopen and commit, including changed defaults or removed sources

#### Scenario: Resolution differs from preceding inheritance
- **WHEN** an earlier action supplies a font and alternatives for later replacements explicitly resolve or inherit that font
- **THEN** both orders reject as ambiguous even if current complete bindings coincide, with unchanged draft, project, history and font bytes

#### Scenario: Chronological chains and global alternatives
- **WHEN** prefixes include freshly resolved fonts, multiple component identities, or longer chains of retained actions
- **THEN** comparison uses actual scoped prefix bindings, preserves exact priority and checks all global alternatives even when earlier equivalent pairs were canonicalized

### Requirement: Explicit native font regression configuration
Native font rendering regressions MUST execute when native tools are configured and MUST fail on missing required configuration, partial configuration or unusable configured dependencies. Only wholly unconfigured optional execution SHALL omit native work. Non-native font lifecycle tests MUST continue running independently of native tools. Required native execution MUST preserve preview, reopen, source-removal, draft, range and export assertions without changing production behavior or persisted formats.

#### Scenario: Unconfigured ordinary test job
- **WHEN** all three native configuration paths are absent and required mode is disabled
- **THEN** native font tests omit native work without launching ambient FFmpeg and non-native font tests execute

#### Scenario: Required or partial configuration
- **WHEN** required mode lacks configuration or only some native paths are configured
- **THEN** native font tests fail with a configuration diagnostic instead of skipping

#### Scenario: Unusable configured dependencies
- **WHEN** complete configuration names an unavailable tool or unreadable font
- **THEN** native font tests fail instead of skipping

#### Scenario: Configured native conformance
- **WHEN** complete usable native configuration is supplied in required mode
- **THEN** both native font regressions execute all existing assertions including mandatory separator parity and pinned fonts after source removal

### Requirement: Mandatory CI native font coverage
The required render-parity job MUST execute the font-resolution integration suite with required native configuration. Exact-command policy validation MUST reject removal, substitution or error masking of this suite while preserving all existing required commands and settings.

#### Scenario: Native font coverage cannot be bypassed
- **WHEN** the font-resolution native command is removed, substituted or given an error-masking suffix
- **THEN** policy validation rejects the workflow
