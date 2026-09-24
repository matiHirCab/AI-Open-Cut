## Context

The [PR #124 Render parity job](https://github.com/matiHirCab/AI-Open-Cut/actions/runs/35934928399/job/107429714234) succeeded in 345 minutes. GitHub's completed log shows 20,448 seconds in `native_golden_render_conformance`, about 50 seconds of initial Rust test compilation, and roughly two minutes for the other native and cache tests. [Pre-PR](https://github.com/matiHirCab/AI-Open-Cut/actions/runs/35883174277) [commits](https://github.com/matiHirCab/AI-Open-Cut/actions/runs/35851585157) also spent 3–5.5 hours in the same native step. The protected command is `cargo test -p opencut-editor-core renderer::golden::native_golden_render_conformance -- --exact`. The golden test invokes seven conformance suites and then one warmup plus three measured captures for the report. It emits no per-suite timings. Its audio comparison scans every offset in a one-frame window over decoded samples, repeatedly, in Rust's unoptimized test profile; this is a plausible CPU cost, but the log does not isolate it from raster and FFmpeg work.

The living render-regression and repository-validation specs require immutable reviewed references, exact protected commands/environments, report validation before upload, and unchanged tolerances. The optimization must preserve these guarantees.

## Goals / Non-Goals

**Goals:** Substantially shorten the required native golden test, attribute time to its seven suites and sampled capture, and keep all existing success/failure evidence. Compare the completed optimized CI run with the 345-minute baseline and record both build and test durations. A successful implementation must show a material reduction on the same Linux runner class; no fixed duration becomes a gate.

**Non-Goals:** Remove golden cases, reduce canvas sizes or sampled states, change fixtures or tolerances, alter production rendering or public contracts, update golden references, move the suite out of required PR CI, or add a general performance budget.

## Decisions

1. **Run only the monolithic golden test with `cargo test --release`.** Retain its exact test filter and the remaining native commands in their existing profile and order. Optimized Rust code should accelerate the CPU-heavy test harness without changing what is asserted. This also avoids modifying the alignment algorithm or golden semantics before per-suite timing is available. Alternatives: drop expensive cases or make the suite optional; rejected because that weakens reviewed evidence. Rewrite audio alignment first; deferred because its share of elapsed time is not yet measured and a new algorithm risks numerical behavior.
2. **Time each existing suite and the sampled capture inside the test harness.** Use monotonic `Instant` spans and print named durations to the Rust test output. Ensure CI uses output capture settings that expose these lines on successful runs without changing assertion behavior. Keep elapsed observations out of fixtures and pass/fail decisions. Alternative: rely only on GitHub step timing; rejected because one step contains the entire monolithic test.
3. **Pin the command in the exact CI policy.** Update the reviewed workflow expectation and negative regression fixtures together. Keep the existing native step, environment, report path, subsequent raster-cache test, strict report validator, and upload. A failed optimized test must stop the step and prevent report publication.

## Risks / Trade-offs

- [Release compilation adds time or changes deterministic output] → Record build and test time separately; the existing golden assertions and protected gate reject output drift. If it fails or the end-to-end job is not materially faster, retain the prior command and revise the design before completion.
- [Timing output is hidden by Rust test capture] → Use a reviewed invocation that exposes successful test output, and verify each suite name appears in the completed CI log.
- [Runner load varies] → Compare with completed historical runs on the same Linux runner class, report raw durations and uncertainty, and do not turn one elapsed value into a universal limit.

## Compatibility, failure, and rollback

No public or persisted contract, schema, migration, dependency, secret, network access, or golden fixture changes. On a failure, the native step remains non-optional and the report upload remains gated. Rollback restores the prior golden command and exact policy expectation together; the reference set remains untouched.

## Verification Plan

Run Rust formatting, focused golden and policy tests, strict Clippy and affected workspace tests, and the repository's protected OpenSpec checks. Run Render parity on the reviewed Linux runner with required FFmpeg, FFprobe, and font settings. Confirm the same golden assertions and report validation pass, inspect named suite durations, and compare total job/test time with the 345-minute run before marking the performance task complete. Complete OpenSpec conformance verification, synchronization, archival, and the final protected gate in repository order.
