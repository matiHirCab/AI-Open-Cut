## Context

The previous fix greedily matched old steps and used an entire component selector array as identity. Review reproduced order-dependent retention and sibling resets. Draft 2 stores selector-keyed steps, so identity-specific preparation must check representability before saving.

## Goals / Non-Goals

Goals: complete bidirectional ambiguity detection, scoped child retention, atomic rejection of unrepresentable results. Non-goals: new contracts, migrations, Unicode rendering or unrelated changes.

## Decisions

- Perform exact structural matching before font-intent matching. Use polynomial maximum bipartite matching and test alternative edge/unmatched outcomes, not permutation enumeration. Compare complete per-destination retention maps; retained versus unresolved is distinct. Deterministic equivalent assignments use original order. Keep operation work bounds.
- Match component kind and explicit component ID; anonymous creates match through overlapping local text IDs. Within a match retain only same local IDs with unchanged selector presence/value, ignoring non-font properties and track/item order. Do not infer identity from content.
- Use a preparation-only map for scoped text bindings, while root add/update retain their existing selector step representation. Apply component retention only to targeted local IDs after applying the operation. Resolve remaining missing bindings under current configuration.
- Capture actual local bindings after each old operation by replaying the old draft through the shared materialization helper. Selector steps alone cannot distinguish an already-bound child from an unresolved sibling sharing selectors or retain a later action's inherited font when an earlier action is removed. Prepare replacement operations chronologically and omit retention entries supplied identically by their actual inherited bindings. Old replay uses validated recorded bindings; replacement preparation may resolve agreed outcomes with current configuration. All other retained versus unresolved outcomes remain distinct. A retained binding conflicting with an already-inherited binding is unrepresentable by draft 2 and fails before publication.
- Build selector steps from newly unresolved identities; reject any differing complete bindings sharing a selector in one step before publication. This keeps draft 2 readable without migration; identity-keyed draft 3 was considered and explicitly rejected by the user.
- Preserve catalog/integrity/revision validation, stage bytes in memory, and publish only after all matching and representability checks succeed. No new dependency edge or external service is needed.

## Risks / Trade-offs

### Approved selector-reversion correction

The user's explicit “Fix component selector reversion” implementation request approves this extension. Preparation distinguishes retained local bindings, unchanged inherited bindings, and explicit resolution markers for every changed selector or new local ID in a matched component action. Markers participate in ambiguity comparisons independently of hashes; explicit resolution is never equivalent to retaining or inheriting a binding. Unmatched actions keep normal operation semantics.

After applying each operation, snapshot the identities normal replay leaves unresolved. Capture and clear actual inherited bindings only for marked local IDs, resolve with the existing resolver and current configuration, then compare complete bindings (all four faces and metadata). If a captured inherited binding differs, reject with descriptive INVALID_ARGUMENT before publication: draft 2 replay would retain it. Equal bindings succeed. Record selector steps only from the original unresolved snapshot, never from artificial clearing. Markers are neither public nor persisted; operations and all versions remain unchanged.

Add red regressions covering family/path/null reversion, repeated replacements, reintroduced IDs, equal-binding lifecycle parity, unchanged siblings, styled-face differences and atomic byte preservation, plus matching outcome coverage. Rerun every required check under Rust 1.97.0 before verification; unrelated active changes still block archival.

Indistinguishable edits can fail even when a user intended a particular pairing; explicit selector changes or preserving exact operations disambiguate them. Unrepresentable component edits fail instead of silently changing fonts. Previously lost bindings cannot be recovered. Polynomial matching must remain bounded by existing operation limits.

## Migration and rollback

### Approved preceding-bindings correction

The user's explicit implementation request approves this extension. Replace the earlier unresolved-only simulation and separate greedy phase consumption with a weighted assignment model maximizing structural matches first, then total compatible matches. Dummy vertices represent unmatched destinations; the square assignment matrix is at most 200 per dimension under existing operation limits. Use integer assignment potentials and zero-reduced-cost alternating cycles to retain every globally optimal candidate, plus a deterministic original-order canonical assignment. Production must not enumerate permutations.

Replay the old draft once for validated bindings. Prepare replacement operations chronologically through the existing store preparation loop. After normal application, compare all globally optimal matching outcomes against actual inherited bindings from the prepared prefix. Equivalent retained/inherited complete bindings normalize identically; explicit resolution remains distinct regardless of current hashes. Canonical assignment must never restrict the alternatives checked at later steps. Only after outcomes agree, apply retention, resolve through current configuration, check representability and record steps using the original unresolved snapshot. Staged fonts remain in memory until every step succeeds. This supersedes the earlier simulation decision above; matching may consume an already-resolved prefix, but never publishes during analysis.

Add red public-API tests for both failures and both orders, exact/intent/fresh prefixes, chains, source changes/removal, local scope, styled faces, lifecycle and byte-atomic rejection. Extend independent exhaustive tests to weighted optimum candidates, canonical order and chronological outcomes; preserve the 100-operation bound test. Reopen verification and rerun all required checks on Rust 1.97.0. Existing atomic rejection, formats, and unrelated archival blockers remain unchanged.

No migration or public type change. Existing schema 19/draft 2/profile v2 files remain readable. Binary rollback restores the old matching defects without converting persisted data.

## Approval

The user's explicit PLEASE IMPLEMENT THIS PLAN approves these artifacts as a transcription of that plan, including atomic representability rejection. No open product decisions remain.
