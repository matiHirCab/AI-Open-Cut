## Why

Main2d748508 fails Windows renderer descendant testing twice before its cleanup assertion. The timeout includes no fixture startup or worker event evidence, so it cannot distinguish shell startup, PowerShell execution, typed renderer failure, or containment behavior. The delegated task now explicitly authorizes investigating and repairing this prerequisite independently of issue60.

## What Changes

- Improve owned test-fixture diagnostics with shell-entry evidence, PowerShell error output and captured worker events while preserving the10-second PID observation deadline,5-second cleanup assertion, exact-child handle and kill-on-close coverage.
- Add a focused Windows diagnostic workflow alongside unchanged required correctness/foundation workflows so startup evidence is available without waiting for the complete desktop workspace suite.
- Collect real Windows evidence before approving any cause-specific correction. A later correction requires an amended reviewed specification; this proposal does not guess the cause or change production execution.
- Compare the original and instrumented fixture bodies in the same native Windows diagnostic process and report only owned process descendants through read-only Windows snapshots. The first focused instrumented run passed in2.37s, which is not a proven correction or explanation of the two original full-suite failures.
- Keep the unchanged original batch body as the sole default descendant fixture. Enable the additional instrumented exercise only through an explicit focused-workflow diagnostic flag, so concurrent probes cannot warm each other during standard full-suite acceptance.
- Report bounded escaped owned PID-file contents on failure, separating missing publication from incomplete/control-byte records without changing PID acceptance.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `repository-validation`: attributable, bounded Windows renderer-startup evidence with unchanged required acceptance gates.

## Impact

The existing headless integration fixture, a portable diagnostic formatter with automated fixture tests, a focused Windows CI workflow and documentation/evidence. No public/persisted/provider contract or production process semantics changes are authorized in this diagnostic phase. The original issue60 work stays untouched in its own checkout.

## Non-goals

No skipped Windows test, blindly extended timeout, relaxed cleanup assertion, speculative production fix, protocol/schema change, unrelated changes, merge, or deployment. Linux zombie-process detection is a distinct concern and must not be used to explain this Windows failure.
