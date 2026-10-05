## Context

The Contract parity job is hermetic and does not install/configure media tools. Its protocol-v1 test currently asserts rendering.ready=true unconditionally, conflicting with the existing legitimate unavailable-renderer status. Production capability reporting and the canonical catalog already have the intended semantics.

## Goals / Non-Goals

Always prove exact readiness-gated protocol capability reporting, in ordinary hermetic tests and explicitly required native runs. Preserve the explicit missing-renderer regression and every public declaration. No runtime, catalog, workflow, schema or golden change.

## Decisions

Read rendering.ready using as_bool().expect rather than coercion. For both default and explicit protocol1 status requests, ready implies the exact canonical rendering list and null error; unavailable implies an empty list and nonretryable DEPENDENCY_UNAVAILABLE. Assert the exact canonical editor list in both cases. Build the expected top-level list from editor capabilities plus the rendering list only when ready, preserving order. Assert linear_light_compositing_v1 occurs exactly once in both rendering/top-level lists iff ready, and never in the editor list.

OPENCUT_GOLDEN_REQUIRED=1 additionally requires ready=true. The test always executes status/capability assertions and never returns early because tools are missing. Controlled unavailable FFmpeg/FFprobe tests run with native-required mode unset; an explicitly configured real native run sets it to1 and proves support. Existing explicit missing health test remains intact. Test renaming is unnecessary, preserving command selectors already used by required contracts.

## Alternatives Considered

Skipping without tools hides the unavailable branch and is rejected. Unconditionally requiring readiness in hermetic jobs incorrectly constrains the environment. Installing media tools or changing workflows is unnecessary; changing production capability reporting would fix the wrong layer. Native-only tests without hermetic exact-list evidence weaken conformance.

## Risks / Trade-offs

Conditional expectations could hide false readiness: exact lists/errors and the explicit native readiness assertion prevent that. Test-only changes leave every renderer input unchanged, so unaffected native golden/TS/Python evidence may be reused only as repository policy permits; affected Rust/contracts/focused native checks rerun.

## Migration Plan

No migration. Parent commits/pushes the reviewed correction to #49 after archive/policy success, then restacks paused #50 work. No branch or publication action by this implementation agent.

## Verification Plan

Run the protocol test with controlled nonexistent FFmpeg/FFprobe without OPENCUT_GOLDEN_REQUIRED, then configured real FFmpeg/FFprobe/reviewed font with OPENCUT_GOLDEN_REQUIRED=1; retain the explicit unavailable health test. Run complete contracts:check, fmt, strict workspace/all-target Clippy and complete unfiltered workspace via reaper. Record unchanged-input reused evidence separately from executed checks. Independent conformance review precedes sync/archive and unchanged Moon/pinned all-strict policy gates.
