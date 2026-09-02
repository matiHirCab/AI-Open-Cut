## Context

`contracts/motion-graphics-v1.json` is fixture-governed under ADR 0002 and remains `fixture_only`. Its current Rust and TypeScript validators check catalog metadata, wrapper fields, finite JSON numbers, selected unsafe substrings, and global reference-string closure. They do not validate the closed shape of each concept payload, reconcile reference metadata with payload fields, or prove every normative negative case in the living specification.

The correction must preserve fixture-governed manual synchronization. It must not turn the catalog into a generated runtime schema, add dependencies, or imply that editor-core, headless, MCP, or the renderer accepts the vocabulary.

## Goals / Non-Goals

**Goals:**

- Make every valid payload closed and typed in both Rust and TypeScript.
- Make logical definitions and references typed, structured, scoped, payload-derived, and deterministically resolvable.
- Make every normative negative scenario concrete and require exact cross-language failure evidence.
- Validate unsafe resources only where a field represents a resource or executable expression.
- Restore a complete living specification after archival.

**Non-Goals:**

- Add production motion-graphics types, migrations, operations, capabilities, stable errors, renderer behavior, or new package dependencies.
- Generate Rust or TypeScript declarations from a schema.
- Edit the archived `2026-09-01-add-motion-graphics-contract-fixtures` change.

## Decisions

### Keep fixture-governed native validators

Rust will define test-only Serde structs/enums with `deny_unknown_fields`; TypeScript will define mirrored strict Zod schemas. Each concept validator will enforce required fields, field types, closed tagged variants, safe integer timing and seeds, finite scalar ranges, positive dimensions/time scales, opacity/anchor bounds, spring constraints, slot constraints, and bounded collection sizes from the catalog.

This follows ADR 0002's hand-authored native declarations. Adding JSON Schema, Ajv, `jsonschema`, or a custom schema language is rejected because it introduces a new canonical type system and dependencies for fixture-only evidence. The JSON examples remain canonical; the two native validators prove that both languages interpret them consistently.

### Use structured scoped references

Every fixture definition/reference becomes an object with closed fields `kind`, `scope`, and `id`. Scope is exactly `project`, `root`, or `component:<component-id>`. Kinds are a closed identifier set for components, layers, transforms, slots, markers, masks, effects, managed assets, sound definitions, audio buses, audio events, and curves.

After strict payload parsing, each concept validator derives the logical definitions and references represented by payload fields. The derived sorted sets must exactly equal fixture metadata. Global closure then resolves the exact `(scope, kind, id)` tuple, rejects duplicates, and applies parent/component dependency cycle and depth checks.

The rule-card component owns its `impact` marker in `component:rule_card`. Root layers and masks remain in `root`; reusable assets, sound definitions, and buses use `project`. External managed assets are declared through a closed top-level managed-resource fixture rather than claimed by unrelated payloads.

### Make negative fixtures complete and executable

Each invalid record remains closed and gains a complete concept payload plus an expected classification and reason key. Both validators must reject the payload and return that exact fixture-level reason. Required cases include:

- hierarchy: missing/cross-scope parent or component, direct/indirect cycles, and depth overflow;
- slots: wrong type, missing required value, invalid default, constraint violation, missing target, and arbitrary property path;
- timing/audio: missing/ambiguous marker, sound definition, variant, and bus;
- graphics/security: non-finite values, unknown variants, collection-limit overflow, executable SVG/event handler, network or filesystem resource, traversal, and renderer expression.

Tests also mutate valid fixtures in memory to prove rejection of string opacity, unknown transform fields, missing required fields, resource fields containing POSIX/Windows/UNC/traversal/URI inputs, and scope mismatch. Required negative fixture IDs and reason keys are compared as exact per-concept sets so a broad global classification cannot hide missing scenarios.

### Validate resource-bearing fields, not arbitrary text

The generic recursive check retains finite-number validation only. Managed asset, SVG/resource, and renderer-expression fields receive dedicated validation: managed IDs must match the canonical logical-ID pattern; filesystem paths, traversal, UNC paths, URI schemes, network resources, scripts, event handlers, and raw FFmpeg/filter expressions are rejected. Text and rich-text slot content may contain ordinary URL-like prose because it is not resolved as a resource.

### Preserve fixture-only compatibility

The catalog remains version 1 with status `fixture_only`. No headless, MCP, error, provider, project, capability, or renderer fixture changes. Because version 1 has not been merged or activated, correcting its preparatory metadata and examples does not require a new major version.

## Verification and Rollback

Run focused malformed/valid/invalid fixture tests in both languages, the contract gate, strict OpenSpec validation, Rust format/Clippy/workspace tests, TypeScript typecheck/lint/unit tests, and `git diff --check`. Integration and packaged smoke remain unaffected unless shared runtime/bootstrap code changes.

Rollback removes this corrective change's catalog/test/spec edits together. There is no data or external-service rollback.
