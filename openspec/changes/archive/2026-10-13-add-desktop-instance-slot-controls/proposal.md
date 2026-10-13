## Why

Issue190's required hero/motif instance inputs currently appear only as stored JSON in the desktop inspector. Users need definition-derived slot controls that preserve shared sources, independent occurrences and existing core safety.

## What Changes

- Present a selected root component instance's authoritative slot definitions, names, binding properties, defaults and overrides with explicit source labels.
- Add bounded controls for text, exact solid hex6 color, managed project asset and Boolean visibility slots. Keep rich text, numbers, enums, durations and component-local occurrences visibly read-only.
- Apply one selected override or remove it to restore the definition default through the existing revisioned component-instance edit, preserving every unrelated override and instance property.
- Provide explicit Apply, clear-override and draft Reset actions; preserve canonical failures, busy/conflict guards and refresh/replanning. Discard drafts when field, scoped selection, tab, history or authoritative revision changes.
- Demonstrate two independent instances and failure/history/reopen behavior in automated tests and actual isolated GPUI interactions.

## Capabilities

### New Capabilities

- `desktop-instance-slots`: definition-derived, presentation-only instance slot controls and attributable GUI verification.

### Modified Capabilities

None. Existing desktop hierarchy/review, slot, component and solid-paint requirements retain their meaning, including component-local read-only inspection and core-owned validation.

## Impact

Desktop inspector/model of local drafts and tests; a deterministic disposable GUI project generator and documentation. No new public operation, request/event/catalog field, persisted schema, renderer, dependency or migration. Compatibility is additive desktop presentation; existing typed core edits and canonical errors remain authoritative.

Verified base is current main `0f069ae4d23a356a60555aa71137ba9d92314b63`. Issue190 is open without comments;182/183 are closed and present. No duplicate PR exists. Verified unmerged PR200 (`6fd01454`) and PR199 (`e2a6e6fd`) remain compatibility targets, without a publishing dependency or unmerged-domain import; re-fetch main and normally preserve any later merged domain before publication.

## Non-goals

No shared-definition editor, arbitrary child/path editor, asset importer, precomposition controls, rich-text flattening, animation controls, parallel domain validator, automatic retry, new scene/slot model, global configuration or credential changes, private production desktop/video access, merge, auto-merge or deployment. Actual isolated GUI execution is required; screenshots of an empty or nonrendering window do not satisfy feature acceptance.
