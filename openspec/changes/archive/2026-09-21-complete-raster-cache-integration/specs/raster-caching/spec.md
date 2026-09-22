## ADDED Requirements

### Requirement: Cross-request agent raster reuse
Successive frame, range, draft and export requests served by an available persistent render worker SHALL share one editor-core renderer cache. Equal keys MUST avoid rasterization across request boundaries while every request repeats canonical preflight and loads its own immutable snapshot. Retained key/payload limits MUST remain 128 entries and 67108864 bytes per renderer. Documentation MUST distinguish retained memory from in-flight work and independent overflow processes. Cache state MUST remain disposable and absent from persisted projects and public render results.

#### Scenario: X1 Prove agent reuse through actual raster work
- **WHEN** an agent renders unchanged text, shapes and SVG through successive worker requests at matching settings
- **THEN** instrumented tests prove the same worker avoids subsequent rasterization and output matches a fresh renderer with correct plans, warnings, layout diagnostics, visual/audio tolerances and timing

#### Scenario: X2 Preserve invalidation and cold restart
- **WHEN** project identity, revision, effective draft content or pixel dependencies change, or a worker is replaced
- **THEN** changed keys or a fresh cache miss, unchanged eligible keys remain reusable, and each snapshot renders its own content without affecting history or revisions

### Requirement: Production-path dependency invalidation evidence
Complete raster identity MUST be backed by valid independently varied inputs submitted through evaluation and materialization. Changed output MUST match a separately constructed renderer, then reuse its own cached bytes, while unrelated retained keys remain reusable. Hash-sensitivity tests alone MUST NOT establish this conformance. Unsupported font profiles MUST be tested as failures rather than presented as valid font changes. Tests MUST distinguish composition-only changes from changes to composed raster sampling.

#### Scenario: X3 Vary valid pixel dependencies
- **WHEN** tests independently vary text/runs/spans, paint stacks, layout, selected valid font bytes, vector geometry/paint/stroke, SVG viewport/order, output dimensions or composed sampling scale at the same revision
- **THEN** changed rasters miss and equal fresh output, later repeats hit, and an unrelated retained raster still hits

#### Scenario: X4 Reuse composition-only changes
- **WHEN** timing, position or opacity changes leave local pixels and sampling density unchanged, including component and repeater occurrences
- **THEN** cached local bytes are reused while independent plan/placement assertions demonstrate the changed composition correctly

### Requirement: Warm preflight conformance evidence
Warm-cache validation tests MUST compare cold and warm error codes, retryability and relevant stages, demonstrate unchanged hit and miss counters, inspect process/artifact events and preserve authoritative project/history/draft state. They MUST cover invalid numbers, missing references, missing/corrupt fonts, unsafe paths, unsupported SVG, aggregate complexity and readiness failures through applicable frame/range/draft/export routes. Existing preflight precedence over export collision and existing workspace-write failure behavior MUST remain effective. Instrumentation MUST be explicitly test-gated and absent from normal wire output.

#### Scenario: X5 Reject before lookup and output side effects
- **WHEN** a populated cache is followed by any specified invalid-input, resource, aggregate-limit or dependency-readiness failure
- **THEN** cold and warm typed errors agree, neither hit nor miss counts advance, prohibited process/artifact side effects do not occur and authoritative state is unchanged

#### Scenario: X6 Preserve precedence and warm materialization failures
- **WHEN** invalid work targets an existing export destination or writing a valid cached raster fails
- **THEN** invalid work retains its canonical error before collision inspection, and a warm write failure retains the established render error, cleanup and no-partial-publication behavior
