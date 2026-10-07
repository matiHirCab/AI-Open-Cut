## ADDED Requirements

### Requirement: Retain known-text alignment in the existing caption lifecycle
Optional knownText on transcription estimate/preview SHALL select known-text mode while omission preserves ordinary transcription. Estimate MUST not infer or publish. Successful preview SHALL retain an owned structurally parsed optional alignment and transcript snapshot, validate it in core against the captured source revision before publishing the expiring opaque token, and expose no provider-only path. Existing caption commit SHALL remain one atomic core revision with source provenance; conflict retry MUST reuse the retained result without inference. Discard, expiry and close MUST remove retention without project mutation. Alignment itself SHALL remain ephemeral, with no new persisted schema or batch timeline operation.

#### Scenario: Estimate and preview known text
- **WHEN** a client estimates then previews valid known text for an allowed asset
- **THEN** estimate performs no inference, preview exposes forced alignment and caption segments, and the project remains unchanged until existing commit

#### Scenario: Retain an owned snapshot
- **WHEN** a provider mutates its returned alignment/segments after preview completion
- **THEN** retained and subsequently committed metadata/timing retain the observed original snapshot

#### Scenario: Retry a conflicted known-text commit
- **WHEN** a retained known-text caption commit conflicts and is retried at the current revision
- **THEN** existing atomic commit uses the same preview without repeated alignment and consumes it only on success

#### Scenario: Preserve source lifecycle after commit
- **WHEN** aligned captions are committed, undone/redone and reopened
- **THEN** existing caption/source provenance remains exact, resolvable and durable while ephemeral alignment introduces no new project fields

#### Scenario: Reject changed or malformed inference
- **WHEN** provider data is malformed or the source revision/reference changes during alignment
- **THEN** existing TRANSCRIPTION_INVALID_OUTPUT or core reference/revision failure applies without publishing a token or project bytes

#### Scenario: Discard or expire known-text preview
- **WHEN** an aligned preview is discarded, expires or the service closes
- **THEN** retention is removed and existing TRANSCRIPTION_PREVIEW_NOT_FOUND applies without project mutation
