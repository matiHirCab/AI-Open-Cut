## Context

Main includes #165 and #166. Cancellation settles an inference promise before the Python child exits, so FIFO can reuse a dying worker and its exit callback can reject a valid successor. Native narration coverage exists but configured CI invokes only predecessor oracles. Complete MCP smoke workers OOM on this host; the responsible allocator is not yet established.

## Goals / Non-goals

Recover queued and immediate alignment/transcription successors after cancellation/timeout, enforce existing native evidence, and complete unweakened source/package suites after profiling. Preserve contracts, thresholds, cache semantics and graceful direct provider close. Do not implement roadmap features or increase limits to hide exhaustion.

## Decisions

1. Give each spawned worker its own output buffer and pending-request ownership. Retirement detaches it synchronously, signals termination and exposes a close promise; FIFO and new control requests wait for retirement. Late output/error/exit from an old generation cannot affect a new worker. Use bounded TERM-to-KILL escalation for a stubborn worker. Merely clearing `#child` before kill would permit overlapping inference; merely awaiting exit without generation isolation leaves concurrent control requests vulnerable.
2. Preserve direct close's graceful active drain, reject new work once closed, and reap its worker. Actual bridge shutdown continues aborting jobs first; cover both paths separately.
3. Add the existing exact native narration command to workflow and validator, plus omission/masking negatives. No oracle, tolerance, golden or fixture is regenerated.
4. Profile SDK validator compilation and invocation independently from bridge/native execution, recording schema size, heap/RSS and phase. Prefer a bounded test-client validator correction preserving full JSON Schema validation if that is the demonstrated cause; do not modify production models based on an OOM symptom alone. Use the SDK-bundled schema interpreter in the conformance helper, with shortcircuit false and exact-schema memoization, after the profiling evidence below. It preserves published schemas, format checks and all errors without generated-code allocation; no production provider or dependency changes. The SDK documents a draft-07 dependencies/reference gap, outside the published 2020-12 catalog; add explicit dialect and nested/format/union negatives rather than assuming equivalence for arbitrary future schemas.

## Risks / Trade-offs

- Process close semantics differ across platforms → real subprocess fixtures, termination escalation, and required supported-platform CI; Linux evidence is explicitly scoped.
- Queue recovery increases cancellation cleanup latency → wait only for terminated worker ownership, preserving typed rejection and concurrency one.
- Schema compilation options can change behavior → retain defaults except demonstrated allocation settings; prove valid/invalid output, formats, unions and nested errors; preserve full existing suites.
- Host PID1 does not reap orphans → use an uncommitted child-reaper harness for independent verification without altering assertions.

## Compatibility, security and rollback

No public schemas/catalogs, persisted fields, migrations, dependencies, error retryability or paths change. No media/token text enters diagnostic logs. Revert the repair commit to roll back; no data migration is needed.

## Verification

First reproduce regressions against unchanged main, then run queued/immediate cancellation and timeout recovery, graceful close, actual signal shutdown, validator negatives, full CI policy tests, strict specs, typecheck/lint/unit, complete integration/package, hermetic Python, workspace fmt/Clippy/tests and configured native narration. Record every failure; archive only after required implementation evidence passes, then rerun protected spec gate.

## Profiling decision evidence

The catalog-expanded largest draft output is 762,279 bytes. Compiling it with the default SDK AJV provider, without bridge/headless/Vitest, was OS-killed (137). Disabling only its optimizer compiled in 2,047.95 ms (RSS506,220,544), but full-suite invocation still grew client RSS to6.9GB with only323MB used JS heap before worker loss. Thus changing only compiler optimization is insufficient; do not ship that attempted mitigation. The SDK interpreter avoids code generation while retaining complete schema validation. Full source/package runs and invalid-output regressions must pass before claiming this resolves the complete acceptance gap. Profiling logs stay outside the repository.
