## Context

Core owns edits and samplers. Typed channels require keys strictly inside item duration; legacy keys allow terminal duration keys and currently split through endpoint interpolation. Arbitrary eased segments cannot be preserved by substituting endpoints. Repeat and ping-pong need the original source phase and finite cycle budget. Original schema is 29; independent issue 46 reserves 30 for tagged preset provenance.

## Goals / Non-Goals

Goals: retain exact source functions across trim/split/duplicate, including fractional inherited clocks, typed compound values, legacy easing and independent audio gain; atomic history and contract parity. Non-goals are those in proposal; unsupported split types retain current errors.

## Decisions

1. Add optional `clock: {offsetMs, sourceDurationMs}` to each AnimationChannel and `legacyAnimationClock` to common visual properties. Absence means offset zero with current item duration. Source offsets and durations are bounded JavaScript-safe integers, signed for offsets; sourceDurationMs is positive. Mapped item window endpoints must remain signed-safe. Keys validate against retained source duration, typed strictly below it, legacy at or below it. Retained keys outside the edited window are deliberate and bounded by existing counts. Clocks without nonempty source animation are rejected. No null clock value is accepted. Applying an offset before loop mapping preserves nonzero first keys, turns, seams and exhausted phases without iteration loops. Negative mapped time holds the first source key. Integer mapping uses checked signed integer arithmetic before interpolation; fractional mapping retains f64 interiors without rounding. No new architecture edges.

2. Split retains source keys/curves/loops on both sides, sets left clock to its existing offset and right clock to existing offset plus left duration, and retains original source duration. Legacy animated supported types follow the same approach, initializing legacy sourceDurationMs to max(original duration, latest source-key time) because existing legacy requests permit keys beyond the current item window. Absent legacy clocks retain that compatibility; source bounds apply only to explicitly retained records. Empty animation records need no clocks. Split rejects group/component (INVALID_ARGUMENT), transition (VALIDATION_FAILED) and boundary cuts (VALIDATION_FAILED) unchanged. Trim composes signed delta newStart-oldStart with each clock, retaining source duration and keys; a same-start right trim leaves source phase unchanged. Extensions sample the original source's held pre-start/post-end values or continuing loop, subject to original finite budget. Move changes placement only. Duplicate clones clocks, arrays, targets, curves, loops and provenance; independent subsequent replacement channels/legacy keyframes clear their respective clocks unless explicitly provided valid typed channel clocks. Preset replacement generates clocks absent only for newly generated/replaced properties; untouched properties retain their exact channel clocks, and legacyAnimationClock remains unchanged; edit preservation keeps provenance as original compilation attribution, not a guarantee of an unsliced current preset window.

3. Marker start expression lifecycle stays unchanged: split clears both; trim clears only when start changes; duplicate shifts offset with placement. Animation trim delta is independent of sourceInMs; media sourceInMs retains its existing omitted/explicit semantics without automatic retiming. Group trim retains its existing child timing behavior; only its own animation channels shift. Component-instance trim remains unsupported. Captions/repeaters/transitions lack supported typed channels and retain existing edit behavior.

4. Introduce schema31 independently, accept migrations1–29 and current31, reject reserved30 until issue46 integration. Existing atomic persistence migration covers all root/component items and undo/redo snapshots under lock, without changing keys/provenance. Reject any clock in a pre31 document to avoid interpreting future data as old semantics. Integration requires carrying issue46's Pack tagged provenance model and constraints forward, changing its supported-version ranges to31 and introducing30→31 migration with both pack records and retained history tests. Neither branch's unrelated code is copied here; later integration cannot relabel incompatible schema30 documents as31 without validating their provenance.

5. Add clocks to governed channel and common visual response/request schemas and canonical catalogs. Existing edit names, aliases, errors and retryability remain. Capability current schema becomes31; minimum versions for existing capabilities remain unchanged. New optional clock payload is additive; persisted schema boundary prevents older readers silently ignoring clocks.

## Risks / Trade-offs

Retaining source keys uses the same bounded storage as the original animation and avoids cumulative resampling errors. Changes to samplers must include render artifact compilation/audio envelopes and visibility analysis, not only scene evaluation; parity tests exercise every intent. Signed overflow is rejected before state publication. Older readers cannot open schema31; preserve bundles and rely on older committed files for rollback. Schema30 adapter is an explicit integration obligation, tested independently when issue46 lands.

## Migration Plan

Add schema31 metadata/defaults and fail-closed pre31 clock checks; migrate all supported current/components/retained snapshots atomically via existing staged migration publication. Update canonical schema claims and test history reopen, including malformed/future/reserved30 rejection without disk mutation. No deployment or merge is authorized.

## Open Questions

None pending independent specification review. Human CODEOWNER review remains a draft PR review request; delegated specification approval is recorded separately.
