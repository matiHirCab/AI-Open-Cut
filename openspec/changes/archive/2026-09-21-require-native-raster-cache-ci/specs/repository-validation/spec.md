## ADDED Requirements

### Requirement: Protected mandatory raster-cache CI evidence
Repository policy MUST enforce the reviewed render-parity sequence including locked bridge dependency installation, configured native core raster-cache conformance, instrumented native worker and bridge reuse checks, and default headless restoration before default transport verification. The policy MUST validate exact commands, feature selection, working directories and step-local required-mode environment. Existing isolated execution, immutable goldens, failure propagation, report validation/upload ordering and foundation aggregation MUST remain enforced.

#### Scenario: P1 Accept complete cache verification
- **WHEN** the reviewed workflow installs locked bridge dependencies, executes all three configured cache suites, restores the default build and verifies default transport before validating and uploading the existing report
- **THEN** policy validation accepts the expanded sequence and preserves every existing required gate

#### Scenario: P2 Reject removed or disabled native evidence
- **WHEN** any new native command, feature selection, required-mode flag, dependency setup or declared working directory is removed, substituted or weakened
- **THEN** policy validation fails before the altered workflow can be accepted

#### Scenario: P3 Reject instrumented compatibility checks or masked failures
- **WHEN** default restoration is removed or reordered after compatibility checks, or a new critical step is conditional, ignores errors or changes its approved command/environment
- **THEN** policy validation rejects the workflow without relaxing the existing protected sequence or golden/report guarantees
