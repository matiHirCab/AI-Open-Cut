# Platform Renderer Packaging Specification

## Purpose

Define portable sealed runtime integrity and canonical renderer availability/fallback evidence through default native packages on Windows, macOS and Linux.

## Requirements

### Requirement: Portable sealed version1 runtime package
The existing package MUST retain version1 and exactly its four runtime roles with unchanged valid default layout. Manifest/entries SHALL be closed typed shapes with safe finite byte counts, SHA256 digests and unique portable canonical relative paths. Validation MUST reject unknown/future/malformed data, duplicate/case collisions, lexical/drive/UNC/backslash/dot escapes, reserved/control paths, missing or non-regular files, symlinks/junctions and undeclared files/directories. Manifest reads MUST be bounded16KiB and executable integrity hashing SHALL stream bounded chunks. Assembly source/name failures MUST be identified before replacing the owned destination. No project/public DTO or model contract SHALL change.

#### Scenario: P1 Assemble and verify a healthy portable package
- **WHEN** four valid runtime sources are assembled on a supported platform
- **THEN** the unchanged version1 manifest/layout verifies actual regular file byte counts/hashes, platform executable suffix/permissions and the exact inventory

#### Scenario: P2 Reject package corruption and escapes
- **WHEN** manifest/data/path/link/duplicate/case/inventory corruption or source destination collisions occur
- **THEN** verification fails closed before accepting the package and preflight source/name failures leave existing destination content intact

### Requirement: Canonical availability and explicit fallback evidence
Source and default assembled-runtime MCP checks MUST obtain rendering/editor/optional-audio availability from existing canonical typed status. Path diagnostics SHALL be distinguished from actual usable dependencies. Missing base filters or media tools MUST report unavailable rendering with stable canonical errors while preserving editor readiness/create/edit/history/reopen; partial optional audio inventories MUST preserve their existing capability/refusal semantics without dropping or approximating requested processing. Healthy native checks MUST render actual PNG and audiovisual range/export, preserve invalid/stale rollback and immutable revision/history, and retain matched SSIM>=0.99/floatPCM RMS<=0.0001/one-frame timing tolerances. No transport SHALL duplicate detection or canonical scene rules.

#### Scenario: P3 Discover healthy and partial feature availability
- **WHEN** actual usable media tools or a controlled incomplete filter inventory are configured through source/default packaged clients
- **THEN** typed status and optional capabilities agree with existing canonical checks, independent path diagnostics remain separate and requested unavailable processing fails with its existing typed error

#### Scenario: P4 Edit and restore with rendering unavailable
- **WHEN** base rendering dependencies are unusable and a client creates/edits/restores/reopens a project or requests render work
- **THEN** editing and history/reopen remain authoritative while rendering fails closed, preserving stable errors, revisions/files and existing artifacts

#### Scenario: P5 Prove native default-package media
- **WHEN** source and verified default assembled-runtime clients execute a bounded healthy audiovisual scene, invalid/stale input and history/fresh reopen
- **THEN** genuine streams/output comparison and independent nonempty media evidence meet retained tolerances and authoritative state/artifact identity remains correct

### Requirement: Mandatory three-platform native execution
A separate platform workflow SHALL execute Windows, macOS and Linux native package/readiness/media checks with repository-pinned toolchain, fixed font and actual FFmpeg/FFprobe, default shipped binary and visible version/output evidence. Required invocation MUST fail for missing dependencies, skipped cases, changed commands/platform inventory or failure-masked checks. Existing required CI workflows/gates/deadlines MUST remain unchanged and authoritative. Local Linux evidence MUST NOT be presented as Windows/macOS proof; issue acceptance requires terminal exact-head platform execution.

#### Scenario: P6 Execute every native platform
- **WHEN** the exact published head is exercised through the new matrix
- **THEN** all three default-runtime package/readiness/fallback/media scenarios run without native skips and report genuine results alongside existing required CI

#### Scenario: P7 Reject weakened platform orchestration
- **WHEN** a platform/toolchain/required command is omitted or modified, failure is masked, or private instrumentation replaces the default package binary
- **THEN** workflow policy tests fail and prior protected workflows/checks remain enforced

### Requirement: Native confined preview cleanup in the existing runtime
The four-role default runtime package SHALL execute the same native confined preview cleanup ownership used by source clients on Linux, macOS and Windows. Cleanup input/output MUST be bounded and private to the application adapter; it MUST NOT add a public editor/MCP operation, arbitrary-path deletion surface or new runtime role. Required tests MUST exercise genuine native directory/file replacement or native sharing refusal, missing outputs, static links/reparse points and retryable failed cleanup. Normal public protocol discovery, packaged integrity, rendering readiness/fallback/media, revisions and authoritative files MUST remain unchanged.

#### Scenario: P8 Execute source and default packaged confined cleanup
- **WHEN** actual source and assembled default-package clients dispose owned preview files and exercise ancestor replacement on each supported platform
- **THEN** native ownership confinement preserves outside files, safe retry/close behavior is observed and the unchanged package inventory and public protocol remain valid

#### Scenario: P9 Fail closed on missing native cleanup proof
- **WHEN** native cleanup prerequisites, private adapter shape or actual required execution are absent, malformed, skipped or failed
- **THEN** verification refuses acceptance without substituting a mock/static path check or weakening the existing three-platform gate
