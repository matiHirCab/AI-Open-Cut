## MODIFIED Requirements

### Requirement: Bounded and justified CI duration exception

An exception to the default CI budget MUST name an accountable owner, cause, measured baseline, evidence link, expiration date, and a positive hard cap no greater than 180 minutes. The protected foundation MUST display the reason and effective cap when the exception applies. An exception MUST be inactive after its expiration date and MUST NOT bypass required test selection, assertions, or failure propagation. The reviewed workflow policy MUST reject missing, duplicated, malformed, unbounded, or unreviewed duration controls. Only the rules-screen parity matrix job MAY use the 180-minute job limit; other required leaf jobs MUST retain their reviewed 135-minute limits and the foundation job MUST retain its 10-minute limit.

#### Scenario: Reviewed exception covers a measured overrun

- **WHEN** a complete, unexpired exception applies and successful required validation finishes above 120 minutes but no later than its hard cap
- **THEN** the protected foundation reports the baseline, evidence, reason, owner, expiration, and measured duration and can succeed

#### Scenario: Exception hard cap is exceeded

- **WHEN** the measured elapsed time exceeds the exception hard cap
- **THEN** the protected foundation fails and reports the hard-cap overrun

#### Scenario: Exception is invalid or expired

- **WHEN** an exception is incomplete, duplicated, malformed, has a cap above 180 minutes, or is past its expiration date
- **THEN** workflow policy validation or the foundation rejects it; the exception cannot authorize an overrun

#### Scenario: Duration control is weakened

- **WHEN** the duration assertion is removed, skipped, masked, detached from the protected foundation, or any required prerequisite is dropped
- **THEN** the repository's CI policy validator rejects the workflow before it can pass the protected gate

#### Scenario: Keep other required jobs bounded

- **WHEN** the rules-screen matrix job is allowed 180 minutes and any other required leaf or foundation job exceeds its reviewed job timeout
- **THEN** repository policy validation rejects the workflow
