## ADDED Requirements

### Requirement: Bounded illustrated scene inventory
The repository MUST maintain a versioned original/synthetic hero-card benchmark and three final composition target inventories. The benchmark MUST identify native layer roles, supported text and managed-image inputs, canvas, frame rate, duration, sample timestamps and inclusive caps on items, instances, geometry, text, artwork and animation. Later targets MUST distinguish acceptance goals from implemented fixtures and MUST NOT imply new palette, pack or precomposition behavior.

#### Scenario: I1 Inspect the initial slice and later targets
- **WHEN** the inventory is loaded
- **THEN** one small hero-card recipe and exactly three separately bounded targets have unique roles, original/synthetic provenance and deterministic supported-input limits without private project files

### Requirement: Independent illustrated native evidence
The benchmark MUST construct editable content through existing editor-core authoring APIs and compare actual 1920×1080 native frame and short range outputs against independently authored layout and interior/exterior color samples. Fixture identity, bounds and expected samples MUST validate before rendering. Required native golden execution MUST include this evidence without replacing or regenerating existing references. Missing configured native tools or font, invalid bounds or divergent expected pixels MUST fail rather than report success. Rendering MUST preserve project revision and authoritative editable content.

#### Scenario: I2 Render the bounded hero card
- **WHEN** the original recipe is authored and rendered at the declared timestamps and short range
- **THEN** named content remains editable, independent geometry/color samples agree within declared codec tolerances, output dimensions and frame count agree, and project state and revision remain unchanged

#### Scenario: I3 Reject fixture or output drift
- **WHEN** fixture bounds, unique roles, input limits or expected output are altered inconsistently, or required configured tools are unavailable
- **THEN** verification fails without accepting a new reference or changing renderer guards

### Requirement: Attributable illustrated performance baseline
The benchmark MUST record exact source and runtime identities, reference host/tool/font identities, fixed workload and directly measured fresh-renderer and repeated-request frame/range observations. Fresh renderer MUST mean a new process-local renderer without claiming cleared OS caches; repeated requests MUST identify applicable raster or encoded-preview reuse without claiming isolated renderer-stage speedups. Successful bounded work and any limiting existing guard MUST be reported separately. Numeric targets MUST be set only after measurements and scoped to the named reference environment; timing MUST remain report-only in portable CI. The three later targets MUST NOT be described as measured or renderer-ready before their own actual verification.

#### Scenario: I4 Capture and interpret the baseline
- **WHEN** the benchmark executes its fixed fresh/repeated sample counts
- **THEN** finite nonnegative frame/range timings carry source/runtime and workload identities, independently verified outputs, cache semantics and existing guard information, and documented reference targets derive from the measured observations rather than universal CI timing assertions
