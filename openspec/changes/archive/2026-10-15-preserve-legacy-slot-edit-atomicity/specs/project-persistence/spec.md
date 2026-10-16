## ADDED Requirements

### Requirement: Atomic legacy slot edit adoption
Core SHALL stage supported legacy project and retained-history migration together with slot definition or component-instance update mutations, including a batch containing either operation. Validation MUST complete before migration or edited project/history/draft/font state is published. Rejected missing binding targets, unknown slot overrides, stale revisions and late batch errors MUST preserve prior authoritative project/history bytes and revision. Valid edits MUST publish one migrated project/history generation with exactly one revision increment through the existing project lock, resource ownership and recoverable commit point. Existing typed errors, alias resolution, source-matched drafts, undo/redo, resource bounds and postcommit recovery SHALL remain unchanged.

#### Scenario: Reject each legacy slot mutation without incidental migration
- **WHEN** a genuine schema44 project with schema44 retained undo/redo receives an invalid slot definition, invalid instance override, or batch whose valid earlier operation precedes an invalid slot operation
- **THEN** the established ITEM_NOT_FOUND is returned and project/history bytes, revision and retained resources are unchanged

#### Scenario: Commit valid adoption or recover its complete generation
- **WHEN** a valid legacy slot definition, instance update or batch is submitted, including interruption before or after the existing journal commit point
- **THEN** validation and migration publish one complete current-schema generation with one revision increment, precommit rejection preserves prior bytes, postcommit interruption recovers the exact committed edit/history, and repeated reopen never reapplies it
