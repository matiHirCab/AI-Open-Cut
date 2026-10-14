## ADDED Requirements

### Requirement: Exact byte conversion and unchanged failures
RGBA decoding SHALL preserve the predecessor's f64 sRGB transfer for every u8 channel, f64 multiplication by the decoded u8 alpha/255 and subsequent f32 cast, including all channel/alpha bit patterns and alpha bits. Existing dimensions, maximum pixels, malformed length/PAM errors, error precedence, zero-size behavior and publication/state semantics MUST remain unchanged. No public or persisted contract/migration SHALL be introduced.

#### Scenario: X1 Exhaustive original bit patterns
- **WHEN** every256channel×256alpha pair is decoded across RGB and alpha
- **THEN** all65536pairs match independently retained predecessor expressions bit-for-bit

#### Scenario: X2 Preserve input and error boundaries
- **WHEN** empty, malformed-size/PAM or excessive/overflowing dimensions are supplied
- **THEN** prior output or typed errors/precedence remain identical without state/publication changes

### Requirement: Immutable bounded transfer reuse
Successful byte conversion SHALL reuse one immutable process-wide256-entry f64 transfer lookup without per-raster/source/project retention or extra pixel copying. Payload MUST be exactly2048bytes; total static scalar/synchronization storage MUST be measured and at most4096bytes, covered by the unchanged existing1048576byte fixed-overhead reservation. Initialization MUST use the predecessor transfer on this runtime, be safely shared across concurrent callers, and retain original alpha/cast order. Resource guards, owner accounting, raster keys, warnings and eligibility MUST remain unchanged.

#### Scenario: X3 One bounded shared lookup
- **WHEN** repeated/concurrent byte conversions initialize or reuse the lookup
- **THEN** callers observe one immutable table, exact bits and bounded payload/header without added retained allocations or copied rasters

#### Scenario: X4 Preserve authoritative resource admission
- **WHEN** bounded scenes or existing admission-boundary failures are processed
- **THEN** original charges, limits and typed failures remain authoritative with no budget increase or second scene estimator

### Requirement: Independent native and lifecycle compatibility
Original independently authored color/alpha/glyph/temporal expectations and established frame/range/draft/export cold/warm parity SHALL remain valid. Existing cache actual-work evidence, history/drafts/resources/preexisting-output invariants and missing-dependency/error controls MUST remain intact. Before/after report artifacts MUST retain corresponding output hashes and validated settings/counts; no expected pixel, tolerance or reference SHALL be regenerated.

#### Scenario: X5 Preserve measured and supported routes
- **WHEN** original fixtures render through existing cold/warm frame/range/draft/export routes and lifecycle checks
- **THEN** independent output/parity/state/resource checks pass with unchanged raster-cache identities and behavior

#### Scenario: X6 Reject output or evidence drift
- **WHEN** conversion/order/alpha/output, source identities, request counts or cache evidence is corrupted
- **THEN** exhaustive/native/report controls reject the drift without adapting expectations

### Requirement: Controlled bounded improvement and protected acceptance
The optimization SHALL be judged using one predeclared before and one after capture of the unchanged seven-workload/fourteen-cell1920x1080/30fps/1000ms matrix, frame500ms and range[0,100)ms: each cell exactly3fresh+1excludedwarmup+3repeat+3raster-warm/encoded-cold requests. Both captures MUST use the same approved measurement driver/settings and issue181Debian13/Xeon8370C/Rust1.97release/FFmpeg7.1.5/font identities, actual matching CPU/memory quotas, and serial acquisition without concurrent repository suites. First singleton initialization cost MUST be included in the standalone after capture; fresh means new Renderer, not flushed OS caches or repeated process-cold initialization. All fourteen after fresh maxima MUST meet5000ms frame/8000ms range. The predeclared row4frame facade maximum MUST decrease by at least5percent versus controlled before, and its median directly measured inclusive native preparation MUST decrease by at least10percent. Direct stage timings remain non-additive; these small-sample bounded results MUST NOT become portable/tail guarantees. Original5414.023545ms counterexample and every failed attempt MUST remain retained; resampling MUST NOT erase failure. Invalid host/data or failed sufficiency MUST block the claim and be reported without scope/guard/tolerance expansion. Actual source/driver/binary/fixture/tool identities, owner work, sampled memory and cold/warm/control differences MUST be recorded. Local repository/native checks, independent conformance, sync/archive, protected acceptance and all18/four-workflow exact published-head CI with attested unchanged default120-minute budget MUST pass before completion.

#### Scenario: X7 Prove exact finite before and after
- **WHEN** the two controlled140request captures and their source/host/resource/output evidence are compared
- **THEN** automated comparison enforces exact identity/count/output correspondence, all after target maxima and predeclared row4facade/preparation improvement, while retaining original failure provenance

#### Scenario: X8 Fail bounded sufficiency or protected acceptance
- **WHEN** captures drift, a threshold/gain fails, a required native/check fails or published-head acceptance is incomplete
- **THEN** completion/sufficiency remains blocked with exact evidence retained and no budget, tolerance, reference or scope workaround
