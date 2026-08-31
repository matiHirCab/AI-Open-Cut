# Rendering and Export Specification

## Purpose

Define deterministic preview and export behavior, dependency readiness, progress, and output publication safety.

## Requirements

### Requirement: Explicit renderer readiness
The renderer MUST verify the configured FFmpeg and FFprobe executables plus the required FFmpeg filters before rendering and SHALL report unavailable dependencies with stable errors.

#### Scenario: FFmpeg is unavailable
- **WHEN** the configured FFmpeg executable cannot be used
- **THEN** readiness or rendering fails with `DEPENDENCY_UNAVAILABLE` instead of starting an incomplete render

#### Scenario: Required media tooling is incomplete
- **WHEN** FFprobe is unavailable or FFmpeg lacks overlay, drawtext, or audio-mix support
- **THEN** renderer readiness fails with `DEPENDENCY_UNAVAILABLE`

### Requirement: Deterministic frame preview
The renderer SHALL produce a preview frame for a validated project revision and timestamp using the same canonical composition and animation rules used by export.

#### Scenario: Render a preview frame
- **WHEN** a caller requests a timestamp within a valid project
- **THEN** the renderer produces a managed preview artifact representing the project at that time

### Requirement: Immutable range preview
Range previews SHALL render the requested valid time span from an immutable project revision and SHALL expose bounded monotonic progress through completion.

#### Scenario: Render a preview range
- **WHEN** a caller queues a valid preview range for the current revision
- **THEN** the resulting video uses final-export rendering behavior and progress remains between zero and one

### Requirement: Safe video export
Video export MUST accept an `.mp4` destination, encode H.264 video with AAC audio in the current MVP, render to a temporary file beside the requested destination, and SHALL publish that file only after FFmpeg completes successfully.

#### Scenario: Reject an unsupported export extension
- **WHEN** an MCP client requests an export destination that does not end in `.mp4`
- **THEN** input validation rejects the request before starting a render job

#### Scenario: Complete an export
- **WHEN** FFmpeg completes a valid export successfully
- **THEN** the temporary file is renamed to the final destination and progress reaches one

#### Scenario: FFmpeg fails
- **WHEN** FFmpeg exits unsuccessfully or emits unusable progress/output
- **THEN** export fails with `FFMPEG_FAILED`, removes its temporary output, and does not publish that partial output as the destination

### Requirement: Explicit overwrite authorization
The renderer MUST NOT replace an existing export unless the caller explicitly sets the overwrite option.

#### Scenario: Reject an export collision
- **WHEN** the destination exists and overwrite is false
- **THEN** export fails with `EXPORT_EXISTS` and leaves the existing file unchanged
