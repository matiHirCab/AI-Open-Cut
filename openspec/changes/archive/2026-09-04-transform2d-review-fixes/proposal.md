## Why
Review of issue #18 found clipped display-rotated media, measured geometry validation after export collision inspection, host-dependent fonts, and native tests ignoring explicit tool configuration.

## What Changes
Implement the user's approved four-fix plan: private read-only oriented-media geometry probing; shared preflight before destination inspection or allocation; redistributable font fixtures; explicitly configured native tests required in CI. Preserve schema 8 and all public contracts and legacy render behavior.

## Capabilities
### New Capabilities
None.
### Modified Capabilities
- motion-graphics-architecture: finalize media dimensions after safe metadata inspection.
- rendering-export: finalize geometry before destination side effects.
- repository-validation: portable correctness tests and required native Transform2D coverage.

## Impact
Core evaluation, renderer orchestration/process port, tests, native CI policy, and documentation. No persistence, public DTO, provider, or module dependency changes.

## Approval
The user explicitly requested PLEASE IMPLEMENT THIS PLAN on 2026-09-04 and supplied the full four-fix plan. These artifacts transcribe that approved scope, including permission for bounded read-only FFprobe metadata inspection before rasterization and changes to the protected native test command.
