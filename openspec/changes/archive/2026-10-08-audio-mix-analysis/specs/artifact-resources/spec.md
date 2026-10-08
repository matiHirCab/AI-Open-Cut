## ADDED Requirements

### Requirement: Bounded audio analysis JSON resources
Completed audio_analysis jobs SHALL add an opaque application/json artifact descriptor/link and typed summary through the existing process-local job ownership, canonical URI, path/symlink/session confinement and retention policy. Default polling/job-status resources MUST remain metadata-only without opening waveform files; includeBinary SHALL not embed JSON. Explicit resources/read SHALL return one UTF8 JSON text item with the matching MIME type, versioned summary and bins, bounded to4MiB using actual opened-handle reads plus an overflow detection byte. Oversized, malformed, nonUTF8, missing or unsafe JSON SHALL produce existing safe non-retryable VALIDATION_FAILED without paths or project changes. Expired/evicted/restarted/foreign jobs retain JOB_NOT_FOUND. Original PNG/WAV/MP4 retrieval and policy SHALL remain unchanged.

#### Scenario: Read bounded owned JSON
- **WHEN** a client explicitly reads the retained completed analysis job's canonical or accepted SDK-normalized artifact URI
- **THEN** it receives its bounded JSON text and matching metadata without changing project state or binary-resource behavior

#### Scenario: Poll analysis without file access
- **WHEN** completed analysis metadata is polled with absent/false/true includeBinary or its job-status resource is read
- **THEN** summary/descriptor/link remain available without opening JSON bytes and no waveform or legacy image/audio block is embedded

#### Scenario: Reject tampering and read races
- **WHEN** an analysis artifact is oversized, grows between stat/read, is malformed/nonUTF8, disappears, or attempts symlink/traversal escape
- **THEN** actual read allocation is bounded and existing safe VALIDATION_FAILED occurs without exposing unrelated bytes or paths

#### Scenario: Preserve registry and binary retention
- **WHEN** an analysis job expires/restarts/is foreign or an existing binary job is read
- **THEN** original JOB_NOT_FOUND and original PNG/WAV/MP4 content/retention semantics remain valid
