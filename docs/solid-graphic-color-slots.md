# Solid graphic color slots

`solid_graphic_color_slots_v1` extends existing typed component slots with three literal bindings:

| Property | Compatible local target |
| --- | --- |
| `shape.fill.color` | Shape with an existing solid fill |
| `shape.stroke.color` | Shape with an existing stroke whose paint is solid |
| `grid.dot.paint.color` | Dot-pattern grid with existing solid paint |

Use kind `color`, empty constraints and the existing `{ "type": "color", "value": "#12AbEf" }` envelope. Upper/lowercase hex is retained in stored values. Each channel becomes its byte divided by255 in unassociated sRGB. The existing paint alpha is preserved exactly; there is no alpha override, hex8, gradient value or implicit paint creation. Stroke metrics, geometry, grid spacing/radius and unrelated visual animation stay unchanged. Ordinary paint editing and `visual.opacity` retain their existing meaning.

For example, a component-local shape named `panel` may declare:

```json
{
  "id": "accent",
  "name": "Accent",
  "kind": "color",
  "required": true,
  "defaultValue": { "type": "color", "value": "#ffcc00" },
  "binding": { "targetLayerId": "panel", "property": "shape.fill.color" },
  "constraints": {}
}
```

Submit slots through existing `component_create`, `component_update` or `component_define_slots`. Customize a root occurrence through existing `component_instance_update` with its complete component/timing tuple and `slotValues: { "accent": { "type": "color", "value": "#00ffff" } }`. An explicit override map replaces the whole map; omission preserves it. Overrides take precedence over defaults and never alter the shared definition or another occurrence. A required slot without a default needs a value in every instance. An omitted optional value retains the original paint.

Use the current expected revision. Existing standalone/batch operations, prior creation aliases, locks, one-step batch history, undo/redo and supported drafts remain applicable. Slot IDs, local target IDs and values are literal; scope comes from the owning definition. A root/foreign-only target is missing locally (`ITEM_NOT_FOUND`); caller-selected scope fields, incompatible targets/kinds, absent/gradient paint, SVG or line-grid targets and unknown paths fail with `INVALID_ARGUMENT`. Changed locked local bindings or locked root instances fail with `TRACK_LOCKED`; stale writes use retryable `REVISION_CONFLICT`. A later batch failure rolls back the entire candidate.

Project schema45 marks the new binding identifiers. Opening supported predecessors migrates current and retained undo/redo snapshots through existing locked atomic persistence, preserving content/provenance/resources. A new graphic binding mislabeled as an older schema, or an unknown future current/history schema, fails closed. Draft format/version2 is unchanged; draft candidate state/preview stays isolated until a current-revision commit. Discard preserves saved state, and stale commit requires explicit rebase.

The versioned [contract fixture](../contracts/solid-graphic-color-slots-v1.json) supplies a small native shape/dot component and two palettes. Canonical Rust/TypeScript parity and actual source/fresh packaged MCP tests cover independent overrides, alpha, failure atomicity, lifecycle and native frame/range/draft pixels with blank-output controls. Those functional checks do not establish portable performance, dense composition capacity, simultaneous heavy-export reliability or separately installed desktop readiness. No new renderer foundation, arbitrary property language, SVG editing, palette UI or dependency is introduced.
