## MODIFIED Requirements

### Requirement: Deliberate atomic golden updates
Golden verification MUST be the default, and replacing reviewed references MUST require an explicit update mode that stages and validates a complete bounded immutable generation before installation, durably persists every retained file and every directory ancestor through the generation root, and only then atomically commits it through a versioned generation pointer. Every native golden invocation that reads, reconciles, renders, compares, captures, publishes, reports, or cleans the shared fixture container MUST first acquire the same blocking exclusive lock on a persistent coordination file and MUST retain that lock for its complete invocation. The coordination file MUST remain installed between invocations and MUST never be treated as cleanup residue. A pointer replacement MUST be considered committed once the atomic rename or replacement selects the new generation, even if a later pointer-directory durability sync fails. Failed content synchronization, generation installation, generation-directory synchronization, or other interrupted work confirmed before the pointer commit MUST leave the prior generation selected and byte-for-byte unchanged. A newly installed but unselected generation whose installation was confirmed successful MAY be removed with best effort or left as strictly recognizable inactive data for bounded reconciliation. When installation reports an error or otherwise has an ambiguous result, the invocation MUST NOT infer ownership merely because the digest path appeared and MUST leave that recognized path for a later locked reconciliation. A preexisting validated digest MUST never be removed because its resynchronization failed. When pointer durability is uncertain after a commit, both the prior and new complete durable generations MUST remain available so reopening can preserve whichever generation the strict pointer selects. Failure to clean an older generation after a successful commit MUST leave the new complete generation selected and retain the older generation for later bounded cleanup rather than reporting publication failure.

#### Scenario: Verify without rewriting selected evidence
- **WHEN** the golden command runs without explicit update mode
- **THEN** it performs conformance checks against the selected immutable generation and does not rewrite any checked-in reference or pointer, while it may remove only strictly recognized orphan data while holding the exclusive coordination lock

#### Scenario: Serialize concurrent golden invocations
- **WHEN** one golden invocation holds the coordination lock while using a selected generation or live stage and another invocation targets the same fixture container
- **THEN** the second invocation blocks before reading or reconciling the container and can clean recognized residue only after the first invocation releases the lock

#### Scenario: Release coordination after failure
- **WHEN** a golden invocation returns an error or unwinds through a panic after acquiring the coordination lock
- **THEN** RAII releases the lock and a later invocation can acquire it without unlinking or recreating the coordination file

#### Scenario: Synchronize a deeply nested reference
- **WHEN** a retained reference is nested below directories such as `frames/nested/deeper/0000.rgb`
- **THEN** the harness synchronizes `deeper`, `nested`, `frames`, and the generation root in deepest-first order before installing the generation

#### Scenario: Preserve an unconfirmed install destination
- **WHEN** generation installation reports an error and the destination digest path is observable afterward
- **THEN** the harness leaves `CURRENT` unchanged and preserves the recognizable destination for a later locked reconciliation instead of deleting it based only on observation

#### Scenario: Fail before pointer commit
- **WHEN** update mode cannot render, decode, hash, validate, synchronize every retained file and ancestor directory, durably install the complete generation, synchronize its required directory entries, or atomically replace the generation pointer before commit
- **THEN** it leaves the complete prior generation selected, removes only confirmed-owned new temporary or inactive output with best effort, and never exposes a missing or partial canonical set

#### Scenario: Fail first publication before pointer commit
- **WHEN** generation content or installation durability fails during an update with no existing `CURRENT`
- **THEN** the harness does not create `CURRENT` and leaves no selected canonical generation

#### Scenario: Reuse an installed digest
- **WHEN** the validated generation digest already exists before update mode attempts to select it
- **THEN** the harness revalidates and resynchronizes that generation before pointer commit and never removes the preexisting generation when synchronization fails

#### Scenario: Fail durability sync after pointer commit
- **WHEN** atomic pointer replacement selects the new complete durable generation but a later pointer-directory durability sync fails
- **THEN** publication remains committed with a non-fatal durability warning and retains both the prior and new generations

#### Scenario: Reopen after uncertain pointer durability
- **WHEN** the harness reopens after an uncertain durability result and `CURRENT` validly selects either the prior or new generation
- **THEN** it preserves that selected complete generation and may clean only the other strictly recognized inactive generation while holding the exclusive coordination lock

#### Scenario: Reconcile recognized orphan data
- **WHEN** any later golden invocation finds stale staging data, a pointer temporary, or an inactive generation with the harness's strict recognized naming and validated layout
- **THEN** it attempts to remove only those recognized inactive entries after acquiring the exclusive coordination lock and never removes the selected generation, persistent coordination file, or an unknown path

#### Scenario: Defer failed cleanup
- **WHEN** startup or post-commit cleanup of recognized inactive data fails
- **THEN** the selected complete generation remains usable and the invocation reports cleanup pending without treating publication or conformance as failed
