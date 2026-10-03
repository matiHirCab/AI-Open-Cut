## ADDED Requirements

### Requirement: Compatible audiovisual review tool
MCP SHALL expose the additive `preview_review_range` tool with resolution `540p|720p|project|{width,height}`, default project, optional fps, and includeAudio default true. Explicit false SHALL opt out. Legacy `preview_render_range` SHALL retain its existing required custom resolution/fps and audio-off omission behavior. Headless SHALL expose additive `render_review_range` with the same typed resolution selection, project selection/fps defaults and audio omission true. Existing `render_preview_range` SHALL preserve its required explicit fields, audio choices, and historically broader positive-only numeric dimension acceptance. Existing job envelopes, cancellation and error retryability SHALL remain unchanged.

#### Scenario: Queue default and selected reviews
- **WHEN** a client submits default, preset, or complete custom input to preview_review_range
- **THEN** the bridge queues the existing range workflow with typed selections and audio true unless explicitly false

#### Scenario: Preserve legacy input
- **WHEN** a client submits a previously valid legacy MCP or headless custom request
- **THEN** dimensions, frame rate, audio selection and job response semantics remain unchanged

#### Scenario: Reject malformed selection
- **WHEN** a caller supplies unknown preset, unknown fields, incorrect types, partial custom selection, or conflicting preset/custom fields
- **THEN** schema/deserialization errors use existing `INVALID_ARGUMENT` behavior and canonical semantic conflicts use `VALIDATION_FAILED`, with no persisted mutation or output

### Requirement: Discoverable review preset support
Ready rendering status SHALL report `preview_review_presets_v1` in subsystem and aggregate headless/MCP capability lists without changing protocol major1. Unready rendering SHALL omit it. Canonical catalogs and cross-language fixtures SHALL govern the new tool, request fields, defaults and capability.

#### Scenario: Discover ready and unavailable review
- **WHEN** status is requested with rendering ready or unavailable
- **THEN** the new capability is present only when ready and existing readiness errors/capabilities remain unchanged
