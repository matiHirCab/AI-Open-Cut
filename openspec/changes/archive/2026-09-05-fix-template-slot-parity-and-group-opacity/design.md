## Context

Issue #23 review demonstrated native preservation but bridge loss of `__proto__` overrides: Zod's record parser skips that key. A valid group with omitted Transform2D also rejects a number slot default of 0.5 because effective application changes its forbidden legacy transform. The original implementation remains archived; this follow-up corrects its uncovered cases.

## Goals / Non-Goals

Restore the existing typed-slot contract for every legal ID and visual group, with automated evidence through native and real source/packaged APIs. Preserve schema 12, protocol 1, published structural schemas, rendering and existing inward dependencies. No dependency upgrade, new public fields, migration, commits or pushes.

## Decisions

1. Replace only the shared override-map parser. Use public Zod APIs over an unknown input, validating a non-null, non-array object and each own enumerable string entry using slotValueSchema.safeParse. Prefix each value issue with its slot key. Reconstruct successful parsed entries with Object.fromEntries, which creates ordinary own data properties even for __proto__. Retain Record<string, z.infer<typeof slotValueSchema>> as the output type through a type annotation backed by that runtime validation. Do not forward inherited values, mutate input objects or change prototypes. Both request and response schemas use this parser. Do not blacklist legal identifiers or use private Zod internals.

2. Derive JSON Schema metadata from the existing z.record(z.string(), slotValueSchema) description, omitting its document-level $schema field. Attach it to the validated parser using public metadata APIs. The planning probe confirmed identical input/output JSON Schema for this strategy. Test the final schema equivalence and actual registered MCP catalog; do not manually weaken the catalog to accommodate the implementation. Unknown keys still reach core so ITEM_NOT_FOUND remains canonical. Values are structurally validated regardless of the key spelling.

3. In apply_slot_value, treat groups and component instances as requiring Transform2D for opacity. Initialize identity Transform2D only on the cloned effective candidate when absent; otherwise update only opacity. Preserve the existing path for ordinary visual items. Run the unchanged complete-candidate validation, retaining bounds and lock checks. Rendering and stored base tracks remain untouched.

4. Extend the canonical template-slots-v1 catalog with a dedicated regression section rather than inserting group fixtures into existing all-kind fixtures that assume a text target. Include three special IDs, valid typed overrides, malformed and unknown-key cases, and group inputs with absent/present nondefault Transform2D and opacity endpoints. Consume these records from Rust, TypeScript and the shared real component workflow. Keep all existing catalog records and consumers valid; update ownership only if new consumer files are introduced.

## Risks / Trade-offs

- Runtime validation and published metadata can diverge: compare input/output JSON schemas exactly and retain behavioral negative tests with full error paths.
- Unsafe JavaScript assignment can alter prototypes: use Object.fromEntries and assert own-key preservation and unchanged prototypes, including null-prototype input and JSON round trips.
- Required and optional defaults can conceal dropped keys: test required values without defaults, overridden defaults, missing values and unknown IDs independently.
- Group validation observes a derived clone: native validation-unit evidence must verify effective opacity and preservation of other transform fields; persistence tests must verify unchanged base tracks, exact slots and overrides.

## Migration Plan

No format migration, protocol change or dependency update. Existing persisted overrides become readable without loss through corrected consumers. Reverting the fix would restore the two defects but not require a data rewrite. Retain original archival evidence and record this correction separately.

## Verification Plan

Add tests before fixes and demonstrate the two known failures. Test special IDs in standalone create/update and aliased batches; reject malformed values with key-prefixed paths and unknown IDs with ITEM_NOT_FOUND. Cover undo/redo/reopen, required/default precedence and prototypes. Test group default and override opacity 0, 0.5 and 1 with absent/existing Transform2D, unchanged fields/base tracks, invalid bounds, locked targets, stale revisions and byte-identical batch rollback. Exercise the same workflows in source integration and packaged smoke.

Run cargo fmt --check --all, cargo clippy --workspace --all-targets -- -D warnings, cargo test --workspace with established FFmpeg6/ffprobe and DejaVuSans native configuration; bridge contracts:check, typecheck, lint, test, test:integration, test:smoke and scripts/run-python-tests.ts. Run focused strict OpenSpec validation, openspec-verify-change and git diff --check. Obtain completed contract-owner review, sync/archive, then run moon run root:openspec-validate (pinned CLI wrapper if needed). Any failed required check blocks completion.

## Open Questions

No implementation decisions remain open. Artifact approval preceded implementation; completed contract review was explicitly approved on 2026-09-05 before final archival.

Verification refinement: Vitest's JSON module transformation omitted the own __proto__ fixture key before exercising the parser. TypeScript tests now read and JSON.parse the canonical source bytes, retaining imported types only. Native consumers already parse the same source bytes. This keeps the fixture data literal and avoids testing a transformed object-literal approximation.
