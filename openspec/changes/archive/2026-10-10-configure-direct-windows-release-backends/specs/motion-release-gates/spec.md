## MODIFIED Requirements

### Requirement: Mandatory release orchestration preserves prior acceptance
A separate pinned release matrix MUST require Windows, macOS and Linux canonical cases and complete native execution, actual dependency/font provenance, strict reports and nonempty evidence artifacts at the published head. Missing/changed platforms/pins/commands, default/instrumented substitutions, skipped cases, golden mutation or failure masking SHALL be rejected by automated policy controls. Drivers SHALL terminate and settle only owned process trees on timeout/error. Existing protected workflows, assertions/deadlines, public/provider contracts, schema44/protocol1 and source fixtures MUST remain unchanged. Completion SHALL require local conformance/archive/protected gates plus terminal exact-head native and prior required CI, with failed/blocked checks disclosed.

#### Scenario: R9 Complete every platform at the published head
- **WHEN** the release matrix and prior required workflows execute the published implementation
- **THEN** genuine all-platform canonical/default/MCP proof, strict reports and actual artifacts are terminal successful before advancing #80

#### Scenario: R10 Reject weakened orchestration
- **WHEN** a platform/pin/native command/report/feature identity is changed, a failure is masked or golden mutation is enabled
- **THEN** policy/required-execution tests reject the change while prior protected gates remain enforced

#### Scenario: R11 Bound owned driver failure cleanup
- **WHEN** a driver-owned process and actual descendant hang or fail
- **THEN** the existing finite driver control terminates and settles that owned tree, preserves diagnostics and returns failure without affecting unrelated work
