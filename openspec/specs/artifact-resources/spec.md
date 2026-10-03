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
Artifact URIs SHALL use only the opaque job identifier in opencut://jobs/{jobId}/artifact; resource readers MUST resolve that identifier through the same server's process-local registry and use only completed supported job outputs. Resource variables MUST be a canonical lowercase UUID and MUST match the canonical URL without query, fragment, encoded UUID bytes or path suffix. The bridge SHALL use the MCP SDK WHATWG URL normalization: equivalent scheme/host casing and plain or percent-encoded dot segments resolving to the same canonical owned job URI SHALL be accepted. Normalization MUST NOT bypass registry ownership, retention or file confinement. Neither requested paths nor speech tokens SHALL be accepted as resource addresses. Frame/range/draft files MUST be lexically restricted to previews beneath the corresponding project root; exports MUST be relative beneath the configured exports root. Readers MUST reject traversal, absolute paths, Windows drive/UNC forms, network forms, symlinked root/ancestor/file escapes, directories and unsupported MIME types. Speech retrieval MUST resolve its retained preview token through the speech service, preserve its expiry/discard policy and confine the returned path beneath one of config.generatedMediaDirectories, including ttsWorkDirectory. Existing transport authentication, origin/host policy and session registry ownership MUST apply unchanged, including resources/read through the same MCP HTTP middleware/session used by tools. Artifact reads MUST NOT mutate project revision/history or switch the active project.

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
