## 1. Specification approval and canonical fixtures

- [x] 1.1 Confirm promoted $openspec-propose artifacts against verified #49 living specs/schema31/canonical expanded digest, retain pinned strict validation evidence and obtain explicit parent/reviewer approval before implementation. Record issue 50 model-only scope and independence from #51 rendering; resolve independent specification review findings.
- [x] 1.2 Add contracts/mask-models-v1.json valid/invalid boundaries for all operations/channels/paint variants/transforms, nested strict decoding and item-local identity; register canonical ownership consumers and CODEOWNER paths before consumers. Retain reused vector/transform contract owners.

## 2. Core model and persisted migration

- [x] 2.1 Add failing Rust canonical fixture/strict JSON tests for every Mask and Bounded stored mask ownership scenario, including duplicate raw JSON keys at observable core/headless Serde boundaries (not parsed MCP/Zod objects), null/unknown/positional shapes, all inclusive/overflow limits, hidden/unused definitions and target eligibility; add core model/validator in owning layer and reuse VectorPath/Paint/Transform2D validators.
- [x] 2.2 Add failing source-version/current+undo+redo+draft/journal migration tests before schema 32 and guards; implement cloned locked atomic migration, premature-field rejection and missing-base stale draft behavior. Prove no partial authoritative/resource writes, schema 32 omitted-default no-rewrite, IDs/revisions/provenance/media integrity and repeat-open determinism (both persistence requirements).

## 3. Core edits and inactive scene semantics

- [x] 3.1 Extend update_item and batch edit types, defaults and timeline mutation; test standalone set/reorder/omit/clear, add→@alias→mask batch, component definitions, draft create/update/commit/discard and failures with byte/revision/resource preservation. Test undo/redo/reopen of exact ordered masked generations.
- [x] 3.2 Add unchanged evaluated-scene semantic-projection/resource/graph tests, enumerate permitted revision/snapshot/output/temp normalization, separately assert metadata commit revisions and preserved revision-scoped cache invalidation for omitted/nonempty masks; retain identity mask stage and reject mask animation targets. Add native frame/range/draft/export lossless pixel/audio/timing equality against otherwise identical #49 scenes, plus existing failure/budget no-publication behavior. No raster implementation or golden regeneration.

## 4. Explicit cross-layer contract synchronization

- [x] 4.1 Synchronize headless edit/request/response declarations and actual protocol tests, capability mask_models_v1/editor status and schema 32 reporting; preserve operation names/errors/major 1.
- [x] 4.2 Synchronize strict bridge mask schemas, existing returned item/component/draft/update unions and capability registration/types. Add shared fixture tests and real MCP standalone/alias/reorder/history/reopen workflows with small bounded canvases; do not weaken native or default contract oracles.
- [x] 4.3 Deliberately synchronize reviewed MCP catalog schemas/$defs and additive capability, compute reviewed new expanded digest and retain exact #49 predecessor projection evidence, including only enumerated projectSchemaVersion/schemaVersion response/status literal32→31 projections; test unauthorized unrelated drift and malformed references. Complete all ownership-listed consumers and designated @matiHirCab contract review evidence; do not auto-generate expected catalogs in tests.
- [x] 4.4 Add docs/mask-models.md with complete typed examples, all limits, coordinates, alpha/luma meaning, ordering/defaults/errors and explicit accepted/persisted/editable but visually inactive scope; explain #51 rendering activation and source-layer/matte deferral.

- [x] 4.5 Synchronize only six existing active animation catalogs' top-level current projectSchemaVersion31→32 and governed exact consumer expectations after explicit amendment approval; preserve all historical source-version cases and every other catalog value. Add pinned predecessor projection and unauthorized-drift tests; retain the mask MCP digest boundary unchanged.

## 5. Required verification

- [x] 5.1 From root with supported activation, run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, focused mask/migration/headless tests, and exact native golden/parity suites required by docs/ci-parity-gates.md. Reuse unchanged passing evidence only as repository policy allows; new mask/output-preservation tests must actually execute.
- [x] 5.2 From apps/agent-bridge run `bun run typecheck`, `bun run lint`, `bun run test:unit`, `bun run contracts:check`, `bun run test:integration`, `bun run test:smoke`; from root run `bun run apps/agent-bridge/scripts/run-python-tests.ts` for required hermetic workers. Record exact counts, failures/skips, exits and full external log paths.
- [x] 5.3 Run `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive` then pre-archive `moon run root:openspec-validate`; only rejection naming this active change is expected, never reported as gate success. Resolve all other failures.

## 6. Independent conformance and archival

- [x] 6.1 Apply $openspec-verify-change and independent implementation review; map each normative scenario to automated evidence in verification.md, resolve all mismatches and finish every required check on stable inputs.
- [x] 6.2 Apply $openspec-sync-specs/$openspec-archive-change for this verified change only. Re-run unchanged `moon run root:openspec-validate` plus pinned strict all-spec validation after archival; completion requires both pass. Parent owns branch/PR/publication actions.
