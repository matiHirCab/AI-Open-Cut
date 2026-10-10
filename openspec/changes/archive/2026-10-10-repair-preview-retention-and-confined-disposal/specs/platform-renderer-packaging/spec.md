## ADDED Requirements

### Requirement: Native confined preview cleanup in the existing runtime
The four-role default runtime package SHALL execute the same native confined preview cleanup ownership used by source clients on Linux, macOS and Windows. Cleanup input/output MUST be bounded and private to the application adapter; it MUST NOT add a public editor/MCP operation, arbitrary-path deletion surface or new runtime role. Required tests MUST exercise genuine native directory/file replacement or native sharing refusal, missing outputs, static links/reparse points and retryable failed cleanup. Normal public protocol discovery, packaged integrity, rendering readiness/fallback/media, revisions and authoritative files MUST remain unchanged.

#### Scenario: P8 Execute source and default packaged confined cleanup
- **WHEN** actual source and assembled default-package clients dispose owned preview files and exercise ancestor replacement on each supported platform
- **THEN** native ownership confinement preserves outside files, safe retry/close behavior is observed and the unchanged package inventory and public protocol remain valid

#### Scenario: P9 Fail closed on missing native cleanup proof
- **WHEN** native cleanup prerequisites, private adapter shape or actual required execution are absent, malformed, skipped or failed
- **THEN** verification refuses acceptance without substituting a mock/static path check or weakening the existing three-platform gate
