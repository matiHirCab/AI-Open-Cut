## MODIFIED Requirements

### Requirement: Readiness-gated linear composition capability
Protocol-version-1 capability reporting MUST add the unique `linear_light_compositing_v1` capability only when the configured local renderer can execute the complete currently supported scene with the normative linear premultiplied pipeline. An absent or unusable renderer MUST omit that capability while preserving existing readiness errors. Canonical headless/MCP capability catalogs and every governed Rust/TypeScript consumer and parity test MUST agree; existing simple clients MUST remain valid and no authored or persisted contract shape MUST change. Ordinary hermetic protocol conformance MUST validate both returned readiness outcomes without assuming media dependencies are installed; explicitly required native conformance using OPENCUT_GOLDEN_REQUIRED=1 MUST additionally require rendering readiness true and MUST NOT skip unavailable dependencies.

#### Scenario: Detect ready corrected rendering
- **WHEN** a client queries protocol-v1 information with a conforming ready renderer
- **THEN** the capability list contains `linear_light_compositing_v1` and the canonical catalogs/consumer parity agree

#### Scenario: Omit unsupported capability
- **WHEN** the renderer executable, required local dependency or complete pipeline is unavailable
- **THEN** protocol information omits `linear_light_compositing_v1` and render requests retain their stable `DEPENDENCY_UNAVAILABLE` behavior without degraded fallback

#### Scenario: Verify unavailable dependencies in hermetic conformance
- **WHEN** protocol-v1 conformance runs with unavailable media executables without explicitly required native mode
- **THEN** it executes exact editor-only top-level capability checks, empty rendering capabilities, boolean readiness false and nonretryable DEPENDENCY_UNAVAILABLE, with no linear capability in editor or rendering lists

#### Scenario: Require configured native readiness evidence
- **WHEN** the same protocol-v1 conformance runs with OPENCUT_GOLDEN_REQUIRED=1
- **THEN** readiness must be true, rendering capabilities equal the complete canonical list, the rendering error is null and the unique linear capability appears exactly once at top-level and rendering scope and never in editor scope
