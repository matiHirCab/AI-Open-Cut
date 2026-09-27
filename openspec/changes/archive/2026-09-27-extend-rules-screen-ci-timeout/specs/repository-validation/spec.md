## ADDED Requirements

### Requirement: Bounded rules-screen CI execution

The required rules-screen parity matrix job MUST give each resolution shard an explicit 180-minute job timeout. Repository policy validation MUST accept that exact value and MUST reject a missing, shorter, longer, or nonnumeric value while preserving the required shard and aggregate result checks.

#### Scenario: Accept the approved render budget

- **WHEN** the three required rules-screen resolution shards each inherit the matrix job's 180-minute timeout and all other protected workflow properties match the reviewed policy
- **THEN** structural CI policy validation accepts the workflow and the shards retain their required test execution and failure propagation

#### Scenario: Reject an altered render budget

- **WHEN** the rules-screen matrix job omits its timeout or declares a value other than numeric 180 minutes
- **THEN** structural CI policy validation rejects the workflow before the protected policy task can attest success
