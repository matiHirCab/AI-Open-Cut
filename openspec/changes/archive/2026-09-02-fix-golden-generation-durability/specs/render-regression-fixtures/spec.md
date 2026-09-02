## MODIFIED Requirements

### Requirement: Deliberate atomic golden updates
Golden verification MUST be the default, and replacing reviewed references MUST require an explicit update mode that stages and validates a complete bounded immutable generation before installation, durably persists every retained file and required directory entry, and only then atomically commits it through a versioned generation pointer. A pointer replacement MUST be considered committed once the atomic rename or replacement selects the new generation, even if a later pointer-directory durability sync fails. Failed content synchronization, generation installation, generation-directory synchronization, or other interrupted work confirmed before the pointer commit MUST leave the prior generation selected and byte-for-byte unchanged. A newly installed but unselected generation MUST be removed with best effort or left as strictly recognizable inactive data for bounded reconciliation, while a preexisting validated digest MUST never be removed because its resynchronization failed. When pointer durability is uncertain after a commit, both the prior and new complete durable generations MUST remain available so reopening can preserve whichever generation the strict pointer selects. Failure to clean an older generation after a successful commit MUST leave the new complete generation selected and retain the older generation for later bounded cleanup rather than reporting publication failure.

#### Scenario: Verify without rewriting selected evidence
- **WHEN** the golden command runs without explicit update mode
- **THEN** it performs conformance checks against the selected immutable generation and does not rewrite any checked-in reference or pointer, while it may remove only strictly recognized orphan data

#### Scenario: Fail before pointer commit
- **WHEN** update mode cannot render, decode, hash, validate, synchronize every retained file, durably install the complete generation, synchronize its required directory entries, or atomically replace the generation pointer before commit
- **THEN** it leaves the complete prior generation selected, removes new recognized temporary or inactive output with best effort, and never exposes a missing or partial canonical set

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
- **THEN** it preserves that selected complete generation and may clean only the other strictly recognized inactive generation

#### Scenario: Reconcile recognized orphan data
- **WHEN** any later golden invocation finds stale staging data, a pointer temporary, or an inactive generation with the harness's strict recognized naming and validated layout
- **THEN** it attempts to remove only those recognized inactive entries before capture and never removes the selected generation or an unknown path

#### Scenario: Defer failed cleanup
- **WHEN** startup or post-commit cleanup of recognized inactive data fails
- **THEN** the selected complete generation remains usable and the invocation reports cleanup pending without treating publication or conformance as failed
