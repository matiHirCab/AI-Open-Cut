## Why

PR #119 CI run 34871014552 fails correctness on all three operating systems and contract parity because two native font tests launch ambient FFmpeg in jobs that do not provision it. The configured render-parity and packaged smoke jobs pass.

## What Changes

- Make native font regressions follow explicit optional/required native configuration, preserving fail-closed required coverage.
- Add configuration regressions and verify both unconfigured and configured execution.
- Run font_resolution in required render parity and extend its exact-command enforcement without weakening existing gates. This extension was explicitly approved by the user ("yes") on 2026-09-14 after discovering that the job did not already run the suite.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `font-resolution`: specify native font regression configuration and mandatory execution in required mode.

## Impact

The font integration-test harness, render-parity workflow command, exact-command validator and its tests, and specifications change. No public contracts, dependencies, migrations, project schema 19, draft version 2, or opencut-text-v2 rendering changes.

## Non-goals

Do not weaken protected CI policy, provision extra tools in ordinary jobs, weaken native assertions, or modify unrelated agent-context work.

## Approval

Explicitly approved by the user on 2026-09-14 ("Approve").
