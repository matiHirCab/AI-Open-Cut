# Typed template slots

Schema 12 adds `slots` to each component definition and `slotValues` to each stored nested instance. Protocol 1 advertises `typed_template_slots`. Slots are validated and persisted now; root component placement and rendering remain issue #24. Adding slots does not change root duration, preview, range preview, draft preview, export, audio, ordering or renderer fallback.

Use `component_define_slots {componentId, slots}` to replace a definition's entire slot list. Headless wraps it in `edit`; MCP exposes the same name. Both support `timeline_batch_edit`/headless `edit_batch`, `projectId` and `expectedRevision`. An earlier component creation alias can be used as `componentId: "@card"`. The slot operation does not produce `resultAlias`; slot and target IDs are literal local identifiers.

`component_create` accepts optional slots (empty by default). `component_update` accepts optional slots: omission preserves them, and an explicit array replaces them. Other update fields still replace the complete definition content. Nested instance requests accept optional slotValues (empty by default); schema-12 persisted documents and returned state require explicit fields.

```json
{
  "id": "heading",
  "name": "Heading",
  "kind": "text",
  "required": true,
  "defaultValue": {"type": "text", "value": "Welcome"},
  "binding": {"targetLayerId": "title", "property": "text.document"},
  "constraints": {"minLength": 1, "maxLength": 120}
}
```

An instance override is `slotValues: {"heading": {"type": "text", "value": "Introduction"}}`. Overrides take precedence over defaults, without changing the shared base tracks. A required slot can lack a default on an uninstantiated definition, but every stored instance must supply it. An absent optional value preserves the base property. Defaults are validated even when overridden. Removing a slot used by an instance, or removing its target, fails atomically; replace dependent instance values before an incompatible definition change.

| Kind | Value and constraints | Binding property |
| --- | --- | --- |
| text | Unicode string; optional scalar-count minLength/maxLength | text.document on text |
| rich_text | `{runs:[{text,bold?,italic?,color?}]}`; optional scalar-count minLength/maxLength | text.document on text |
| color | Six-digit `#RRGGBB`; empty constraints | text.color on text |
| number | Finite JSON number; optional inclusive min/max | visual.opacity, value in [0,1] |
| boolean | JSON true/false; empty constraints | visual.hidden |
| enum | String in required unique choices; alignment choices restricted to left/center/right | text.alignment on text |
| duration | Nonnegative safe integer milliseconds; optional inclusive integer min/max | item.durationMs; effective item duration must be positive |
| asset | `{kind:"asset",scope:"project",id:"managed-id"}`; optional unique nonempty assetKinds (image/video/audio) | media.asset on media |

Values always have `{type,value}` with the matching type tag. Null, unknown fields, coercion and inapplicable constraints are rejected. Rich text has no HTML, SVG, URL, font or executable fields and is not flattened into a rendering fallback. Ordinary text remains ordinary prose, including URL-like strings. Asset values accept managed project identifiers only, never paths or network resources.

Bindings resolve stable IDs within the owning component. They cannot address root, another definition, arbitrary JSON paths or another slot. Two slots cannot write the same target/property. Track reordering does not change identity. Core checks the complete derived candidate, including local duration bounds, source trims, instance time scales, media compatibility and inherited existing target rules; it never clamps invalid values. Coordinates and timing remain component-local, with half-open integer-millisecond intervals and existing stacking rules.

Limits: 128 slots per definition, 4096 slots per project snapshot, 128 overrides per instance, 4096 Unicode scalars per text/rich-text value, 1–256 rich-text runs, 1–128 enum choices of 1–128 scalars, and 1048576 aggregate default/override text scalars per snapshot. Slot IDs use `[A-Za-z0-9_-]{1,128}`; names are nonblank and at most 256 UTF-8 bytes. Numbers must be finite; millisecond fields and numeric duration constraints must not exceed 9007199254740991. Existing target rules can impose tighter limits (for example text items retain their nonempty 4096-byte text bound, and alignment has three choices). Hidden and unused definitions receive the same validation.

Successful batches commit one revision and one undo step. Missing components, targets and override slot IDs return ITEM_NOT_FOUND; a missing safe managed asset returns ASSET_NOT_FOUND. Invalid types, constraints, bindings and resource forms return non-retryable INVALID_ARGUMENT. Changed slots targeting locked tracks return TRACK_LOCKED; unchanged slots remain legal. Stale revisions retain retryable REVISION_CONFLICT. Failures preserve the entire prior project and history generation. Undo, redo, drafts and deterministic reopen retain exact slot data.

Supported schemas 1–11 migrate current state and all retained undo/redo snapshots atomically under the project lock, adding empty slot fields while preserving other values, media, revisions and output. Schema 12 requires its fields; malformed current/history data and unknown future schemas fail closed without publication. Older binaries cannot read schema 12; no downgrade is supplied. Managed assets referenced only by defaults or overrides remain protected in current state, history and durable drafts, even when a default is overridden.

The runtime catalog is `contracts/template-slots-v1.json`. `motion-graphics-v1.json` remains preparatory fixture-only vocabulary. Canonical Rust/TypeScript tests distinguish structural rejection from core semantic validation; transport schemas do not duplicate core binding resolution.

All legal slot IDs, including __proto__, constructor and toString, are preserved as own data keys through native and MCP requests/responses. Their values receive the same validation and default/required rules as other IDs; unknown IDs still fail in core. Group opacity slots use effective Transform2D opacity even when the group omits Transform2D, preserving other transform fields and stored base tracks.

Closed records reject every unknown own enumerable field before bridge parsing can discard it. This includes `__proto__`, `constructor`, `toString` and ordinary unknown names on definitions, bindings, constraints, all `{type,value}` envelopes, rich-text documents/runs and managed-asset references. Bridge diagnostics retain the full nested record path and offending key names; malformed values retain their nested value paths. Override maps remain open to legal string slot IDs. Malformed standalone and batch requests preserve project state and history. This correction changes neither schema 12 nor protocol 1 or published input/output schemas.

The catalog's `regressions.closedRecords` matrix covers every closed-record location with raw JSON negative examples. Rust and TypeScript consumers read those bytes directly so JavaScript object-literal or bundler handling cannot remove special keys before validation. Native decoding, bridge request/response checks and the shared source/packaged workflow consume the same evidence.
