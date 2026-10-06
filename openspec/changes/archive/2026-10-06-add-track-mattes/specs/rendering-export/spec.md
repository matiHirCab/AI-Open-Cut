## ADDED Requirements

### Requirement: Shared evaluated track matte semantics
Immutable EvaluatedScene MUST own scoped matte DAG bindings, provider occurrence membership, direct-draw/matteOnly roles, exact temporal dependencies and certified budgets/resources. Frame, audiovisual range, draft and export MUST consume those same evaluated semantics before graph/artifact publication, with canonical paint ordering retained. Exact renderer bypass MUST require both every matte=None and every matteOnly=false; an unreferenced matteOnly=true leaf MUST still suppress direct drawing and preserve audio/timing. Provider execution may precede its normal destination position, but SHALL NOT reorder final destination drawing, sample opaque destination backgrounds or add audio duplicates. Revision/project/cache identities MUST retain existing behavior; tests SHALL compare render semantics and decoded oracles rather than entire revision-bearing plan hashes.

#### Scenario: Compare analytic and actual native all-intent output
- **WHEN** asymmetric animated/masked/transformed matte scenes with alpha/luma, provider chains and matteOnly are previewed, range-rendered, drafted and exported using configured required native tools/fonts
- **THEN** each intent matches independent numeric stage equations and shared decoded visual/audio/timing tolerance, with exact same-intent lossless identity where applicable and unchanged audio/duration

#### Scenario: Validate dependencies before cache hits or output
- **WHEN** a matteOnly/offscreen provider has invalid resource integrity, failed complete readiness or excessive certified graph/time/work/memory
- **THEN** ordinary validation rejects before cache shortcuts, raster/decoder execution or artifact publication and no degraded output appears

#### Scenario: Require both defaults for exact bypass
- **WHEN** a project has no references but an unreferenced eligible leaf has matteOnly=true
- **THEN** that leaf’s direct drawing is suppressed with audio/timing unchanged; exact prior-render bypass occurs only when every reference is None and every matteOnly is false
