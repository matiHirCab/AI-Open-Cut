## Why

The delivered motion-graphics audit reproduced an alignment cancellation/reuse race, found that required native CI omits the integrated narration oracle, and could not complete source/package MCP suites because their test workers exhausted the host. Repair those acceptance gaps without weakening contracts or rendering evidence.

## What Changes

- Retire terminated transcription/alignment workers before FIFO successor dispatch, preserving cancellation/timeout error codes and concurrency-one semantics.
- Preserve and document direct provider close as graceful draining; verify bridge shutdown cancels jobs before provider disposal.
- Require the existing integrated six-cue native narration test in configured CI and the CI policy validator, including an omission-negative policy test.
- Profile complete MCP source/package failures. Correct only a demonstrated allocation cause in test/SDK validation or production lifecycle, retaining full output validation and every existing assertion. Record evidence and remaining limits.

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `known-text-alignment`: explicit isolated recovery after worker cancellation/timeout and graceful direct-close semantics.
- `narration-driven-fixture`: explicitly enforce the existing integrated native oracle in configured required CI.
- `agent-bridge`: bounded reusable output-schema validation in complete source/package conformance evidence, conditional on profiling the demonstrated cause.

## Impact

Bridge provider orchestration, test fixtures and MCP client validation helpers; native CI invocation/policy; living requirements and conformance evidence. No new runtime dependencies, public tool/schema/error/provider protocol changes, persisted migrations, or changed render thresholds are intended.

## Non-goals

Do not implement open MG-M6 roadmap work, alter preview-cache semantics, change graceful close to abrupt disposal, skip tests, increase memory/deadline limits to hide failures, merge, or deploy. The repair starts at main `4b0485d2172cbc7614d7674994d80dcb405fd6f9`, which already includes #165 and #166.

## Authorization

The user's remediation instruction explicitly authorizes these issue-scoped repairs and supplies standing specification approval authority. This bounded proposal records that scope before implementation; discoveries beyond it require a separately authorized proposal.
