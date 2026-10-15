## Why
Schema44 slot edits currently publish schema45 project/history migration before validating the requested mutation. A rejected definition, instance update or late batch can therefore rewrite both files while returning ITEM_NOT_FOUND at the original revision.

## What Changes
- Route slot definition and instance-update operations through existing staged migration and one recoverable publication after full validation.
- Add exact-byte rejection, successful adoption, revision, history and fault/recovery regressions.

## Capabilities
### New Capabilities
None.
### Modified Capabilities
- `project-persistence`: make legacy slot-edit adoption and requested mutation one atomic generation.

## Impact
Core staging classification and regression tests only; no new public/persisted schema, operation, dependency or rendering behavior. Existing transport/provider contracts remain unchanged.

## Non-goals
No #207 narration features, general persistence redesign, new migration version, merge or deployment. The #207 working tree remains isolated in source/.
