## ADDED Requirements

### Requirement: Doctor readiness is independent from cleanup ownership
The doctor CLI SHALL release only its unchanged owning runtime guard after diagnostics return and all owned runtime shutdown cleanup succeeds, even if its report is unready because of unavailable subsystems, optional models or diagnostic prerequisites. It SHALL preserve existing unsuccessful diagnostic reports and exit codes. A thrown doctor cleanup failure, failed service shutdown, unconfirmed worker termination, process crash or unsafe/foreign ownership SHALL retain the guard without admitting new producers or deleting foreign artifacts. This correction SHALL preserve all existing job envelopes, capabilities, stable errors, provider contracts, project revisions and crash-reconciliation semantics.

#### Scenario: Complete unsuccessful diagnostics with clean shutdown
- **WHEN** doctor returns an unready report and every runtime service closes successfully
- **THEN** doctor exits unsuccessfully but removes only its own guard, and a fresh runtime using that root can admit an unrelated job

#### Scenario: Complete successful diagnostics with clean shutdown
- **WHEN** doctor returns a ready report and all runtime services close successfully
- **THEN** doctor exits successfully and releases only its own guard

#### Scenario: Preserve ownership after diagnostic cleanup failure
- **WHEN** doctor fixture or writeability-probe cleanup fails, including a later probe after an earlier ordinary write error
- **THEN** unsuccessful shutdown preserves its guard for operator reconciliation even if provider close succeeds

#### Scenario: Preserve ownership after service cleanup failure
- **WHEN** headless, speech or transcription close fails after diagnostics returned
- **THEN** doctor exits unsuccessfully and retains its guard, and a fresh runtime cannot invoke a producer

#### Scenario: Preserve a predecessor guard
- **WHEN** doctor starts in a root containing an existing foreign guard
- **THEN** admission fails with the existing safe retryable error and the foreign marker and private artifacts remain unchanged
