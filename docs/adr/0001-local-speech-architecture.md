# ADR 0001: Local speech architecture

- Status: Accepted
- Date: 2026-08-27

## Context

OpenCut needs private local speech generation without coupling project files or timeline behavior to one model. The implementation crosses Rust, TypeScript, and Python boundaries and must retain media provenance, survive contract evolution safely, and isolate provider and headless failures.

## Decision

### Provider boundary

The agent bridge exposes provider-neutral speech workflows. Kokoro is an adapter behind that abstraction and runs as a replaceable CPU-only worker. The worker knows how to describe voices, estimate and synthesize speech, and produce WAV files; it has no project or timeline knowledge. Adding another provider must implement the bridge provider contract rather than change editor-domain behavior.

### Persisted intent and provenance

Generated assets persist provider/model/voice provenance and provider-neutral speech intent. Intent includes the original text and deterministic normalization, pronunciation, chunking, pause, language, voice, and speed options needed for regeneration. The Rust domain owns atomic insertion or replacement and keeps logical asset and timeline identity rules independent of the provider.

### Cross-language contracts and errors

Canonical catalogs under `contracts` define shared stable errors and other cross-language expectations. Rust, TypeScript, and Python tests enforce parity. Public errors use stable codes, catalog-defined retryability, and safe descriptions; provider internals, private paths, text, and tokens are not exposed.

### Jobs and reusable artifacts

Speech and other long-running bridge work uses a bounded, process-local job registry. Jobs do not survive bridge restart. Synthesized preview or revision-conflict audio can be retained behind opaque, expiring, process-local tokens so a client can commit it without rerunning inference. Cancellation, expiry, successful consumption, and shutdown clean owned files.

### Schema migration

Project schemas migrate deterministically under the project lock. Migrations cover current state and retained undo/redo snapshots, preserve provenance, and verify managed media before publishing the new state. Unknown future schema versions fail closed instead of being interpreted by an older binary.

### Headless isolation

The bridge starts one typed headless process per request. This keeps crash, cancellation, temporary-file, and protocol boundaries simple. A release benchmark of one warm-up plus 30 status requests measured 41.16 ms median and 43.17 ms p95 on the recorded development machine, below the 100 ms median and 250 ms p95 follow-up thresholds. The isolation model is retained until repeated measurements show startup cost is material.

## Consequences

- New providers remain outside the project and timeline model, but must satisfy the provider-neutral contract and parity tests.
- Persisted intent supports audit and regeneration while keeping provider-specific inference details out of the domain.
- Local execution keeps speech text and audio on the user's machine, subject to the privacy behavior of any future provider a user configures.
- Pending jobs and artifact tokens are intentionally lost on restart; clients must resubmit work after `JOB_NOT_FOUND` or `GENERATED_ARTIFACT_NOT_FOUND`.
- Schema changes carry migration and integrity-test obligations across current state and history.
- Process-per-request startup is paid on every headless call in exchange for strong isolation. Crossing either recorded latency threshold creates an architecture follow-up; it does not silently change the transport model.
