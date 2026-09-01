## Why

ADR 0004 locks the motion-graphics renderer boundary but does not define what clients observe when the preferred graphics backend is unavailable or cannot execute a complete evaluated scene. Issue #10 explicitly requires fallback semantics, so the architecture must distinguish conforming backend failover from prohibited silent degradation before renderer implementation begins.

## What Changes

- Define deterministic failover to another locally available graphics backend only when it supports the complete `EvaluatedScene` and preserves the same semantics and documented output tolerance.
- Require preview and export to use the same backend-selection policy.
- Prohibit silent omission, approximation, downgrade, network acquisition, and partial or degraded artifact publication.
- Require `DEPENDENCY_UNAVAILABLE` before rasterization, FFmpeg execution, or artifact publication when no conforming backend is ready.
- Extend ADR 0004, the living motion-graphics architecture requirement, and deterministic ADR coverage with the fallback policy.

This is a documentation-only correction. It does not implement backend selection, change runtime behavior, introduce a new error code, or alter any public or persisted contract.

### Non-goals

- Select or implement a graphics backend, backend registry, capability-discovery API, or runtime priority mechanism.
- Add public fields, operations, capabilities, warnings, dependencies, migrations, or contract fixtures.
- Permit degraded rendering as a successful result.
- Modify the archived `record-motion-graphics-architecture` change.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `motion-graphics-architecture`: Add client-observable conforming-failover and fail-closed semantics to the hybrid renderer boundary.

## Impact

- Documentation: extend ADR 0004 with one normative fallback subsection.
- Living requirements: modify the existing hybrid renderer boundary requirement with conforming-failover and no-conforming-backend scenarios.
- Tests: strengthen the ADR documentation contract and add a focused omitted-fallback failure fixture.
- Compatibility: reuse `DEPENDENCY_UNAVAILABLE`; schema version 6, capability reporting, public catalogs, and versioned fixtures remain unchanged.
