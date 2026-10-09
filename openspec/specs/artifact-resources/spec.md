# Artifact Resources Specification

## Purpose

Define metadata-first process-local MCP artifact delivery, explicit binary retrieval, ownership, path confinement, retention and compatibility.

## Requirements

### Requirement: Metadata-first job content
The bridge SHALL return existing structured job metadata and a matching text block without reading artifact bytes by default, including when includeBinary is absent or false. Completed frame, draft-frame, range, export and speech-preview jobs SHALL expose an optional `artifactResource` descriptor containing an opaque job-owned URI, MIME type, name, expiration, and size when known, plus one matching MCP resource_link content block. Names SHALL be fixed display names derived from job kind/format, never paths or speech output filenames. Pending, failed, cancelled and non-artifact jobs SHALL remain metadata-only. The job-status JSON resource SHALL expose the same descriptor without binary content.

#### Scenario: Poll completed outputs without embedding bytes
- **WHEN** a client polls a completed PNG, MP4 or WAV job without binary opt-in or reads its job-status resource
- **THEN** metadata and the resource descriptor remain available without opening or base64-encoding the file, even if the backing file disappeared

#### Scenario: Poll a job without output
- **WHEN** a job is queued, running, failed, cancelled or completed without a supported artifact
- **THEN** existing status, progress, results and typed errors are preserved and no binary or artifact link is included

### Requirement: Explicit binary delivery and compatibility
The bridge SHALL accept optional boolean includeBinary on job_get_status. Only true SHALL embed the legacy PNG image or WAV audio content alongside metadata and its resource link; MP4 SHALL remain linked and be retrieved by explicit resources/read. The canonical artifact-delivery content contract SHALL be major version 2 and discovery SHALL advertise artifact_resources_v2; existing clients requiring legacy PNG/WAV blocks MUST migrate to includeBinary=true. Existing request envelopes and structured job fields SHALL remain valid. Headless, provider, error and persisted project contracts SHALL remain unchanged.

#### Scenario: Explicitly request legacy preview content
- **WHEN** includeBinary=true is supplied for a completed PNG or WAV job
- **THEN** the matching base64 image or audio block is returned with identical file bytes and metadata

#### Scenario: Validate opt-in and discover the changed content policy
- **WHEN** a client discovers tools and editor status or submits an invalid non-boolean includeBinary
- **THEN** the canonical schema and capability expose the version-2 policy and malformed input is rejected before artifact access

### Requirement: Owned local resource retrieval
Artifact URIs SHALL use only the opaque job identifier in opencut://jobs/{jobId}/artifact; resource readers MUST resolve that identifier through the same server's process-local registry and use only completed supported job outputs. Resource variables MUST be a canonical lowercase UUID and MUST match the canonical URL without query, fragment, encoded UUID bytes or path suffix. The bridge SHALL use the MCP SDK WHATWG URL normalization: equivalent scheme/host casing and plain or percent-encoded dot segments resolving to the same canonical owned job URI SHALL be accepted. Normalization MUST NOT bypass registry ownership, retention or file confinement. Neither requested paths nor speech tokens SHALL be accepted as resource addresses. Frame/range/draft files MUST be lexically restricted to previews beneath the corresponding project root; exports MUST be relative beneath the configured exports root. Readers MUST reject traversal, absolute paths, Windows drive/UNC forms, network forms, a symlinked configured root or descendants, directories and unsupported MIME types. Readers SHALL anchor confinement at the configured root's canonical real path and permit trusted parent aliases above that root; the opened target MUST match that canonical root plus the validated relative path. Speech retrieval MUST resolve its retained preview token through the speech service, preserve its expiry/discard policy and confine the returned path beneath one of config.generatedMediaDirectories, including ttsWorkDirectory. Existing transport authentication, origin/host policy and session registry ownership MUST apply unchanged, including resources/read through the same MCP HTTP middleware/session used by tools. Artifact reads MUST NOT mutate project revision/history or switch the active project.

#### Scenario: Read an owned artifact
- **WHEN** an authenticated client explicitly reads the canonical resource URI or an SDK-normalized equivalent for a completed job retained by its server
- **THEN** resources/read returns one blob with the output MIME type and identical PNG/WAV/MP4 bytes

#### Scenario: Reject path, URI and ownership attacks
- **WHEN** a resource names a noncanonical/foreign identifier or a registered artifact attempts path traversal, symlink escape, network/absolute paths or an unsupported output
- **THEN** the reader rejects without disclosing or reading unrelated file bytes and without changing project state

### Requirement: Retention and safe failures
Descriptors SHALL expire at job retention, bounded by speech-preview expiration for speech; metadata polling SHALL not extend retention or imply current file availability. Expired, evicted, restarted, foreign or unknown jobs MUST retain JOB_NOT_FOUND; consumed, discarded or expired speech tokens MUST retain GENERATED_ARTIFACT_NOT_FOUND. Missing/unreadable render files, invalid paths and unsupported output access MUST use existing safe non-retryable VALIDATION_FAILED failures. MCP resource failures MUST include the safe stable code while omitting file paths, filesystem messages, authentication tokens and speech preview tokens. The existing speechPreview.token and generatedArtifact.token fields SHALL remain intentional process-local commit handles, and SHALL never be copied into resource links or errors.

#### Scenario: Read expired or missing output
- **WHEN** the job expires, speech is discarded/consumed/expires, or an output file is missing
- **THEN** the documented stable failure is returned and metadata polling of a still-retained job continues to work

#### Scenario: Preserve project and failure semantics
- **WHEN** binary retrieval fails or metadata is repeatedly polled after unrelated edits or revision conflicts
- **THEN** no synthesis/render or project mutation occurs, existing conflict metadata stays authoritative, and no sensitive paths/tokens are emitted in errors

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

### Requirement: Bounded disposable bridge preview artifacts
The process-local bridge MUST retain at most32 completed PNG/MP4 preview artifacts and67108864 total declared encoded bytes by default, accepting both inclusive bounds and evicting oldest settled preview jobs before admitting a new output. An individually oversized preview MUST be safely discarded and fail with existing retryable JOB_REGISTRY_FULL. Failed disposal MUST remain charged and cannot silently allow capacity growth. Expiration, eviction, cancelled late completion and graceful close MUST delete only the originating process's confined renderer-generated preview files; exports, speech/analysis policies, project/media/history, foreign files and predecessor-process artifacts SHALL remain unchanged. Missing outputs SHALL be treated as already disposed; unsafe paths/symlinks and I/O failures MUST fail safely without unrelated deletion or sensitive diagnostics. Expired/evicted resource access MUST retain JOB_NOT_FOUND, metadata polling MUST not extend TTL and registry restart MUST remain cold.

#### Scenario: L4 Enforce exact artifact budgets
- **WHEN** successful preview completions meet/exceed count or byte capacity or a single preview exceeds byte capacity
- **THEN** exact bounds succeed, oldest settled outputs are disposed before replacement and oversized outputs fail safely without exceeding retained accounting

#### Scenario: L5 Dispose only owned expired outputs
- **WHEN** preview retention expires, eviction or close occurs, or a cancelled producer returns an output
- **THEN** its safe disposable file is removed, unavailable jobs retain JOB_NOT_FOUND and unrelated files/exports/project state remain intact

#### Scenario: L6 Fail closed during unsafe disposal
- **WHEN** output deletion encounters traversal, a symlink, a foreign filename/project or an I/O failure
- **THEN** unrelated files remain untouched, diagnostics stay safe and failed retained disposal cannot free charged capacity
