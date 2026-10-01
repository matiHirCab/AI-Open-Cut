# Approved: bounded pre-publication analysis

The original proposal was approved in this chat on 2026-09-30. Implementation exposed an omitted observable failure case in the requirement to validate **every reachable sample before commit**. The user explicitly approved this amendment in this chat on 2026-09-30 with “Aprove”. Implementation of its bounded conservative rejection rule is authorized.

## Concrete problem

Checking authored endpoints does not establish coupled crop safety. For example, a crop with static width 0.5 and X animated from 0 to 0.5 between 0 and 500 ms has valid endpoints. A spring with mass 1, stiffness 100, damping 1 and initialVelocity 0 overshoots near 150 ms, so sampled X plus width exceeds 1 although every scalar remains within its individual bound. Paths, effect supports and inherited transforms likewise need bounds over intermediate samples, not just endpoint checks.

Exhaustively checking every integer timestamp is also insufficient as a bounded implementation strategy: keyframe offsets can span very large intervals, loops can combine with inherited clocks, and the existing channel/keyframe-count limits do not bound analysis work. The approved design did not specify what to do when safety cannot be proved within finite analysis work.

## Proposed decision

- Use deterministic interval analysis of canonical curve/value bounds, including correlations where they can be established, to certify coupled geometry and expanded work before publication.
- Give a final candidate, including its expanded and retained content, a cumulative budget of **65,536 interval-analysis nodes**. A node is one bounded time interval evaluated for one visual occurrence; repeated subdivision consumes additional nodes. Process occurrences in canonical order and subdivide left before right.
- Return stable non-retryable `INVALID_ARGUMENT` if a bound proves unsafe **or safety remains unresolved when that budget is exhausted**. Preserve all project/history/draft/resource bytes and revision on either failure. This deliberately permits conservative rejection of newly supported work whose safety cannot be certified within the budget.
- Keep old accepted requests and projects without extended properties on their prior path. New channels, crop and effects remain additive; this budget is reported in the new canonical extended-visual contract. Schema remains 27.
- Render preflight still validates the requested sampled output before destination inspection/publication. Candidate certification and actual rendering consume the same canonical sampler and bounds; renderer or transport code must not invent alternative domain rules.

## Verification additions

Add automated scenarios for the spring crop above, provably safe correlated crop channels, curved path/effect envelopes under inherited transforms, exact analysis-budget exhaustion, deterministic order, failed-batch/draft rollback, and no changes to legacy behavior. Record accepted/rejected canonical cross-language fixtures and scenario evidence before marking these tasks complete.

## Implementation status

The approved certification and bounded lossless streaming preparation are implemented. Required local suites, native extended channel/draft/audio and legacy render conformance, effect/budget tests, CODEOWNER review, implementation verification, synchronization and archival are complete. Both final gates pass. See tasks.md and verification.md for the complete scenario mapping, full log locations and local-versus-remote evidence limits.
