# Durable production requests

## Purpose

Preserve bounded, explicitly opted-in rendering intent and truthful known outcomes across reconnects, separately from process-local jobs and artifact capabilities.

## Requirements


### Requirement: Opt-in stable durable render intent
The bridge SHALL provide an explicitly enabled bounded durable request registry for documented frame, range, audiovisual review and final-export intents connected to existing native producers. Admission MUST durably record stable caller request identity, canonical typed intent and captured project/revision before any producer executes. Identical repeated admission SHALL return the existing identity without duplicating work; conflicting identity reuse MUST fail explicitly. Execution SHALL require a separate explicit start and preserve core-owned validation, optimistic revisions, publication rules and existing runtime admission guards.

#### Scenario: Admit and explicitly start existing rendering
- **WHEN** an enabled authorized caller admits a supported original rendering intent and explicitly starts it
- **THEN** queued metadata precedes execution, core validates the captured request, and one process-local job produces the result without changing existing job/token contracts

#### Scenario: Repeat identity and concurrent start
- **WHEN** callers repeat identical admission, reuse an identity with different intent or concurrently start the same request
- **THEN** identical admission is idempotent, conflicting intent fails and at most one producer starts; no retry or reconnect automatically dispatches work

### Requirement: Truthful restart inventory and outcomes
Queued intent and known completed, failed or cancelled terminal metadata SHALL survive process restart within documented retention. Persisted running intent from an interrupted owner SHALL be exposed as uncertain, never inferred failed solely from restart or a missing ephemeral job. Failed or unconfirmed terminal persistence MUST expose uncertainty rather than claiming a durable success or a failed effect. Durable records MUST NOT store/adopt private artifact resources, generated tokens or opaque predecessor job identifiers as reusable execution handles. Export/project state remains authoritative for reconciliation; missing/expired metadata SHALL NOT establish absence of an effect.

#### Scenario: Restart queued running and terminal states
- **WHEN** real processes stop or crash around admission, native dispatch or terminal metadata publication
- **THEN** surviving queued and known terminal records remain distinguishable from uncertain interrupted execution and predecessor tokens/jobs are not rehydrated or replayed

#### Scenario: Lose terminal acknowledgement or storage
- **WHEN** rendering or export may complete but terminal persistence or its acknowledgement cannot be confirmed
- **THEN** inventory reports uncertainty without automatically rerunning effects, falsely claiming durable completion or labelling the effect failed

### Requirement: Bounded private ownership and fail-closed storage
The registry SHALL enforce documented record-count, serialized-byte and retention-age limits and a fixed bounded staging/ownership inventory across repeated crashes. These limits MUST NOT be represented as a quota on exported media, project assets or arbitrary foreign files. Unavailable, unsafe, malformed, future-version or foreign-owned storage MUST fail safely before producer side effects. Private configured runtime roots SHALL remain isolated; intentionally shared authorized runtime access and existing authentication remain unchanged. Crash inventory reads SHALL remain available where safe, while producer admission continues to require existing operator reconciliation and MUST NOT steal locks based on PID or age.

#### Scenario: Exercise capacity retention and malformed storage
- **WHEN** count/byte/age bounds, admission/terminal write faults, malformed/future records or repeated process crashes are exercised
- **THEN** the documented inventory bound holds, no partial producer admission occurs, unknown/foreign data is preserved and failures remain explicit and sanitized

#### Scenario: Isolate runtimes and preserve crash guards
- **WHEN** private clients use distinct configured roots, another owner occupies storage or an interrupted runtime guard exists
- **THEN** clients cannot adopt another root's records or outputs, unsafe writes/start fail before effects and safe reconciliation reads do not bypass the existing crash guard

### Requirement: Governed compatibility and measured recovery evidence
The additive durable request contract and capability SHALL have independently authored versioned fixtures, ownership entries and actual required parity consumers. Existing ephemeral_jobs_v1 clients, default behavior, job envelopes, stable error meanings, native/headless/project/provider schemas and restart guidance MUST remain compatible. ADR0005 SHALL explain the separate opted-in metadata decision using measured representative original multi-output rendering workloads and explicit limitations. Verification MUST cover actual producer integration and real admission/terminal crashes, beyond isolated storage helpers; exact-head protected and required checks remain mandatory before completion.

#### Scenario: Preserve disabled legacy clients
- **WHEN** durable storage is not opted in or a legacy client uses existing job/status/resource surfaces
- **THEN** existing ephemeral behavior remains unchanged and no durable producer, private token adoption or automatic replay is introduced

#### Scenario: Verify real multi-output recovery and canonical parity
- **WHEN** original multi-output workloads, independent fixture parity and actual native/MCP/packaged crash recovery checks run
- **THEN** measured outcomes and uncertainty match the documented bounded contract, unchanged predecessor proofs remain enforced and all required verification evidence is reported without claiming resumability or exactly-once effects

#### Scenario: Coexist with an independently governed source-planning extension
- **WHEN** the approved durable registry and separately approved source-planning CLI coexist after branch integration
- **THEN** both complete runtime implementations and required consumers remain present, exact independent inverses preserve each peer and restore unchanged independently frozen predecessor bytes within each pretty or compact representation in either order, and missing/narrowed additions or unrelated drift remain detected without repinning or relaxing checks

#### Scenario: Preserve the historical native hero tool inventory
- **WHEN** actual native hero MCP acceptance runs with the five additive durable operations registered
- **THEN** each independently fixture-owned durable tool is present exactly once, the unchanged full canonical inventory matches, and removing only approved additive tools preserves the historical78-tool oracle and all original native media and lifecycle checks
