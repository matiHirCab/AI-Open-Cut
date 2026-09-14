## ADDED Requirements

### Requirement: Explicit native font regression configuration
Native font rendering regressions MUST execute when native tools are configured and MUST fail on missing required configuration, partial configuration or unusable configured dependencies. Only wholly unconfigured optional execution SHALL omit native work. Non-native font lifecycle tests MUST continue running independently of native tools. Required native execution MUST preserve preview, reopen, source-removal, draft, range and export assertions without changing production behavior or persisted formats.

#### Scenario: Unconfigured ordinary test job
- **WHEN** all three native configuration paths are absent and required mode is disabled
- **THEN** native font tests omit native work without launching ambient FFmpeg and non-native font tests execute

#### Scenario: Required or partial configuration
- **WHEN** required mode lacks configuration or only some native paths are configured
- **THEN** native font tests fail with a configuration diagnostic instead of skipping

#### Scenario: Unusable configured dependencies
- **WHEN** complete configuration names an unavailable tool or unreadable font
- **THEN** native font tests fail instead of skipping

#### Scenario: Configured native conformance
- **WHEN** complete usable native configuration is supplied in required mode
- **THEN** both native font regressions execute all existing assertions including mandatory separator parity and pinned fonts after source removal

### Requirement: Mandatory CI native font coverage
The required render-parity job MUST execute the font-resolution integration suite with required native configuration. Exact-command policy validation MUST reject removal, substitution or error masking of this suite while preserving all existing required commands and settings.

#### Scenario: Native font coverage cannot be bypassed
- **WHEN** the font-resolution native command is removed, substituted or given an error-masking suffix
- **THEN** policy validation rejects the workflow
