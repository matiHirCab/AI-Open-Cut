## Context

The stage benchmark is test-private and layered on deterministic golden conformance. Review showed that a boolean named for memory sampling controls only the sampler while benchmark probes always run, and that CI duplicates only a subset of the canonical report checks. Update reconciliation also needs to distinguish a legacy schema-2 migration source from a current schema-3 generation.

## Goals / Non-Goals

**Goals:** preserve ordinary conformance behavior, make benchmark capture explicit, validate the exact uploaded artifact with one canonical validator, fail closed for malformed current generations, and cover every measured process failure.

**Non-Goals:** change stage meanings, performance values, fixture references, renderer production APIs, public telemetry, performance budgets, or project persistence.

## Decisions

### Represent capture intent explicitly

Replace the ambiguous sampling boolean with a test-private capture mode. Conformance mode does not construct observations or invoke benchmark processes. Benchmark mode always gathers the three intent observations and independently selects whether process-tree memory is sampled. Warm-up uses benchmark mode without sampling; each measured capture uses benchmark mode with sampling.

Alternative considered: infer benchmark execution from the memory flag. Rejected because warm-up deliberately runs benchmark work without memory sampling.

### Validate generated and uploaded reports through editor-core

Immediately validate the aggregated typed report before returning it. Add a test-only file validator that reads the report from an explicit path, strictly deserializes it with unknown-field rejection, and applies the existing semantic validator. Required Linux CI invokes this validator against the exact report produced by conformance before upload, replacing the partial Bun field check.

Alternative considered: expand the JavaScript checks. Rejected because that would duplicate renderer-owned schema semantics outside editor-core.

### Separate current validation from legacy migration

Centralize generation classification: current revision 3 uses full manifest/report validation; only revision 2 with performance schema 2 may use migration validation. Other revisions and malformed current reports fail. Cleanup recognizes an inactive generation using the same classification, avoiding broader deletion authority.

### Inject benchmark probe execution by named stage

Use a test-private executor interface carrying `Decode` or `Composite` stage identity. Native execution delegates to the existing bounded process runner. Tests use a recording/failing executor together with the existing production process executor to verify ordering, early termination, cleanup, and state preservation without invoking external tools.

## Failure and Compatibility

All benchmark process failures reject the observation before publication. RAII continues to remove render-plan workspaces; encoded temporary output is removed on success and failure. Current fixtures fail before capture if strict validation fails. No compatibility surface changes, and no persisted migration is introduced.

## Verification

Add focused tests for capture isolation, sample counts, strict report read-back, revision classification, and each process-failure stage. Then run the repository-required Rust, native Linux, bridge, Python, contract, smoke, and OpenSpec checks.
