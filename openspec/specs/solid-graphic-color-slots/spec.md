# Solid graphic color slots specification

## Purpose

Provide three explicit typed solid-paint color bindings with exact RGB/alpha semantics, independent transactional instances, governed schema adoption and attributable native verification.

## Requirements

### Requirement: Closed solid graphic binding matrix
Core MUST accept exactly three additional color-slot properties: `shape.fill.color` on an existing solid shape fill, `shape.stroke.color` on an existing solid shape stroke paint, and `grid.dot.paint.color` on an existing solid dot-pattern grid paint. The slot kind MUST be color. Missing local targets, including identifiers that exist only in root or foreign compositions, MUST retain non-retryable ITEM_NOT_FOUND and MUST NOT resolve outside the owner. Incompatible kinds/items, absent paints, gradients, non-dot grids, SVG targets, arbitrary property paths and explicit caller-selected scope fields MUST fail with non-retryable INVALID_ARGUMENT. Existing local-identity, duplicate-writer, hidden/unused validation and collection bounds MUST remain unchanged.

#### Scenario: G1 accept exact targets
- **WHEN** definitions bind color defaults or required/optional values to each compatible local solid target
- **THEN** native and TypeScript structural contracts accept the three identifiers and core validates the effective candidate without creating paint or changing the base definition

#### Scenario: G2 reject incompatible targets and kinds
- **WHEN** a graphic slot has a wrong kind, missing/foreign/root target, duplicate writer, gradient/absent paint, SVG item, non-dot grid or arbitrary path
- **THEN** core or structural parsing returns the specified existing failure before project/history/draft/artifact publication, including unused and hidden definitions

### Requirement: Exact RGB replacement with preserved alpha
Graphic bindings MUST reuse the existing closed `{type:"color",value:"#[0-9A-Fa-f]{6}"}` slot value and empty color constraints. Each hexadecimal channel byte MUST become its unassociated sRGB value divided by255. The target solid paint's linear alpha MUST remain exactly unchanged, including0, fractional values and1. Defaults and overrides MUST retain their exact typed persisted strings and precedence; absent optional values MUST preserve base paint. Geometry, stroke width/dashes/offset/caps/joins/miter limit, grid dimensions/pattern metrics, visual properties and unrelated animation MUST remain unchanged. Hex8, objects, coercion and malformed/unknown fields MUST retain existing rejection.

#### Scenario: G3 preserve alpha and other properties
- **WHEN** mixed-case RGB values and alpha0,0.5,1 are resolved for fill, stroke and dots
- **THEN** effective RGB channels are exactly the declared byte fractions while alpha and every unrelated property remain equal to the original target, and invalid values do not mutate stored content

### Requirement: Independent transactional instance resolution
Each component occurrence MUST resolve its own graphic overrides before defaults through existing core component evaluation without modifying shared definitions or other occurrences. Existing standalone definition/instance edits, component slot replacement, earlier batch creation aliases, locks, optimistic revisions and one-step batch history MUST apply. Stale requests MUST return retryable REVISION_CONFLICT, affected locked tracks MUST return non-retryable TRACK_LOCKED, and a later failed batch edit MUST preserve complete prior revision and byte-identical project/history/resource files.

#### Scenario: G4 customize independent occurrences
- **WHEN** two occurrences of one definition have different fill/stroke/dot overrides and one occurrence is updated, duplicated or cleared to defaults
- **THEN** each evaluates its own exact colors while the shared definition, the other occurrence, geometry and unrelated channels remain unchanged

#### Scenario: G5 preserve standalone and batch failure atomicity
- **WHEN** a standalone slot/instance edit or an aliased batch has an invalid target/value, changed locked binding, locked occurrence, stale revision or invalid trailing operation
- **THEN** the existing typed error and retryability are preserved and no partial candidate, revision, history or artifact is published

### Requirement: Versioned persistence and supported drafts
New graphic binding identifiers MUST require project schema45. Existing current documents and retained undo/redo snapshots from supported predecessor schemas MUST migrate under the project lock and existing atomic publication rules, preserving content except established migrations and version adoption. New bindings claiming predecessor schema and unknown future current/history schemas MUST fail closed without partial writes. Undo/redo/reopen MUST retain exact slot values, alpha, paints and unaffected properties. Existing supported draft operations and envelope versions MUST support these graphic bindings through isolated materialization/preview/update/rebase/commit/discard without introducing a new draft format or publishing candidate state prematurely.

#### Scenario: G6 migrate and reopen retained state
- **WHEN** a schema44 project with existing slots and retained undo/redo history is opened repeatedly, or a new graphic project is undone/redone/reopened
- **THEN** supported documents become schema45 deterministically with resources/provenance and prior content preserved, and new graphic overrides render equivalently after lifecycle operations

#### Scenario: G7 reject mislabeled or future documents atomically
- **WHEN** current or retained history contains a new binding under schema44 or an unknown future schema
- **THEN** opening rejects it without publishing any migrated project/history/resource bytes

#### Scenario: G8 isolate and commit supported drafts
- **WHEN** a supported draft changes graphic slots/instance overrides, previews, reopens, rebases or commits while its revision is current versus stale
- **THEN** candidate state is isolated, committed content occupies one undo step, discard preserves durable state and stale commits retain retryable REVISION_CONFLICT with the draft available

### Requirement: Governed additive contracts and independent native evidence
The extension MUST advertise `solid_graphic_color_slots_v1` and manually synchronize canonical artifacts and governed native/TypeScript/MCP consumers while preserving existing protocol-v1 requests, slot kinds, errors, renderer limits and unrelated catalog sections. Actual source and freshly packaged MCP native checks MUST render two differently colored occurrences in frame and short range output plus an isolated draft preview. Independently authored expected pixels MUST demonstrate each binding and preserved alpha; blank/wrong-size controls MUST fail. Native dimensions/frame count, saved-state invariance and lifecycle MUST be checked. Existing platform RGB rounding bounds, required assertions, deadlines and protected gates MUST remain unchanged.

#### Scenario: G9 prove actual native colors and alpha
- **WHEN** the small solid-graphic fixture renders through actual source and fresh packaged entry points on required platforms
- **THEN** independent interior/stroke/dot samples match both occurrences and draft override with preserved alpha, frame/range facts agree within existing tolerances, blank/wrong-size evidence fails and project/history content remains unchanged by rendering

#### Scenario: G10 enforce canonical parity and prior compatibility
- **WHEN** Rust, TypeScript/Zod, MCP and capability checks consume the reviewed extension and unchanged predecessor sections
- **THEN** identifiers, fixtures, schema adoption, structural catalogs and errors agree without changing prior request/value meanings or admitting unsupported properties
