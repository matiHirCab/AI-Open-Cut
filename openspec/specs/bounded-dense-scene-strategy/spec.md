# Bounded dense-scene strategy

## Purpose
Define finite attributable native-scene evaluation, unchanged resource admission and independent parity proof, and evidence-backed conditional rendering strategy decisions.

## Requirements

### Requirement: Finite original workload and attributable measurement
The evaluation SHALL use only the existing original issue181 hero baseline, completed native hero/motif examples and role-team minimum/maximum row/grid examples at1920x1080,30fps and1000ms. The closed seven-workload matrix MUST be: issue181 baseline, native hero recipe with its two default instances, motif gallery with its ten declared default/alternate instances, row1, row4, grid1 and grid4. Each cell MUST additionally have exactly three raster-warm/encoded-cold control requests after that cell's warm-up. It MUST measure frame500ms and range[0,100)ms using three fresh Renderer requests, one excluded warm-up and three same-Renderer repeats for each cell. Fresh MUST NOT imply flushed operating-system caches; repeat reports MUST identify encoded-preview reuse separately from raster reuse. Exact source/driver/fixture/binary/font/tool identities, release/debug status, CPU/quota/memory and source dirtiness MUST accompany finite nonnegative observations. The decision statistic MUST be the maximum of the three fresh-renderer samples per workload/intent against the unchanged5000ms frame/8000ms range thresholds; all fourteen cells MUST meet their threshold to select current direct rendering. Repeat/control observations MUST be reported separately and MUST NOT substitute for failed fresh coverage. Timing MUST remain report-only in portable CI; comparisons to issue181's5000ms frame/8000ms three-frame-range targets MUST explicitly name the reference host and workload differences. The unimplemented96-leaf/six-instance/3000ms target envelope MUST NOT be claimed measured.

#### Scenario: D1 Capture the fixed matrix
- **WHEN** native evaluation runs on the approved original fixtures
- **THEN** each workload and intent has exact settings/sample counts, attributable observations and explicit admitted or typed-rejected status

#### Scenario: D2 Reject misleading or malformed evidence
- **WHEN** report settings/counts/identities are absent or drift, values are nonfinite, or unimplemented target readiness is asserted
- **THEN** automated validation rejects the report instead of accepting inferred performance

### Requirement: Existing resource admission and source raster evidence
The report MUST retain authoritative existing core accounting with exact-versus-conservative units and unchanged limits, and MUST report the maximum sampled resident bytes of the test process and recursively discovered child processes under the existing joined sampler. Request-scoped reset/readout MUST isolate owner-local cfg(test) observations at existing charge_render_work and certify_composition_memory, the successful raster miss closure, native visual preparation and final SystemProcessExecutor execution. Preparation/final-execution direct timings MUST be labeled inclusive/non-additive and absent stages MUST be labeled unobserved rather than inferred. These seams MUST add no production behavior, public API, feature flag or dependency edge. Test-only instrumentation MUST record actual raster hit/miss work and directly timed successful source rasterization, distinct from facade time, encoded-cache hits and composition/backend work. Independent timings MUST NOT be summed/subtracted to claim unmeasured stages. Representative equal-component or repeated-request cache reuse MUST have pixel parity and actual-work evidence. Any guard rejection MUST retain its original typed error/retryability and no publication or state mutation; intentional guards MUST NOT be called renderer defects.

#### Scenario: D3 Distinguish reuse from composition
- **WHEN** fresh, raster-warm but encoded-cold and identical repeated requests render the same immutable fixture
- **THEN** counters and direct source timings identify actual avoided raster work while total measured rendering and sampled memory remain separately attributed

#### Scenario: D4 Report costs and safe failures
- **WHEN** the fixed workload is admitted or an existing guard rejects it
- **THEN** existing owner-certified charged work or the original sanitized diagnostic is retained with units/limits, and project/history/resources and preexisting outputs remain unchanged

### Requirement: Independent native output and evaluation controls
The evaluation MUST reuse independently authored fixture geometry/color/timing expectations and unchanged established tolerances, validate dimensions/frame counts, and compare decoded fresh/warm outputs. Blank, wrong-size, truncated, wrong-paint and false-reuse evidence MUST fail automated controls. Required configured tools/font MUST fail closed. All observations MUST preserve authoritative current/history state. Source/package/MCP/native predecessor verification MUST remain intact without golden regeneration.

#### Scenario: D5 Verify every measured output
- **WHEN** measured frame/range requests return artifacts
- **THEN** independent original oracles and fresh/warm parity validate output and authoritative bytes remain unchanged

#### Scenario: D6 Reject false output or reuse
- **WHEN** blank/drifted output or fabricated cache evidence is supplied
- **THEN** the evaluation fails without adapting expected pixels, tolerances or references

### Requirement: Evidence-backed bounded decision and conditional disposition
A reviewed ADR/report SHALL select exactly one outcome: existing direct rendering meets the bounded reference targets, one small identified separately scoped optimization candidate is selected (its sufficiency remains unverified until that implementation passes its own required evidence), or static source-preserving precomposition is justified. It MUST explain observed limiting paths, exact proposed change or deliberate absence of runtime change, compatibility/resource proof, independent parity cases and reference performance threshold. Any direct optimization MUST have a separate implementation issue linked to180 before implementation. Issues188/189 MUST be explicitly activated or closed as not planned with the decision linked; implementation MUST follow the selected dependency chain. Uncertain policy choices reserved to the owner MUST be surfaced. Release MUST remain blocked until all selected implementation work is verified. No renderer rewrite, guard inflation, precomposition implementation, merge or deployment SHALL occur here.

#### Scenario: D7 Resolve the decision using measured evidence
- **WHEN** finite accepted observations and resource/reuse evidence are reviewed
- **THEN** the ADR selects one supported outcome with bounded follow-up scope and explicit188/189 disposition, without implementing the selected optimization inside187

#### Scenario: D8 Preserve protected acceptance
- **WHEN** the decision/evaluation is completed and published
- **THEN** approved requirements/tasks and actual checks are verified, synced and archived, predecessor behavior remains compatible, and all required workflows plus attested unchanged-budget foundation pass at the exact remote head
