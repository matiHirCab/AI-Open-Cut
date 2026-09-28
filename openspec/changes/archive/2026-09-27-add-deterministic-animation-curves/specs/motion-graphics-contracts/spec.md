## ADDED Requirements

### Requirement: Governed active parameterized curve contract
Canonical checked-in channel and motion-graphics catalogs MUST describe the active `cubic_bezier` and `spring` tagged records, named finite parameter limits, existing string `hold`/`linear` compatibility, and an additive capability/version marker. The governed Rust, TypeScript/Zod, headless, and MCP consumers MUST accept and reject the same canonical valid and invalid fixtures. The existing `set_animation_channels` and batch operations MUST remain typed and additive; unknown variants and fields MUST fail without coercion. Stable error codes and retryability MUST remain unchanged.

#### Scenario: Compare cross-language curve acceptance
- **WHEN** each canonical boundary, malformed, non-finite, unknown-field, and unknown-variant fixture is checked in Rust and TypeScript
- **THEN** both languages agree on its wire shape, named bounds, activation, and stable result

#### Scenario: Discover runtime support
- **WHEN** a client reads capability and version reporting
- **THEN** it can distinguish active Bézier/spring channels from deferred marker, loop, and inactive property behavior

#### Scenario: Retain old clients
- **WHEN** a client sends a previously valid channel or legacy keyframe request
- **THEN** its wire format, result shape, and error behavior remain compatible
